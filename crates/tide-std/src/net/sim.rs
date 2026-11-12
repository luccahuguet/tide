use super::*;
use crate::task::Signal;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Duration;

const CAPACITY: usize = 65_536;
const BACKLOG: usize = 128;
#[derive(Default)]
pub(super) struct SimNet {
    listeners: BTreeMap<u16, Bound>,
    conns: BTreeMap<usize, Connection>,
    next: usize,
    waiters: Vec<Arc<Signal>>,
}
// The baton makes allocation atomic; release the net lock before drawing from the runtime.
fn port(state: &Mutex<SimNet>, c: &NetContext) -> io::Result<u16> {
    let taken = {
        let s = state.lock().unwrap();
        s.listeners
            .keys()
            .copied()
            .chain(
                s.conns
                    .values()
                    .flat_map(|conn| conn.ends.iter().filter(|e| e.open).map(|e| e.addr.port())),
            )
            .collect::<BTreeSet<_>>()
    };
    if (49152..=65535).all(|p| taken.contains(&p)) {
        return Err(ErrorKind::AddrNotAvailable.into());
    }
    loop {
        let p = 49152 + c.draw(16384) as u16;
        if !taken.contains(&p) {
            return Ok(p);
        }
    }
}

#[derive(Debug)]
struct Bound {
    backlog: VecDeque<usize>,
}
#[derive(Debug)]
struct Connection {
    ends: [End; 2],
}
#[derive(Debug)]
struct End {
    addr: SocketAddr,
    open: bool,
    read_shut: bool,
    write_shut: bool,
    reset_at: Option<Duration>,
    inbox: VecDeque<Segment>,
    bytes: usize,
}
#[derive(Debug)]
struct Segment {
    arrive: Duration,
    bytes: VecDeque<u8>,
    fin: bool,
}
impl End {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            open: true,
            read_shut: false,
            write_shut: false,
            reset_at: None,
            inbox: VecDeque::new(),
            bytes: 0,
        }
    }
    fn queue(&mut self, now: Duration, latency: Duration, bytes: &[u8], fin: bool) {
        let arrive = (now + latency).max(self.inbox.back().map_or(now, |s| s.arrive));
        self.bytes += bytes.len();
        self.inbox.push_back(Segment {
            arrive,
            bytes: bytes.iter().copied().collect(),
            fin,
        });
    }
}
impl SimNet {
    fn changed(&mut self) {
        for signal in self.waiters.drain(..) {
            signal.wake();
        }
    }
    fn signal(&mut self) -> Arc<Signal> {
        let signal = Arc::new(Signal::new());
        self.waiters.push(signal.clone());
        signal
    }
}

fn wait(
    state: &Mutex<SimNet>,
    c: &NetContext,
    name: &'static str,
    id: usize,
    span: Span,
    until: Option<Duration>,
    signal: Arc<Signal>,
) -> io::Result<()> {
    let result = c.wait(name, id, span, until, Some(signal.clone()));
    state
        .lock()
        .unwrap()
        .waiters
        .retain(|s| !Arc::ptr_eq(s, &signal));
    result
}

pub(super) struct Listener {
    pub(super) addr: SocketAddr,
    state: Arc<Mutex<SimNet>>,
    network: Network,
}
pub(super) struct Stream {
    pub(super) local: SocketAddr,
    pub(super) peer: SocketAddr,
    state: Arc<Mutex<SimNet>>,
    network: Network,
    id: usize,
    side: usize,
}

pub(super) fn bind(
    net: &Net,
    state: &Arc<Mutex<SimNet>>,
    c: &NetContext,
    mut addr: SocketAddr,
    span: Span,
) -> io::Result<Listener> {
    c.begin("net.bind", 0, span)?;
    let result = (|| {
        if addr.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST)
            && addr.ip() != IpAddr::V4(Ipv4Addr::UNSPECIFIED)
        {
            return Err(ErrorKind::AddrNotAvailable.into());
        }
        if addr.port() == 0 {
            addr.set_port(port(state, c)?);
        }
        let mut s = state.lock().unwrap();
        if s.listeners.contains_key(&addr.port()) {
            return Err(ErrorKind::AddrInUse.into());
        }
        s.listeners.insert(
            addr.port(),
            Bound {
                backlog: VecDeque::new(),
            },
        );
        Ok(Listener {
            addr,
            state: state.clone(),
            network: net.network.clone(),
        })
    })();
    c.record(
        "net.bind",
        0,
        span,
        &format!("addr={addr} result={}", outcome(&result)),
    );
    result
}

pub(super) fn connect(
    net: &Net,
    state: &Arc<Mutex<SimNet>>,
    c: &NetContext,
    addr: SocketAddr,
    span: Span,
) -> io::Result<Stream> {
    c.check()?;
    let latency = c.latency() + c.latency();
    let until = c.now() + latency;
    let result = (|| {
        c.wait("net.connect", addr.port() as usize, span, Some(until), None)?;
        loop {
            c.check()?;
            let signal = {
                let mut s = state.lock().unwrap();
                if addr.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST)
                    || !s.listeners.contains_key(&addr.port())
                {
                    return Err(ErrorKind::ConnectionRefused.into());
                }
                if s.listeners[&addr.port()].backlog.len() < BACKLOG {
                    break;
                }
                s.signal()
            };
            wait(
                state,
                c,
                "net.connect",
                addr.port() as usize,
                span,
                None,
                signal,
            )?;
        }
        let port = port(state, c)?;
        let local = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
        let mut s = state.lock().unwrap();
        let id = s.next;
        s.next += 1;
        s.conns.insert(
            id,
            Connection {
                ends: [End::new(local), End::new(addr)],
            },
        );
        s.listeners
            .get_mut(&addr.port())
            .unwrap()
            .backlog
            .push_back(id);
        s.changed();
        Ok(Stream {
            local,
            peer: addr,
            state: state.clone(),
            network: net.network.clone(),
            id,
            side: 0,
        })
    })();
    c.wake();
    c.record(
        "net.connect",
        result.as_ref().map_or(0, |s| s.id),
        span,
        &format!(
            "addr={addr} latency={} result={}",
            latency.as_nanos(),
            outcome(&result)
        ),
    );
    result
}
impl Listener {
    pub(super) fn accept(&self, c: &NetContext, span: Span) -> io::Result<(Stream, SocketAddr)> {
        let port = self.addr.port();
        c.begin("net.accept", port as usize, span)?;
        let result = (|| loop {
            c.check()?;
            let signal = {
                let mut s = self.state.lock().unwrap();
                if let Some(id) = s.listeners.get_mut(&port).unwrap().backlog.pop_front() {
                    let peer = s.conns[&id].ends[0].addr;
                    let local = s.conns[&id].ends[1].addr;
                    s.changed();
                    break Ok((
                        Stream {
                            local,
                            peer,
                            state: self.state.clone(),
                            network: self.network.clone(),
                            id,
                            side: 1,
                        },
                        peer,
                    ));
                }
                s.signal()
            };
            wait(
                &self.state,
                c,
                "net.accept",
                port as usize,
                span,
                None,
                signal,
            )?;
        })();
        c.wake();
        let detail = match &result {
            Ok((stream, peer)) => format!("conn={} peer={peer} result=ok", stream.id),
            Err(_) => format!("result={}", outcome(&result)),
        };
        c.record("net.accept", port as usize, span, &detail);
        result
    }
}
impl Stream {
    pub(super) fn read(&self, c: &NetContext, buf: &mut [u8], span: Span) -> io::Result<usize> {
        c.begin("net.read", self.id, span)?;
        let result = (|| loop {
            c.check()?;
            let now = c.now();
            let (signal, until) = {
                let mut s = self.state.lock().unwrap();
                let end = &mut s.conns.get_mut(&self.id).unwrap().ends[self.side];
                if buf.is_empty() || end.read_shut {
                    break Ok(0);
                }
                if end.reset_at.is_some_and(|t| t <= now) {
                    break Err(ErrorKind::ConnectionReset.into());
                }
                let mut n = 0;
                while let Some(segment) = end.inbox.front_mut() {
                    if segment.arrive > now || n == buf.len() {
                        break;
                    }
                    if segment.fin {
                        break;
                    }
                    while n < buf.len() {
                        let Some(byte) = segment.bytes.pop_front() else {
                            break;
                        };
                        buf[n] = byte;
                        n += 1;
                        end.bytes -= 1;
                    }
                    if segment.bytes.is_empty() {
                        end.inbox.pop_front();
                    }
                }
                if n != 0 {
                    s.changed();
                    break Ok(n);
                }
                if end
                    .inbox
                    .front()
                    .is_some_and(|seg| seg.fin && seg.arrive <= now)
                {
                    break Ok(0);
                }
                let until = end
                    .inbox
                    .front()
                    .map(|seg| seg.arrive)
                    .into_iter()
                    .chain(end.reset_at)
                    .min();
                (s.signal(), until)
            };
            wait(&self.state, c, "net.read", self.id, span, until, signal)?;
        })();
        c.wake();
        c.record(
            "net.read",
            self.id,
            span,
            &format!(
                "n={} result={}",
                result.as_ref().copied().unwrap_or(0),
                outcome(&result)
            ),
        );
        result
    }
    pub(super) fn write(&self, c: &NetContext, buf: &[u8], span: Span) -> io::Result<usize> {
        c.begin("net.write", self.id, span)?;
        let mut latency = Duration::ZERO;
        let result = (|| loop {
            c.check()?;
            let now = c.now();
            let signal = {
                let mut s = self.state.lock().unwrap();
                let ends = &s.conns[&self.id].ends;
                if ends[self.side].write_shut || ends[self.side].reset_at.is_some_and(|t| t <= now)
                {
                    break Err(ErrorKind::BrokenPipe.into());
                }
                if buf.is_empty() {
                    break Ok(0);
                }
                if !ends[1 - self.side].open
                    || ends[1 - self.side].read_shut
                    || ends[1 - self.side].bytes < CAPACITY
                {
                    break Ok(buf.len().min(CAPACITY - ends[1 - self.side].bytes));
                }
                s.signal()
            };
            wait(&self.state, c, "net.write", self.id, span, None, signal)?;
        })()
        .and_then(|n| {
            if n == 0 {
                return Ok(0);
            }
            let now = c.now();
            latency = c.latency();
            let peer_open = self.state.lock().unwrap().conns[&self.id].ends[1 - self.side].open;
            let reset = if peer_open {
                None
            } else {
                Some(now + latency + c.latency())
            };
            let mut s = self.state.lock().unwrap();
            let ends = &mut s.conns.get_mut(&self.id).unwrap().ends;
            if !peer_open {
                if ends[self.side].reset_at.is_none() {
                    ends[self.side].reset_at = reset;
                }
            } else if !ends[1 - self.side].read_shut {
                ends[1 - self.side].queue(now, latency, &buf[..n], false);
            }
            s.changed();
            Ok(n)
        });
        c.wake();
        c.record(
            "net.write",
            self.id,
            span,
            &format!(
                "n={} latency={} result={}",
                result.as_ref().copied().unwrap_or(0),
                latency.as_nanos(),
                outcome(&result)
            ),
        );
        result
    }
    pub(super) fn shutdown(&self, c: &NetContext, how: Shutdown, span: Span) -> io::Result<()> {
        c.begin("net.shutdown", self.id, span)?;
        self.finish(c, how, false, span);
        Ok(())
    }
    fn finish(&self, c: &NetContext, how: Shutdown, close: bool, span: Span) {
        let now = c.now();
        let write = matches!(how, Shutdown::Write | Shutdown::Both);
        let send_fin = write
            && (close || !self.state.lock().unwrap().conns[&self.id].ends[self.side].write_shut);
        let latency = if send_fin && !std::thread::panicking() {
            c.latency()
        } else {
            Duration::ZERO
        };
        {
            let mut s = self.state.lock().unwrap();
            let ends = &mut s.conns.get_mut(&self.id).unwrap().ends;
            if write {
                ends[self.side].write_shut = true;
            }
            if matches!(how, Shutdown::Read | Shutdown::Both) {
                ends[self.side].read_shut = true;
                ends[self.side].inbox.clear();
                ends[self.side].bytes = 0;
            }
            if send_fin && ends[1 - self.side].open && !ends[1 - self.side].read_shut {
                ends[1 - self.side].queue(now, latency, &[], true);
            }
            if close {
                ends[self.side].open = false;
            }
            if ends.iter().all(|e| !e.open) {
                s.conns.remove(&self.id);
            }
            s.changed();
        }
        c.wake();
        c.record(
            if close { "net.close" } else { "net.shutdown" },
            self.id,
            span,
            &format!("how={how:?} latency={}", latency.as_nanos()),
        );
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        if let Some(c) = self.network.closing() {
            self.finish(&c, Shutdown::Both, true, std::panic::Location::caller());
        }
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(c) = self.network.closing() {
            let now = c.now();
            {
                let mut s = self.state.lock().unwrap();
                if let Some(listener) = s.listeners.remove(&self.addr.port()) {
                    for id in listener.backlog {
                        let conn = s.conns.get_mut(&id).unwrap();
                        conn.ends[0].reset_at = Some(now);
                        conn.ends[1].open = false;
                        if !conn.ends[0].open {
                            s.conns.remove(&id);
                        }
                    }
                }
                s.changed();
            }
            c.wake();
            c.record(
                "net.close_listener",
                self.addr.port() as usize,
                std::panic::Location::caller(),
                &self.addr.to_string(),
            );
        }
    }
}
fn outcome<T>(result: &io::Result<T>) -> String {
    result
        .as_ref()
        .map_or_else(|e| format!("{:?}", e.kind()), |_| "ok".into())
}

impl std::fmt::Debug for Listener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpListener")
            .field("addr", &self.addr)
            .finish()
    }
}
impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpStream")
            .field("local", &self.local)
            .field("peer", &self.peer)
            .finish()
    }
}

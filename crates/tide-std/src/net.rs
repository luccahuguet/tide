// TCP authority and handles (reference §6.1, §6.7; research/21-channels-tcp.md §§1.9–1.13).
// Linux only. Real sockets are nonblocking and waits observe scope cancellation/deadlines.
// Simulated loopback uses bounded streams and seed-drawn 1–10 ms packet delays.
// Other real hostnames use Rust's inline resolver, which cannot yet be abandoned on cancel;
// simulation resolves only IP literals and localhost. HTTP, UDP, options and owned I/O are pending.
use crate::io::{self, ErrorKind, Read, Write};
use crate::task::{NetContext, Network};
pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use std::sync::{Arc, Mutex};
#[path = "net/real.rs"]
mod tcp_real;
#[path = "net/sim.rs"]
mod tcp_sim;
use tcp_sim::SimNet;
type Span = &'static std::panic::Location<'static>;

/// Address data accepted by `Net`, sealed so resolution cannot escape its authority.
pub trait ToSocketAddrs: address::Sealed {}
impl<T: address::Sealed + ?Sized> ToSocketAddrs for T {}
mod address {
    use super::*;
    pub enum Address {
        Socket(SocketAddr),
        Host(String, u16),
    }
    pub trait Sealed {
        fn address(&self) -> io::Result<Address>;
    }
    impl Sealed for SocketAddr {
        fn address(&self) -> io::Result<Address> {
            Ok(Address::Socket(*self))
        }
    }
    impl<T: Sealed + ?Sized> Sealed for &T {
        fn address(&self) -> io::Result<Address> {
            (**self).address()
        }
    }
    impl Sealed for str {
        fn address(&self) -> io::Result<Address> {
            if let Ok(addr) = self.parse() {
                return Ok(Address::Socket(addr));
            }
            let (host, port) = self.rsplit_once(':').ok_or(ErrorKind::InvalidInput)?;
            if host.is_empty() || host.contains(':') {
                return Err(ErrorKind::InvalidInput.into());
            }
            Ok(Address::Host(
                host.to_owned(),
                port.parse()
                    .map_err(|_| io::Error::from(ErrorKind::InvalidInput))?,
            ))
        }
    }
    impl Sealed for String {
        fn address(&self) -> io::Result<Address> {
            self.as_str().address()
        }
    }
    impl Sealed for (IpAddr, u16) {
        fn address(&self) -> io::Result<Address> {
            SocketAddr::new(self.0, self.1).address()
        }
    }
    impl Sealed for (Ipv4Addr, u16) {
        fn address(&self) -> io::Result<Address> {
            (IpAddr::V4(self.0), self.1).address()
        }
    }
    impl Sealed for (Ipv6Addr, u16) {
        fn address(&self) -> io::Result<Address> {
            (IpAddr::V6(self.0), self.1).address()
        }
    }
    impl Sealed for (&str, u16) {
        fn address(&self) -> io::Result<Address> {
            Ok(match self.0.parse::<IpAddr>() {
                Ok(ip) => Address::Socket(SocketAddr::new(ip, self.1)),
                Err(_) if self.0.is_empty() => return Err(ErrorKind::InvalidInput.into()),
                Err(_) => Address::Host(self.0.to_owned(), self.1),
            })
        }
    }
    impl Address {
        pub(super) fn key(&self) -> (String, u16) {
            match self {
                Self::Socket(a) => (a.ip().to_string(), a.port()),
                Self::Host(h, p) => (
                    h.parse::<IpAddr>()
                        .map_or_else(|_| h.to_ascii_lowercase(), |ip| ip.to_string()),
                    *p,
                ),
            }
        }
        pub(super) fn resolve(self, simulated: bool) -> io::Result<Vec<SocketAddr>> {
            match self {
                Self::Socket(a) => Ok(vec![a]),
                Self::Host(h, p) => {
                    if let Ok(ip) = h.parse() {
                        return Ok(vec![SocketAddr::new(ip, p)]);
                    }
                    if h.eq_ignore_ascii_case("localhost") {
                        return Ok(vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), p)]);
                    }
                    if simulated {
                        return Err(ErrorKind::InvalidInput.into());
                    }
                    // shortcut: inline DNS cannot be cancelled, replace when Net::resolve owns resolution.
                    std::net::ToSocketAddrs::to_socket_addrs(&(h.as_str(), p))
                        .map(|it| it.collect())
                        .map_err(io::from_std)
                }
            }
        }
    }
}

/// Network authority. Attenuations return intersected clones and never expand authority.
#[derive(Clone)]
pub struct Net {
    network: Network,
    sim: Option<Arc<Mutex<SimNet>>>,
    only: Option<Arc<[(String, u16)]>>,
    may_bind: bool,
}
impl Net {
    pub(crate) fn new(tasks: Option<crate::task::Tasks>) -> Self {
        Self {
            sim: tasks
                .as_ref()
                .map(|_| Arc::new(Mutex::new(SimNet::default()))),
            network: Network::new(tasks),
            only: None,
            may_bind: true,
        }
    }
    pub fn no_bind(&self) -> Self {
        Self {
            may_bind: false,
            ..self.clone()
        }
    }
    /// Literal case-insensitive host and exact port matching, without lookup. Invalid entries grant nothing.
    pub fn only(&self, hosts: impl IntoIterator<Item: AsRef<str>>) -> Self {
        use address::Sealed;
        let hosts: Vec<_> = hosts
            .into_iter()
            .filter_map(|h| h.as_ref().address().ok().map(|a| a.key()))
            .filter(|key| self.only.as_ref().is_none_or(|old| old.contains(key)))
            .collect();
        Self {
            only: Some(hosts.into()),
            ..self.clone()
        }
    }
    #[track_caller]
    pub fn bind(&self, addr: impl ToSocketAddrs) -> io::Result<TcpListener> {
        let span = std::panic::Location::caller();
        let c = self.network.context();
        c.check()?;
        if !self.may_bind {
            return Err(ErrorKind::PermissionDenied.into());
        }
        let addrs = address::Sealed::address(&addr)?.resolve(self.sim.is_some())?;
        let mut error = ErrorKind::InvalidInput.into();
        for addr in addrs {
            let result = if let Some(sim) = &self.sim {
                tcp_sim::bind(self, sim, &c, addr, span).map(Listener::Sim)
            } else {
                tcp_real::bind(&c, addr).map(Listener::Real)
            };
            match result {
                Ok(inner) => {
                    return Ok(TcpListener {
                        inner,
                        network: self.network.clone(),
                    });
                }
                Err(e) => error = e,
            }
        }
        Err(error)
    }
    #[track_caller]
    pub fn connect(&self, addr: impl ToSocketAddrs) -> io::Result<TcpStream> {
        let span = std::panic::Location::caller();
        let c = self.network.context();
        c.check()?;
        let addr = address::Sealed::address(&addr)?;
        if self
            .only
            .as_ref()
            .is_some_and(|hosts| !hosts.contains(&addr.key()))
        {
            return Err(ErrorKind::PermissionDenied.into());
        }
        let mut error = ErrorKind::InvalidInput.into();
        for addr in addr.resolve(self.sim.is_some())? {
            let result = if let Some(sim) = &self.sim {
                tcp_sim::connect(self, sim, &c, addr, span).map(Stream::Sim)
            } else {
                tcp_real::connect(&c, addr, span).map(Stream::Real)
            };
            match result {
                Ok(inner) => {
                    return Ok(TcpStream {
                        inner,
                        network: self.network.clone(),
                    });
                }
                Err(e) if e.is_cancelled() => return Err(e),
                Err(e) => error = e,
            }
        }
        Err(error)
    }
}

#[derive(Debug)]
enum Listener {
    Real(std::net::TcpListener),
    Sim(tcp_sim::Listener),
}
#[derive(Debug)]
enum Stream {
    Real(std::net::TcpStream),
    Sim(tcp_sim::Stream),
}
/// A TCP listener made only through `Net::bind`.
pub struct TcpListener {
    inner: Listener,
    network: Network,
}
/// A TCP byte stream made only through `Net` or an authorized listener.
pub struct TcpStream {
    inner: Stream,
    network: Network,
}
impl std::fmt::Debug for TcpListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
impl std::fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
impl TcpListener {
    #[track_caller]
    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let c = self.network.context();
        let span = std::panic::Location::caller();
        c.check()?;
        let (inner, peer) = match &self.inner {
            Listener::Real(l) => {
                tcp_real::retry(&c, l, false, span, || l.accept()).and_then(|(s, a)| {
                    s.set_nonblocking(true).map_err(io::from_std)?;
                    Ok((Stream::Real(s), a))
                })?
            }
            Listener::Sim(l) => {
                let (s, a) = l.accept(&c, span)?;
                (Stream::Sim(s), a)
            }
        };
        Ok((
            TcpStream {
                inner,
                network: self.network.clone(),
            },
            peer,
        ))
    }
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.network.context();
        match &self.inner {
            Listener::Real(l) => l.local_addr().map_err(io::from_std),
            Listener::Sim(l) => Ok(l.addr),
        }
    }
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming {
            listener: self,
            ended: false,
        }
    }
}
/// Yields cancellation once, then ends, like `io::Lines`.
pub struct Incoming<'a> {
    listener: &'a TcpListener,
    ended: bool,
}
impl Iterator for Incoming<'_> {
    type Item = io::Result<TcpStream>;
    #[track_caller]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }
        let result = self.listener.accept().map(|(s, _)| s);
        self.ended = result.as_ref().is_err_and(|e| e.is_cancelled());
        Some(result)
    }
}
impl TcpStream {
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.network.context();
        match &self.inner {
            Stream::Real(s) => s.local_addr().map_err(io::from_std),
            Stream::Sim(s) => Ok(s.local),
        }
    }
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.network.context();
        match &self.inner {
            Stream::Real(s) => s.peer_addr().map_err(io::from_std),
            Stream::Sim(s) => Ok(s.peer),
        }
    }
    #[track_caller]
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let c = self.network.context();
        c.check()?;
        match &self.inner {
            Stream::Real(s) => s.shutdown(how).map_err(io::from_std),
            Stream::Sim(s) => s.shutdown(&c, how, std::panic::Location::caller()),
        }
    }
}
impl Read for &TcpStream {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let c = self.network.context();
        c.check()?;
        let span = std::panic::Location::caller();
        match &self.inner {
            Stream::Real(_) if buf.is_empty() => Ok(0),
            Stream::Real(s) => {
                tcp_real::retry(&c, s, false, span, || std::io::Read::read(&mut &*s, buf))
            }
            Stream::Sim(s) => s.read(&c, buf, span),
        }
    }
}
impl Write for &TcpStream {
    #[track_caller]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let c = self.network.context();
        c.check()?;
        let span = std::panic::Location::caller();
        match &self.inner {
            Stream::Real(s) => {
                tcp_real::retry(&c, s, true, span, || std::io::Write::write(&mut &*s, buf))
            }
            Stream::Sim(s) => s.write(&c, buf, span),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.network.context();
        Ok(())
    }
}
impl Read for TcpStream {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&*self).read(buf)
    }
}
impl Write for TcpStream {
    #[track_caller]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self).write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    #[test]
    fn handles_never_cross_real_or_simulated_worlds() {
        let net;
        let listener;
        {
            let sim = crate::Root::sim(17);
            net = sim.net.clone();
            listener = net.bind("127.0.0.1:0").unwrap();
            assert!(catch_unwind(AssertUnwindSafe(|| Net::new(None).bind("127.0.0.1:0"))).is_err());
        }
        assert!(catch_unwind(AssertUnwindSafe(|| net.bind("127.0.0.1:0"))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| listener.local_addr())).is_err());
        let other = crate::Root::sim(18);
        assert!(catch_unwind(AssertUnwindSafe(|| listener.accept())).is_err());
        other.tasks.checkpoint().unwrap();
        // Outliving handles may drop without a draw or a record in the other world.
        drop(listener);
        assert!(!other.trace().contains("net.close"));
    }
}

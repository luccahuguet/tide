// TCP authority and handles (reference §6.1, §6.7; research/21-channels-tcp.md §§1.9–1.13).
// Linux only. Real sockets are nonblocking and waits observe scope cancellation/deadlines.
// Simulated loopback uses bounded streams and seed-drawn 1–10 ms packet delays.
// Other real hostnames go through NET-RESOLVE (`net/resolve.rs`), which cancellation abandons;
// simulation resolves only IP literals and localhost for TCP. `only` keys every outbound peer
// by NET-HOST (`net/host.rs`). `Net::get` (`net/http.rs`) is one HTTP/1.1 GET; simulated gets
// reach only `sim.world.http` peers. `Net::resolve` returns a granted host's addresses
// (research/48); simulation gives `world.http` domains synthetic 198.18.0.n addresses.
// UDP, options and owned I/O are pending.
use crate::io::{self, ErrorKind, Read, Write};
use crate::task::{NetContext, Network};
pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use std::sync::{Arc, Mutex};
#[path = "net/real.rs"]
mod tcp_real;
#[path = "net/sim.rs"]
mod tcp_sim;
use tcp_sim::SimNet;
#[path = "net/host.rs"]
mod host;
use host::{Host, Key};
#[path = "net/resolve.rs"]
mod resolve;
#[path = "net/http.rs"]
mod http;
pub use http::{NetError, Url, UrlError};
pub(crate) use http::BODY_LIMIT;
pub(crate) use tcp_sim::Peer;
type Span = &'static std::panic::Location<'static>;

/// Address data accepted by `Net`, sealed so resolution cannot escape its authority.
pub trait ToSocketAddrs: address::Sealed {}
impl<T: address::Sealed + ?Sized> ToSocketAddrs for T {}
mod address {
    use super::*;
    pub enum Address {
        Socket(SocketAddr),
        Host(Host, u16),
    }
    pub trait Sealed {
        fn address(&self) -> io::Result<Address>;
    }
    impl Sealed for SocketAddr {
        fn address(&self) -> io::Result<Address> {
            // NET-HOST refuses zone identifiers.
            if matches!(self, SocketAddr::V6(a) if a.scope_id() != 0) {
                return Err(ErrorKind::InvalidInput.into());
            }
            Ok(Address::Socket(*self))
        }
    }
    impl<T: Sealed + ?Sized> Sealed for &T {
        fn address(&self) -> io::Result<Address> {
            (**self).address()
        }
    }
    fn written(written: &str, port: u16) -> io::Result<Address> {
        Ok(match host::host(written).ok_or(ErrorKind::InvalidInput)? {
            Host::Ip(ip) => Address::Socket(SocketAddr::new(ip, port)),
            name => Address::Host(name, port),
        })
    }
    impl Sealed for str {
        fn address(&self) -> io::Result<Address> {
            let (host, port) = host::split(self).ok_or(ErrorKind::InvalidInput)?;
            written(host, port)
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
            match self.0.parse::<IpAddr>() {
                Ok(ip) => Ok(Address::Socket(SocketAddr::new(ip, self.1))),
                Err(_) => written(self.0, self.1),
            }
        }
    }
    impl Address {
        /// NET-HOST's key, the only thing `only` compares.
        pub(super) fn key(&self) -> Key {
            match self {
                Self::Socket(a) => Key {
                    host: host::ip(a.ip()),
                    port: a.port(),
                },
                Self::Host(h, p) => Key {
                    host: h.clone(),
                    port: *p,
                },
            }
        }
        /// IP literals and localhost need no lookup; simulation resolves no other name, and
        /// the real backend asks NET-RESOLVE, which the caller's cancellation abandons.
        pub(super) fn resolve(
            self,
            c: &NetContext,
            simulated: bool,
            span: Span,
        ) -> io::Result<Vec<SocketAddr>> {
            match self {
                Self::Socket(a) => Ok(vec![a]),
                Self::Host(h, p) => match (h.literal(), &h) {
                    (Some(ip), _) => Ok(vec![SocketAddr::new(ip, p)]),
                    _ if simulated => Err(ErrorKind::InvalidInput.into()),
                    (None, Host::Name(name)) => resolve::lookup(c, name, p, None, span),
                    (None, Host::Ip(_)) => unreachable!(),
                },
            }
        }
    }
}

/// Network authority. Attenuations return intersected clones and never expand authority.
#[derive(Clone)]
pub struct Net {
    network: Network,
    sim: Option<Arc<Mutex<SimNet>>>,
    only: Option<Arc<[Key]>>,
    may_bind: bool,
    /// A test root that only crate tests add (research/42 §14); no public root override exists.
    #[cfg(test)]
    test_roots: Option<Arc<rustls::RootCertStore>>,
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
            #[cfg(test)]
            test_roots: None,
        }
    }
    fn allows(&self, key: &Key) -> bool {
        self.only.as_ref().is_none_or(|keys| keys.contains(key))
    }
    pub fn no_bind(&self) -> Self {
        self.network.context();
        Self {
            may_bind: false,
            ..self.clone()
        }
    }
    /// NET-HOST: `"host:port"` entries keyed as `connect` and `get` key their peers, matched by
    /// equality before any lookup. Invalid entries grant nothing. It bounds outbound peer
    /// operations only: `bind` and accepted peers stay open unless `no_bind` is added too.
    pub fn only(&self, hosts: impl IntoIterator<Item: AsRef<str>>) -> Self {
        self.network.context();
        let hosts: Vec<_> = hosts
            .into_iter()
            .filter_map(|h| host::entry(h.as_ref()))
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
        let addrs = address::Sealed::address(&addr)?.resolve(&c, self.sim.is_some(), span)?;
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
        if !self.allows(&addr.key()) {
            return Err(ErrorKind::PermissionDenied.into());
        }
        let mut error = ErrorKind::InvalidInput.into();
        for addr in addr.resolve(&c, self.sim.is_some(), span)? {
            let result = if let Some(sim) = &self.sim {
                tcp_sim::connect(self, sim, &c, addr, span).map(Stream::Sim)
            } else {
                tcp_real::connect(&c, addr, span, None).map(Stream::Real)
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

impl Net {
    /// One HTTP/1.1 GET (NET-GET, research/42 §3): a 2xx body, or the `NetError` that stopped
    /// it. Cancellation, the URL's shape and `only` are checked before any lookup, connection
    /// or simulated peer; `deadline` bounds the whole call.
    #[track_caller]
    pub fn get(&self, url: &Url, deadline: crate::time::Instant) -> Result<Vec<u8>, NetError> {
        let span = std::panic::Location::caller();
        let c = self.network.context();
        let checked = url.key();
        let mut delays = Vec::new();
        let result = (|| {
            c.check().map_err(|_| NetError::Cancelled)?;
            let (tls, key) = checked
                .as_ref()
                .ok_or_else(|| NetError::Io(ErrorKind::InvalidInput.into()))?;
            if !self.allows(key) {
                return Err(NetError::Denied);
            }
            match &self.sim {
                Some(sim) => tcp_sim::get(sim, &c, url, key, deadline.0, span, &mut delays),
                None => http::get(self, &c, url, *tls, key, deadline.0, span),
            }
        })();
        if self.sim.is_some() {
            tcp_sim::record_get(&c, span, url, checked.as_ref().map(|(_, k)| k), &delays, &result);
        }
        result
    }
    /// One name lookup (NET-RESOLVE-CALL, research/48 §3): the host's addresses, never empty,
    /// or the `NetError` that stopped it. Cancellation, the host's shape and `only` (an entry
    /// for the host on any port) are checked before any lookup; IP literals and `localhost`
    /// need none. It binds no later call: `connect` and `get` look the name up again.
    #[track_caller]
    pub fn resolve(&self, host: &str, deadline: crate::time::Instant) -> Result<Vec<IpAddr>, NetError> {
        let span = std::panic::Location::caller();
        let c = self.network.context();
        let checked = host::tuple(host);
        let result = (|| {
            c.check().map_err(|_| NetError::Cancelled)?;
            let host = checked
                .as_ref()
                .ok_or_else(|| NetError::Io(ErrorKind::InvalidInput.into()))?;
            if !self.only.as_ref().is_none_or(|keys| keys.iter().any(|k| k.host == *host)) {
                return Err(NetError::Denied);
            }
            match &self.sim {
                Some(sim) => tcp_sim::resolve(sim, &c, host, deadline.0),
                None => http::resolve(&c, host, deadline.0, span),
            }
        })();
        if self.sim.is_some() {
            tcp_sim::record_resolve(&c, span, checked.as_ref(), &result);
        }
        result
    }
    #[track_caller]
    pub(crate) fn world_http(&self, addr: &str, peer: Peer) -> String {
        let c = self.network.context();
        tcp_sim::world_http(self.sim.as_ref().unwrap(), &c, addr, peer, std::panic::Location::caller())
    }
    pub(crate) fn clear_http_peers(&self) {
        tcp_sim::clear_http_peers(self.sim.as_ref().unwrap());
    }
    #[cfg(test)]
    pub(crate) fn with_test_root(&self, der: &[u8]) -> Self {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(rustls::pki_types::CertificateDer::from(der.to_vec())).unwrap();
        Self {
            test_roots: Some(Arc::new(roots)),
            ..self.clone()
        }
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
        self.network.context();
        self.inner.fmt(f)
    }
}
impl std::fmt::Debug for TcpStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.network.context();
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
                tcp_real::retry(&c, l, false, span, None, || l.accept()).and_then(|(s, a)| {
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
        self.network.context();
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
        self.listener.network.context();
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
                tcp_real::retry(&c, s, false, span, None, || std::io::Read::read(&mut &*s, buf))
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
                tcp_real::retry(&c, s, true, span, None, || std::io::Write::write(&mut &*s, buf))
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
#[path = "net/tests.rs"]
mod http_tests;

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
        assert!(catch_unwind(AssertUnwindSafe(|| net.no_bind())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| net.only(["127.0.0.1:80"]))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| net.get(&Url::parse("http://127.0.0.1/").unwrap(), other.clock.now()))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| listener.incoming())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| format!("{listener:?}"))).is_err());
        let mut incoming = Incoming { listener: &listener, ended: true };
        assert!(catch_unwind(AssertUnwindSafe(|| incoming.next())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| listener.accept())).is_err());
        other.tasks.checkpoint().unwrap();
        // Outliving handles may drop without a draw or a record in the other world.
        drop(listener);
        assert!(!other.trace().contains("net.close"));
    }
}

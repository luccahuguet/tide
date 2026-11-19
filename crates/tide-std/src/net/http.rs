// Net::get's types and real backend (research/42 §§2, 3, 5, 8; companion 21). One HTTP/1.1 GET
// per call over the nonblocking TCP path, TLS through rustls (ring, compiled-in roots,
// NoKeyLog, no resumption), heads parsed by httparse and framed here (§3 step 9).
use super::*;
use std::io::{Read as _, Write as _};
use std::time::Duration;

/// Bodies larger than this are `Io(InvalidData)` (provisional; research/42 §3).
pub(crate) const BODY_LIMIT: usize = 64 << 20;
const HEAD_LIMIT: usize = 64 << 10;

/// Why a `Net::get` failed (companion 21). Not `PartialEq`: test it with `matches!`.
#[derive(Debug)]
#[non_exhaustive]
pub enum NetError {
    /// The call's deadline passed.
    Timeout,
    /// The scope was cancelled.
    Cancelled,
    /// The host and port are outside `Net::only`.
    Denied,
    /// Every address refused the connection.
    Refused,
    /// The connection was reset or closed before the response was complete.
    Reset,
    /// The name did not resolve.
    Dns,
    /// A final status outside 200–299, 3xx included.
    Status(u16),
    /// Any other failure: `InvalidInput` for a URL `get` refuses, `InvalidData` for a TLS or
    /// malformed, oversized or unsupported response.
    Io(io::Error),
}
impl NetError {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
    /// `Timeout`, `Reset`, `Refused`, and `Status` 408, 429 or 5xx.
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Timeout | Self::Reset | Self::Refused => true,
            Self::Status(code) => matches!(code, 408 | 429 | 500..=599),
            _ => false,
        }
    }
}
impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Timeout => "timed out",
            Self::Cancelled => "cancelled",
            Self::Denied => "denied by Net::only",
            Self::Refused => "connection refused",
            Self::Reset => "connection reset",
            Self::Dns => "name did not resolve",
            Self::Status(code) => return write!(f, "HTTP status {code}"),
            Self::Io(e) => return e.fmt(f),
        })
    }
}
impl std::error::Error for NetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
pub(crate) fn invalid_data(message: impl Into<String>) -> NetError {
    NetError::Io(io::Error::new(ErrorKind::InvalidData, message.into()))
}

/// One parsed URL (WHATWG), immutable, with v0's host restrictions (research/42 §2). It holds
/// no way to resolve or reach its host: only `Net::get` does that.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url(url::Url);
/// Why `Url::parse` refused its input.
#[derive(Debug)]
pub struct UrlError(&'static str);
impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid URL: {}", self.0)
    }
}
impl std::error::Error for UrlError {}

impl Url {
    /// Refuses non-ASCII input and, in an `http` or `https` host as written, a `%`, an empty
    /// label or an `xn--` label, before the WHATWG parser runs.
    pub fn parse(input: &str) -> Result<Url, UrlError> {
        if !input.is_ascii() {
            return Err(UrlError("non-ASCII input"));
        }
        if let Some(written) = host::url_host(input) {
            if host::host(&written).is_none() {
                return Err(UrlError("host outside v0's ASCII host rule"));
            }
        }
        url::Url::parse(input)
            .map(Url)
            .map_err(|_| UrlError("not a URL"))
    }
    pub fn scheme(&self) -> &str {
        self.0.scheme()
    }
    pub fn host_str(&self) -> Option<&str> {
        self.0.host_str()
    }
    pub fn port(&self) -> Option<u16> {
        self.0.port()
    }
    pub fn port_or_known_default(&self) -> Option<u16> {
        self.0.port_or_known_default()
    }
    pub fn path(&self) -> &str {
        self.0.path()
    }
    pub fn query(&self) -> Option<&str> {
        self.0.query()
    }
    pub fn fragment(&self) -> Option<&str> {
        self.0.fragment()
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    /// NET-GET step 2 then NET-HOST: `None` is `Io(InvalidInput)`.
    pub(crate) fn key(&self) -> Option<(bool, Key)> {
        let tls = match self.0.scheme() {
            "http" => false,
            "https" => true,
            _ => return None,
        };
        if !self.0.username().is_empty() || self.0.password().is_some() {
            return None;
        }
        let host = host::key_host(self.0.host()?);
        Some((tls, Key { host, port: self.0.port_or_known_default()? }))
    }
    /// The URL without its fragment, as a simulated peer sees it.
    pub(crate) fn without_fragment(&self) -> Url {
        let mut url = self.0.clone();
        url.set_fragment(None);
        Url(url)
    }
    /// The request target: path and query, as the parser serialized them.
    fn target(&self) -> String {
        match self.0.query() {
            Some(q) => format!("{}?{q}", self.0.path()),
            None => self.0.path().to_owned(),
        }
    }
}
impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl std::fmt::Debug for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Url({:?})", self.as_str())
    }
}
// A JSON string, read back through `Url::parse` (research/42 §2).
impl crate::__codec::Encode for Url {
    fn __encode(&self, out: &mut crate::__codec::Encoder) {
        out.string(self.as_str());
    }
}
impl crate::__codec::Decode for Url {
    fn __decode(input: &mut crate::__codec::Decoder<'_>) -> Result<Self, crate::json::Error> {
        input.parsed(Url::parse)
    }
}
impl std::str::FromStr for Url {
    type Err = UrlError;
    fn from_str(s: &str) -> Result<Url, UrlError> {
        Url::parse(s)
    }
}

/// The exact request of NET-GET step 8.
pub(crate) fn request(url: &Url, tls: bool, key: &Key) -> String {
    let default = if tls { 443 } else { 80 };
    let port = if key.port == default { String::new() } else { format!(":{}", key.port) };
    format!(
        "GET {} HTTP/1.1\r\nHost: {}{port}\r\nUser-Agent: tide\r\nAccept: */*\r\nConnection: close\r\n\r\n",
        url.target(),
        key.host
    )
}

fn expired(until: Duration) -> bool {
    crate::time::monotonic() >= until
}
fn fail(e: io::Error, until: Duration) -> NetError {
    match e.kind() {
        _ if e.is_cancelled() => NetError::Cancelled,
        ErrorKind::TimedOut if expired(until) => NetError::Timeout,
        ErrorKind::ConnectionReset
        | ErrorKind::ConnectionAborted
        | ErrorKind::BrokenPipe
        | ErrorKind::UnexpectedEof => NetError::Reset,
        _ => NetError::Io(e),
    }
}

fn config(extra: Option<&rustls::RootCertStore>) -> Arc<rustls::ClientConfig> {
    let build = |roots: rustls::RootCertStore| {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut config = rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .expect("ring supports TLS 1.2 and 1.3")
            .with_root_certificates(roots)
            .with_no_client_auth();
        // The TLS guard (research/42 §8): no key log whatever SSLKEYLOGFILE says, no resumption.
        config.key_log = Arc::new(rustls::NoKeyLog);
        config.resumption = rustls::client::Resumption::disabled();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Arc::new(config)
    };
    if let Some(roots) = extra {
        return build(roots.clone());
    }
    static CONFIG: std::sync::OnceLock<Arc<rustls::ClientConfig>> = std::sync::OnceLock::new();
    CONFIG
        .get_or_init(|| {
            build(rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            })
        })
        .clone()
}

/// A connection with its deadline: plain TCP, or TLS over it.
struct Conn<'a> {
    c: &'a NetContext,
    socket: std::net::TcpStream,
    tls: Option<rustls::ClientConnection>,
    pending: Vec<u8>,
    eof: bool,
    until: Duration,
    span: Span,
}
impl Conn<'_> {
    fn raw_read(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        let s = &self.socket;
        tcp_real::retry(self.c, s, false, self.span, Some(self.until), || (&*s).read(buf))
            .map_err(|e| fail(e, self.until))
    }
    fn raw_write(&mut self, mut buf: &[u8]) -> Result<(), NetError> {
        while !buf.is_empty() {
            let s = &self.socket;
            let n = tcp_real::retry(self.c, s, true, self.span, Some(self.until), || {
                (&*s).write(buf)
            })
            .map_err(|e| fail(e, self.until))?;
            buf = &buf[n..];
        }
        Ok(())
    }
    fn flush_tls(&mut self) -> Result<(), NetError> {
        let mut out = Vec::new();
        while let Some(tls) = self.tls.as_mut().filter(|t| t.wants_write()) {
            tls.write_tls(&mut out).map_err(|e| NetError::Io(io::from_std(e)))?;
        }
        self.raw_write(&out)
    }
    /// Feeds TLS records from the socket; at EOF tells rustls the transport closed.
    fn fill_tls(&mut self) -> Result<(), NetError> {
        if self.pending.is_empty() && !self.eof {
            let mut chunk = [0; 16 << 10];
            let n = self.raw_read(&mut chunk)?;
            self.eof = n == 0;
            self.pending.extend_from_slice(&chunk[..n]);
        }
        let tls = self.tls.as_mut().unwrap();
        loop {
            let mut rest = &self.pending[..];
            let n = tls.read_tls(&mut rest).map_err(|e| NetError::Io(io::from_std(e)))?;
            self.pending.drain(..n);
            tls.process_new_packets()
                .map_err(|e| invalid_data(format!("TLS: {e}")))?;
            if n == 0 || self.pending.is_empty() {
                break;
            }
        }
        self.flush_tls()
    }
    fn handshake(&mut self) -> Result<(), NetError> {
        loop {
            self.flush_tls()?;
            if !self.tls.as_ref().unwrap().is_handshaking() {
                return Ok(());
            }
            if self.eof {
                return Err(invalid_data("TLS: connection closed during the handshake"));
            }
            self.fill_tls()?;
        }
    }
    fn send(&mut self, mut request: &[u8]) -> Result<(), NetError> {
        if self.tls.is_none() {
            return self.raw_write(request);
        }
        while !request.is_empty() {
            let n = self.tls.as_mut().unwrap().writer().write(request)
                .map_err(|e| NetError::Io(io::from_std(e)))?;
            if n == 0 {
                return Err(NetError::Io(ErrorKind::WriteZero.into()));
            }
            self.flush_tls()?;
            request = &request[n..];
        }
        Ok(())
    }
    /// Ok(0) is a clean end: a plain EOF, or TLS `close_notify`. A TLS EOF without it is `Reset`.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        if self.tls.is_none() {
            return self.raw_read(buf);
        }
        loop {
            self.c.check().map_err(|_| NetError::Cancelled)?;
            if expired(self.until) {
                return Err(NetError::Timeout);
            }
            match self.tls.as_mut().unwrap().reader().read(buf) {
                Ok(n) => return Ok(n),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Err(NetError::Reset);
                }
                Err(e) => return Err(NetError::Io(io::from_std(e))),
            }
            if self.eof && self.pending.is_empty() {
                return Err(NetError::Reset);
            }
            self.fill_tls()?;
        }
    }
}

// RFC 9112 §7.1.1 / RFC 9110 §5.6: httparse converts the size but ignores extension syntax.
fn chunk_line(mut line: &[u8]) -> Option<()> {
    fn bws(line: &[u8]) -> &[u8] {
        &line[line.iter().take_while(|&&b| matches!(b, b' ' | b'\t')).count()..]
    }
    fn token(line: &[u8]) -> Option<&[u8]> {
        let n = line.iter().take_while(|&&b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)).count();
        (n != 0).then_some(&line[n..])
    }
    let digits = line.iter().take_while(|b| b.is_ascii_hexdigit()).count();
    if digits == 0 { return None; }
    line = &line[digits..];
    while !line.is_empty() {
        line = bws(bws(line).strip_prefix(b";")?);
        line = token(line)?;
        if let Some(value) = bws(line).strip_prefix(b"=") {
            line = bws(value);
            if let Some(value) = line.strip_prefix(b"\"") {
                line = value;
                loop {
                    let (&byte, rest) = line.split_first()?;
                    line = rest;
                    match byte {
                        b'"' => break,
                        b'\\' => {
                            let (&escaped, rest) = line.split_first()?;
                            if !matches!(escaped, b'\t' | b' '..=b'~' | 0x80..=0xff) { return None; }
                            line = rest;
                        }
                        b'\t' | b' ' | b'!' | b'#'..=b'[' | b']'..=b'~' | 0x80..=0xff => {}
                        _ => return None,
                    }
                }
            } else {
                line = token(line)?;
            }
        }
    }
    Some(())
}

/// The response reader (§3 step 9). It owns framing; `httparse` reads heads and chunk sizes.
struct Response<'a, 'b> {
    conn: &'a mut Conn<'b>,
    buf: Vec<u8>,
    pos: usize,
}
impl Response<'_, '_> {
    /// Reads more bytes; false at a clean end.
    fn more(&mut self) -> Result<bool, NetError> {
        let mut chunk = [0; 64 << 10];
        let n = self.conn.read(&mut chunk)?;
        self.buf.extend_from_slice(&chunk[..n]);
        Ok(n != 0)
    }
    fn need(&mut self, len: usize) -> Result<(), NetError> {
        while self.buf.len() - self.pos < len {
            if !self.more()? {
                return Err(NetError::Reset);
            }
        }
        Ok(())
    }
    fn result(&mut self) -> Result<Vec<u8>, NetError> {
        let mut heads = 0;
        let (code, chunked, length) = loop {
            let mut headers = [httparse::EMPTY_HEADER; 100];
            let mut head = httparse::Response::new(&mut headers);
            let len = match head.parse(&self.buf[self.pos..]) {
                Ok(httparse::Status::Complete(len)) => len,
                Ok(httparse::Status::Partial) => {
                    if heads + self.buf.len() - self.pos > HEAD_LIMIT {
                        return Err(invalid_data("response heads over 64 KiB"));
                    }
                    if !self.more()? {
                        return Err(NetError::Reset);
                    }
                    continue;
                }
                Err(e) => return Err(invalid_data(format!("malformed response head: {e}"))),
            };
            heads += len;
            if heads > HEAD_LIMIT {
                return Err(invalid_data("response heads over 64 KiB"));
            }
            let code = head.code.unwrap();
            if !(100..=599).contains(&code) {
                return Err(invalid_data("invalid HTTP status code"));
            }
            match code {
                101 => return Err(invalid_data("101 Switching Protocols was not asked for")),
                100..=199 => {
                    self.pos += len;
                    continue;
                }
                204 => return Ok(Vec::new()),
                304 => return Err(NetError::Status(304)),
                _ => {}
            }
            let (mut codings, mut lengths) = (Vec::new(), Vec::new());
            for h in head.headers.iter() {
                let values = if h.name.eq_ignore_ascii_case("transfer-encoding") {
                    &mut codings
                } else if h.name.eq_ignore_ascii_case("content-length") {
                    &mut lengths
                } else {
                    continue;
                };
                let value = std::str::from_utf8(h.value)
                    .map_err(|_| invalid_data("non-ASCII framing header"))?;
                values.extend(value.split(',').map(|p| p.trim_matches([' ', '\t']).to_ascii_lowercase()));
            }
            if !codings.is_empty() && !lengths.is_empty() {
                return Err(invalid_data("both Transfer-Encoding and Content-Length"));
            }
            if !codings.is_empty() && codings != ["chunked"] {
                return Err(invalid_data("unsupported transfer coding"));
            }
            let length = match lengths.first() {
                None => None,
                Some(first) => {
                    if lengths.iter().any(|l| l != first || l.is_empty() || !l.bytes().all(|b| b.is_ascii_digit())) {
                        return Err(invalid_data("invalid Content-Length"));
                    }
                    Some(first.parse::<u64>().map_err(|_| invalid_data("invalid Content-Length"))?)
                }
            };
            self.pos += len;
            break (code, !codings.is_empty(), length);
        };
        let body = self.body(chunked, length)?;
        if (200..300).contains(&code) {
            Ok(body)
        } else {
            Err(NetError::Status(code))
        }
    }
    fn body(&mut self, chunked: bool, length: Option<u64>) -> Result<Vec<u8>, NetError> {
        let oversized = || invalid_data("response body over 64 MiB");
        if let Some(n) = length {
            let n = usize::try_from(n).ok().filter(|&n| n <= BODY_LIMIT).ok_or_else(oversized)?;
            self.need(n)?;
            return Ok(self.buf[self.pos..self.pos + n].to_vec());
        }
        if !chunked {
            while self.more()? {
                if self.buf.len() - self.pos > BODY_LIMIT {
                    return Err(oversized());
                }
            }
            if self.buf.len() - self.pos > BODY_LIMIT {
                return Err(oversized());
            }
            return Ok(self.buf.split_off(self.pos));
        }
        let mut body = Vec::new();
        loop {
            let (len, size) = match httparse::parse_chunk_size(&self.buf[self.pos..]) {
                Ok(httparse::Status::Complete(found)) => found,
                Ok(httparse::Status::Partial) => {
                    if self.buf.len() - self.pos > HEAD_LIMIT {
                        return Err(invalid_data("malformed chunk size"));
                    }
                    if !self.more()? {
                        return Err(NetError::Reset);
                    }
                    continue;
                }
                Err(_) => return Err(invalid_data("malformed chunk size")),
            };
            if chunk_line(&self.buf[self.pos..self.pos + len - 2]).is_none() {
                return Err(invalid_data("malformed chunk size or extension"));
            }
            self.pos += len;
            if size == 0 {
                break;
            }
            let size = usize::try_from(size)
                .ok()
                .filter(|&s| s <= BODY_LIMIT - body.len())
                .ok_or_else(oversized)?;
            self.need(size + 2)?;
            body.extend_from_slice(&self.buf[self.pos..self.pos + size]);
            if &self.buf[self.pos + size..self.pos + size + 2] != b"\r\n" {
                return Err(invalid_data("malformed chunk"));
            }
            self.pos += size + 2;
            // Keep the buffer to what is unread.
            self.buf.drain(..self.pos);
            self.pos = 0;
        }
        // Validate discarded trailers, including the byte bound on a completed block.
        loop {
            let bytes = &self.buf[self.pos..];
            let mut headers = vec![httparse::EMPTY_HEADER; bytes.iter().filter(|&&b| b == b'\n').count() + 1];
            match httparse::parse_headers(bytes, &mut headers) {
                Ok(httparse::Status::Complete((len, _))) => {
                    if len > HEAD_LIMIT {
                        return Err(invalid_data("trailers over 64 KiB"));
                    }
                    return Ok(body);
                }
                Ok(httparse::Status::Partial) if bytes.len() > HEAD_LIMIT => {
                    return Err(invalid_data("trailers over 64 KiB"));
                }
                Ok(httparse::Status::Partial) => {
                    if !self.more()? {
                        return Err(NetError::Reset);
                    }
                }
                Err(e) => return Err(invalid_data(format!("malformed trailers: {e}"))),
            }
        }
    }
}

/// NET-GET step 6: the addresses in order. All refused is `Refused`; otherwise `Io` of the first
/// non-refusal error. Cancellation and the deadline keep precedence.
pub(super) fn dial(
    c: &NetContext,
    addrs: Vec<SocketAddr>,
    span: Span,
    until: Duration,
) -> Result<std::net::TcpStream, NetError> {
    let mut first = None;
    for addr in addrs {
        match tcp_real::connect(c, addr, span, Some(until)) {
            Ok(s) => return Ok(s),
            Err(e) if e.is_cancelled() => return Err(NetError::Cancelled),
            Err(e) if e.kind() == ErrorKind::TimedOut && expired(until) => {
                return Err(NetError::Timeout);
            }
            Err(e) if e.kind() == ErrorKind::ConnectionRefused => {}
            Err(e) => first = first.or(Some(e)),
        }
    }
    Err(first.map_or(NetError::Refused, NetError::Io))
}

/// Every lookup failure is `Dns`; cancellation and the passed deadline keep their variants. A
/// resolver's own `TimedOut` before the deadline is a resolver failure.
fn lookup_error(e: io::Error, until: Duration) -> NetError {
    match e.kind() {
        _ if e.is_cancelled() => NetError::Cancelled,
        ErrorKind::TimedOut if expired(until) => NetError::Timeout,
        _ => NetError::Dns,
    }
}

/// NET-RESOLVE-CALL steps 4–6 on the real backend (research/48 §3); steps 1–3 ran in
/// `Net::resolve`. Port 0 asks the resolver what `connect`'s lookup asks, without a port.
pub(super) fn resolve(c: &NetContext, host: &Host, until: Duration, span: Span) -> Result<Vec<IpAddr>, NetError> {
    if expired(until) {
        return Err(NetError::Timeout);
    }
    if let Some(ip) = host.literal() {
        return Ok(vec![ip]);
    }
    let Host::Name(name) = host else { unreachable!() };
    let addrs = resolve::lookup(c, name, 0, Some(until), span).map_err(|e| lookup_error(e, until))?;
    Ok(addrs.into_iter().map(|a| a.ip()).collect())
}

/// NET-GET steps 4–10 on the real backend; steps 1–3 ran in `Net::get`.
pub(super) fn get(
    net: &Net,
    c: &NetContext,
    url: &Url,
    tls: bool,
    key: &Key,
    until: Duration,
    span: Span,
) -> Result<Vec<u8>, NetError> {
    if expired(until) {
        return Err(NetError::Timeout);
    }
    let addrs = match key.host.literal() {
        Some(ip) => vec![SocketAddr::new(ip, key.port)],
        None => {
            let Host::Name(name) = &key.host else { unreachable!() };
            resolve::lookup(c, name, key.port, Some(until), span).map_err(|e| lookup_error(e, until))?
        }
    };
    let socket = dial(c, addrs, span, until)?;
    let tls = if tls {
        let name = match &key.host {
            Host::Name(n) => rustls::pki_types::ServerName::try_from(n.clone())
                .map_err(|_| invalid_data("TLS: host is not a DNS name"))?,
            Host::Ip(ip) => rustls::pki_types::ServerName::IpAddress((*ip).into()),
        };
        #[cfg(test)]
        let config = config(net.test_roots.as_deref());
        #[cfg(not(test))]
        let config = config(None);
        let _ = net;
        Some(rustls::ClientConnection::new(config, name).map_err(|e| invalid_data(format!("TLS: {e}")))?)
    } else {
        None
    };
    let mut conn = Conn {
        c,
        socket,
        tls,
        pending: Vec::new(),
        eof: false,
        until,
        span,
    };
    if conn.tls.is_some() {
        conn.handshake()?;
    }
    conn.send(request(url, conn.tls.is_some(), key).as_bytes())?;
    Response {
        conn: &mut conn,
        buf: Vec::new(),
        pos: 0,
    }
    .result()
    // The connection closes when `conn` drops, before `get` returns.
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunked_with_buffered_trailers(trailers: &[u8], rest: &[u8]) -> Result<Vec<u8>, NetError> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let socket = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        socket.set_nonblocking(true).unwrap();
        server.write_all(rest).unwrap();
        let c = Network::new(None).context();
        let mut conn = Conn {
            c: &c, socket, tls: None, pending: Vec::new(), eof: false,
            until: crate::time::monotonic() + Duration::from_secs(1),
            span: std::panic::Location::caller(),
        };
        let mut bytes = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n".to_vec();
        bytes.extend_from_slice(trailers);
        Response { conn: &mut conn, buf: bytes, pos: 0 }.result()
    }

    #[test]
    fn completed_trailers_still_obey_the_byte_limit() {
        // Prebuffer the complete block so the partial-read limit cannot hide the defect.
        let trailers = format!("X: {}\r\n\r\n", "x".repeat(HEAD_LIMIT));
        let result = chunked_with_buffered_trailers(trailers.as_bytes(), b"");
        assert!(matches!(result, Err(NetError::Io(e)) if e.kind() == ErrorKind::InvalidData));
    }

    #[test]
    fn partial_and_many_trailer_fields_remain_valid() {
        assert_eq!(chunked_with_buffered_trailers(b"X", b": \xff\r\n\r\n").unwrap(), b"");
        let trailers = format!("{}\r\n", "X: v\r\n".repeat(101));
        assert_eq!(chunked_with_buffered_trailers(trailers.as_bytes(), b"").unwrap(), b"");
    }
}

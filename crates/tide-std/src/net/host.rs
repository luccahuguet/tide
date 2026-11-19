// NET-HOST (03 §1.1, research/42 §4): the one key function for `only` entries, `connect`/`bind`
// addresses and `get` URLs. The v0 restrictions read the host as written, before the URL
// Standard's host parser (and so before its percent-decoding and IDNA); the key comes from
// that parser for special schemes, with IPv4-mapped IPv6 keyed as IPv4 and one trailing dot
// dropped from a domain.
use super::*;

/// A checked destination: equality of keys is the whole of `only`'s matching.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Key {
    pub(crate) host: Host,
    pub(crate) port: u16,
}
/// `pub` only because the sealed `Address` carries it; its module is private, so no program can
/// name it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Host {
    Name(String),
    Ip(IpAddr),
}
impl Host {
    /// IP hosts and `localhost` need no lookup (research/42 §3 step 5).
    pub(crate) fn literal(&self) -> Option<IpAddr> {
        match self {
            Self::Ip(ip) => Some(*ip),
            Self::Name(n) if n == "localhost" => Some(Ipv4Addr::LOCALHOST.into()),
            Self::Name(_) => None,
        }
    }
}
impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(n) => f.write_str(n),
            Self::Ip(IpAddr::V4(ip)) => write!(f, "{ip}"),
            Self::Ip(IpAddr::V6(ip)) => write!(f, "[{ip}]"),
        }
    }
}
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

pub(crate) fn ip(ip: IpAddr) -> Host {
    Host::Ip(match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(IpAddr::V6(v6), IpAddr::V4),
        v4 => v4,
    })
}

pub(crate) fn key_host(parsed: url::Host<impl AsRef<str>>) -> Host {
    match parsed {
        url::Host::Domain(domain) => {
            let domain = domain.as_ref();
            Host::Name(domain.strip_suffix('.').unwrap_or(domain).to_owned())
        }
        url::Host::Ipv4(v4) => Host::Ip(v4.into()),
        url::Host::Ipv6(v6) => ip(v6.into()),
    }
}

/// The v0 restrictions on a host as written, then its key host. `None` refuses it.
pub(crate) fn host(written: &str) -> Option<Host> {
    if written.is_empty() || !written.is_ascii() || written.contains(['%', '*']) {
        return None;
    }
    if let Some(inner) = written.strip_prefix('[') {
        // Brackets hold an IPv6 address and nothing else (no zone identifier).
        let v6: Ipv6Addr = inner.strip_suffix(']')?.parse().ok()?;
        return Some(ip(v6.into()));
    }
    let labels = written.strip_suffix('.').unwrap_or(written);
    if labels
        .split('.')
        .any(|l| l.is_empty() || l.get(..4).is_some_and(|p| p.eq_ignore_ascii_case("xn--")))
    {
        return None;
    }
    url::Host::parse(written).ok().map(key_host)
}

/// `resolve`'s host, keyed as `connect((host, port))` keys its host (research/48 §2): an IP
/// address as written, unbracketed or bracketed for IPv6, else the v0 host rule.
pub(crate) fn tuple(written: &str) -> Option<Host> {
    written.parse::<IpAddr>().map_or_else(|_| host(written), |a| Some(ip(a)))
}

/// An `only` entry or a `connect`/`bind` address written `"host:port"` (IPv6 in brackets).
pub(crate) fn split(address: &str) -> Option<(&str, u16)> {
    let (host, port) = address.rsplit_once(':')?;
    if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
        return None;
    }
    if port.is_empty() || !port.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((host, port.parse().ok()?))
}

/// An `only` entry's key; a port of 0 or an invalid entry grants nothing.
pub(crate) fn entry(entry: &str) -> Option<Key> {
    let (written, port) = split(entry)?;
    Some(Key { host: host(written)?, port }).filter(|k| k.port != 0)
}

/// The host as written in an `http` or `https` URL: the span the basic URL parser hands to
/// its host parser (research/42 §4, *Host as written*). `None` when `input` is not one.
pub(crate) fn url_host(input: &str) -> Option<String> {
    let trimmed = input.trim_matches(|c: char| c <= ' ');
    let text: String = trimmed.chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect();
    let (scheme, rest) = text.split_once(':')?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let rest = rest.trim_start_matches(['/', '\\']);
    let authority = &rest[..rest.find(['/', '\\', '?', '#']).unwrap_or(rest.len())];
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let mut inside = false;
    let end = host
        .char_indices()
        .find(|&(_, c)| {
            match c {
                '[' => inside = true,
                ']' => inside = false,
                ':' if !inside => return true,
                _ => {}
            }
            false
        })
        .map_or(host.len(), |(i, _)| i);
    Some(host[..end].to_owned())
}

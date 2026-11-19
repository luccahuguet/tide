// The simulated HTTP peer's request and reply (companion 17, research/42 §9). A peer models an
// application response, not TLS certificate validation or wire parsing.
use crate::net::Url;
use crate::time::Duration;

/// What a `sim.world.http` peer is asked (provisional): the request URL, fragment removed.
#[derive(Debug)]
pub struct Request {
    pub(crate) url: Url,
}
impl Request {
    pub fn url(&self) -> &Url {
        &self.url
    }
}

/// A simulated peer's reply. Every reply obeys `Net::get`'s 64 MiB body limit.
#[derive(Debug)]
pub struct Reply {
    pub(crate) answer: Answer,
    pub(crate) after: Duration,
}
#[derive(Debug)]
pub(crate) enum Answer {
    Ok(Vec<u8>),
    Status(u16),
    Hang,
    Reset,
    TlsError,
}
impl Reply {
    fn new(answer: Answer) -> Reply {
        Reply {
            answer,
            after: Duration::ZERO,
        }
    }
    /// A `200` with this body.
    pub fn ok(body: impl Into<Vec<u8>>) -> Reply {
        Reply::new(Answer::Ok(body.into()))
    }
    /// A final status, 200–599; a 2xx gives an empty body.
    ///
    /// # Panics
    ///
    /// Outside 200–599: no simulated peer sends an interim or `101` response.
    #[track_caller]
    pub fn status(code: u16) -> Reply {
        assert!(
            (200..=599).contains(&code),
            "Reply::status({code}): a simulated peer sends a final status, 200-599"
        );
        Reply::new(Answer::Status(code))
    }
    /// Never replies: the request ends at its deadline.
    pub fn hang() -> Reply {
        Reply::new(Answer::Hang)
    }
    /// The connection is reset: `NetError::Reset`.
    pub fn reset() -> Reply {
        Reply::new(Answer::Reset)
    }
    /// A failed handshake or certificate: `NetError::Io` with `InvalidData`.
    pub fn tls_error() -> Reply {
        Reply::new(Answer::TlsError)
    }
    /// Delays the reply by `d`, beyond the packet delays.
    pub fn after(self, d: Duration) -> Reply {
        Reply { after: d, ..self }
    }
}

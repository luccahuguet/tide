# hyper 0.14: idle connections are never closed without `runtime`

A deterministic reproduction, in plain Rust, of
[hyperium/hyper#2808](https://github.com/hyperium/hyper/issues/2808) against
hyper 0.14.32, the latest 0.14 release. It is a standalone Cargo package, not
a member of Tide's workspace, and Tide's tests do not run it. To run it, put
the two files below in a new directory and run:

```sh
cargo test                     # fails: the idle connection is never closed
cargo test --features runtime  # passes: the pool's interval closes it within 181 s
```

The client is the same in both builds. It is handed an executor (hyper
without `runtime` requires one) and a connector to an in-memory HTTP/1 server,
makes one request, and leaves the connection idle in its pool with a 90 s idle
timeout, hyper's default. The test then waits up to 181 s for the server to
see the connection close. It runs on tokio's paused clock
(`#[tokio::test(start_paused = true)]`), so the 181 s pass in virtual time
and both outcomes repeat exactly. The executor in the test is tokio's own, so
that the paused clock drives everything; the failure does not depend on the
executor, only on hyper's `runtime` feature, which gates the pool's idle
interval.

`Cargo.toml`:

```toml
[package]
name = "hyper-idle-reaping"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
hyper = { version = "=0.14.32", default-features = false, features = ["client", "server", "http1"] }
tokio = { version = "1", features = ["rt", "macros", "time", "test-util", "io-util", "sync"] }

[features]
# Turns on hyper's `runtime` feature, and with it the pool's idle interval.
runtime = ["hyper/runtime"]

[workspace]
```

`src/lib.rs`:

```rust
#![cfg(test)]
//! hyper 0.14's client pool closes an idle connection only when hyper's `runtime` feature is
//! on. The client below is the same in both builds: it is handed an executor and a connector,
//! and runs on tokio's paused clock, so virtual time jumps ahead whenever every task waits.
//!
//!     cargo test                     # fails: the idle connection is never closed
//!     cargo test --features runtime  # passes: the pool's interval closes it within 181 s
use hyper::client::connect::{Connected, Connection};
use hyper::server::conn::Http;
use hyper::service::{service_fn, Service};
use hyper::{Body, Client, Response, Uri};
use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{duplex, AsyncRead, AsyncWrite, DuplexStream, ReadBuf};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

/// The caller's executor. hyper without `runtime` needs one; this one happens to be tokio's.
#[derive(Clone)]
struct Spawn;
impl<F: Future<Output = ()> + Send + 'static> hyper::rt::Executor<F> for Spawn {
    fn execute(&self, fut: F) {
        tokio::spawn(fut);
    }
}

/// Connects each connection to its own in-memory HTTP/1 server, which reports on `closed` when
/// the client closes it.
#[derive(Clone)]
struct Connector {
    closed: UnboundedSender<()>,
}

impl Service<Uri> for Connector {
    type Response = Conn;
    type Error = Infallible;
    type Future = Ready<Result<Conn, Infallible>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Uri) -> Self::Future {
        let (client, server) = duplex(64 * 1024);
        let closed = self.closed.clone();
        tokio::spawn(async move {
            let ok = service_fn(|_| async { Ok::<_, Infallible>(Response::new(Body::from("ok"))) });
            let _ = Http::new().serve_connection(server, ok).await;
            let _ = closed.send(());
        });
        ready(Ok(Conn(client)))
    }
}

struct Conn(DuplexStream);

impl Connection for Conn {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl AsyncRead for Conn {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for Conn {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

#[tokio::test(start_paused = true)]
async fn an_idle_connection_is_closed_after_the_timeout() {
    let idle_timeout = Duration::from_secs(90); // hyper's default
    let (closed, mut closes) = unbounded_channel();
    let client: Client<Connector, Body> = Client::builder()
        .executor(Spawn)
        .pool_idle_timeout(idle_timeout)
        .build(Connector { closed });
    let response = client.get(Uri::from_static("http://pool.test/")).await.unwrap();
    hyper::body::to_bytes(response.into_body()).await.unwrap();
    // The connection now sits idle in the pool, and the client is still alive.
    let waited = 2 * idle_timeout + Duration::from_secs(1);
    let close = tokio::time::timeout(waited, closes.recv()).await;
    assert!(
        close.is_ok(),
        "the idle connection was still open {waited:?} after its request"
    );
    drop(client);
}
```

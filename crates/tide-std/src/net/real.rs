use super::*;
use std::os::fd::AsFd;

pub(super) fn bind(c: &NetContext, addr: SocketAddr) -> io::Result<std::net::TcpListener> {
    c.check()?;
    let l = std::net::TcpListener::bind(addr).map_err(io::from_std)?;
    l.set_nonblocking(true).map_err(io::from_std)?;
    Ok(l)
}

pub(super) fn connect(
    c: &NetContext,
    addr: SocketAddr,
    span: Span,
) -> io::Result<std::net::TcpStream> {
    use rustix::net::{AddressFamily, SocketFlags, SocketType, socket_with};
    c.check()?;
    let socket = socket_with(
        if addr.is_ipv4() {
            AddressFamily::INET
        } else {
            AddressFamily::INET6
        },
        SocketType::STREAM,
        SocketFlags::NONBLOCK | SocketFlags::CLOEXEC,
        None,
    )
    .map_err(|e| io::from_std(e.into()))?;
    match rustix::net::connect(&socket, &addr) {
        Ok(()) => {}
        Err(rustix::io::Errno::INPROGRESS) => {
            c.socket(socket.as_fd(), true, span)?;
            rustix::net::sockopt::socket_error(&socket)
                .and_then(|r| r)
                .map_err(|e| io::from_std(e.into()))?;
        }
        Err(e) => return Err(io::from_std(e.into())),
    }
    Ok(socket.into())
}

pub(super) fn retry<T>(
    c: &NetContext,
    socket: &impl AsFd,
    write: bool,
    span: Span,
    mut f: impl FnMut() -> std::io::Result<T>,
) -> io::Result<T> {
    loop {
        c.check()?;
        match f() {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                c.socket(socket.as_fd(), write, span)?
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            result => return result.map_err(io::from_std),
        }
    }
}

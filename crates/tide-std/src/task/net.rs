// Network adapter to the runtime's cancellation, timers, scheduler and per-thread poll waker.
use super::*;
use rustix::event::{EventfdFlags, PollFd, PollFlags, eventfd};
use std::os::fd::{BorrowedFd, OwnedFd};

thread_local! {
    static POLL_EVENT: RefCell<Option<Arc<OwnedFd>>> = const { RefCell::new(None) };
}
fn poll_event() -> crate::io::Result<Arc<OwnedFd>> {
    POLL_EVENT.with(|slot| {
        let mut cached = slot.borrow_mut();
        if let Some(event) = &*cached {
            return Ok(event.clone());
        }
        let event = Arc::new(
            eventfd(0, EventfdFlags::NONBLOCK | EventfdFlags::CLOEXEC)
                .map_err(|e| crate::io::from_std(e.into()))?,
        );
        *cached = Some(event.clone());
        Ok(event)
    })
}
pub(super) fn poll(
    fd: BorrowedFd<'_>,
    flags: PollFlags,
    event: &OwnedFd,
    left: Option<Duration>,
) -> crate::io::Result<bool> {
    let mut fds = [PollFd::new(&fd, flags), PollFd::new(event, PollFlags::IN)];
    let timeout = left.map(|d| {
        rustix::event::Timespec::try_from(d.min(Duration::from_secs(i32::MAX as u64))).unwrap()
    });
    match rustix::event::poll(&mut fds, timeout.as_ref()) {
        Ok(_) => {
            let mut bytes = [0; 8];
            let _ = rustix::io::read(event, &mut bytes);
            Ok(!fds[0].revents().is_empty())
        }
        Err(rustix::io::Errno::INTR) => Ok(false),
        Err(e) => Err(crate::io::from_std(e.into())),
    }
}

#[derive(Clone)]
pub(crate) struct Network(Option<Tasks>);
pub(crate) struct NetContext(Context);
impl Network {
    pub(crate) fn new(tasks: Option<Tasks>) -> Self {
        Self(tasks)
    }
    pub(crate) fn context(&self) -> NetContext {
        NetContext(match &self.0 {
            Some(tasks) => tasks.context().unwrap(),
            None => Context::of(&None).unwrap_or_else(Context::detached),
        })
    }
    // Ended/foreign simulations cannot be touched; live unwinding still closes resources.
    pub(crate) fn closing(&self) -> Option<NetContext> {
        let tasks = self.0.as_ref()?;
        let context = Context::current()?;
        if !Arc::ptr_eq(&context.runtime, tasks.simulation.as_ref().unwrap())
            || context.lock().simulation_ended
        {
            return None;
        }
        Some(NetContext(context))
    }
}
impl NetContext {
    pub(crate) fn check(&self) -> crate::io::Result<()> {
        let mut runtime = self.0.lock();
        if self.0.at.is_some_and(|at| runtime.observe(at)) {
            Err(crate::io::ErrorKind::Cancelled.into())
        } else {
            Ok(())
        }
    }
    pub(crate) fn begin(&self, name: &'static str, id: usize, span: Span) -> crate::io::Result<()> {
        self.check()?;
        if self.0.task.is_some() {
            self.0.point(Operation::Net(name, id), span);
        }
        self.check()
    }
    pub(crate) fn now(&self) -> Duration {
        self.0.lock().baton.as_ref().unwrap().scheduler.now
    }
    pub(crate) fn draw(&self, bound: u64) -> u64 {
        self.0
            .lock()
            .baton
            .as_mut()
            .unwrap()
            .scheduler
            .net_draw(bound)
    }
    pub(crate) fn latency(&self) -> Duration {
        Duration::from_nanos(1_000_000 + self.draw(9_000_001))
    }
    pub(crate) fn wait(
        &self,
        name: &'static str,
        id: usize,
        span: Span,
        until: Option<Duration>,
        signal: Option<Arc<Signal>>,
    ) -> crate::io::Result<()> {
        match self.0.wait_for(
            Operation::Net(name, id),
            span,
            until,
            Cancellation::Checkpoint,
            signal,
        ) {
            Wake::Cancelled => Err(crate::io::ErrorKind::Cancelled.into()),
            _ => Ok(()),
        }
    }
    pub(crate) fn socket(
        &self,
        fd: BorrowedFd<'_>,
        write: bool,
        span: Span,
    ) -> crate::io::Result<()> {
        self.check()?;
        // Allocate before the runtime lock or waiter registration; failure leaves both intact.
        let event = poll_event()?;
        match self.0.wait_with_socket(
            Operation::Net("net.poll", 0),
            span,
            None,
            Cancellation::Checkpoint,
            None,
            Some((
                fd,
                if write { PollFlags::OUT } else { PollFlags::IN },
                event,
            )),
        )? {
            Wake::Cancelled => Err(crate::io::ErrorKind::Cancelled.into()),
            _ => Ok(()),
        }
    }
    pub(crate) fn wake(&self) {
        self.0.lock().wake_waiters();
    }
    pub(crate) fn record(&self, name: &'static str, id: usize, span: Span, detail: &str) {
        if thread::panicking() {
            return;
        }
        if let Some(task) = self.0.task {
            self.0
                .lock()
                .baton
                .as_mut()
                .unwrap()
                .scheduler
                .net_event(task, name, id, span, detail);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn socket_wait_failures_preserve_runtime_and_can_retry() {
        const CHILD: &str = "TIDE_TCP_EVENTFD_LIMIT_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new("timeout")
                .arg("15")
                .arg(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "task::net::tests::socket_wait_failures_preserve_runtime_and_can_retry",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CHILD, "1")
                .stdin(std::process::Stdio::null())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                String::from_utf8_lossy(&output.stdout).contains("TCP_EVENTFD_LIMIT_CHILD_RAN")
            );
            return;
        }
        #[repr(C)]
        struct Limit {
            current: std::ffi::c_ulong,
            maximum: std::ffi::c_ulong,
        }
        unsafe extern "C" {
            fn getrlimit(resource: std::ffi::c_int, limit: *mut Limit) -> std::ffi::c_int;
            fn setrlimit(resource: std::ffi::c_int, limit: *const Limit) -> std::ffi::c_int;
        }
        const RLIMIT_NOFILE: std::ffi::c_int = 7; // Linux resource ABI.
        let tasks = Tasks::real();
        let net = crate::net::Net::new(None);
        let listener = net.bind("127.0.0.1:0").unwrap();
        let socket = net.connect(listener.local_addr().unwrap()).unwrap();
        let (_peer, _) = listener.accept().unwrap();
        // A fresh thread has no cached eventfd; recv needs no descriptor allocation itself.
        let worker = tasks.clone();
        tasks
            .spawn_isolated(move || {
                use crate::io::Read;
                let mut original = Limit {
                    current: 0,
                    maximum: 0,
                };
                // Only this fresh subprocess lowers its soft limit; existing sockets stay valid.
                unsafe {
                    assert_eq!(getrlimit(RLIMIT_NOFILE, &mut original), 0);
                    assert_eq!(
                        setrlimit(
                            RLIMIT_NOFILE,
                            &Limit {
                                current: 0,
                                maximum: original.maximum
                            }
                        ),
                        0
                    );
                }
                worker
                    .scope::<_, TaskError, _>(|_| {
                        assert_eq!(
                            (&socket).read(&mut [0; 1]).unwrap_err().raw_os_error(),
                            Some(24)
                        ); // EMFILE
                        assert!(
                            Context::current()
                                .unwrap()
                                .lock()
                                .waiting
                                .iter()
                                .all(|w| w.thread.id() != thread::current().id())
                        );
                        worker.checkpoint().unwrap();
                        Ok(())
                    })
                    .unwrap();
                unsafe {
                    assert_eq!(setrlimit(RLIMIT_NOFILE, &original), 0);
                }
                let _cached = poll_event().unwrap();
                // poll's two fds exceed this child-only limit even though both already exist.
                unsafe {
                    assert_eq!(
                        setrlimit(
                            RLIMIT_NOFILE,
                            &Limit {
                                current: 1,
                                maximum: original.maximum
                            }
                        ),
                        0
                    );
                }
                worker
                    .scope::<_, TaskError, _>(|_| {
                        assert_eq!(
                            (&socket).read(&mut [0; 1]).unwrap_err().raw_os_error(),
                            Some(22)
                        ); // EINVAL
                        assert!(
                            Context::current()
                                .unwrap()
                                .lock()
                                .waiting
                                .iter()
                                .all(|w| w.thread.id() != thread::current().id())
                        );
                        worker.checkpoint().unwrap();
                        Ok(())
                    })
                    .unwrap();
                unsafe {
                    assert_eq!(setrlimit(RLIMIT_NOFILE, &original), 0);
                }
                worker
                    .scope::<_, TaskError, _>(|s| {
                        let clock = crate::time::Clock::new(Time::real());
                        s.set_deadline(clock.now() + Duration::from_millis(2));
                        assert!((&socket).read(&mut [0; 1]).unwrap_err().is_cancelled());
                        assert!(
                            Context::current()
                                .unwrap()
                                .lock()
                                .waiting
                                .iter()
                                .all(|w| w.thread.id() != thread::current().id())
                        );
                        Ok(())
                    })
                    .unwrap();
                worker.checkpoint().unwrap();
            })
            .unwrap();
        println!("TCP_EVENTFD_LIMIT_CHILD_RAN");
    }
}

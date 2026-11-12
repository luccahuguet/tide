// Shared verbatim with the conformance fixture.
use crate::task::{Cancelled, Time};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::sync::OnceLock;
pub use std::time::Duration;

/// Authority to read time and sleep (03 §1.1). A simulated clock reads virtual time, which
/// starts at zero and advances only when every task is blocked (03 §9.3).
#[derive(Clone)]
pub struct Clock {
    time: Time,
    wall: bool,
}

impl Clock {
    pub(crate) fn new(time: Time) -> Self {
        Self { time, wall: true }
    }

    /// The current instant on the monotonic clock.
    #[track_caller]
    pub fn now(&self) -> Instant {
        let span = std::panic::Location::caller();
        Instant(self.time.read(false, span).unwrap_or_else(monotonic))
    }

    /// The current wall-clock time. A real clock set before 1970 reads as the Unix epoch.
    ///
    /// # Panics
    ///
    /// On a `Clock` from `monotonic_only()`, which holds no authority to read wall time.
    /// Also when virtual elapsed time exceeds `Duration::MAX - Duration::from_secs(946_684_800)`:
    /// adding the simulated wall epoch (2000-01-01) would exceed the unsigned offset range.
    #[track_caller]
    pub fn system_time(&self) -> SystemTime {
        assert!(
            self.wall,
            "`system_time()` called on a `Clock` from `monotonic_only()`, which cannot read wall \
             time"
        );
        let span = std::panic::Location::caller();
        SystemTime(self.time.read(true, span).unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
        }))
    }

    /// Sleeps for at least `d`: `sleep_until(now + d)`, where a sum past the last instant never
    /// arrives, so the sleep ends only by cancellation.
    #[track_caller]
    pub fn sleep(&self, d: Duration) -> Result<(), Cancelled> {
        let span = std::panic::Location::caller();
        self.time
            .sleep("clock.sleep", |now| now.checked_add(d), span)
    }

    /// Sleeps until `t`, an instant of this clock. It is a checkpoint: it returns `Cancelled`
    /// at once in a cancelled scope and when its scope is cancelled while it sleeps. A `t` that
    /// is not after now checks for cancellation and returns without blocking.
    #[track_caller]
    pub fn sleep_until(&self, t: Instant) -> Result<(), Cancelled> {
        let span = std::panic::Location::caller();
        self.time.sleep("clock.sleep_until", |_| Some(t.0), span)
    }

    /// The same clock without the authority to read wall time: `system_time()` panics on it.
    pub fn monotonic_only(&self) -> Clock {
        Clock {
            time: self.time.clone(),
            wall: false,
        }
    }
}

/// The real monotonic clock, as the offset from a process anchor set at its first use.
pub(crate) fn monotonic() -> Duration {
    static ANCHOR: OnceLock<std::time::Instant> = OnceLock::new();
    ANCHOR.get_or_init(std::time::Instant::now).elapsed()
}

/// A point on a `Clock`'s monotonic time: Rust's `Instant` without `now()` and `elapsed()`,
/// which read the clock (03 CAP-1). Only `Clock::now` makes one. It is the offset from its
/// clock's origin (the start of a simulation, or a point at or before the first reading of the
/// real clock), so, as in Rust, arithmetic that leaves the representable range panics and the
/// `checked_*` methods return `None`: an instant before the origin is not representable.
/// Instants of different clocks compare as plain offsets; nothing detects the mix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(pub(crate) Duration);

impl Instant {
    /// The time from `earlier` to `self`, or zero when `earlier` is later.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.saturating_sub(earlier.0)
    }

    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_sub(earlier.0)
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.duration_since(earlier)
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_add(duration).map(Instant)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_sub(duration).map(Instant)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, other: Duration) -> Instant {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

/// A point in wall-clock time: Rust's `SystemTime` without `now()` and `elapsed()` (03 CAP-1).
/// Only `Clock::system_time` and arithmetic from `UNIX_EPOCH` make one. It is the offset from
/// the Unix epoch, so a time before 1970 is not representable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(Duration);

/// 1970-01-01T00:00:00Z.
pub const UNIX_EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::ZERO);

    /// The time from `earlier` to `self`. Wall time is not monotonic: when `earlier` is later,
    /// the error holds how much later.
    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
        self.0
            .checked_sub(earlier.0)
            .ok_or_else(|| SystemTimeError(earlier.0 - self.0))
    }

    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        self.0.checked_add(duration).map(SystemTime)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        self.0.checked_sub(duration).map(SystemTime)
    }
}

impl Add<Duration> for SystemTime {
    type Output = SystemTime;
    fn add(self, other: Duration) -> SystemTime {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for SystemTime {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Sub<Duration> for SystemTime {
    type Output = SystemTime;
    fn sub(self, other: Duration) -> SystemTime {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for SystemTime {
    fn sub_assign(&mut self, other: Duration) {
        *self = *self - other;
    }
}

/// `SystemTime::duration_since` was given a later time.
#[derive(Clone, Debug)]
pub struct SystemTimeError(Duration);

impl SystemTimeError {
    /// How much later the other time was.
    pub fn duration(&self) -> Duration {
        self.0
    }
}

impl fmt::Display for SystemTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("second time provided was later than self")
    }
}

impl std::error::Error for SystemTimeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{TaskError, Tasks};
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn clock() -> Clock {
        Clock::new(Time::real())
    }

    #[test]
    fn instants_and_system_times_follow_rusts_arithmetic() {
        let clock = clock();
        let t = clock.now();
        let d = Duration::from_millis(1500);
        assert_eq!((t + d) - t, d);
        assert_eq!((t + d) - d, t);
        assert_eq!((t + d).duration_since(t), d);
        assert_eq!((t + d).checked_duration_since(t), Some(d));
        // A later `earlier` saturates, as in Rust since 1.60.
        assert_eq!(t.duration_since(t + d), Duration::ZERO);
        assert_eq!(t - (t + d), Duration::ZERO);
        assert_eq!(t.saturating_duration_since(t + d), Duration::ZERO);
        assert_eq!(t.checked_duration_since(t + d), None);
        assert_eq!(t.checked_add(Duration::MAX), None);
        assert_eq!((t + d).checked_sub(d), Some(t));
        assert_eq!(t.checked_sub(Duration::MAX), None);
        let mut u = t;
        u += d;
        u -= d;
        assert_eq!(u, t);
        assert!(t < t + d && clock.now() >= t);
        assert!(catch_unwind(|| t + Duration::MAX).is_err());
        assert!(catch_unwind(|| t - Duration::MAX).is_err());

        let wall = clock.system_time();
        let since = wall.duration_since(UNIX_EPOCH).unwrap();
        assert!(since > Duration::from_secs(946_684_800));
        assert_eq!(UNIX_EPOCH, SystemTime::UNIX_EPOCH);
        assert_eq!(UNIX_EPOCH + since, wall);
        assert_eq!(wall - since, UNIX_EPOCH);
        let error = UNIX_EPOCH.duration_since(wall).unwrap_err();
        assert_eq!(error.duration(), since);
        assert_eq!(
            error.to_string(),
            "second time provided was later than self"
        );
        assert_eq!(wall.checked_add(Duration::MAX), None);
        assert_eq!(UNIX_EPOCH.checked_sub(d), None);
        assert_eq!((wall + d).checked_sub(d), Some(wall));
        let mut later = wall;
        later += d;
        assert_eq!(later.duration_since(wall).unwrap(), d);
        later -= d;
        assert_eq!(later, wall);
        assert!(catch_unwind(|| UNIX_EPOCH - Duration::from_nanos(1)).is_err());
    }

    #[test]
    fn real_sleeps_take_at_least_their_duration() {
        let clock = clock();
        let d = Duration::from_millis(20);
        let (real, start) = (std::time::Instant::now(), clock.now());
        clock.sleep(d).unwrap();
        let woke = clock.now();
        assert!(real.elapsed() >= d && woke - start >= d);
        clock.sleep_until(woke + d).unwrap();
        assert!(real.elapsed() >= 2 * d && clock.now() >= woke + d);
        // Zero and past targets return without sleeping.
        clock.sleep(Duration::ZERO).unwrap();
        clock.sleep_until(start).unwrap();
    }

    #[test]
    fn cancellation_ends_a_real_sleep() {
        let (clock, tasks) = (clock(), Tasks::real());
        let clock = &clock;
        let real = std::time::Instant::now();
        let long = Duration::from_secs(30);
        // A sibling's failure unparks a sleeper; so does an explicit cancel.
        for explicit in [false, true] {
            let result: Result<(), TaskError> = tasks.scope(|scope| {
                let sleeper = scope.spawn(|| clock.sleep(long).map_err(|_| TaskError::Cancelled));
                let until = clock.now() + long;
                scope.spawn(move || clock.sleep_until(until).map_err(|_| TaskError::Cancelled));
                std::thread::sleep(Duration::from_millis(30));
                if explicit {
                    scope.cancel();
                } else {
                    scope.spawn(|| Err::<(), _>(TaskError::DeadlineExceeded));
                }
                assert_eq!(sleeper.join(), Err(TaskError::Cancelled));
                // In a cancelled scope every sleep fails at once, whatever its target.
                assert_eq!(clock.sleep(long), Err(Cancelled));
                assert_eq!(clock.sleep(Duration::MAX), Err(Cancelled));
                assert_eq!(clock.sleep(Duration::ZERO), Err(Cancelled));
                assert_eq!(clock.sleep_until(clock.now()), Err(Cancelled));
                Ok(())
            });
            let expected = if explicit {
                TaskError::Cancelled
            } else {
                TaskError::DeadlineExceeded
            };
            assert_eq!(result, Err(expected));
        }
        assert!(real.elapsed() < long);
        clock.sleep(Duration::ZERO).unwrap();
    }

    #[test]
    fn monotonic_only_keeps_time_and_sleep_and_panics_on_wall_time() {
        let clock = clock().monotonic_only().clone();
        let t = clock.now();
        clock.sleep(Duration::from_millis(1)).unwrap();
        assert!(clock.monotonic_only().now() > t);
        let panic = catch_unwind(AssertUnwindSafe(|| clock.system_time())).unwrap_err();
        assert_eq!(
            panic.downcast_ref::<&str>().copied(),
            Some(
                "`system_time()` called on a `Clock` from `monotonic_only()`, which cannot read \
                 wall time"
            )
        );
    }
}

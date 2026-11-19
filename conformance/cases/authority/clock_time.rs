//@ rules: CAP-1
// Shared `std::time` signatures; the driver also runs these assertions against production std.
use std::task::{Cancelled, TaskError, Tasks};
use std::time::{Clock, Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

fn arithmetic(clock: &Clock) {
    let t: Instant = clock.now();
    let d = Duration::from_millis(1500);
    assert_eq!((t + d) - t, d);
    assert_eq!((t + d) - d, t);
    assert_eq!((t + d).duration_since(t), d);
    assert_eq!(t.duration_since(t + d), Duration::ZERO);
    assert_eq!(t.saturating_duration_since(t + d), Duration::ZERO);
    assert_eq!(t.checked_duration_since(t + d), None);
    assert_eq!(t.checked_add(Duration::MAX), None);
    assert_eq!((t + d).checked_sub(d), Some(t));
    let mut u = t;
    u += d;
    u -= d;
    assert!(u == t && t < t + d && clock.now() >= t);

    let wall: SystemTime = clock.system_time();
    let since = wall.duration_since(UNIX_EPOCH).unwrap();
    assert!(since > Duration::from_secs(946_684_800));
    assert_eq!(SystemTime::UNIX_EPOCH + since, wall);
    assert_eq!(wall - since, UNIX_EPOCH);
    let error: SystemTimeError = UNIX_EPOCH.duration_since(wall).unwrap_err();
    assert_eq!(error.duration(), since);
    assert_eq!(wall.checked_add(Duration::MAX), None);
    assert_eq!(UNIX_EPOCH.checked_sub(d), None);
}

fn sleeps(clock: &Clock, tasks: &Tasks) -> Result<(), Cancelled> {
    let d = Duration::from_millis(20);
    let start = clock.now();
    clock.sleep(d)?;
    let woke = clock.now();
    assert!(woke - start >= d);
    clock.sleep_until(woke + d)?;
    assert!(clock.now() >= woke + d);
    clock.sleep(Duration::ZERO)?;
    clock.sleep_until(start)?;

    let long = Duration::from_secs(600);
    let failed: Result<(), TaskError> = tasks.scope(|s| {
        // The sibling's failure cancels the scope and ends this sleep.
        let sleeper = s.spawn(|| Ok(clock.sleep(long)?));
        s.spawn(|| Err::<(), _>(TaskError::DeadlineExceeded));
        assert_eq!(sleeper.join(), Err(TaskError::Cancelled));
        assert_eq!(clock.sleep(long), Err(Cancelled));
        assert_eq!(clock.sleep(Duration::MAX), Err(Cancelled));
        assert_eq!(clock.sleep_until(clock.now() + long), Err(Cancelled));
        Ok(())
    });
    assert_eq!(failed, Err(TaskError::DeadlineExceeded));
    assert!(clock.now() - start < long);

    let monotonic = clock.monotonic_only();
    let before = monotonic.now();
    monotonic.sleep(Duration::from_millis(1))?;
    assert!(monotonic.now() > before);
    Ok(())
}

#[tide::main]
fn main(root: Root) -> Result<(), Cancelled> {
    arithmetic(&root.clock);
    sleeps(&root.clock, &root.tasks)?;
    println!("TIME_RAN");
    Ok(())
}

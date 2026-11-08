use std::time::{Duration, Instant};
#[test]
fn case1() {
    let start = Instant::now();
    assert_eq!(
        run_batch(
            &[10, 500, 30].map(Duration::from_millis),
            Duration::from_millis(100)
        ),
        vec![Outcome::Done, Outcome::TimedOut, Outcome::Done]
    );
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(100) && elapsed < Duration::from_millis(300));
}
#[test]
fn case2() {
    let start = Instant::now();
    assert_eq!(
        run_batch(&[5, 6].map(Duration::from_millis), Duration::from_secs(1)),
        vec![Outcome::Done, Outcome::Done]
    );
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(6) && elapsed < Duration::from_millis(500));
}
#[test]
fn case3() {
    let start = Instant::now();
    assert_eq!(run_batch(&[], Duration::from_secs(1)), vec![]);
    assert!(start.elapsed() < Duration::from_millis(100));
}

use std::time::Duration;
#[test]
fn case1() {
    let mut answers = [true, true, false, true, false, false].into_iter();
    let r = watch(Duration::from_millis(100), 2, || answers.next().unwrap());
    assert_eq!((r.calls, r.last_ok), (6, Some(3)));
    assert!(r.elapsed >= Duration::from_millis(500) && r.elapsed < Duration::from_millis(650));
}
#[test]
fn case2() {
    let mut first = true;
    let r = watch(Duration::from_millis(100), 1, || {
        if first {
            first = false;
            std::thread::sleep(Duration::from_millis(250));
            true
        } else {
            false
        }
    });
    assert_eq!((r.calls, r.last_ok), (2, Some(0)));
    assert!(r.elapsed >= Duration::from_millis(300) && r.elapsed < Duration::from_millis(450));
}
#[test]
fn case3() {
    let r = watch(Duration::from_millis(100), 1, || false);
    assert_eq!((r.calls, r.last_ok), (1, None));
    assert!(r.elapsed < Duration::from_millis(150));
}
#[test]
#[should_panic]
fn case4() {
    watch(Duration::from_millis(100), 0, || false);
}

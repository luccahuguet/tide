use std::thread;
#[derive(Debug, PartialEq)]
pub struct TooLong;
#[derive(Debug, PartialEq)]
pub struct Summary {
    pub ok: u64,
    pub done: u32,
    pub too_long: u32,
    pub panicked: u32,
}
pub fn steps(mut n: u64) -> Result<u32, TooLong> {
    assert!(n > 0);
    let mut count = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        count += 1;
        if count > 200 {
            return Err(TooLong);
        }
    }
    Ok(count)
}
pub fn run_jobs(inputs: &[u64], workers: usize) -> Summary {
    assert!(workers > 0);
    let mut out = Summary {
        ok: 0,
        done: 0,
        too_long: 0,
        panicked: 0,
    };
    for batch in inputs.chunks(workers) {
        thread::scope(|s| {
            let hs: Vec<_> = batch.iter().map(|&n| s.spawn(move || steps(n))).collect();
            for h in hs {
                match h.join() {
                    Ok(Ok(n)) => {
                        out.ok += u64::from(n);
                        out.done += 1;
                    }
                    Ok(Err(TooLong)) => out.too_long += 1,
                    Err(_) => out.panicked += 1,
                }
            }
        });
    }
    out
}

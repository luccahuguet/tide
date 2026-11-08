use std::thread;
pub fn histogram(values: &[u32], edges: &[u32]) -> Vec<u64> {
    assert!(edges.len() >= 2 && edges.windows(2).all(|w| w[0] < w[1]));
    thread::scope(|s| {
        let jobs: Vec<_> = values
            .chunks(4096)
            .map(|chunk| {
                s.spawn(move || {
                    let mut buckets = vec![0; edges.len() - 1];
                    for v in chunk {
                        let i = edges.partition_point(|e| e <= v);
                        if i > 0 && i < edges.len() {
                            buckets[i - 1] += 1;
                        }
                    }
                    buckets
                })
            })
            .collect();
        let mut result = vec![0; edges.len() - 1];
        for job in jobs {
            for (out, n) in result.iter_mut().zip(job.join().unwrap()) {
                *out += n;
            }
        }
        result
    })
}

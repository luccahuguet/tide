pub fn partition(v: &mut [i32], pivot: i32) -> usize {
    let mut low = 0;
    for i in 0..v.len() {
        if v[i] < pivot {
            v.swap(low, i);
            low += 1;
        }
    }
    low
}
pub fn select_kth(mut v: &mut [i32], mut k: usize) -> i32 {
    assert!(k < v.len());
    loop {
        let pivot = v[v.len() / 2];
        let low = partition(v, pivot);
        let equal = v[low..].iter().filter(|&&n| n == pivot).count();
        // Group equal pivots too, so duplicate-heavy inputs always make progress.
        let mut end = low;
        for i in low..v.len() {
            if v[i] == pivot {
                v.swap(end, i);
                end += 1;
            }
        }
        if k < low {
            v = &mut v[..low];
        } else if k < low + equal {
            return pivot;
        } else {
            v = &mut v[end..];
            k -= end;
        }
    }
}

use std::par::join;
fn merge<T, K: Ord, F: Fn(&T) -> K>(v: &mut [T], mid: usize, key: &F) {
    let len = v.len();
    if mid == 0 || mid == len || key(&v[mid - 1]) <= key(&v[mid]) {
        return;
    }
    if len == 2 {
        v.swap(0, 1);
        return;
    }
    let (a, b) = if mid > len - mid {
        let a = mid / 2;
        let pivot = key(&v[a]);
        let b = mid + v[mid..].partition_point(|x| key(x) < pivot);
        (a, b)
    } else {
        let b = mid + (len - mid) / 2;
        let pivot = key(&v[b]);
        let a = v[..mid].partition_point(|x| key(x) <= pivot);
        (a, b)
    };
    v[a..b].rotate_left(mid - a);
    let cut = a + b - mid;
    let (left, right) = v.split_at_mut(cut);
    merge(left, a, key);
    merge(right, mid - a, key);
}
fn sort<T: Send + Pure, K: Ord, F: Fn(&T) -> K + Sync + Pure>(v: &mut [T], key: &F) {
    if v.len() <= 2048 {
        v.sort_by_key(key);
        return;
    }
    let mid = v.len() / 2;
    {
        let (a, b) = v.split_at_mut(mid);
        join(|| sort(a, key), || sort(b, key));
    }
    merge(v, mid, key);
}
pub fn sort_parallel(v: &mut [i64]) {
    sort(v, &|x| *x);
}
pub fn sort_by_key_parallel<T: Send + Pure, K: Ord, F: Fn(&T) -> K + Sync + Pure>(
    v: &mut [T],
    key: F,
) {
    sort(v, &key);
}

//! The rerun tool's statistics, its round order, and its record encoding (fx-6u8m.6), apart
//! from anything timed, so that `tests/benchmarks.rs` tests them in the
//! workspace suite.
use std::ops::Range;

/// A series is stable when its coefficient of variation is at most this (1%).
pub const CV_LIMIT: f64 = 0.01;

/// The median, or `None` for no values.
pub fn median(values: &[f64]) -> Option<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(sorted[n / 2]),
        _ => Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0),
    }
}

/// The coefficient of variation: the sample standard deviation (n - 1) over the mean. `None`
/// for fewer than two values or a mean that is not positive.
pub fn cv(values: &[f64]) -> Option<f64> {
    let n = values.len();
    if n < 2 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    if mean <= 0.0 {
        return None;
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    Some(variance.sqrt() / mean)
}

/// Whether `values` vary by at most `CV_LIMIT`.
pub fn stable(values: &[f64]) -> bool {
    cv(values).is_some_and(|cv| cv <= CV_LIMIT)
}

/// How many more rounds a group of series runs after `done` rounds: none once every series is
/// stable or `done` has reached `cap`, else `step`, cut so the total never passes `cap`.
pub fn more_rounds(series: &[Vec<f64>], done: usize, step: usize, cap: usize) -> usize {
    if done >= cap || series.iter().all(|s| stable(s)) {
        0
    } else {
        step.min(cap - done)
    }
}

/// The interleaved order of `rounds`: each round measures every arm once, in arm order, so two
/// arms run A B A B and none runs all its rounds in one stretch. Pairs are (round, arm).
pub fn interleaved(arms: usize, rounds: Range<usize>) -> Vec<(usize, usize)> {
    rounds
        .flat_map(|round| (0..arms).map(move |arm| (round, arm)))
        .collect()
}

/// `text` as a JSON string literal.
pub fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

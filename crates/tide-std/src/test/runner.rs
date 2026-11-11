use std::any::Any;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::process::{ExitCode, Termination};
use std::sync::Once;

use super::{Choice, Gen, Kind, mix};

/// Classify the success value after the macro has handled every resolved `Result` layer.
#[doc(hidden)]
pub fn property_result<R: Termination>(value: R) -> Result<(), String> {
    if value.report() == ExitCode::SUCCESS {
        Ok(())
    } else {
        Err("property returned a failing exit status".into())
    }
}

const CASES: u64 = 100;
const RUN_SEED: u64 = 0x6a09e667f3bcc909;
const SHRINK_BUDGET: usize = 1000;
const STALLED_BUDGET: usize = 128;

// The hook delegates on every thread except the one currently evaluating a case.
thread_local! {
    static CAPTURE: RefCell<Option<Option<String>>> = const { RefCell::new(None) };
}

fn install_hook() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let captured = CAPTURE.try_with(|capture| {
                let mut capture = capture.borrow_mut();
                if let Some(location) = capture.as_mut() {
                    *location = info.location().map(ToString::to_string);
                    true
                } else {
                    false
                }
            });
            if captured != Ok(true) {
                previous(info);
            }
        }));
    });
}

enum Failure {
    Panic {
        payload: Box<dyn Any + Send>,
        location: Option<String>,
    },
    Exit(String),
}

impl Failure {
    fn same_kind(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exit(_), Self::Exit(_)) => true,
            (
                Self::Panic {
                    payload: a,
                    location: at,
                },
                Self::Panic {
                    payload: b,
                    location: bt,
                },
            ) => at == bt && a.as_ref().type_id() == b.as_ref().type_id(),
            _ => false,
        }
    }

    fn report(self) -> ! {
        match self {
            Self::Panic { payload, location } => {
                let message = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("panic with a non-string payload");
                if let Some(location) = location {
                    eprintln!("  cause: {message} at {location}");
                } else {
                    eprintln!("  cause: {message}");
                }
                panic::resume_unwind(payload)
            }
            Self::Exit(message) => {
                eprintln!("  cause: {message}");
                panic!("property returned a failing exit status")
            }
        }
    }
}

struct FailedCase {
    input: String,
    choices: Vec<Choice>,
    failure: Failure,
}

fn evaluate(
    seed: u64,
    replay: Option<Vec<Choice>>,
    body: &mut impl FnMut(&mut Gen) -> Result<(), String>,
) -> Option<FailedCase> {
    let mut generator = Gen::new(seed, replay);
    let previous = CAPTURE.replace(Some(None));
    let result = panic::catch_unwind(AssertUnwindSafe(|| body(&mut generator)));
    let location = CAPTURE.replace(previous).flatten();
    if generator.replay_overrun {
        return None;
    }
    let failure = match result {
        Ok(Ok(())) => return None,
        Ok(Err(message)) => Failure::Exit(message),
        Err(payload) => Failure::Panic { payload, location },
    };
    Some(FailedCase {
        input: generator.input,
        choices: generator.choices,
        failure,
    })
}

/// Execute the generated cases for a libtest entry.
///
/// `path` is the crate-qualified test path. The callback draws and records
/// arguments before invoking the test body. A decimal `TIDE_TEST_SEED` replays
/// one case, including its shrink sequence on failure.
/// The callback cannot capture caller state and returns a silently classified result.
#[doc(hidden)]
pub fn run(path: &str, mut body: fn(&mut Gen) -> Result<(), String>) {
    let name = path.split_once("::").map_or(path, |(_, name)| name);
    let replay = std::env::var_os("TIDE_TEST_SEED").map(|seed| {
        seed.to_str()
            .and_then(|seed| seed.parse::<u64>().ok())
            .expect("TIDE_TEST_SEED must be a decimal u64")
    });
    let cases = if replay.is_some() { 1 } else { CASES };
    // Stable path hash so replay does not depend on the host's hash builder.
    let path_hash = path.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    install_hook();
    for index in 0..cases {
        let seed = replay.unwrap_or_else(|| mix(RUN_SEED ^ path_hash ^ mix(index)));
        if let Some(best) = evaluate(seed, None, &mut body) {
            eprintln!("property failed: {name}");
            eprintln!("  seed: {seed}");
            eprintln!("  replay: tide test {name} --seed {seed}");
            let executable = std::env::current_exe()
                .map(|path| path.to_string_lossy().replace('\'', "'\\''"))
                .unwrap_or_else(|_| "<test-binary>".into());
            eprintln!("  replay: TIDE_TEST_SEED={seed} '{executable}' --exact {name}");
            let mut shrinker = Shrinker {
                seed,
                body: &mut body,
                best,
                evaluations: 0,
                last_improvement: 0,
            };
            shrinker.shrink();
            eprintln!("  input: {}", shrinker.best.input);
            eprintln!("  shrink evaluations: {}", shrinker.evaluations);
            if shrinker.exhausted() {
                eprintln!("  shrink budget reached");
            }
            shrinker.best.failure.report();
        }
    }
    eprintln!("property {name}: {cases}/{cases} cases passed");
}

struct Shrinker<'a, F> {
    seed: u64,
    body: &'a mut F,
    best: FailedCase,
    evaluations: usize,
    last_improvement: usize,
}

impl<F: FnMut(&mut Gen) -> Result<(), String>> Shrinker<'_, F> {
    fn exhausted(&self) -> bool {
        // Stop spending time on expensive properties when no candidate improves the input.
        self.evaluations == SHRINK_BUDGET
            || self.evaluations - self.last_improvement == STALLED_BUDGET
    }

    fn try_choices(&mut self, choices: Vec<Choice>) -> bool {
        if self.exhausted() {
            return false;
        }
        self.evaluations += 1;
        let Some(candidate) = evaluate(self.seed, Some(choices), self.body) else {
            return false;
        };
        let shorter = (candidate.choices.len(), &candidate.choices)
            < (self.best.choices.len(), &self.best.choices);
        if shorter && candidate.failure.same_kind(&self.best.failure) {
            self.best = candidate;
            self.last_improvement = self.evaluations;
            true
        } else {
            false
        }
    }

    fn try_value(&mut self, index: usize, value: u128) -> bool {
        let mut candidate = self.best.choices.clone();
        candidate[index].value = value;
        self.try_choices(candidate)
    }

    fn shrink(&mut self) {
        while !self.exhausted() {
            let before = self.best.choices.clone();
            self.delete_spans();
            self.minimize();
            for index in 0..self.best.choices.len() {
                if self.exhausted() || index >= self.best.choices.len() {
                    break;
                }
                if self.best.choices[index].value > 0 {
                    self.try_value(index, 0);
                }
            }
            if self.best.choices == before {
                break;
            }
        }
    }

    fn delete_spans(&mut self) {
        let mut width = self.best.choices.len();
        while width > 0 && !self.exhausted() {
            let mut start = 0;
            while start + width <= self.best.choices.len() && !self.exhausted() {
                let mut candidate = self.best.choices.clone();
                candidate.drain(start..start + width);
                if !self.try_choices(candidate) {
                    start += 1;
                }
            }
            width /= 2;
        }
    }

    fn minimize(&mut self) {
        for index in 0..self.best.choices.len() {
            let Some(choice) = self.best.choices.get(index).copied() else {
                break;
            };
            // Signed and float choices put the sign in the low bit. Searching
            // the magnitude keeps a positive boundary from alternating signs.
            let shift = u32::from(matches!(choice.kind, Kind::Signed(_) | Kind::Float(_)));
            let sign = choice.value & u128::from(shift);
            let (mut low, mut high) = (1 - sign, choice.value >> shift);
            while low < high && !self.exhausted() {
                let middle = low + (high - low) / 2;
                if self.try_value(index, (middle << shift) | sign) {
                    if self
                        .best
                        .choices
                        .get(index)
                        .is_none_or(|new| new.kind != choice.kind)
                    {
                        break;
                    }
                    high = self.best.choices[index].value >> shift;
                } else {
                    low = middle + 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn a_slow_stalled_search_finishes_within_the_harness_budget() {
        install_hook();
        let started = Instant::now();
        let mut calls = 0;
        let mut body = |generator: &mut Gen| {
            calls += 1;
            let values: [u8; 512] = std::array::from_fn(|_| generator.index(16) as u8);
            generator.record("values", &values);
            std::thread::sleep(Duration::from_millis(50));
            // Seed 4 fails at this boundary and lowering a draw makes it pass.
            assert!(values.iter().map(|n| u32::from(*n)).sum::<u32>() < 3958);
            Ok(())
        };
        let best = evaluate(4, None, &mut body).expect("the initial case must fail");
        let mut shrinker = Shrinker {
            seed: 4,
            body: &mut body,
            best,
            evaluations: 0,
            last_improvement: 0,
        };
        shrinker.shrink();
        assert!(shrinker.exhausted());
        assert_eq!(shrinker.evaluations, STALLED_BUDGET);
        assert!(shrinker.best.input.starts_with("values = ["));
        drop(shrinker);
        assert_eq!(calls, STALLED_BUDGET + 1);
        assert!(started.elapsed() < Duration::from_secs(10));
    }
}

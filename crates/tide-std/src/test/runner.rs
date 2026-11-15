use std::any::Any;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::process::{ExitCode, Termination};
use std::sync::Once;

use super::{Choice, Gen, Kind, mix};
use crate::__contract::{self, Clause, Kind as ContractKind, Violation};

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

/// Source bounds used to distinguish direct calls from deeper contract failures.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Source {
    pub file: &'static str,
    pub start: (u32, u32),
    pub end: (u32, u32),
}
impl Source {
    fn contains(self, violation: &Violation) -> bool {
        self.file == violation.file
            && self.start <= (violation.line, violation.column)
            && (violation.line, violation.column) <= self.end
    }
}
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct Config {
    pub cases: u64,
    pub plain: bool,
    pub body: Source,
    pub call: Source,
    pub excluded: &'static [Source],
    pub should_panic: bool,
    pub simulation: bool,
    pub fixed_seed: bool,
    pub allow_vacuous: Option<&'static str>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            cases: CASES,
            plain: false,
            body: Source::default(),
            call: Source::default(),
            excluded: &[],
            should_panic: false,
            simulation: false,
            fixed_seed: false,
            allow_vacuous: None,
        }
    }
}

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

struct GenerationRejected(Clause);

/// A public constructor/mutator may reject its arguments before entering its body.
#[doc(hidden)]
pub fn attempt<T>(function: &str, call: Source, body: impl FnOnce() -> T) -> Option<T> {
    __contract::take_violation();
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => Some(value),
        Err(payload) => {
            let violation = __contract::take_violation();
            if let Some(v) = violation
                && v.clause.kind == ContractKind::Requires
                && v.clause.function == function
                && call.contains(&v)
                && payload
                    .downcast_ref::<String>()
                    .is_some_and(|s| s.starts_with("requires violated:"))
            {
                REJECTION.with(|r| *r.borrow_mut() = Some(v.clause));
                None
            } else {
                panic::resume_unwind(payload)
            }
        }
    }
}
thread_local! { static REJECTION: RefCell<Option<Clause>> = const { RefCell::new(None) }; }
#[doc(hidden)]
pub fn reject_generation() -> ! {
    let clause = REJECTION
        .with(|r| r.borrow_mut().take())
        .expect("generation rejection needs a requires clause");
    panic::panic_any(GenerationRejected(clause))
}

enum Failure {
    Panic {
        payload: Box<dyn Any + Send>,
        location: Option<String>,
        violation: Option<Violation>,
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
                    violation: av,
                },
                Self::Panic {
                    payload: b,
                    location: bt,
                    violation: bv,
                },
            ) => at == bt && av == bv && a.as_ref().type_id() == b.as_ref().type_id(),
            _ => false,
        }
    }

    fn report(self) -> ! {
        match self {
            Self::Panic {
                payload, location, ..
            } => {
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

enum Outcome {
    Passed,
    Rejected(Clause),
    Failed(FailedCase),
}
struct Evaluation {
    outcome: Outcome,
    trace: crate::task::Trace,
}
impl Evaluation {
    fn signature(&self) -> String {
        match &self.outcome {
            Outcome::Passed => "passed".into(),
            Outcome::Rejected(clause) => format!("rejected: {clause:?}"),
            Outcome::Failed(case) => match &case.failure {
                Failure::Exit(message) => format!("exit: {message}"),
                Failure::Panic {
                    payload,
                    location,
                    violation,
                } => format!(
                    "panic: {:?} {location:?} {violation:?}",
                    payload
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| payload.downcast_ref::<&str>().copied())
                ),
            },
        }
    }
}

fn evaluate(
    seed: u64,
    replay: Option<Vec<Choice>>,
    config: Config,
    body: &mut impl FnMut(&mut Gen) -> Result<(), String>,
) -> Evaluation {
    evaluate_inner(seed, replay, config, body, None, false)
}

fn evaluate_inner(
    seed: u64,
    replay: Option<Vec<Choice>>,
    config: Config,
    body: &mut impl FnMut(&mut Gen) -> Result<(), String>,
    recording: Option<(&str, u64)>,
    strict: bool,
) -> Evaluation {
    let nested = crate::sim::manifest::active();
    let _suspended = nested.then(crate::sim::manifest::suspend);
    let mut generator = Gen::new(seed, replay);
    generator.strict = strict;
    generator.origin = (!config.call.file.is_empty()).then_some(config.call);
    if !nested {
        crate::sim::manifest::begin(recording.map_or("", |r| r.0), seed, recording.map_or(0, |r| r.1), config, recording.is_some());
    }
    // Seed-start records belong to the outermost simulation; a nested one would mislead the parent.
    if config.simulation && !nested {
        crate::sim::runner::start(seed);
    }
    if config.simulation || config.fixed_seed {
        generator.random = mix(seed ^ RUN_SEED);
    }
    __contract::take_violation();
    let previous = CAPTURE.replace(Some(None));
    let result = panic::catch_unwind(AssertUnwindSafe(|| body(&mut generator)));
    let location = CAPTURE.replace(previous).flatten();
    let violation = __contract::take_violation();
    if strict {
        assert!(generator.strict_error.is_none(), "{}", generator.strict_error.as_deref().unwrap_or(""));
        assert!(generator.replay.as_ref().is_none_or(|r| r.is_empty()), "replay choice tape has leftovers");
    }
    let choices = generator.choices.clone();
    let outcome = if generator.replay_overrun {
        Outcome::Passed
    } else {
        match result {
            Ok(Ok(())) => Outcome::Passed,
            Ok(Err(message)) => Outcome::Failed(FailedCase {
                input: generator.input,
                choices: generator.choices,
                failure: Failure::Exit(message),
            }),
            Err(payload) if payload.is::<GenerationRejected>() => {
                Outcome::Rejected(payload.downcast::<GenerationRejected>().unwrap().0)
            }
            Err(payload) => {
                // A caught earlier clause must not turn a later, unrelated panic into a discard.
                let rejected = violation.as_ref().filter(|v| {
                    !config.should_panic && !config.plain
                        && v.clause.kind == ContractKind::Requires
                        && (config.body.contains(v) || config.call.contains(v))
                        && !config.excluded.iter().any(|source| source.contains(v))
                        && location.as_deref()
                            == Some(format!("{}:{}:{}", v.file, v.line, v.column).as_str())
                        && payload.downcast_ref::<String>().is_some_and(|s| {
                            s.starts_with(&format!(
                                "requires violated: {} (in `{}`",
                                v.clause.text, v.clause.function
                            ))
                        })
                });
                if let Some(v) = rejected {
                    Outcome::Rejected(v.clause)
                } else {
                    Outcome::Failed(FailedCase {
                        input: generator.input,
                        choices: generator.choices,
                        failure: Failure::Panic {
                            payload,
                            location,
                            violation,
                        },
                    })
                }
            }
        }
    };
    let mut evaluation = Evaluation { outcome, trace: Default::default() };
    // Native libtest owns Termination::report after an ordinary function returns its value.
    let signature = if config.plain && matches!(evaluation.outcome, Outcome::Passed) {
        "returned".into()
    } else {
        evaluation.signature()
    };
    if !nested {
        evaluation.trace = crate::sim::manifest::finish(&choices, &signature, matches!(evaluation.outcome, Outcome::Failed(_)));
    }
    evaluation
}

/// Record an ordinary test without consuming its return value or intercepting nested calls.
#[doc(hidden)]
pub fn run_plain<R>(path: &str, config: Config, body: fn() -> R, classify: fn(&R) -> Result<(), String>) -> R {
    if crate::sim::manifest::active() {
        return body();
    }
    let expected = crate::sim::manifest::context().and_then(|c| c.replay);
    let seed = selected_seed(config, expected.as_ref()).unwrap_or_else(|| case_seed(path, config, 0));
    let tape = expected.as_ref().map(|m| m.choices.iter().map(Choice::from_wire).collect());
    install_hook();
    let mut value = None;
    let evaluation = evaluate_inner(seed, tape, config, &mut |_| {
        let returned = body();
        let outcome = classify(&returned);
        value = Some(returned);
        outcome
    }, Some((path, expected.as_ref().map_or(0, |m| m.evaluation))), expected.is_some());
    if let Outcome::Failed(FailedCase { failure: failure @ Failure::Panic { .. }, .. }) = evaluation.outcome {
        failure.report();
    }
    value.expect("ordinary test must return or panic")
}

fn seed_var(config: Config) -> &'static str {
    if config.simulation { "TIDE_SIM_SEED" } else { "TIDE_TEST_SEED" }
}

fn selected_seed(config: Config, expected: Option<&crate::replay::Manifest>) -> Option<u64> {
    let seed_var = seed_var(config);
    let fresh = std::env::var_os(seed_var).map(|seed| {
        seed.to_str().and_then(|seed| seed.parse::<u64>().ok())
            .unwrap_or_else(|| panic!("{seed_var} must be a decimal u64"))
    });
    expected.map(|m| m.seed.parse::<u64>().unwrap()).or(fresh)
}

fn case_seed(path: &str, config: Config, index: u64) -> u64 {
    let path_hash = path.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    if config.simulation { index } else { mix(RUN_SEED ^ path_hash ^ mix(index)) }
}

/// One owner for accepted-case counting, requires discards, replay and shrinking.
/// Callbacks cannot capture caller state across a caught panic.
#[doc(hidden)]
pub fn run_config(path: &str, config: Config, mut body: fn(&mut Gen) -> Result<(), String>) {
    let name = path.split_once("::").map_or(path, |(_, name)| name);
    let seed_var = seed_var(config);
    let nested = crate::sim::manifest::active();
    let expected = if nested { None } else { crate::sim::manifest::context().and_then(|c| c.replay) };
    // Driver controls select the outer evaluation, not a nested test's case range.
    let selected_seed = if nested { None } else { selected_seed(config, expected.as_ref()) };
    let cases = if selected_seed.is_some() { 1 } else { config.cases };
    let limit = if selected_seed.is_some() {
        1
    } else {
        cases.saturating_mul(100)
    };
    install_hook();
    let mut accepted = 0;
    let mut generated = 0;
    let mut failures = 0;
    let mut first_failure = None;
    // A failing case that did not panic: a returned error or a failed determinism self-check.
    let mut unpanicked = false;
    let mut rejections: Vec<(Clause, u64)> = Vec::new();
    for index in 0..limit {
        if accepted == cases {
            break;
        }
        let seed = selected_seed.unwrap_or_else(|| case_seed(path, config, index));
        generated += 1;
        let tape = expected.as_ref().map(|m| m.choices.iter().map(Choice::from_wire).collect());
        let mut evaluation = evaluate_inner(seed, tape, config, &mut body, Some((path, expected.as_ref().map_or(index, |m| m.evaluation))), expected.is_some());
        if !nested && expected.is_none() && config.simulation && (index == 0 || matches!(evaluation.outcome, Outcome::Failed(_))) {
            let again = evaluate(seed, None, config, &mut body);
            let difference = evaluation.trace.first_difference(&again.trace);
            if let Some((line, a, b)) = &difference {
                eprintln!(
                    "simulation nondeterminism seed={seed}: first differing trace line {line}\n  first: {a:?}\n  second: {b:?}"
                );
            }
            let different_outcome = evaluation.signature() != again.signature();
            if different_outcome {
                eprintln!("simulation nondeterminism seed={seed}: outcomes differ");
            }
            if difference.is_some() || different_outcome {
                evaluation.outcome = Outcome::Failed(FailedCase {
                    input: String::new(),
                    choices: Vec::new(),
                    failure: Failure::Exit("determinism self-check failed".into()),
                });
            }
        }
        match evaluation.outcome {
            Outcome::Passed => accepted += 1,
            Outcome::Rejected(clause) => {
                if let Some((_, count)) = rejections.iter_mut().find(|(c, _)| *c == clause) {
                    *count += 1;
                } else {
                    rejections.push((clause, 1));
                }
            }
            Outcome::Failed(best) => {
                accepted += 1;
                failures += 1;
                eprintln!("property failed: {name}");
                eprintln!("  seed: {seed}");
                eprintln!("  fresh rerun (planned frontend): tide test {name} --seed {seed}");
                if let Some(context) = crate::sim::manifest::context() {
                    let mut command = context.command;
                    command.extend(["--tide-run-test".into(), name.into(), "--seed".into(), seed.to_string()]);
                    eprintln!("  fresh rerun: {}", command.iter().map(|s| crate::replay::shell(s)).collect::<Vec<_>>().join(" "));
                } else {
                    let executable = std::env::current_exe()
                        .map(|path| path.to_string_lossy().replace('\'', "'\\''"))
                        .unwrap_or_else(|_| "<test-binary>".into());
                    eprintln!("  fresh rerun: {seed_var}={seed} '{executable}' --exact {name}");
                }
                if config.simulation {
                    eprintln!("simulation failure seed={seed} test={name}");
                    eprint!("{}", evaluation.trace.render());
                }
                let mut shrinker = Shrinker {
                    seed,
                    body: &mut body,
                    config,
                    best,
                    evaluations: 0,
                    last_improvement: 0,
                };
                if expected.is_none() { shrinker.shrink(); }
                eprintln!("  input: {}", shrinker.best.input);
                eprintln!("  shrink evaluations: {}", shrinker.evaluations);
                if shrinker.exhausted() {
                    eprintln!("  shrink budget reached");
                }
                if config.simulation {
                    let failure = shrinker.best.failure;
                    match &failure {
                        Failure::Exit(message) => eprintln!("  cause: {message}"),
                        Failure::Panic { payload, .. } => eprintln!(
                            "  cause: {}",
                            payload
                                .downcast_ref::<String>()
                                .map(String::as_str)
                                .or_else(|| payload.downcast_ref::<&str>().copied())
                                .unwrap_or("non-string panic")
                        ),
                    }
                    unpanicked |= matches!(failure, Failure::Exit(_));
                    if first_failure.is_none() {
                        first_failure = Some(failure);
                    }
                } else {
                    let failure = shrinker.best.failure;
                    if config.should_panic
                        && let Failure::Exit(message) = &failure
                    {
                        eprintln!("  cause: {message}");
                        eprintln!(
                            "property {name}: the failing case did not panic; should_panic not satisfied"
                        );
                        return;
                    }
                    failure.report();
                }
            }
        }
    }
    if let Some(failure) = first_failure {
        eprintln!(
            "simulation {name}: {} passed; {failures} failed ({generated} seeds)",
            accepted - failures
        );
        if unpanicked && config.should_panic {
            // A returned error or failed self-check is a failure, not the expected panic, and
            // another seed's panic does not excuse it. Returning fails the test as "did not panic".
            eprintln!(
                "simulation {name}: a failing seed did not panic; should_panic not satisfied"
            );
            return;
        }
        failure.report();
    }
    if accepted != cases {
        // Source order, rather than the first case to reject, breaks count ties.
        rejections.sort_by_key(|(c, _)| (c.file, c.line, c.column, c.index, c.conjunct));
        let most = rejections
            .iter()
            .fold(None, |best: Option<&(Clause, u64)>, next| {
                if best.is_none_or(|best| next.1 > best.1) {
                    Some(next)
                } else {
                    best
                }
            });
        let (text, count) = most.map_or(("<unknown requires>", 0), |(clause, count)| {
            (clause.text, *count)
        });
        if config.simulation {
            eprintln!("simulation {name}: vacuous ({generated} seeds)");
        }
        if let Some(reason) = config.allow_vacuous {
            eprintln!(
                "unchecked (vacuous, allowed: {reason}): {name}; {accepted}/{generated} accepted/generated; most rejected by `{text}` ({count})"
            );
            return;
        }
        panic!(
            "vacuous: {accepted} of {generated} generated cases satisfied requires; most rejected by `{text}` ({count})"
        );
    }
    if config.simulation {
        eprintln!(
            "simulation {name}: {accepted} passed; 0 failed ({generated} seeds); {accepted}/{generated} cases passed (accepted/generated)"
        );
    } else if !config.plain {
        eprintln!("property {name}: {accepted}/{generated} cases passed (accepted/generated)");
    }
}

struct Shrinker<'a, F> {
    seed: u64,
    config: Config,
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
        let Outcome::Failed(candidate) =
            evaluate(self.seed, Some(choices), self.config, self.body).outcome
        else {
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
    fn fixed_seed_parameters_share_the_simulation_stream_and_replay() {
        let draws = |config| {
            let mut values = Vec::new();
            evaluate(47, None, config, &mut |g| {
                assert_eq!(g.seed(), 47);
                values = (0..4).map(|_| g.index(usize::MAX)).collect();
                Ok(())
            });
            values
        };
        let plain = Config {
            fixed_seed: true,
            ..Config::default()
        };
        let simulated = Config {
            simulation: true,
            ..Config::default()
        };
        assert_eq!(draws(plain), draws(simulated));
        assert_eq!(draws(plain), draws(plain));
        assert_ne!(draws(plain), draws(Config::default()));
    }

    #[test]
    fn caught_clause_records_do_not_discard_an_unrelated_panic_or_should_panic_case() {
        install_hook();
        let clause = Clause {
            kind: ContractKind::Requires,
            text: "false",
            function: "f",
            file: file!(),
            line: line!(),
            column: 1,
            index: 0,
            conjunct: 0,
        };
        let config = Config {
            body: Source {
                file: file!(),
                start: (0, 0),
                end: (u32::MAX, u32::MAX),
            },
            ..Config::default()
        };
        let mut body = |_: &mut Gen| -> Result<(), String> {
            let _ = panic::catch_unwind(|| __contract::requires(clause, || false));
            panic!("ordinary failure");
        };
        assert!(matches!(
            evaluate(1, None, config, &mut body).outcome,
            Outcome::Failed(_)
        ));
        let mut body = |_: &mut Gen| -> Result<(), String> {
            __contract::requires(clause, || false);
            Ok(())
        };
        assert!(matches!(
            evaluate(
                1,
                None,
                Config {
                    should_panic: true,
                    ..config
                },
                &mut body
            )
            .outcome,
            Outcome::Failed(_)
        ));
        let violation = Violation {
            clause,
            file: file!(),
            line: 1,
            column: 1,
        };
        let failure = |violation| Failure::Panic {
            payload: Box::new(String::from("same payload type")),
            location: None,
            violation: Some(violation),
        };
        assert!(!failure(violation.clone()).same_kind(&failure(Violation {
            clause: Clause {
                conjunct: 1,
                ..clause
            },
            ..violation
        })));
    }

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
        let Outcome::Failed(best) = evaluate(4, None, Config::default(), &mut body).outcome else {
            panic!("the initial case must fail")
        };
        let mut shrinker = Shrinker {
            seed: 4,
            config: Config::default(),
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

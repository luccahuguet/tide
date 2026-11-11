use super::Outcome;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::time::Duration;

pub(super) type Span = &'static std::panic::Location<'static>;
pub(super) type TaskId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation {
    Scope(usize),
    Spawn(TaskId),
    Join(TaskId),
    ScopeExit(usize),
    Cancel(usize),
    Checkpoint,
    Yield,
    CellNew(usize),
    CellAcquire(usize),
    CellRelease(usize),
    ClockNow,
    ClockSystemTime(Duration),
    /// `clock.sleep` or `clock.sleep_until`, by `name`; `until: None` never ends.
    Sleep {
        name: &'static str,
        until: Option<Duration>,
    },
    Advance,
    Deadlock,
    Exit,
    Sink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Pending {
    pub(super) operation: Operation,
    pub(super) span: Span,
}

enum State {
    Starting,
    Ready(Option<Pending>),
    Blocked(Pending),
    Finished,
}

struct TaskState {
    state: State,
    spawn_span: Span,
    timer: Option<TimerKey>,
}

// (deadline, class, id). Class 0 is reserved for scope deadlines keyed by scope id, so an
// enclosing deadline fires before a nested one and both before any operation timer of the same
// instant (TASK-2, TASK-4); class 1 is an operation timer with a simulation-local sequence id.
type TimerKey = (Duration, u8, usize);
const OPERATION_TIMER: u8 = 1;

/// Simulated wall time at virtual time zero: 2000-01-01T00:00:00Z (03 §9.3).
const WALL_START: Duration = Duration::from_secs(946_684_800);

/// Scheduling points allowed while a timer waits and virtual time stands still (03 §9.3).
const STALL_LIMIT: usize = 1_000_000;

// Decision state has no OS resources. Index order is task registration order.
pub(super) struct Scheduler {
    tasks: Vec<TaskState>,
    pub(super) current: Option<TaskId>,
    next_cell: usize,
    rng: u64,
    step: usize,
    pub(super) trace: String,
    pub(super) records: Vec<crate::sim::Record>,
    // Virtual time moves only in `pick`, when no task is runnable.
    pub(super) now: Duration,
    timers: BTreeMap<TimerKey, TaskId>,
    next_timer: usize,
    stalled: usize,
    pub(super) stall_limit: usize,
}

impl Scheduler {
    pub(super) fn new(seed: u64, span: Span) -> Self {
        Self {
            tasks: vec![TaskState {
                state: State::Ready(None),
                spawn_span: span,
                timer: None,
            }],
            current: Some(0),
            next_cell: 0,
            rng: seed ^ 0x7469_6465_7363_6864, // "tideschd"
            step: 0,
            trace: String::new(),
            records: Vec::new(),
            now: Duration::ZERO,
            timers: BTreeMap::new(),
            next_timer: 0,
            stalled: 0,
            stall_limit: STALL_LIMIT,
        }
    }

    /// Simulated wall time, as an offset from the Unix epoch.
    pub(super) fn wall(&self) -> Option<Duration> {
        WALL_START.checked_add(self.now)
    }

    pub(super) fn register(&mut self, span: Span) -> TaskId {
        let id = self.tasks.len();
        self.tasks.push(TaskState {
            state: State::Starting,
            spawn_span: span,
            timer: None,
        });
        id
    }

    pub(super) fn allocate_cell(&mut self) -> usize {
        let id = self.next_cell;
        self.next_cell += 1;
        id
    }

    pub(super) fn start(&mut self, task: TaskId) {
        self.tasks[task].state = State::Ready(None);
    }

    pub(super) fn abandon(&mut self, task: TaskId) {
        self.tasks[task].state = State::Finished;
    }

    /// Records a scheduling point. `Err` is the stall report: too many points passed while a
    /// timer waited, so the tasks that keep running are holding virtual time still.
    pub(super) fn point(&mut self, task: TaskId, pending: Pending) -> Result<(), String> {
        self.tasks[task].state = State::Ready(Some(pending));
        self.event(task, pending, pending.operation.kind(), Some("ready"));
        self.trace.push_str("}\n");
        if self.timers.is_empty() {
            return Ok(());
        }
        self.stalled += 1;
        if self.stalled <= self.stall_limit {
            return Ok(());
        }
        self.stalled = 0;
        let mut report = format!(
            "simulation stalled at virtual time {:?}: {} scheduling points passed while a timer \
             waited. Virtual time advances only when every task is blocked, so a task that polls \
             with `yield_now`, `checkpoint`, `now` or a zero sleep never reaches the timer",
            self.now, self.stall_limit,
        );
        for (id, state) in self.tasks.iter().enumerate() {
            if let State::Ready(Some(ready)) = state.state {
                write!(
                    report,
                    "\n  task {id} runnable in {} at {}",
                    ready.operation, ready.span
                )
                .unwrap();
            }
        }
        Err(report)
    }

    /// Blocks `task`; with `until`, an operation timer wakes it at that instant, which the
    /// caller has checked is after `now`.
    pub(super) fn block(&mut self, task: TaskId, pending: Pending, until: Option<Duration>) {
        self.tasks[task].state = State::Blocked(pending);
        if let Some(until) = until {
            debug_assert!(until > self.now);
            let key = (until, OPERATION_TIMER, self.next_timer);
            self.next_timer += 1;
            self.timers.insert(key, task);
            self.tasks[task].timer = Some(key);
        }
        self.event(task, pending, pending.operation.kind(), Some("blocked"));
        self.trace.push_str("}\n");
    }

    pub(super) fn blocked(&self) -> impl Iterator<Item = (TaskId, Pending)> + '_ {
        self.tasks
            .iter()
            .enumerate()
            .filter_map(|(id, task)| match task.state {
                State::Blocked(wait) => Some((id, wait)),
                _ => None,
            })
    }

    pub(super) fn wake(&mut self, task: TaskId, pending: Pending) {
        // A waiter woken early takes its timer along: a stale one would cause a jump that
        // wakes nobody.
        if let Some(key) = self.tasks[task].timer.take() {
            self.timers.remove(&key);
            if self.timers.is_empty() {
                self.stalled = 0;
            }
        }
        self.tasks[task].state = State::Ready(Some(pending));
        self.event(task, pending, pending.operation.kind(), Some("ready"));
        self.trace.push_str(",\"action\":\"wake\"}\n");
    }

    pub(super) fn cancel(&mut self, task: TaskId, scope: usize, span: Span) {
        let pending = Pending {
            operation: Operation::Cancel(scope),
            span,
        };
        self.event(task, pending, "cancel", None);
        self.trace.push_str(",\"action\":\"cancelled\"}\n");
    }

    pub(super) fn finish(&mut self, task: TaskId, outcome: Outcome) -> Pending {
        self.tasks[task].state = State::Finished;
        let pending = Pending {
            operation: Operation::Exit,
            span: self.tasks[task].spawn_span,
        };
        self.event(task, pending, "exit", None);
        let outcome = match outcome {
            Outcome::Ok => "ok",
            Outcome::Error => "error",
            Outcome::Echo => "echo",
            Outcome::Panic => "panic",
        };
        writeln!(self.trace, ",\"outcome\":\"{outcome}\"}}").unwrap();
        pending
    }

    pub(super) fn sink(&mut self, task: TaskId, stream: &str, message: &str, span: Span) {
        self.event(
            task,
            Pending {
                operation: Operation::Sink,
                span,
            },
            "sink",
            None,
        );
        self.trace.push_str(",\"stream\":");
        quoted(&mut self.trace, stream);
        self.trace.push_str(",\"message\":");
        quoted(&mut self.trace, message);
        self.trace.push_str("}\n");
        self.records.push(crate::sim::Record {
            node: "main".into(),
            time: self.now,
            sink: match stream {
                "out" => crate::sim::Sink::Out,
                "err" => crate::sim::Sink::Err,
                _ => unreachable!("unsupported sink"),
            },
            message: message.into(),
        });
    }

    /// Records what a blocking capability operation returned.
    pub(super) fn result(&mut self, task: TaskId, pending: Pending, ok: bool) {
        self.event(task, pending, pending.operation.kind(), None);
        let result = if ok { "ok" } else { "cancelled" };
        writeln!(self.trace, ",\"result\":\"{result}\"}}").unwrap();
    }

    /// The global deadlock report (03 §9.3): every blocked task, what it waits for, and the span
    /// of its blocking call. The same list is the trace's last event.
    pub(super) fn deadlock(&mut self, task: TaskId, pending: Pending) -> String {
        let blocked: Vec<_> = self.blocked().collect();
        let pending = Pending {
            operation: Operation::Deadlock,
            span: pending.span,
        };
        self.event(task, pending, "sched", None);
        self.trace.push_str(",\"blocked\":[");
        let mut report = format!(
            "deadlock: no runnable task in the simulator and no timer pending (virtual time {:?})",
            self.now
        );
        for (index, (id, wait)) in blocked.into_iter().enumerate() {
            let comma = if index == 0 { "" } else { "," };
            write!(self.trace, "{comma}{{\"task\":{id}").unwrap();
            self.operation(wait);
            self.trace.push('}');
            write!(
                report,
                "\n  task {id} blocked in {} at {}",
                wait.operation, wait.span
            )
            .unwrap();
        }
        self.trace.push_str("]}\n");
        report
    }

    pub(super) fn finished(&self, task: TaskId) -> bool {
        matches!(self.tasks[task].state, State::Finished)
    }

    pub(super) fn pick(&mut self, task: TaskId, pending: Pending) -> Option<TaskId> {
        let runnable = loop {
            let runnable: Vec<_> = self
                .tasks
                .iter()
                .enumerate()
                .filter_map(|(id, task)| matches!(task.state, State::Ready(_)).then_some(id))
                .collect();
            if !runnable.is_empty() {
                break runnable;
            }
            // Every task is blocked: jump to the earliest timer and fire all of its instant, in
            // key order, before any task runs. Without a timer nothing can run again.
            let Some((&(to, _, _), _)) = self.timers.first_key_value() else {
                self.current = None;
                return None;
            };
            assert!(to > self.now, "a timer was registered in the past");
            let advance = Pending {
                operation: Operation::Advance,
                span: pending.span,
            };
            self.event(task, advance, "sched", None);
            writeln!(
                self.trace,
                ",\"from\":{},\"to\":{}}}",
                self.now.as_nanos(),
                to.as_nanos()
            )
            .unwrap();
            self.now = to;
            self.stalled = 0;
            while let Some(timer) = self
                .timers
                .first_entry()
                .filter(|timer| timer.key().0 == to)
            {
                let woken = *timer.get();
                let State::Blocked(wait) = self.tasks[woken].state else {
                    unreachable!("a timer outlived its wait")
                };
                self.wake(woken, wait);
            }
        };
        self.current = match runnable.as_slice() {
            [only] => Some(*only),
            _ => {
                let chosen = runnable[self.index(runnable.len() as u64) as usize];
                let wait = match self.tasks[task].state {
                    State::Ready(Some(_)) => Some("ready"),
                    State::Blocked(_) => Some("blocked"),
                    _ => None,
                };
                self.event(task, pending, "sched", wait);
                self.trace.push_str(",\"runnable\":[");
                for (index, id) in runnable.iter().enumerate() {
                    let comma = if index == 0 { "" } else { "," };
                    write!(self.trace, "{comma}{id}").unwrap();
                }
                writeln!(self.trace, "],\"chosen\":{chosen}}}").unwrap();
                Some(chosen)
            }
        };
        self.current
    }

    fn index(&mut self, bound: u64) -> u64 {
        // Rejection avoids giving lower indices extra weight when 2^64 is not divisible.
        let threshold = bound.wrapping_neg() % bound;
        loop {
            self.rng = self.rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let z = self.rng;
            let z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            let draw = z ^ (z >> 31);
            if draw >= threshold {
                return draw % bound;
            }
        }
    }

    fn event(&mut self, task: TaskId, pending: Pending, kind: &str, wait: Option<&str>) {
        write!(
            self.trace,
            "{{\"step\":{},\"time\":{},\"task\":{task},\"kind\":\"{kind}\"",
            self.step,
            self.now.as_nanos(),
        )
        .unwrap();
        self.step += 1;
        self.operation(pending);
        match wait {
            Some(wait) => write!(self.trace, ",\"wait\":\"{wait}\"").unwrap(),
            None => self.trace.push_str(",\"wait\":null"),
        }
        if kind == "cap" {
            let extra = match pending.operation {
                Operation::ClockNow => write!(self.trace, ",\"result\":{}", self.now.as_nanos()),
                Operation::ClockSystemTime(reading) => {
                    write!(self.trace, ",\"result\":{}", reading.as_nanos())
                }
                Operation::Sleep {
                    until: Some(until), ..
                } => write!(self.trace, ",\"until\":{}", until.as_nanos()),
                Operation::Sleep { until: None, .. } => write!(self.trace, ",\"until\":null"),
                _ => Ok(()),
            };
            extra.unwrap();
        }
    }

    fn operation(&mut self, pending: Pending) {
        let (operation, target) = pending.operation.name();
        write!(self.trace, ",\"operation\":\"{operation}\"").unwrap();
        if let Some(target) = target {
            write!(self.trace, ",\"target\":{target}").unwrap();
        }
        self.trace.push_str(",\"span\":{\"file\":");
        quoted(&mut self.trace, pending.span.file());
        write!(
            self.trace,
            ",\"line\":{},\"column\":{}}}",
            pending.span.line(),
            pending.span.column(),
        )
        .unwrap();
    }
}

// JSON strings for spans and sink messages; each record must stay on one physical line.
fn quoted(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => write!(out, "\\u{:04x}", u32::from(ch)).unwrap(),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

impl Operation {
    fn name(self) -> (&'static str, Option<usize>) {
        match self {
            Self::Scope(id) => ("scope", Some(id)),
            Self::Spawn(id) => ("spawn", Some(id)),
            Self::Join(id) => ("join", Some(id)),
            Self::ScopeExit(id) => ("scope_exit", Some(id)),
            Self::Cancel(id) => ("cancel", Some(id)),
            Self::Checkpoint => ("checkpoint", None),
            Self::Yield => ("yield_now", None),
            Self::CellNew(id) => ("cell_new", Some(id)),
            Self::CellAcquire(id) => ("cell_acquire", Some(id)),
            Self::CellRelease(id) => ("cell_release", Some(id)),
            Self::ClockNow => ("clock.now", None),
            Self::ClockSystemTime(_) => ("clock.system_time", None),
            Self::Sleep { name, .. } => (name, None),
            Self::Advance => ("advance", None),
            Self::Deadlock => ("deadlock", None),
            Self::Exit => ("exit", None),
            Self::Sink => ("sink", None),
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Self::CellNew(_) | Self::CellAcquire(_) | Self::CellRelease(_) => "cell",
            Self::ClockNow | Self::ClockSystemTime(_) | Self::Sleep { .. } => "cap",
            _ => self.name().0,
        }
    }
}

// The wait as the stall and deadlock reports name it: `join(1)`, `clock.sleep until 5s`.
impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, target) = self.name();
        f.write_str(name)?;
        if let Some(target) = target {
            write!(f, "({target})")?;
        }
        match self {
            Self::Sleep {
                until: Some(until), ..
            } => write!(f, " until {until:?}"),
            Self::Sleep { until: None, .. } => f.write_str(" forever"),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelling_the_last_sleeper_resets_the_stall_count_before_another_registers() {
        let span = std::panic::Location::caller();
        let mut scheduler = Scheduler::new(0, span);
        scheduler.stall_limit = 3;
        let polling = Pending {
            operation: Operation::Yield,
            span,
        };
        let until = Duration::from_secs(1);
        let sleeping = Pending {
            operation: Operation::Sleep {
                name: "clock.sleep",
                until: Some(until),
            },
            span,
        };
        let first = scheduler.register(span);
        scheduler.start(first);
        scheduler.block(first, sleeping, Some(until));
        for _ in 0..scheduler.stall_limit {
            scheduler.point(0, polling).unwrap();
        }
        scheduler.cancel(0, 0, span);
        scheduler.wake(first, sleeping);
        // No point observes the empty timer map before the next sleeper registers.
        let second = scheduler.register(span);
        scheduler.start(second);
        scheduler.block(second, sleeping, Some(until));
        for _ in 0..scheduler.stall_limit {
            scheduler.point(0, polling).unwrap();
        }
        assert!(scheduler.point(0, polling).is_err());
        assert_eq!(scheduler.now, Duration::ZERO);
    }

    #[test]
    fn scheduling_decisions_omit_capability_results_and_sleep_targets() {
        let span = std::panic::Location::caller();
        for (operation, extra) in [
            (Operation::ClockNow, ",\"result\":0}"),
            (
                Operation::ClockSystemTime(Duration::from_secs(1)),
                ",\"result\":1000000000}",
            ),
            (
                Operation::Sleep {
                    name: "clock.sleep",
                    until: Some(Duration::ZERO),
                },
                ",\"until\":0}",
            ),
            (
                Operation::Sleep {
                    name: "clock.sleep",
                    until: None,
                },
                ",\"until\":null}",
            ),
        ] {
            let mut scheduler = Scheduler::new(0, span);
            let second = scheduler.register(span);
            scheduler.start(second);
            let pending = Pending { operation, span };
            scheduler.point(0, pending).unwrap();
            scheduler.pick(0, pending).unwrap();
            let mut events = scheduler.trace.lines();
            let capability = events.next().unwrap();
            let decision = events.next().unwrap();
            assert!(capability.contains("\"kind\":\"cap\""), "{capability}");
            assert!(capability.ends_with(extra), "{capability}");
            assert!(decision.contains("\"kind\":\"sched\""), "{decision}");
            assert!(!decision.contains("\"result\":"), "{decision}");
            assert!(!decision.contains("\"until\":"), "{decision}");
        }
    }

    #[test]
    fn no_runnable_task_lists_every_blocked_operation_and_span() {
        let first_span = std::panic::Location::caller();
        let second_span = std::panic::Location::caller();
        for operations in [
            [Operation::Join(1), Operation::Join(0)],
            [Operation::CellAcquire(4), Operation::CellAcquire(5)],
        ] {
            let mut scheduler = Scheduler::new(0, first_span);
            let second = scheduler.register(second_span);
            scheduler.start(second);
            let [first_wait, second_wait] = [
                Pending {
                    operation: operations[0],
                    span: first_span,
                },
                Pending {
                    operation: operations[1],
                    span: second_span,
                },
            ];

            let rng = scheduler.rng;
            scheduler.block(0, first_wait, None);
            assert_eq!(scheduler.pick(0, first_wait), Some(second));
            scheduler.block(second, second_wait, None);
            assert_eq!(scheduler.pick(second, second_wait), None);
            assert_eq!(scheduler.rng, rng);
            assert!(!scheduler.trace.contains("\"kind\":\"sched\""));
            let report = scheduler.deadlock(second, second_wait);
            assert_eq!(report.lines().count(), 3, "{report}");
            for (task, wait) in [(0, first_wait), (second, second_wait)] {
                let line = format!(
                    "\n  task {task} blocked in {} at {}",
                    wait.operation, wait.span
                );
                assert!(report.contains(&line), "{report}");
            }
            let last = scheduler.trace.lines().last().unwrap();
            assert!(
                last.contains("\"kind\":\"sched\",\"operation\":\"deadlock\"")
                    && last.contains(",\"blocked\":[{\"task\":0,\"operation\":")
                    && last.contains("},{\"task\":1,\"operation\":"),
                "{last}"
            );
            assert_eq!(
                scheduler.blocked().collect::<Vec<_>>(),
                [(0, first_wait), (second, second_wait)]
            );
            assert_eq!(
                scheduler
                    .trace
                    .lines()
                    .filter(|line| line.contains("\"wait\":\"blocked\""))
                    .count(),
                2
            );
        }
    }
}

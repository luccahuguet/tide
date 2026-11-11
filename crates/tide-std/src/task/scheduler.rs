use super::Outcome;
use std::fmt::Write;

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
    Exit,
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
}

// Decision state has no OS resources. Index order is task registration order.
pub(super) struct Scheduler {
    tasks: Vec<TaskState>,
    pub(super) current: Option<TaskId>,
    next_cell: usize,
    rng: u64,
    step: usize,
    pub(super) trace: String,
}

impl Scheduler {
    pub(super) fn new(seed: u64, span: Span) -> Self {
        Self {
            tasks: vec![TaskState {
                state: State::Ready(None),
                spawn_span: span,
            }],
            current: Some(0),
            next_cell: 0,
            rng: seed ^ 0x7469_6465_7363_6864, // "tideschd"
            step: 0,
            trace: String::new(),
        }
    }

    pub(super) fn register(&mut self, span: Span) -> TaskId {
        let id = self.tasks.len();
        self.tasks.push(TaskState {
            state: State::Starting,
            spawn_span: span,
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

    pub(super) fn point(&mut self, task: TaskId, pending: Pending) {
        self.tasks[task].state = State::Ready(Some(pending));
        self.event(task, pending, pending.operation.kind(), Some("ready"));
        self.trace.push_str("}\n");
    }

    pub(super) fn block(&mut self, task: TaskId, pending: Pending) {
        self.tasks[task].state = State::Blocked(pending);
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

    pub(super) fn finished(&self, task: TaskId) -> bool {
        matches!(self.tasks[task].state, State::Finished)
    }

    pub(super) fn pick(&mut self, task: TaskId, pending: Pending) -> Option<TaskId> {
        let runnable: Vec<_> = self
            .tasks
            .iter()
            .enumerate()
            .filter_map(|(id, task)| matches!(task.state, State::Ready(_)).then_some(id))
            .collect();
        self.current = match runnable.as_slice() {
            [] => None,
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
        let (operation, target) = pending.operation.name();
        write!(
            self.trace,
            "{{\"step\":{},\"task\":{task},\"kind\":\"{kind}\",\"operation\":\"{operation}\"",
            self.step,
        )
        .unwrap();
        self.step += 1;
        if let Some(target) = target {
            write!(self.trace, ",\"target\":{target}").unwrap();
        }
        self.trace.push_str(",\"span\":{\"file\":\"");
        for ch in pending.span.file().chars() {
            match ch {
                '"' => self.trace.push_str("\\\""),
                '\\' => self.trace.push_str("\\\\"),
                '\u{0}'..='\u{1f}' => write!(self.trace, "\\u{:04x}", u32::from(ch)).unwrap(),
                _ => self.trace.push(ch),
            }
        }
        write!(
            self.trace,
            "\",\"line\":{},\"column\":{}}}",
            pending.span.line(),
            pending.span.column(),
        )
        .unwrap();
        match wait {
            Some(wait) => write!(self.trace, ",\"wait\":\"{wait}\"").unwrap(),
            None => self.trace.push_str(",\"wait\":null"),
        }
    }
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
            Self::Exit => ("exit", None),
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Self::CellNew(_) | Self::CellAcquire(_) | Self::CellRelease(_) => "cell",
            _ => self.name().0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            scheduler.block(0, first_wait);
            assert_eq!(scheduler.pick(0, first_wait), Some(second));
            scheduler.block(second, second_wait);
            assert_eq!(scheduler.pick(second, second_wait), None);
            assert_eq!(scheduler.rng, rng);
            assert!(!scheduler.trace.contains("\"kind\":\"sched\""));
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

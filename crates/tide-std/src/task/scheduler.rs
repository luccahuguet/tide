use super::{Cause, Outcome};
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
    Failure(usize),
    Deadline(usize, Duration),
    SetDeadline(usize, Duration),
    Limit(usize),
    Isolated(TaskId),
    Checkpoint,
    Yield,
    CellNew(usize),
    CellsAcquire,
    CellsRelease,
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
    File(&'static str),
    WorldFile,
    Net(&'static str, usize),
    Input(&'static str),
    Channel(&'static str, usize),
    Arc(&'static str, usize),
}

#[derive(Clone, PartialEq)]
pub(super) enum ChannelEvent {
    New(Option<usize>),
    Result {
        name: &'static str,
        result: &'static str,
        until: Option<Duration>,
    },
    Close {
        end: &'static str,
        senders: Option<usize>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Pending {
    pub(super) operation: Operation,
    pub(super) span: Span,
}

// One trace line kept as data: recording copies a few words, and the NDJSON text is written
// only when the trace is read. `render` is a pure function of the record, so equal records
// render equal lines, and the determinism check compares records before any text.
#[derive(Clone, PartialEq)]
struct Record {
    step: usize,
    time: Duration,
    task: TaskId,
    kind: &'static str,
    pending: Pending,
    origin: Option<crate::test::Source>,
    // The task's cell set when the record was made, for `cells_acquire`.
    targets: Option<Box<[usize]>>,
    wait: Option<&'static str>,
    tail: Tail,
}

// Fields after the common ones, before the record's closing brace.
#[derive(Clone, PartialEq)]
enum Tail {
    None,
    Text(&'static str),
    Owned(String),
    Sched {
        runnable: Vec<TaskId>,
        chosen: TaskId,
    },
    Advance {
        from: Duration,
        to: Duration,
    },
    Channel(ChannelEvent),
    Sink {
        stream: &'static str,
        message: Vec<u8>,
    },
}

impl Record {
    fn render(&self, out: &mut String) {
        write!(
            out,
            "{{\"step\":{},\"time\":{},\"task\":{},\"kind\":\"{}\"",
            self.step,
            self.time.as_nanos(),
            self.task,
            self.kind,
        )
        .unwrap();
        operation(out, self.pending, self.targets.as_deref().unwrap_or(&[]), self.origin);
        match self.wait {
            Some(wait) => write!(out, ",\"wait\":\"{wait}\"").unwrap(),
            None => out.push_str(",\"wait\":null"),
        }
        if let Operation::Deadline(_, at) | Operation::SetDeadline(_, at) = self.pending.operation {
            write!(out, ",\"until\":{}", at.as_nanos()).unwrap();
        }
        if matches!(self.pending.operation, Operation::Isolated(_)) {
            out.push_str(",\"isolated\":true");
        }
        if self.kind == "cap" {
            let extra = match self.pending.operation {
                Operation::ClockNow => write!(out, ",\"result\":{}", self.time.as_nanos()),
                Operation::ClockSystemTime(reading) => {
                    write!(out, ",\"result\":{}", reading.as_nanos())
                }
                Operation::Sleep {
                    until: Some(until), ..
                } => write!(out, ",\"until\":{}", until.as_nanos()),
                Operation::Sleep { until: None, .. } => write!(out, ",\"until\":null"),
                _ => Ok(()),
            };
            extra.unwrap();
        }
        match &self.tail {
            Tail::None => {}
            Tail::Text(text) => out.push_str(text),
            Tail::Owned(text) => out.push_str(text),
            Tail::Sched { runnable, chosen } => {
                out.push_str(",\"runnable\":[");
                for (index, id) in runnable.iter().enumerate() {
                    let comma = if index == 0 { "" } else { "," };
                    write!(out, "{comma}{id}").unwrap();
                }
                write!(out, "],\"chosen\":{chosen}").unwrap();
            }
            Tail::Advance { from, to } => write!(
                out,
                ",\"from\":{},\"to\":{}",
                from.as_nanos(),
                to.as_nanos()
            )
            .unwrap(),
            Tail::Channel(event) => match event {
                ChannelEvent::New(bound) => match bound {
                    Some(bound) => write!(out, ",\"bound\":{bound}").unwrap(),
                    None => out.push_str(",\"bound\":null"),
                },
                ChannelEvent::Result { result, until, .. } => {
                    if let Some(until) = until {
                        write!(out, ",\"until\":{}", until.as_nanos()).unwrap();
                    }
                    out.push_str(",\"result\":");
                    quoted(out, result);
                }
                ChannelEvent::Close { end, senders } => {
                    out.push_str(",\"end\":");
                    quoted(out, end);
                    if let Some(senders) = senders {
                        write!(out, ",\"senders\":{senders}").unwrap();
                    }
                }
            },
            Tail::Sink { stream, message } => {
                out.push_str(",\"stream\":");
                quoted(out, stream);
                out.push_str(",\"message\":");
                quoted(out, &String::from_utf8_lossy(message));
                // Preserve raw bytes as well: lossy display must not erase replay differences.
                write!(out, ",\"bytes\":{:?}", &message[..]).unwrap();
            }
        }
        out.push_str("}\n");
    }
}

/// A simulation's trace as data: NDJSON with one line per record, written on demand.
#[derive(Default)]
pub(crate) struct Trace(Vec<Record>);

fn render(records: &[Record]) -> String {
    let mut out = String::new();
    for record in records {
        record.render(&mut out);
    }
    out
}

impl Trace {
    pub(crate) fn append(&mut self, other: Self) { self.0.extend(other.0); }

    pub(crate) fn render(&self) -> String {
        render(&self.0)
    }

    /// The first line, numbered from 1, whose text differs between two traces, with both lines
    /// (`<end of trace>` past either end). Records are compared first; text only where they
    /// differ, so the answer is the one a comparison of the rendered traces gives.
    pub(crate) fn first_difference(&self, other: &Self) -> Option<(usize, String, String)> {
        let line = |record: Option<&Record>| {
            record.map_or_else(
                || "<end of trace>".to_owned(),
                |record| {
                    let mut out = String::new();
                    record.render(&mut out);
                    out
                },
            )
        };
        let (mut first, mut second) = (self.0.iter(), other.0.iter());
        for number in 1.. {
            match (first.next(), second.next()) {
                (None, None) => return None,
                (a, b) if a == b => {}
                (a, b) => {
                    let (a, b) = (line(a), line(b));
                    if a != b {
                        return Some((number, a, b));
                    }
                }
            }
        }
        unreachable!()
    }
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
    cells: Vec<usize>,
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
    pub(super) main_origin: Option<crate::test::Source>,
    next_cell: usize,
    next_limit: usize,
    next_channel: usize,
    next_arc: usize,
    rng: u64,
    fs_rng: u64,
    net_rng: u64,
    step: usize,
    events: Vec<Record>,
    sink_events: Vec<(&'static str, Vec<u8>, Duration)>,
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
                cells: Vec::new(),
            }],
            current: Some(0),
            main_origin: None,
            next_cell: 0,
            next_limit: 0,
            next_channel: 0,
            next_arc: 0,
            rng: seed ^ 0x7469_6465_7363_6864,     // "tideschd"
            fs_rng: seed ^ 0x7469_6465_6669_6c65,  // "tidefile"
            net_rng: seed ^ 0x7469_6465_6e65_7421, // "tidenet!"
            step: 0,
            events: Vec::new(),
            sink_events: Vec::new(),
            now: Duration::ZERO,
            timers: BTreeMap::new(),
            next_timer: 0,
            stalled: 0,
            stall_limit: STALL_LIMIT,
        }
    }

    pub(super) fn cell_set(&mut self, task: TaskId, ids: &[usize]) {
        self.tasks[task].cells.clear();
        self.tasks[task].cells.extend_from_slice(ids);
    }

    // Cell records within a set share one trace step; only the set boundaries schedule.
    pub(super) fn cells(
        &mut self,
        task: TaskId,
        ids: impl IntoIterator<Item = usize>,
        acquire: bool,
        span: Span,
    ) {
        let step = self.step;
        let mut recorded = false;
        for id in ids {
            recorded = true;
            let operation = if acquire {
                Operation::CellAcquire(id)
            } else {
                Operation::CellRelease(id)
            };
            self.event(task, Pending { operation, span }, "cell", None, Tail::None);
            self.step = step;
        }
        if recorded {
            self.step += 1;
        }
    }

    /// The trace as NDJSON text.
    pub(super) fn trace(&self) -> String {
        render(&self.events)
    }

    pub(super) fn take_trace(&mut self) -> Trace {
        Trace(std::mem::take(&mut self.events))
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
            cells: Vec::new(),
        });
        id
    }

    pub(super) fn allocate_cell(&mut self) -> usize {
        let id = self.next_cell;
        self.next_cell += 1;
        id
    }

    pub(super) fn allocate_arc(&mut self) -> usize {
        let id = self.next_arc;
        self.next_arc += 1;
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
        self.event(
            task,
            pending,
            pending.operation.kind(),
            Some("ready"),
            Tail::None,
        );
        if !self.has_operation_timer() {
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
        self.event(
            task,
            pending,
            pending.operation.kind(),
            Some("blocked"),
            Tail::None,
        );
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
            if !self.has_operation_timer() {
                self.stalled = 0;
            }
        }
        self.tasks[task].state = State::Ready(Some(pending));
        self.event(
            task,
            pending,
            pending.operation.kind(),
            Some("ready"),
            Tail::Text(",\"action\":\"wake\""),
        );
    }

    pub(super) fn cancel(&mut self, task: TaskId, scope: usize, span: Span, cause: Cause) {
        let pending = Pending {
            operation: match cause {
                Cause::Failure(_) => Operation::Failure(scope),
                Cause::Explicit(_) => Operation::Cancel(scope),
                Cause::Deadline(_) => unreachable!("deadline events include their instant"),
            },
            span,
        };
        self.event(
            task,
            pending,
            "cancel",
            None,
            Tail::Text(",\"action\":\"cancelled\""),
        );
    }

    pub(super) fn finish(&mut self, task: TaskId, outcome: Outcome) -> Pending {
        self.tasks[task].state = State::Finished;
        let pending = Pending {
            operation: Operation::Exit,
            span: self.tasks[task].spawn_span,
        };
        let outcome = match outcome {
            Outcome::Ok => ",\"outcome\":\"ok\"",
            Outcome::Error => ",\"outcome\":\"error\"",
            Outcome::Echo => ",\"outcome\":\"echo\"",
            Outcome::Deadline => ",\"outcome\":\"deadline\"",
            Outcome::Panic => ",\"outcome\":\"panic\"",
        };
        self.event(task, pending, "exit", None, Tail::Text(outcome));
        pending
    }

    pub(super) fn finish_isolated(&mut self, task: TaskId, outcome: Outcome) -> Pending {
        self.tasks[task].state = State::Finished;
        let pending = Pending {
            operation: Operation::Exit,
            span: self.tasks[task].spawn_span,
        };
        let result = if outcome == Outcome::Ok {
            ",\"isolated\":true,\"result\":\"ok\""
        } else {
            ",\"isolated\":true,\"result\":\"panicked\""
        };
        self.event(task, pending, "exit", None, Tail::Text(result));
        pending
    }

    pub(super) fn sink(&mut self, task: TaskId, stream: &'static str, message: &[u8], span: Span) {
        self.event(
            task,
            Pending {
                operation: Operation::Sink,
                span,
            },
            "sink",
            None,
            Tail::Sink {
                stream,
                message: message.to_vec(),
            },
        );
        self.sink_events.push((stream, message.to_vec(), self.now));
    }

    pub(super) fn records(&self) -> Vec<crate::sim::Record> {
        use crate::sim::{Record, Sink};
        #[cfg(test)]
        let mut scanned = 0;
        let mut records = Vec::new();
        let mut pending: Vec<(&str, Vec<u8>, Duration)> = Vec::new();
        for (stream, bytes, time) in &self.sink_events {
            let i = pending.iter().position(|(s, _, _)| s == stream).unwrap_or_else(|| {
                pending.push((stream, Vec::new(), *time));
                pending.len() - 1
            });
            let (_, buf, last_time) = &mut pending[i];
            let previous_len = buf.len();
            buf.extend(bytes);
            *last_time = *time;
            if !matches!(*stream, "out" | "err" | "dbg") { buf.push(b'\n'); }
            let mut consumed = 0;
            for (offset, byte) in buf[previous_len..].iter().enumerate() {
                #[cfg(test)]
                { scanned += 1; }
                if *byte != b'\n' { continue; }
                let end = previous_len + offset;
                records.push(Record {
                    node: "main".into(), time: *time,
                    sink: match *stream {
                        "out" => Sink::Out, "err" => Sink::Err, "dbg" => Sink::Dbg,
                        name => Sink::Log(crate::log::Level::from_name(name)),
                    },
                    message: String::from_utf8_lossy(&buf[consumed..end]).into_owned(),
                });
                consumed = end + 1;
            }
            buf.drain(..consumed);
        }
        #[cfg(test)]
        assert_eq!(scanned, self.sink_events.iter().map(|(stream, bytes, _)| {
            bytes.len() + usize::from(!matches!(*stream, "out" | "err" | "dbg"))
        }).sum::<usize>(), "line assembly must scan each appended byte exactly once");
        for (stream, bytes, time) in pending {
            if !bytes.is_empty() {
                records.push(Record {
                    node: "main".into(), time,
                    sink: match stream { "out" => Sink::Out, "err" => Sink::Err, "dbg" => Sink::Dbg, name => Sink::Log(crate::log::Level::from_name(name)) },
                    message: String::from_utf8_lossy(&bytes).into_owned(),
                });
            }
        }
        records
    }

    /// Records what a blocking capability operation returned.
    pub(super) fn result(&mut self, task: TaskId, pending: Pending, ok: bool) {
        let result = if ok {
            ",\"result\":\"ok\""
        } else {
            ",\"result\":\"cancelled\""
        };
        self.event(
            task,
            pending,
            pending.operation.kind(),
            None,
            Tail::Text(result),
        );
    }

    /// The global deadlock report (03 §9.3): every blocked task, what it waits for, and the span
    /// of its blocking call. The same list is the trace's last event.
    pub(super) fn deadlock(&mut self, task: TaskId, pending: Pending) -> String {
        let blocked: Vec<_> = self.blocked().collect();
        let pending = Pending {
            operation: Operation::Deadlock,
            span: pending.span,
        };
        let mut list = String::from(",\"blocked\":[");
        let mut report = format!(
            "deadlock: no runnable task in the simulator and no timer pending (virtual time {:?})",
            self.now
        );
        for (index, (id, wait)) in blocked.into_iter().enumerate() {
            let comma = if index == 0 { "" } else { "," };
            write!(list, "{comma}{{\"task\":{id}").unwrap();
            operation(&mut list, wait, &self.tasks[id].cells, None);
            list.push('}');
            write!(
                report,
                "\n  task {id} blocked in {} at {}",
                wait.operation, wait.span
            )
            .unwrap();
        }
        list.push(']');
        self.event(task, pending, "sched", None, Tail::Owned(list));
        report
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
                self.event(
                    task,
                    pending,
                    "sched",
                    wait,
                    Tail::Sched { runnable, chosen },
                );
                Some(chosen)
            }
        };
        self.current
    }

    pub(super) fn allocate_channel(&mut self) -> usize {
        let id = self.next_channel;
        self.next_channel += 1;
        id
    }

    pub(super) fn channel(&mut self, task: TaskId, id: usize, span: Span, event: ChannelEvent) {
        let name = match event {
            ChannelEvent::New(_) => "new",
            ChannelEvent::Result { name, .. } => name,
            ChannelEvent::Close { .. } => "close",
        };
        self.event(
            task,
            Pending {
                operation: Operation::Channel(name, id),
                span,
            },
            "chan",
            None,
            Tail::Channel(event),
        );
    }

    pub(super) fn allocate_limit(&mut self) -> usize {
        let id = self.next_limit;
        self.next_limit += 1;
        id
    }

    pub(super) fn has_operation_timer(&self) -> bool {
        self.timers.keys().any(|key| key.1 == OPERATION_TIMER)
    }

    pub(super) fn deadline(&mut self, scope: usize, old: Option<Duration>, at: Option<Duration>) {
        if let Some(old) = old {
            self.timers.remove(&(old, 0, scope));
        }
        if let Some(at) = at {
            self.timers.insert((at, 0, scope), scope);
        }
    }

    // Runtime fires the timers in key order before choosing a runnable task.
    pub(super) fn advance(&mut self, task: TaskId, pending: Pending) -> bool {
        let Some((&(to, _, _), _)) = self.timers.first_key_value() else {
            return false;
        };
        assert!(to > self.now, "a timer was registered in the past");
        self.event(
            task,
            Pending {
                operation: Operation::Advance,
                span: pending.span,
            },
            "sched",
            None,
            Tail::Advance { from: self.now, to },
        );
        self.now = to;
        self.stalled = 0;
        true
    }

    pub(super) fn fire_timer(&mut self) -> Option<(u8, usize)> {
        let (&key, &id) = self
            .timers
            .first_key_value()
            .filter(|(key, _)| key.0 == self.now)?;
        if key.1 == 0 {
            self.timers.remove(&key);
        } else {
            let State::Blocked(wait) = self.tasks[id].state else {
                unreachable!("a timer outlived its wait")
            };
            self.wake(id, wait);
        }
        Some((key.1, id))
    }

    pub(super) fn event_result(&mut self, task: TaskId, pending: Pending, result: &str) {
        let mut tail = String::from(",\"result\":");
        quoted(&mut tail, result);
        self.event(
            task,
            pending,
            pending.operation.kind(),
            None,
            Tail::Owned(tail),
        );
    }

    fn index(&mut self, bound: u64) -> u64 {
        index(&mut self.rng, bound)
    }

    pub(super) fn net_draw(&mut self, bound: u64) -> u64 {
        index(&mut self.net_rng, bound)
    }

    pub(super) fn net_event(
        &mut self,
        task: TaskId,
        name: &'static str,
        id: usize,
        span: Span,
        detail: &str,
    ) {
        let mut tail = String::from(",\"net\":");
        quoted(&mut tail, detail);
        self.event(
            task,
            Pending {
                operation: Operation::Net(name, id),
                span,
            },
            "cap",
            None,
            Tail::Owned(tail),
        );
    }

    pub(super) fn file_latency(&mut self) -> Duration {
        Duration::from_nanos(index(&mut self.fs_rng, 1_000_001))
    }

    pub(super) fn file_event(
        &mut self,
        task: TaskId,
        operation: Operation,
        span: Span,
        path: &std::path::Path,
        latency: Duration,
        result: &str,
    ) {
        let kind = if operation == Operation::WorldFile {
            "world"
        } else {
            "fs"
        };
        let mut tail = String::from(",\"path\":");
        quoted(&mut tail, &path.to_string_lossy());
        write!(tail, ",\"latency\":{},\"result\":", latency.as_nanos()).unwrap();
        quoted(&mut tail, result);
        self.event(
            task,
            Pending { operation, span },
            kind,
            None,
            Tail::Owned(tail),
        );
    }

    fn event(
        &mut self,
        task: TaskId,
        pending: Pending,
        kind: &'static str,
        wait: Option<&'static str>,
        tail: Tail,
    ) {
        let targets = (pending.operation == Operation::CellsAcquire)
            .then(|| self.tasks[task].cells.clone().into_boxed_slice());
        self.events.push(Record {
            step: self.step,
            time: self.now,
            task,
            kind,
            pending,
            origin: (task == 0 && pending.operation == Operation::Exit).then_some(self.main_origin).flatten(),
            targets,
            wait,
            tail,
        });
        self.step += 1;
    }
}

fn operation(out: &mut String, pending: Pending, cells: &[usize], origin: Option<crate::test::Source>) {
    let (operation, target) = pending.operation.name();
    write!(out, ",\"operation\":\"{operation}\"").unwrap();
    if let Some(target) = target {
        write!(out, ",\"target\":{target}").unwrap();
    }
    if pending.operation == Operation::CellsAcquire {
        write!(out, ",\"targets\":{cells:?}").unwrap();
    }
    out.push_str(",\"span\":{\"file\":");
    quoted(out, origin.map_or(pending.span.file(), |s| s.file));
    write!(
        out,
        ",\"line\":{},\"column\":{}}}",
        origin.map_or(pending.span.line(), |s| s.start.0),
        origin.map_or(pending.span.column(), |s| s.start.1),
    )
    .unwrap();
}

// Rejection avoids giving lower indices extra weight when 2^64 is not divisible.
fn index(rng: &mut u64, bound: u64) -> u64 {
    let threshold = bound.wrapping_neg() % bound;
    loop {
        *rng = rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let z = *rng;
        let z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let draw = z ^ (z >> 31);
        if draw >= threshold {
            return draw % bound;
        }
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
            Self::Cancel(id) => ("explicit", Some(id)),
            Self::Failure(id) => ("failure", Some(id)),
            Self::Deadline(id, _) => ("deadline", Some(id)),
            Self::SetDeadline(id, _) => ("set_deadline", Some(id)),
            Self::Limit(id) => ("limit", Some(id)),
            Self::Isolated(id) => ("isolated", Some(id)),
            Self::Checkpoint => ("checkpoint", None),
            Self::Yield => ("yield_now", None),
            Self::CellNew(id) => ("cell_new", Some(id)),
            Self::CellsAcquire => ("cells_acquire", None),
            Self::CellsRelease => ("cells_release", None),
            Self::CellAcquire(id) => ("cell_acquire", Some(id)),
            Self::CellRelease(id) => ("cell_release", Some(id)),
            Self::ClockNow => ("clock.now", None),
            Self::ClockSystemTime(_) => ("clock.system_time", None),
            Self::Sleep { name, .. } => (name, None),
            Self::Advance => ("advance", None),
            Self::Deadlock => ("deadlock", None),
            Self::Exit => ("exit", None),
            Self::Sink => ("sink", None),
            Self::File(name) => (name, None),
            Self::WorldFile => ("world.file", None),
            Self::Net(name, id) => (name, Some(id)),
            Self::Input(name) => (name, None),
            Self::Channel(name, id) => (name, Some(id)),
            Self::Arc(name, id) => (name, Some(id)),
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Self::CellNew(_)
            | Self::CellAcquire(_)
            | Self::CellRelease(_)
            | Self::CellsAcquire
            | Self::CellsRelease => "cell",
            Self::ClockNow
            | Self::ClockSystemTime(_)
            | Self::Sleep { .. }
            | Self::Input(_)
            | Self::Net(..) => "cap",
            Self::File(_) => "io_wait",
            Self::Channel(..) => "chan",
            Self::Arc(..) => "arc",
            Self::Cancel(_) | Self::Failure(_) | Self::Deadline(..) | Self::SetDeadline(..) => {
                "cancel"
            }
            Self::Limit(_) => "spawn",
            Self::Isolated(_) => "spawn",
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
            if matches!(self, Self::Channel(..)) {
                write!(f, "(chan {target})")?;
            } else if let Self::Net(name, _) = self {
                let resource = if matches!(*name, "net.accept" | "net.connect") {
                    "port"
                } else {
                    "conn"
                };
                write!(f, "({resource} {target})")?;
            } else {
                write!(f, "({target})")?;
            }
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
    fn fragmented_sink_records_scan_each_byte_once() {
        use crate::sim::{Record, Sink};
        fn event(stream: &'static str, bytes: &[u8], time: u64) -> (&'static str, Vec<u8>, Duration) {
            (stream, bytes.to_vec(), Duration::from_secs(time))
        }
        // records() counts actual byte comparisons in test builds and asserts the exact
        // linear bound, including implicit log newlines; doubling fragments cannot rescan tails.
        for fragments in [2048, 4096] {
            let mut scheduler = Scheduler::new(0, std::panic::Location::caller());
            for _ in 0..fragments {
                scheduler.sink_events.push(event("out", b"x", 0));
            }
            scheduler.sink_events.extend([
                event("err", b"err\n", 1),
                event("out", b"\xc3", 2),
                event("out", b"\xa9\nA\n\xff", 3),
                event("debug", b"note\n", 4),
                event("err", b"tail", 5),
            ]);
            let expected: Vec<_> = [
                (Sink::Err, "err".into(), 1),
                (Sink::Out, format!("{}é", "x".repeat(fragments)), 3),
                (Sink::Out, "A".into(), 3),
                (Sink::Log(crate::log::Level::Debug), "note".into(), 4),
                (Sink::Log(crate::log::Level::Debug), "".into(), 4),
                (Sink::Out, "�".into(), 3),
                (Sink::Err, "tail".into(), 5),
            ].into_iter().map(|(sink, message, time)| Record {
                node: "main".into(), time: Duration::from_secs(time), sink, message,
            }).collect();
            assert_eq!(scheduler.records(), expected);
            assert_eq!(scheduler.records(), expected);
        }
    }

    #[test]
    fn trace_difference_names_the_first_changed_or_missing_line() {
        let span = std::panic::Location::caller();
        let yielding = Pending {
            operation: Operation::Yield,
            span,
        };
        let run = |values: &[&[u8]]| {
            let mut scheduler = Scheduler::new(0, span);
            for value in values {
                scheduler.point(0, yielding).unwrap();
                scheduler.sink(0, "out", value, span);
            }
            let text = scheduler.trace();
            (scheduler.take_trace(), text)
        };
        let (base, text) = run(&[b"a", b"b"]);
        let lines: Vec<_> = text.split_inclusive('\n').map(str::to_owned).collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(base.first_difference(&run(&[b"a", b"b"]).0), None);
        let (changed, changed_text) = run(&[b"a", b"c"]);
        let expected = changed_text
            .split_inclusive('\n')
            .nth(3)
            .unwrap()
            .to_owned();
        assert_eq!(
            base.first_difference(&changed),
            Some((4, lines[3].clone(), expected))
        );
        let end = || "<end of trace>".to_owned();
        let (short, _) = run(&[b"a"]);
        assert_eq!(
            short.first_difference(&base),
            Some((3, end(), lines[2].clone()))
        );
        assert_eq!(
            base.first_difference(&short),
            Some((3, lines[2].clone(), end()))
        );
        assert_eq!(Trace::default().first_difference(&Trace::default()), None);
    }

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
        scheduler.cancel(0, 0, span, Cause::Explicit(0));
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
            let trace = scheduler.trace();
            let mut events = trace.lines();
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
            assert!(!scheduler.trace().contains("\"kind\":\"sched\""));
            let report = scheduler.deadlock(second, second_wait);
            assert_eq!(report.lines().count(), 3, "{report}");
            for (task, wait) in [(0, first_wait), (second, second_wait)] {
                let line = format!(
                    "\n  task {task} blocked in {} at {}",
                    wait.operation, wait.span
                );
                assert!(report.contains(&line), "{report}");
            }
            let trace = scheduler.trace();
            let last = trace.lines().last().unwrap();
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
                trace
                    .lines()
                    .filter(|line| line.contains("\"wait\":\"blocked\""))
                    .count(),
                2
            );
        }
    }
}

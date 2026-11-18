// `sim.world.proc` models (03 §9.9 lifecycle subset; research/40 §6). A model is owned,
// declarative data: which command shapes it accepts and an ordered list of steps that consume
// input and timers and emit output, endpoint closure and exit. It has no callbacks and cannot
// reach the host. The builder spelling is provisional.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A declared simulated program. Accepts any arguments, environment, directory and supported
/// stdio unless restricted; with no `exit` step it exits with code 0 after its last step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcessModel {
    pub(crate) missing: bool,
    pub(crate) args: Option<Vec<OsString>>,
    pub(crate) env: Vec<(OsString, OsString)>,
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) steps: Vec<Step>,
    pub(crate) ignores_sigpipe: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    /// Consumes exactly these stdin bytes, in any chunking.
    Read(Vec<u8>),
    /// Waits for stdin to close with no further bytes.
    ReadEof,
    /// Copies stdin to stdout until stdin closes.
    Copy,
    /// Writes to stdout (1) or stderr (2).
    Write(u8, Vec<u8>),
    /// Closes the model's end of stdin (0), stdout (1) or stderr (2).
    Close(u8),
    /// Waits a delay drawn from the process sub-stream within these bounds.
    Delay(Duration, Duration),
    /// Continues with one alternative's steps, chosen by the process sub-stream.
    Choose(Vec<Vec<Step>>),
    Exit(i32),
}

impl ProcessModel {
    pub fn new() -> Self {
        Self::default()
    }
    /// A program that does not exist: spawning it returns `NotFound` and creates no instance.
    pub fn missing() -> Self {
        Self { missing: true, ..Self::default() }
    }
    /// Accepts only these arguments after argv[0].
    pub fn args(mut self, args: impl IntoIterator<Item: AsRef<OsStr>>) -> Self {
        self.args = Some(args.into_iter().map(|a| a.as_ref().to_owned()).collect());
        self
    }
    /// Accepts only a child environment in which `key` is `value`.
    pub fn env(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.env.push((key.as_ref().to_owned(), value.as_ref().to_owned()));
        self
    }
    /// Accepts only this working directory (simulated cwd is `/` when unset).
    pub fn current_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.cwd = Some(dir.as_ref().to_owned());
        self
    }
    pub fn read(self, bytes: impl AsRef<[u8]>) -> Self {
        self.step(Step::Read(bytes.as_ref().to_vec()))
    }
    pub fn read_to_end(self) -> Self {
        self.step(Step::ReadEof)
    }
    pub fn copy(self) -> Self {
        self.step(Step::Copy)
    }
    pub fn write(self, bytes: impl AsRef<[u8]>) -> Self {
        self.step(Step::Write(1, bytes.as_ref().to_vec()))
    }
    pub fn write_err(self, bytes: impl AsRef<[u8]>) -> Self {
        self.step(Step::Write(2, bytes.as_ref().to_vec()))
    }
    pub fn close_stdin(self) -> Self {
        self.step(Step::Close(0))
    }
    pub fn close_stdout(self) -> Self {
        self.step(Step::Close(1))
    }
    pub fn close_stderr(self) -> Self {
        self.step(Step::Close(2))
    }
    pub fn delay(self, min: Duration, max: Duration) -> Self {
        self.step(Step::Delay(min, max))
    }
    /// Continues with one of these models' steps; their other settings must be unset.
    pub fn choose(self, alternatives: impl IntoIterator<Item = ProcessModel>) -> Self {
        let alternatives = alternatives
            .into_iter()
            .map(|m| {
                assert!(
                    m == (ProcessModel { steps: m.steps.clone(), ..ProcessModel::default() }),
                    "a model alternative may only declare steps"
                );
                m.steps
            })
            .collect();
        self.step(Step::Choose(alternatives))
    }
    /// A write to a closed stdout or stderr fails with `BrokenPipe` and the model continues,
    /// as a program that ignores SIGPIPE does; by default it dies with no code (SIGPIPE).
    pub fn ignore_sigpipe(mut self) -> Self {
        self.ignores_sigpipe = true;
        self
    }
    /// Exits with `code` (0..=255).
    pub fn exit(self, code: i32) -> Self {
        self.step(Step::Exit(code))
    }
    fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    /// Panics on a malformed model: steps on `missing`, a step after an exit, reversed delay
    /// bounds, an empty choice, or an exit code outside 0..=255.
    pub(crate) fn validate(&self) {
        assert!(!self.missing || *self == Self::missing(), "a missing program declares nothing else");
        fn steps(list: &[Step]) {
            for (i, step) in list.iter().enumerate() {
                match step {
                    Step::Exit(code) => {
                        assert!((0..=255).contains(code), "exit code {code} is outside 0..=255");
                        assert!(i + 1 == list.len(), "a model step follows its exit");
                    }
                    Step::Delay(min, max) => assert!(min <= max, "reversed delay bounds"),
                    Step::Choose(alternatives) => {
                        assert!(!alternatives.is_empty(), "an empty choice");
                        alternatives.iter().for_each(|a| steps(a));
                    }
                    _ => {}
                }
            }
        }
        steps(&self.steps);
    }
}

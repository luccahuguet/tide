//! Process arguments and environment, reached only through `Env` (03 §1.1).
use crate::io;
use crate::task::Tasks;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
pub use std::env::VarError;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// Owned simulation inputs, shared by every capability in one simulated process.
#[derive(Default)]
pub(crate) struct Inputs {
    pub(crate) args: Vec<OsString>,
    pub(crate) env: BTreeMap<OsString, OsString>,
    pub(crate) stdin: VecDeque<u8>,
}

/// Authority to read process inputs. Clones retain all attenuation.
#[derive(Clone)]
pub struct Env {
    tasks: Option<Tasks>,
    only: Option<BTreeSet<OsString>>,
    cwd: bool,
}
impl Env {
    pub(crate) fn new(tasks: Option<Tasks>) -> Self {
        Self {
            tasks,
            only: None,
            cwd: true,
        }
    }

    fn allowed(&self, key: &OsStr) -> bool {
        self.only.as_ref().is_none_or(|keys| keys.contains(key))
    }

    #[track_caller]
    fn read<T>(
        &self,
        name: &'static str,
        sim: impl FnOnce(&mut Inputs) -> T,
        real: impl FnOnce() -> T,
    ) -> T {
        match &self.tasks {
            Some(tasks) => tasks.input_op(name, sim),
            None => {
                crate::task::outside_simulation();
                real()
            }
        }
    }

    /// Arguments include the program name when supplied. Non-Unicode items panic on iteration.
    #[track_caller]
    pub fn args(&self) -> Args {
        Args(
            self.read(
                "env.args",
                |inputs| inputs.args.clone(),
                || std::env::args_os().collect(),
            )
            .into_iter(),
        )
    }

    #[track_caller]
    pub fn var(&self, key: impl AsRef<OsStr>) -> Result<String, VarError> {
        let key = key.as_ref();
        self.read(
            "env.var",
            |inputs| {
                if self.allowed(key) {
                    inputs.env.get(key).cloned()
                } else {
                    None
                }
            },
            || {
                if self.allowed(key) {
                    std::env::var_os(key)
                } else {
                    None
                }
            },
        )
        .ok_or(VarError::NotPresent)?
        .into_string()
        .map_err(VarError::NotUnicode)
    }

    /// A snapshot; non-Unicode keys or values panic at the offending iteration item.
    #[track_caller]
    pub fn vars(&self) -> Vars {
        let vars = self.read(
            "env.vars",
            |inputs| {
                inputs
                    .env
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<_>>()
            },
            || std::env::vars_os().collect(),
        );
        Vars(
            vars.into_iter()
                .filter(|(k, _)| self.allowed(k))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    #[track_caller]
    pub fn current_dir(&self) -> io::Result<PathBuf> {
        let denied = || Err(io::ErrorKind::PermissionDenied.into());
        self.read(
            "env.current_dir",
            |_| {
                if self.cwd {
                    Ok(PathBuf::from("/"))
                } else {
                    denied()
                }
            },
            || {
                if self.cwd {
                    std::env::current_dir().map_err(io::from_std)
                } else {
                    denied()
                }
            },
        )
    }

    /// Narrows variables by exact name (v0 Linux). Repeated calls intersect.
    pub fn only(&self, keys: impl IntoIterator<Item: AsRef<OsStr>>) -> Self {
        let only = keys
            .into_iter()
            .map(|k| k.as_ref().to_owned())
            .filter(|k| self.allowed(k))
            .collect();
        Self {
            only: Some(only),
            ..self.clone()
        }
    }
    /// Keeps arguments, denying variables and the working directory.
    pub fn args_only(&self) -> Self {
        Self {
            only: Some(BTreeSet::new()),
            cwd: false,
            ..self.clone()
        }
    }
}

#[derive(Debug)]
pub struct Args(std::vec::IntoIter<OsString>);
fn unicode(value: OsString) -> String {
    value.into_string().unwrap()
}
impl Iterator for Args {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.0.next().map(unicode)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<String> {
        self.0.next_back().map(unicode)
    }
}
impl ExactSizeIterator for Args {}

#[derive(Debug)]
pub struct Vars(std::vec::IntoIter<(OsString, OsString)>);
fn pair((key, value): (OsString, OsString)) -> (String, String) {
    (unicode(key), unicode(value))
}
impl Iterator for Vars {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(pair)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl DoubleEndedIterator for Vars {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(pair)
    }
}
impl ExactSizeIterator for Vars {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn simulated_os_inputs_keep_nonunicode_until_consumed() {
        let sim = crate::Root::sim(2);
        let bad = OsString::from_vec(vec![0xff]);
        sim.world.args([OsString::from("valid"), bad.clone()]);
        sim.world.env_var("BAD", &bad);
        let mut args = sim.env.args();
        assert_eq!(args.next().unwrap(), "valid");
        assert!(catch_unwind(AssertUnwindSafe(|| args.next_back())).is_err());
        assert_eq!(sim.env.var("BAD"), Err(VarError::NotUnicode(bad)));
        let mut vars = sim.env.vars();
        assert!(catch_unwind(AssertUnwindSafe(|| vars.next())).is_err());
        assert_eq!(sim.env.only(["missing"]).vars().len(), 0);
        assert_eq!(sim.elapsed(), crate::time::Duration::ZERO);
        drop(sim);
        // Reject a real handle used inside a simulation, and a stale simulated handle.
        let real = Env::new(None);
        let env = {
            let sim = crate::Root::sim(1);
            assert!(catch_unwind(AssertUnwindSafe(|| real.args())).is_err());
            sim.env.clone()
        };
        assert!(catch_unwind(AssertUnwindSafe(|| env.var("BAD"))).is_err());
    }
}

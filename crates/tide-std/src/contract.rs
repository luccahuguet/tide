//! Runtime and definition markers for executable clauses (04 §1.8).
//! Hidden compiler/runner interface, shared verbatim with the conformance fixture.
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;
use std::panic::{AssertUnwindSafe, Location, catch_unwind, panic_any};

pub const STEP_BUDGET: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Requires,
    Ensures,
    Invariant,
}
impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Requires => "requires",
            Self::Ensures => "ensures",
            Self::Invariant => "invariant",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clause {
    pub kind: Kind,
    pub text: &'static str,
    pub function: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub index: usize,
    pub conjunct: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub clause: Clause,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unchecked {
    pub clause: Clause,
    pub reason: &'static str,
}

type ClauseId = (&'static str, u32, u32, usize, usize);

struct Budget {
    remaining: usize,
    exhausted: bool,
}
thread_local! {
    static VIOLATION: RefCell<Option<Violation>> = const { RefCell::new(None) };
    static UNCHECKED: RefCell<(Vec<Unchecked>, HashSet<ClauseId>)> = RefCell::new((Vec::new(), HashSet::new()));
    static BUDGETS: RefCell<Vec<Budget>> = const { RefCell::new(Vec::new()) };
}

/// Current OS thread only: child-task panic propagation does not transport these records.
/// Runners consume before each case and treat missing child metadata as an ordinary failure.
pub fn take_violation() -> Option<Violation> {
    VIOLATION.with(|v| v.borrow_mut().take())
}
/// First-seen order, current OS thread only; child records are not collected by a parent.
pub fn take_unchecked() -> Vec<Unchecked> {
    UNCHECKED.with(|v| {
        let (records, seen) = &mut *v.borrow_mut();
        seen.clear();
        std::mem::take(records)
    })
}

/// Type-directed old snapshots: references must clone their referent, never the reference.
pub struct Old<'a, T: ?Sized>(pub &'a T);
pub trait RefOld<T: ?Sized> {
    fn snapshot(self) -> T
    where
        T: Clone;
}
impl<T: ?Sized> RefOld<T> for &&Old<'_, &T> {
    fn snapshot(self) -> T
    where
        T: Clone,
    {
        (*self.0).clone()
    }
}
impl<T: ?Sized> RefOld<T> for &&Old<'_, &mut T> {
    fn snapshot(self) -> T
    where
        T: Clone,
    {
        (**self.0).clone()
    }
}
pub trait ValueOld {
    type Value;
    fn snapshot(self) -> Self::Value;
}
impl<T: Clone> ValueOld for &Old<'_, T> {
    type Value = T;
    fn snapshot(self) -> T {
        self.0.clone()
    }
}
pub fn unchecked(clause: Clause, reason: &'static str) {
    UNCHECKED.with(|v| {
        let id = (
            clause.file,
            clause.line,
            clause.column,
            clause.index,
            clause.conjunct,
        );
        let (records, seen) = &mut *v.borrow_mut();
        if seen.insert(id) {
            records.push(Unchecked { clause, reason });
        }
    });
}

#[track_caller]
fn violated(clause: Clause, panic: Option<Box<dyn std::any::Any + Send>>) -> ! {
    let caller = Location::caller();
    let (file, line, column) = if clause.kind == Kind::Requires {
        (caller.file(), caller.line(), caller.column())
    } else {
        (clause.file, clause.line, clause.column)
    };
    VIOLATION.with(|v| {
        *v.borrow_mut() = Some(Violation {
            clause,
            file,
            line,
            column,
        })
    });
    let mut message = format!(
        "{} violated: {} (in `{}` at {}:{}:{})",
        clause.kind.name(),
        clause.text,
        clause.function,
        file,
        line,
        column
    );
    if let Some(panic) = panic {
        let cause = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("non-string panic");
        message.push_str(&format!(" (clause panicked: {cause})"));
    }
    panic_any(message)
}

#[track_caller]
fn evaluate(clause: Clause, predicate: impl FnOnce() -> bool) {
    BUDGETS.with(|b| {
        b.borrow_mut().push(Budget {
            remaining: STEP_BUDGET,
            exhausted: false,
        })
    });
    let result = catch_unwind(AssertUnwindSafe(predicate));
    let budget = BUDGETS.with(|b| b.borrow_mut().pop().unwrap());
    if budget.exhausted {
        unchecked(clause, "step budget exceeded");
        return;
    }
    match result {
        Ok(true) => (),
        Ok(false) => violated(clause, None),
        Err(p) => violated(clause, Some(p)),
    }
}

// Separate callees let admission and clause hashing identify their closure arguments.
#[track_caller]
pub fn requires(clause: Clause, predicate: impl FnOnce() -> bool) {
    evaluate(clause, predicate);
}
#[track_caller]
pub fn ensures(clause: Clause, predicate: impl FnOnce() -> bool) {
    evaluate(clause, predicate);
}
/// Definition-identifiable ensures marker for const bodies, which cannot call `ensures`.
pub const fn const_ensures_marker() {}

#[track_caller]
pub fn invariant(clause: Clause, predicate: impl FnOnce() -> bool) {
    evaluate(clause, predicate);
}

#[track_caller]
pub fn old<T>(clause: Clause, snapshot: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(snapshot)) {
        Ok(v) => v,
        Err(p) => violated(clause, Some(p)),
    }
}
#[track_caller]
pub fn entry<T>(clause: Clause, snapshot: impl FnOnce() -> T) -> T {
    old(clause, snapshot)
}

fn step() -> bool {
    BUDGETS.with(|b| {
        let mut b = b.borrow_mut();
        let Some(b) = b.last_mut() else {
            return true;
        };
        if b.remaining == 0 {
            b.exhausted = true;
            false
        } else {
            b.remaining -= 1;
            true
        }
    })
}

pub fn forall<I: IntoIterator>(values: I, mut predicate: impl FnMut(I::Item) -> bool) -> bool {
    for value in values {
        if !step() {
            return true;
        }
        if !predicate(value) {
            return false;
        }
    }
    true
}
pub fn exists<I: IntoIterator>(values: I, mut predicate: impl FnMut(I::Item) -> bool) -> bool {
    for value in values {
        if !step() {
            return false;
        }
        if predicate(value) {
            return true;
        }
    }
    false
}
pub fn forall_in<'a, T: Clone + 'a, I: IntoIterator<Item = &'a T>>(
    values: I,
    mut predicate: impl FnMut(T) -> bool,
) -> bool {
    forall(values, |v| predicate(v.clone()))
}
pub fn exists_in<'a, T: Clone + 'a, I: IntoIterator<Item = &'a T>>(
    values: I,
    mut predicate: impl FnMut(T) -> bool,
) -> bool {
    exists(values, |v| predicate(v.clone()))
}

pub fn sampled_head(head: u64) -> bool {
    head <= 64 || head.is_power_of_two()
}

/// Contextualizes closure arguments, including results borrowing a reborrowed argument.
pub fn call<A, R>(body: impl FnOnce(A) -> R, arguments: A) -> R {
    body(arguments)
}

#[diagnostic::on_unimplemented(
    message = "const ensures needs Copy entry parameters; use a runtime wrapper for Clone or borrow-in-place snapshots"
)]
pub trait ConstEntry: Copy {}
impl<T: Copy> ConstEntry for T {}
pub const fn const_entry<T: ConstEntry>(value: &T) -> T {
    *value
}

pub struct Entry<'a, T: ?Sized>(pub &'a T);
impl<T: ?Sized> Copy for Entry<'_, T> {}
impl<T: ?Sized> Clone for Entry<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
pub struct Owned<T>(T);
impl<T> Deref for Owned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
// Autoref dispatch sees definition-site bounds; Copy wins over Clone, then a live borrow.
pub trait CopyEntry {
    type Output;
    fn snapshot(self) -> Self::Output;
}
pub trait CloneEntry {
    type Output;
    fn snapshot(self) -> Self::Output;
}
pub trait BorrowEntry {
    type Output;
    fn snapshot(self) -> Self::Output;
}
pub trait MutableAliasEntry {
    type Output;
    fn snapshot(self) -> Self::Output;
}
pub struct MutableAlias<T>(std::marker::PhantomData<fn() -> T>);
impl<T> Deref for MutableAlias<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unreachable!("mutable-reference aliases need explicit reborrow lowering")
    }
}
impl<'m, T: ?Sized> MutableAliasEntry for &&&Entry<'_, &'m mut T> {
    type Output = MutableAlias<&'m mut T>;
    fn snapshot(self) -> Self::Output {
        MutableAlias(std::marker::PhantomData)
    }
}
#[diagnostic::on_unimplemented(
    message = "mutable-reference alias in ensures is not supported; spell the parameter as &mut T so the contract macro can reborrow it"
)]
pub trait SupportedEntry {}
impl<T> SupportedEntry for Owned<T> {}
impl<T: ?Sized> SupportedEntry for &T {}
pub fn supported_entry<T: SupportedEntry>(_: &T) {}
impl<T: Copy> CopyEntry for &&Entry<'_, T> {
    type Output = Owned<T>;
    fn snapshot(self) -> Self::Output {
        Owned(*self.0)
    }
}
impl<T: Clone> CloneEntry for &Entry<'_, T> {
    type Output = Owned<T>;
    fn snapshot(self) -> Self::Output {
        Owned(self.0.clone())
    }
}
impl<'a, T: ?Sized> BorrowEntry for Entry<'a, T> {
    type Output = &'a T;
    fn snapshot(self) -> Self::Output {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CLAUSE: Clause = Clause {
        kind: Kind::Requires,
        text: "b > 0",
        function: "pair",
        file: "clause.tide",
        line: 7,
        column: 3,
        index: 2,
        conjunct: 1,
    };

    #[test]
    fn string_panics_keep_structured_blame_and_snapshot_cause() {
        take_violation();
        let line = line!() + 1;
        let panic = catch_unwind(|| requires(CLAUSE, || false)).unwrap_err();
        assert!(
            panic
                .downcast_ref::<String>()
                .unwrap()
                .starts_with("requires violated: b > 0")
        );
        let violation = take_violation().unwrap();
        assert_eq!(violation.clause, CLAUSE);
        assert_eq!(violation.line, line);
        assert!(violation.file.ends_with("contract.rs"));
        assert!(take_violation().is_none());
        let clause = Clause {
            kind: Kind::Ensures,
            ..CLAUSE
        };
        let panic = catch_unwind(|| old(clause, || panic!("snapshot failed"))).unwrap_err();
        assert!(
            panic
                .downcast_ref::<String>()
                .unwrap()
                .contains("(clause panicked: snapshot failed)")
        );
        assert_eq!(take_violation().unwrap().file, "clause.tide");
    }

    #[test]
    fn unchecked_deduplicates_by_source_identity_and_resets_on_take() {
        take_unchecked();
        unchecked(CLAUSE, "first");
        unchecked(
            Clause {
                text: "different text",
                ..CLAUSE
            },
            "second",
        );
        let other = Clause {
            conjunct: CLAUSE.conjunct + 1,
            ..CLAUSE
        };
        unchecked(other, "first");
        let records = take_unchecked();
        assert_eq!(
            records,
            vec![
                Unchecked {
                    clause: CLAUSE,
                    reason: "first"
                },
                Unchecked {
                    clause: other,
                    reason: "first"
                },
            ]
        );
        unchecked(CLAUSE, "after take");
        assert_eq!(take_unchecked()[0].reason, "after take");
        assert!(take_unchecked().is_empty());
    }

    #[test]
    fn quantifiers_short_circuit_share_budget_and_record_exhaustion() {
        take_unchecked();
        let mut visits = 0;
        ensures(CLAUSE, || {
            assert!(!forall(0..10, |_| {
                visits += 1;
                false
            }));
            assert!(exists(0..10, |_| {
                visits += 1;
                true
            }));
            assert!(forall(0..0, |_| false));
            assert!(!exists(0..0, |_| true));
            true
        });
        assert_eq!(visits, 2);
        ensures(CLAUSE, || forall(0..101, |_| forall(0..101, |_| true)));
        let records = take_unchecked();
        assert_eq!(
            records,
            vec![Unchecked {
                clause: CLAUSE,
                reason: "step budget exceeded"
            }]
        );
        ensures(CLAUSE, || forall(0..STEP_BUDGET, |_| true));
        assert!(take_unchecked().is_empty());
        assert!(sampled_head(64));
        assert!(!sampled_head(65));
        assert!(sampled_head(128));
    }
}

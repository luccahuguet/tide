/// Channels with cancellation-aware checkpoints on both runtimes.
pub mod mpsc {
    include!("sync/mpsc.rs");
}

use crate::marker::Pure;
use crate::task::{ArcHandle, Context, SimCell, Tasks};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc as RustArc, Condvar, Mutex, MutexGuard, PoisonError, TryLockError};

/// A reference-counted synchronization handle, created with task authority (GLOBAL-6).
/// Unlike Rust's `Arc`, it has no count observers, `get_mut`, `try_unwrap`, or `into_inner`.
/// Neither this handle nor its weak observers are `Pure`.
pub struct Arc<T: ?Sized>(RustArc<T>, ArcHandle);

impl<T> Arc<T> {
    /// Allocates a shared value using the caller's task authority.
    #[track_caller]
    pub fn new(value: T, tasks: &Tasks) -> Self {
        let runtime = ArcHandle::new(tasks, std::panic::Location::caller());
        Self(RustArc::new(value), runtime)
    }

    /// Creates a weak observer using the caller's task authority.
    #[track_caller]
    pub fn downgrade(this: &Self, tasks: &Tasks) -> Weak<T> {
        let runtime = this.1.downgrade(tasks, std::panic::Location::caller());
        Weak(RustArc::downgrade(&this.0), runtime)
    }
}

impl<T: Clone> Arc<T> {
    /// Makes the value unique, cloning if shared and dissociating weak observers otherwise.
    #[track_caller]
    pub fn make_mut(this: &mut Self) -> &mut T {
        this.1.begin("arc.make_mut", std::panic::Location::caller());
        RustArc::make_mut(&mut this.0)
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
    fn ne(&self, other: &Self) -> bool {
        self.0.ne(&other.0)
    }
}
impl<T: ?Sized + Eq> Eq for Arc<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for Arc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
    fn lt(&self, other: &Self) -> bool {
        self.0.lt(&other.0)
    }
    fn le(&self, other: &Self) -> bool {
        self.0.le(&other.0)
    }
    fn gt(&self, other: &Self) -> bool {
        self.0.gt(&other.0)
    }
    fn ge(&self, other: &Self) -> bool {
        self.0.ge(&other.0)
    }
}
impl<T: ?Sized + Ord> Ord for Arc<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T: ?Sized + Hash> Hash for Arc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<T: ?Sized + fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl<T: ?Sized + fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
impl<T: ?Sized> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.0, f)
    }
}

/// A weak observer of an [`Arc`], created by [`Arc::downgrade`].
pub struct Weak<T: ?Sized>(std::sync::Weak<T>, ArcHandle);

impl<T> Weak<T> {
    /// Returns a strong handle if the value is still alive.
    /// Simulation schedules before the attempt and after a successful acquisition.
    #[track_caller]
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let span = std::panic::Location::caller();
        self.1.begin("weak.upgrade", span);
        self.0.upgrade().map(|value| {
            let acquired = Arc(value, self.1.clone());
            self.1.begin("weak.upgrade_acquired", span);
            acquired
        })
    }
}
impl<T: ?Sized> Clone for Weak<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
impl<T: ?Sized> fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

type Span = &'static std::panic::Location<'static>;
static NEXT_CELL: AtomicU64 = AtomicU64::new(0);

/// A shared cell. Cloning the handle keeps the same cell.
/// Simulated calls take one point before acquisition and one after release, including empty sets;
/// an aliased call returns before either point.
pub struct Shared<T>(RustArc<SharedCell<T>>);

struct Cell {
    simulation: Option<SimCell>,
    id: u64,
    next: AtomicUsize,
    ready: Condvar,
}

struct SharedCell<T> {
    cell: Cell,
    state: Mutex<State<T>>,
}

struct Control {
    serving: usize,
    poisoned: bool,
}

struct State<T> {
    control: Control,
    value: T,
}

impl Cell {
    fn order_key(&self) -> u64 {
        self.id
    }
}

impl<T> SharedCell<T> {
    fn acquire(&self) -> MutexGuard<'_, State<T>> {
        // Reserve lazily, after acquiring every lower-id operand. Eager tickets can deadlock.
        let ticket = self.cell.next.fetch_add(1, Ordering::Relaxed);
        if self.cell.simulation.is_some() {
            let state = match self.state.try_lock() {
                Ok(state) => state,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
                Err(TryLockError::WouldBlock) => panic!("contended cell under the simulator"),
            };
            assert_eq!(
                state.control.serving, ticket,
                "contended cell under the simulator"
            );
            state
        } else {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            while state.control.serving != ticket {
                state = self
                    .cell
                    .ready
                    .wait(state)
                    .unwrap_or_else(PoisonError::into_inner);
            }
            state
        }
    }
}

// Both homogeneous sets and heterogeneous pairs use the same ticket/poison/release protocol.
trait Held {
    fn control(&mut self) -> &mut Control;
}

impl<T> Held for MutexGuard<'_, State<T>> {
    fn control(&mut self) -> &mut Control {
        &mut self.control
    }
}

enum PairGuard<'a, T, U> {
    First(MutexGuard<'a, State<T>>),
    Second(MutexGuard<'a, State<U>>),
}

impl<T, U> Held for PairGuard<'_, T, U> {
    fn control(&mut self) -> &mut Control {
        match self {
            Self::First(state) => &mut state.control,
            Self::Second(state) => &mut state.control,
        }
    }
}

struct HeldCell<'a, G> {
    index: usize,
    cell: &'a Cell,
    state: G,
}

fn acquire_set<'a, G: Held>(
    cells: &[&'a Cell],
    mut acquire: impl FnMut(usize) -> G,
    allow_poison: bool,
    span: Span,
) -> Result<(Vec<HeldCell<'a, G>>, Option<Context>), LockError> {
    // Validate every domain before ordering, detecting aliases, or taking any ticket.
    let context = SimCell::context(cells.iter().map(|cell| cell.simulation.as_ref()));
    let mut ordered: Vec<_> = cells.iter().enumerate().map(|(i, &c)| (i, c)).collect();
    ordered.sort_unstable_by_key(|(_, cell)| cell.order_key());
    if ordered
        .windows(2)
        .any(|pair| std::ptr::eq(pair[0].1, pair[1].1))
    {
        return Err(LockError::Aliased);
    }
    SimCell::begin_set(
        context.as_ref(),
        ordered.iter().map(|(_, c)| c.order_key() as usize),
        span,
    );
    let mut held = Vec::with_capacity(cells.len());
    for (index, cell) in ordered {
        let mut state = acquire(index);
        let poisoned = state.control().poisoned;
        held.push(HeldCell { index, cell, state });
        if poisoned && !allow_poison {
            SimCell::record_set(
                context.as_ref(),
                held.iter().map(|h| h.cell.order_key() as usize),
                true,
                span,
            );
            release_set(held, context.as_ref(), span);
            return Err(LockError::Poisoned);
        }
    }
    SimCell::record_set(
        context.as_ref(),
        held.iter().map(|h| h.cell.order_key() as usize),
        true,
        span,
    );
    held.sort_unstable_by_key(|h| h.index);
    Ok((held, context))
}

fn release_set<G: Held>(mut held: Vec<HeldCell<'_, G>>, context: Option<&Context>, span: Span) {
    held.sort_unstable_by_key(|h| std::cmp::Reverse(h.cell.order_key()));
    SimCell::record_set(
        context,
        held.iter().map(|h| h.cell.order_key() as usize),
        false,
        span,
    );
    for h in &mut held {
        let control = h.state.control();
        control.serving = control.serving.wrapping_add(1);
    }
    // Every path advances all cells before dropping guards, then notifies each unlocked cell.
    for h in held.drain(..) {
        let cell = h.cell;
        drop(h.state);
        cell.ready.notify_all();
    }
    SimCell::end_set(context, span);
}

fn run_set<G: Held, R>(
    (mut held, context): (Vec<HeldCell<'_, G>>, Option<Context>),
    f: impl FnOnce(&mut [HeldCell<'_, G>]) -> R,
    span: Span,
) -> R {
    let outcome = catch_unwind(AssertUnwindSafe(|| f(&mut held)));
    if outcome.is_err() {
        for h in &mut held {
            h.state.control().poisoned = true;
        }
    }
    release_set(held, context.as_ref(), span);
    match outcome {
        Ok(value) => value,
        Err(panic) => resume_unwind(panic),
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Send + Pure> Shared<T> {
    /// Creates a cell using the caller's task authority.
    #[track_caller]
    pub fn new(v: T, tasks: &Tasks) -> Shared<T> {
        let simulation = tasks.sim_cell(std::panic::Location::caller());
        let id = simulation
            .as_ref()
            .map_or_else(|| NEXT_CELL.fetch_add(1, Ordering::Relaxed), SimCell::id);
        Self(RustArc::new(SharedCell {
            cell: Cell {
                simulation,
                id,
                next: AtomicUsize::new(0),
                ready: Condvar::new(),
            },
            state: Mutex::new(State {
                control: Control {
                    serving: 0,
                    poisoned: false,
                },
                value: v,
            }),
        }))
    }

    /// Runs `f` with exclusive access, in FIFO acquisition order, even in a cancelled scope.
    /// A panic propagates and poisons the cell. Later calls skip `f` and return `Poisoned`.
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R + Pure) -> Result<R, LockError> {
        let span = std::panic::Location::caller();
        let held = acquire_set(&[&self.0.cell], |_| self.0.acquire(), false, span)?;
        Ok(run_set(held, |held| f(&mut held[0].state.value), span))
    }

    /// Runs `f` holding both cells, acquiring in cell-id order. Aliases skip `f`, even if poisoned.
    /// FIFO is per cell by arrival at that cell, not by arrival at the whole operation.
    /// A panic poisons both cells and propagates.
    #[track_caller]
    pub fn with2<U: Send + Pure, R>(
        a: &Shared<T>,
        b: &Shared<U>,
        f: impl FnOnce(&mut T, &mut U) -> R + Pure,
    ) -> Result<R, LockError> {
        let span = std::panic::Location::caller();
        let held = acquire_set(
            &[&a.0.cell, &b.0.cell],
            |index| match index {
                0 => PairGuard::First(a.0.acquire()),
                _ => PairGuard::Second(b.0.acquire()),
            },
            false,
            span,
        )?;
        Ok(run_set(
            held,
            |held| {
                let (first, second) = held.split_at_mut(1);
                let PairGuard::First(a) = &mut first[0].state else {
                    unreachable!()
                };
                let PairGuard::Second(b) = &mut second[0].state else {
                    unreachable!()
                };
                f(&mut a.value, &mut b.value)
            },
            span,
        ))
    }

    /// Runs `f` holding all cells, with payload references in the caller's operand order.
    /// An empty set runs `f` on an empty slice. Aliases skip `f`; a panic poisons every cell.
    /// Acquisition is globally ordered, with FIFO arrival at each cell, even when cancelled.
    #[track_caller]
    pub fn with_all<R>(
        cells: &[Shared<T>],
        f: impl FnOnce(&mut [&mut T]) -> R + Pure,
    ) -> Result<R, LockError> {
        let span = std::panic::Location::caller();
        let operands: Vec<_> = cells.iter().map(|c| &c.0.cell).collect();
        let held = acquire_set(&operands, |index| cells[index].0.acquire(), false, span)?;
        Ok(run_set(
            held,
            |held| {
                let mut values: Vec<_> = held.iter_mut().map(|h| &mut h.state.value).collect();
                f(&mut values)
            },
            span,
        ))
    }

    /// Compares cell identity without acquiring or scheduling.
    pub fn same(a: &Shared<T>, b: &Shared<T>) -> bool {
        RustArc::ptr_eq(&a.0, &b.0)
    }

    /// Replaces the payload and clears poison, acquiring in FIFO order even when cancelled.
    /// The old payload drops after release; a destructor panic leaves the replacement usable.
    #[track_caller]
    pub fn reset(&self, v: T) {
        let span = std::panic::Location::caller();
        let held = acquire_set(&[&self.0.cell], |_| self.0.acquire(), true, span).unwrap();
        let old = run_set(
            held,
            |held| {
                held[0].state.control.poisoned = false;
                std::mem::replace(&mut held[0].state.value, v)
            },
            span,
        );
        drop(old);
    }
}

/// Failure to access a shared cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockError {
    /// Multiple operands name the same cell.
    Aliased,
    /// A previous callback panicked while holding the cell.
    Poisoned,
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aliased => f.write_str("shared cell appears more than once"),
            Self::Poisoned => f.write_str("shared cell poisoned"),
        }
    }
}

impl std::error::Error for LockError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskError;
    use std::panic::panic_any;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    const TIMEOUT: Duration = Duration::from_secs(20);

    #[test]
    fn arc_operations_validate_domains_but_payload_access_does_not() {
        let real_tasks = Tasks::real();
        let real = Arc::new(1, &real_tasks);
        let real_weak = Arc::downgrade(&real, &real_tasks);
        let (mut old, old_weak);
        {
            let sim = crate::Root::sim(0);
            old = Arc::new(1, &sim.tasks);
            old_weak = Arc::downgrade(&old, &sim.tasks);
        }
        assert!(catch_unwind(AssertUnwindSafe(|| old_weak.upgrade())).is_err());
        assert_eq!(old.clone(), real);
        assert_eq!(format!("{:?}", old_weak.clone()), "(Weak)");
        let sim = crate::Root::sim(1);
        let before = sim.trace();
        assert_eq!(*old, 1);
        assert_eq!(old.clone(), real.clone());
        assert_eq!(format!("{old} {old:?}"), "1 1");
        assert_eq!(
            format!("{:?} {:?}", old_weak.clone(), real_weak.clone()),
            "(Weak) (Weak)"
        );
        assert_eq!(sim.trace(), before);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                Arc::make_mut(&mut old);
            }))
            .is_err()
        );
        assert!(catch_unwind(AssertUnwindSafe(|| Arc::downgrade(&old, &sim.tasks))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| real_weak.upgrade())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| Arc::new(1, &real_tasks))).is_err());
        let before = sim.trace();
        drop((old, old_weak, real, real_weak));
        assert_eq!(sim.trace(), before);
        sim.tasks.checkpoint().unwrap();
    }

    fn wait_until(ready: impl Fn() -> bool, failure: &str) {
        let deadline = Instant::now() + TIMEOUT;
        while !ready() {
            assert!(Instant::now() < deadline, "{failure}");
            thread::yield_now();
        }
    }

    #[test]
    fn real_sets_do_not_collect_simulation_ids() {
        let tasks = Tasks::real();
        tasks
            .scope::<_, TaskError, _>(|_| {
                let cell = Shared::new(0, &tasks);
                let context = SimCell::context([cell.0.cell.simulation.as_ref()]);
                assert!(context.is_none());
                let ids = || std::iter::once_with(|| panic!("real set collected a simulation id"));
                let span = std::panic::Location::caller();
                SimCell::begin_set(context.as_ref(), ids(), span);
                SimCell::record_set(context.as_ref(), ids(), true, span);
                SimCell::record_set(context.as_ref(), ids(), false, span);
                SimCell::end_set(context.as_ref(), span);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn shared_serializes_cloned_handles_in_fifo_order() {
        let tasks = Tasks::real();
        let order = Shared::new(Vec::new(), &tasks);
        let result: Result<(), TaskError> = tasks.scope(|scope| {
            let (entered, in_cell) = mpsc::channel();
            let (release, held) = mpsc::channel();
            drop(scope.spawn(|| {
                order
                    .with(move |xs| {
                        xs.push(0);
                        entered.send(()).unwrap();
                        let _ = held.recv();
                    })
                    .unwrap();
                Ok(())
            }));
            in_cell.recv_timeout(TIMEOUT).unwrap();
            for n in 1..=3 {
                let handle = order.clone();
                drop(scope.spawn(move || {
                    handle.with(|xs| xs.push(n)).unwrap();
                    Ok(())
                }));
                // Establish arrival order while the first callback still holds the cell.
                wait_until(
                    || order.0.cell.next.load(Ordering::Relaxed) > n,
                    "waiter did not take a ticket",
                );
            }
            release.send(()).unwrap();
            Ok(())
        });
        result.unwrap();
        assert_eq!(order.with(|xs| xs.clone()), Ok(vec![0, 1, 2, 3]));
    }

    #[test]
    fn shared_propagates_panics_and_wakes_poisoned_waiters() {
        const WAITERS: usize = 3;
        let tasks = Tasks::real();
        let cell = Shared::new(0, &tasks);
        let (release, held) = mpsc::channel();
        let (completed, results) = mpsc::channel();
        let scoped_cell = cell.clone();
        // Keep the test's deadline outside the scope, which must join even stuck children.
        let run = thread::spawn(move || {
            let _: Result<(), TaskError> = tasks.scope(move |scope| {
                let (entered, in_cell) = mpsc::channel();
                let holder = scoped_cell.clone();
                drop(scope.spawn(move || {
                    holder
                        .with(move |value| {
                            *value = 1;
                            entered.send(()).unwrap();
                            held.recv().unwrap();
                            panic_any(73u32);
                        })
                        .unwrap();
                    Ok(())
                }));
                in_cell.recv_timeout(TIMEOUT).unwrap();
                for _ in 0..WAITERS {
                    let handle = scoped_cell.clone();
                    let completed = completed.clone();
                    drop(scope.spawn(move || {
                        let mut ran = false;
                        let result = handle.with(|_| ran = true);
                        completed.send((result, ran)).unwrap();
                        Ok(())
                    }));
                }
                Ok(())
            });
        });
        wait_until(
            || cell.0.cell.next.load(Ordering::Relaxed) == WAITERS + 1,
            "queued callers did not reserve their tickets",
        );
        release.send(()).unwrap();
        for _ in 0..WAITERS {
            assert_eq!(
                results
                    .recv_timeout(TIMEOUT)
                    .expect("queued caller did not finish"),
                (Err(LockError::Poisoned), false)
            );
        }
        wait_until(|| run.is_finished(), "panicking scope did not finish");
        let panic = run.join().unwrap_err();
        assert_eq!(*panic.downcast::<u32>().unwrap(), 73);
    }

    #[test]
    fn multi_cell_tickets_are_lazy_and_poison_releases_earlier_cells() {
        let tasks = Tasks::real();
        let a = Shared::new(1, &tasks);
        let b = Shared::new(2, &tasks);
        let (entered, in_b) = mpsc::channel();
        let (release, held_b) = mpsc::channel();
        let (done, completed) = mpsc::channel();
        let (aa, bb) = (a.clone(), b.clone());
        let worker = thread::spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| {
                tasks
                    .scope::<_, TaskError, _>(|scope| {
                        let holder = bb.clone();
                        scope.spawn(move || {
                            holder
                                .with(|_| {
                                    entered.send(()).unwrap();
                                    held_b.recv().unwrap();
                                })
                                .unwrap();
                            Ok(())
                        });
                        in_b.recv_timeout(TIMEOUT).unwrap();
                        scope.spawn(|| {
                            Shared::with2(&bb, &aa, |b, a| {
                                *a += 1;
                                *b -= 1;
                            })
                            .unwrap();
                            Ok(())
                        });
                        wait_until(
                            || bb.0.cell.next.load(Ordering::Relaxed) == 2,
                            "pair did not reach B",
                        );
                        scope.spawn(|| {
                            Shared::with_all(&[aa.clone(), bb.clone()], |xs| {
                                assert_eq!(*xs[0] + *xs[1], 3);
                            })
                            .unwrap();
                            Ok(())
                        });
                        Ok(())
                    })
                    .unwrap();
            }));
            done.send(result).unwrap();
        });
        wait_until(
            || a.0.cell.next.load(Ordering::Relaxed) == 2,
            "set did not queue on A",
        );
        assert_eq!(
            b.0.cell.next.load(Ordering::Relaxed),
            2,
            "set reserved B before acquiring A"
        );
        release.send(()).unwrap();
        completed
            .recv_timeout(TIMEOUT)
            .expect("overlapping calls deadlocked")
            .unwrap();
        worker.join().unwrap();

        let _ = catch_unwind(AssertUnwindSafe(|| b.with(|_| panic!("poison B"))));
        let ticket = b.0.cell.next.load(Ordering::Relaxed);
        assert_eq!(
            Shared::with2(&b, &b, |_, _| panic!("aliased callback ran")),
            Err(LockError::Aliased)
        );
        assert_eq!(b.0.cell.next.load(Ordering::Relaxed), ticket);
        assert_eq!(
            Shared::with_all(&[a.clone(), b.clone(), a.clone()], |_| panic!(
                "aliased callback ran"
            )),
            Err(LockError::Aliased)
        );
        assert_eq!(b.0.cell.next.load(Ordering::Relaxed), ticket);
        assert_eq!(
            Shared::with_all(&[a.clone(), b.clone()], |_| panic!("poisoned callback ran")),
            Err(LockError::Poisoned)
        );
        assert_eq!(a.with(|n| *n), Ok(2));
        b.reset(4);
        assert_eq!(Shared::with2(&a, &b, |a, b| *a + *b), Ok(6));
    }

    struct DropBomb(bool);
    impl Drop for DropBomb {
        fn drop(&mut self) {
            assert!(!self.0, "old payload dropped");
        }
    }

    #[test]
    fn reset_releases_before_dropping_the_old_payload() {
        let tasks = Tasks::real();
        let cell = Shared::new(DropBomb(true), &tasks);
        let _ = catch_unwind(AssertUnwindSafe(|| cell.with(|_| panic!("poison"))));
        assert!(catch_unwind(AssertUnwindSafe(|| cell.reset(DropBomb(false)))).is_err());
        assert_eq!(cell.with(|v| v.0), Ok(false));
        cell.reset(DropBomb(false)); // Healthy cells are replaceable too.
    }

    #[test]
    fn simulated_sets_have_ordered_events_without_internal_scheduling() {
        fn observed() -> String {
            let simulation = crate::task::Simulation::new(19);
            let tasks = simulation.tasks();
            let a = Shared::new(1, tasks);
            let b = Shared::new(2, tasks);
            tasks
                .scope::<_, TaskError, _>(|scope| {
                    for _ in 0..2 {
                        scope.spawn(|| {
                            Shared::with2(&b, &a, |b, a| {
                                *b += 1;
                                *a -= 1;
                            })
                            .unwrap();
                            Shared::with_all(&[b.clone(), a.clone()], |xs| {
                                assert_eq!(*xs[0] + *xs[1], 3);
                            })
                            .unwrap();
                            Ok(())
                        });
                    }
                    Ok(())
                })
                .unwrap();
            let trace = tasks.sim_trace();
            let mut acquisitions = 0;
            let mut releases = 0;
            for line in trace.lines() {
                if line.contains("\"operation\":\"cells_acquire\"") {
                    assert!(line.contains("\"targets\":[0, 1]"), "{line}");
                }
            }
            for pair in trace.lines().collect::<Vec<_>>().windows(2) {
                if pair[0].contains("\"operation\":\"cell_acquire\",\"target\":0") {
                    assert!(pair[1].contains("\"operation\":\"cell_acquire\",\"target\":1"));
                    assert_eq!(
                        pair[0].split(",\"time\"").next(),
                        pair[1].split(",\"time\"").next()
                    );
                    acquisitions += 1;
                }
                if pair[0].contains("\"operation\":\"cell_release\",\"target\":1") {
                    assert!(pair[1].contains("\"operation\":\"cell_release\",\"target\":0"));
                    assert_eq!(
                        pair[0].split(",\"time\"").next(),
                        pair[1].split(",\"time\"").next()
                    );
                    releases += 1;
                }
            }
            assert_eq!((acquisitions, releases), (4, 4));
            trace
        }
        assert_eq!(observed(), observed());
    }

    #[test]
    fn set_domain_checks_precede_tickets_and_aliases() {
        let real = Shared::new(0, &Tasks::real());
        let old = {
            let simulation = crate::task::Simulation::new(1);
            Shared::new(0, simulation.tasks())
        };
        let simulation = crate::task::Simulation::new(2);
        let current = Shared::new(0, simulation.tasks());
        for foreign in [&real, &old] {
            assert!(
                catch_unwind(AssertUnwindSafe(|| Shared::with2(
                    &current,
                    foreign,
                    |_, _| ()
                )))
                .is_err()
            );
            assert!(
                catch_unwind(AssertUnwindSafe(|| Shared::with_all(
                    &[current.clone(), foreign.clone(), current.clone()],
                    |_| ()
                )))
                .is_err()
            );
            assert_eq!(current.0.cell.next.load(Ordering::Relaxed), 0);
            assert_eq!(foreign.0.cell.next.load(Ordering::Relaxed), 0);
        }
        let before = simulation.tasks().sim_trace();
        assert!(Shared::same(&current, &current.clone()));
        assert_eq!(simulation.tasks().sim_trace(), before);
        assert_eq!(Shared::<i32>::with_all(&[], |xs| xs.len()), Ok(0));
        let trace = simulation.tasks().sim_trace();
        assert!(trace[before.len()..].contains("\"operation\":\"cells_acquire\""));
        assert!(trace[before.len()..].contains("\"operation\":\"cells_release\""));
    }
}

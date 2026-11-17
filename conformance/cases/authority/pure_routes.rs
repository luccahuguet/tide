//@ rules: PURE-1, LIFE-3
// `Pure` obligations that no expression's type shows: those of the `Deref` impl that an
// autoderef or a deref coercion selects; of a `Drop` impl and of a trait impl's method or
// associated type, whose where-clauses drop glue (of values that a constructor passed as a
// function or a method's type argument makes too), a trait call, and the trait's bounds rely
// on, and which rustc lets bound a parameter by `Pure` where the type and the trait do not; of a
// trait's associated type, whose where-clauses a use that names it meets and its impls assume;
// of an opaque type's bounds on its associated types, which its users meet without selecting an
// impl; and of the `Termination` impl that rustc's entry point (a `main` without
// `#[tide::main]` is TD0512) and the test harness report a result through.
use std::ops::Deref;
use std::process::{ExitCode, Termination};
use std::time::{Clock, Instant};

#[derive(Default)]
pub struct Checked<T>(pub T);

impl<T: Pure> Deref for Checked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Pure> Termination for Checked<T> {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}

pub struct Holder {
    pub clock: Clock,
}

fn read(clock: &Clock) -> Instant {
    clock.now()
}

pub fn method(c: &Checked<Clock>) -> Instant {
    c.now() //~ TD0517
}

pub fn field(c: &Checked<Holder>) -> Instant {
    c.clock.now() //~ TD0517
}

pub fn coercion(c: &Checked<Clock>) -> Instant {
    read(c) //~ TD0517
}

pub fn pure_target(c: &Checked<u64>) -> u64 {
    c.pow(2)
}

pub struct Guard<T>(pub T);

impl<T: Pure> Drop for Guard<T> {
    fn drop(&mut self) {}
}

pub fn guarded(clock: Clock) {
    let _ = Guard(clock); //~ TD0517
}

pub fn guarded_data(ticks: u64) {
    let _ = Guard(ticks);
}

pub fn guard_clocks() -> impl Fn(Clock) -> Guard<Clock> { //~ TD0517
    Guard
}

pub struct Maker;

impl Maker {
    pub fn make<V: Default>(&self) {
        let _ = V::default();
    }
}

pub fn guarded_inside(maker: &Maker) {
    maker.make::<Vec<Guard<Clock>>>(); //~ TD0517
}

pub struct Logged<T>(pub T);

impl<T> Drop for Logged<T> {
    fn drop(&mut self)
    where
        T: Pure, //~ TD0517
    {
    }
}

pub trait Show {
    fn show(&self);
    fn show_pure(&self)
    where
        Self: Pure;
}

impl<T> Show for Logged<T> {
    fn show(&self)
    where
        T: Pure, //~ TD0517
    {
    }
    fn show_pure(&self)
    where
        Self: Pure,
    {
    }
}

pub trait Wrap {
    type Out<U>: Deref; //~ TD0403
}

impl Wrap for Holder {
    type Out<U> = Checked<U> where U: Pure; //~ TD0403 TD0517
}

pub trait Supply {
    type Supplied<U>: Deref + Default //~ TD0403
    where
        U: Pure;
}

impl Supply for Holder {
    type Supplied<U> = Checked<Option<U>> where U: Pure; //~ TD0403
}

pub fn supplied<T: Supply>() {
    let _ = &*<T::Supplied<Clock>>::default(); //~ TD0517
}

pub fn supplied_data<T: Supply>() {
    let _ = &*<T::Supplied<u64>>::default();
}

pub fn checked_all<T>(items: Vec<T>) -> impl Iterator<Item: Deref> { //~ TD0517
    items.into_iter().map(Checked)
}

pub fn checked_data(items: Vec<u64>) -> impl Iterator<Item: Deref> {
    items.into_iter().map(Checked)
}

#[test] //~ TD0517
fn checked_test() -> Checked<Clock> {
    todo!()
}

fn main() -> Checked<Option<Clock>> { //~ TD0512 TD0517
    Checked(None)
}

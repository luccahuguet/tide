//@ rules: PURE-1, OPAQUE-2
// `Pure` obligations in declarations, which no expression shows: a field's type, a required
// trait method's signature, an associated type, and the `impl Trait` result of a trait
// method's default body, whose hidden type must meet the `Pure` its bounds state.
use std::time::Clock;

pub struct NeedsPure<T: Pure>(pub T);

pub struct Config {
    pub ticks: NeedsPure<u64>,
    pub clock: NeedsPure<Clock>, //~ TD0517
}

pub trait Source {
    type Out;
    fn open(ticks: u64) -> NeedsPure<Clock>; //~ TD0517
    fn reader(&self, clock: Clock) -> impl Fn() -> u64 + Pure + Send { //~ TD0517
        move || {
            let _ = clock.now();
            1
        }
    }
}

impl Source for u8 {
    type Out = NeedsPure<Clock>; //~ TD0517
    fn open(_ticks: u64) -> NeedsPure<Clock> { //~ TD0517
        todo!()
    }
}

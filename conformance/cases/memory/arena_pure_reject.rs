//@ rules: PURE-1, AFREE-1
use std::arena::Arena;
use std::fs::Fs;

fn needs_pure<T: Pure>(_: &T) {}
pub fn check(arena: &Arena<Fs>) {
    needs_pure(arena); //~ TD0517
}
#[pure]
pub fn authority(arena: &Arena<Fs>) {} //~ TD0511

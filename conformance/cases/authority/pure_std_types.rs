//@ rules: PURE-1
// std's types by 02 §9's table: the listing types `ReadDir` and `DirEntry` act on the file
// system, so they are not `Pure`, while `Metadata`, `FileType`, and the map hasher
// `RandomState` are data. A type argument std holds only as a marker counts too, so
// `PhantomData<Clock>` is not `Pure`.
use std::collections::hash_map::RandomState;
use std::fs::{DirEntry, FileType, Metadata, ReadDir};
use std::marker::PhantomData;
use std::time::Clock;

fn needs_pure<T: Pure>(_t: &T) {}

pub fn data(m: &Metadata, t: &FileType, s: &RandomState) {
    needs_pure(m);
    needs_pure(t);
    needs_pure(s);
}

pub fn listing(r: &ReadDir) {
    needs_pure(r) //~ FX0517
}

pub fn entry(e: &DirEntry) {
    needs_pure(e) //~ FX0517
}

pub fn marker(p: &PhantomData<Clock>) {
    needs_pure(p) //~ FX0517
}

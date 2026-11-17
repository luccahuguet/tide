//@ rules: PURE-1
// The shared std's collection keys, values, and custom builders all participate in Pure.
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};
use std::net::Net;

fn pure<T: Pure>(_: &T) {}

pub fn map_value(map: &HashMap<u8, Net>) {
    pure(map); //~ TD0517
}
pub fn map_key(map: &HashMap<Net, u8>) {
    pure(map); //~ TD0517
}
pub fn map_hasher(map: &HashMap<u8, u8, Net>) {
    pure(map); //~ TD0517
}
pub fn set_value(set: &HashSet<Net>) {
    pure(set); //~ TD0517
}
pub fn set_hasher(set: &HashSet<u8, Net>) {
    pure(set); //~ TD0517
}
pub fn tree_map(map: &BTreeMap<u8, Net>) {
    pure(map); //~ TD0517
}
pub fn tree_set(set: &BTreeSet<Net>) {
    pure(set); //~ TD0517
}
pub fn queue(queue: &VecDeque<Net>) {
    pure(queue); //~ TD0517
}
pub fn heap(heap: &BinaryHeap<Net>) {
    pure(heap); //~ TD0517
}

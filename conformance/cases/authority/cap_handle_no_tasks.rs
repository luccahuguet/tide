//@ rules: CAP-2
// A synchronization handle's constructor takes `&Tasks`.
use std::sync::Shared;
use std::sync::mpsc::sync_channel;

pub fn make() {
    let _hits = Shared::new(0u64); //~ E0061 TD0505
    let (_tx, _rx) = sync_channel::<u32>(8); //~ E0061 TD0505
    let make = Shared::<u64>::new;
    let _hits = make(0u64); //~ E0061 TD0505
}

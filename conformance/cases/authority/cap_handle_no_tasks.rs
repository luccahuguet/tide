//@ rules: CAP-2
// A synchronization handle's constructor takes `&Tasks`.
use std::sync::Shared;
use std::sync::mpsc::sync_channel;

pub fn make() {
    let _hits = Shared::new(0u64); //~ E0061 FX0505
    let (_tx, _rx) = sync_channel::<u32>(8); //~ E0061 FX0505
}

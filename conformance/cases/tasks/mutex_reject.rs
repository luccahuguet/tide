//@ rules: REF-0604
// Mutex and RwLock habits belong to Shared; raw synchronization keeps its own owner.
use std::sync::Mutex as Cell; //~ E0603 FX0604
pub fn create() { let _ = Cell::new(1); } //~ FX0604
pub fn mutex() { let _ = std::sync::Mutex::new(1); } //~ E0603 FX0604
pub fn read_write() { let _ = std::sync::RwLock::new(2); } //~ E0433 FX0604

//! Stand-in for the facade's Pure surface (the real one is spikes/d18/facade).
#![deny(unsafe_code)]

/// Marker: rustc accepts every bound; tide-admit owns the structural analysis (FX0510).
#[allow(unsafe_code)]
pub unsafe trait Pure {}
#[allow(unsafe_code)]
unsafe impl<T: ?Sized> Pure for T {}

/// A capability (impure).
pub struct Fs {
    _private: (),
}
impl Fs {
    pub fn new_for_test() -> Fs {
        Fs { _private: () }
    }
}

/// A synchronization handle (impure).
pub struct Shared<T> {
    pub value: T,
}

// `AsDynError` and `AsDisplay` are adapted from thiserror (src/aserror.rs and src/display.rs,
// https://github.com/dtolnay/thiserror, MIT OR Apache-2.0; see NOTICE).

// Method resolution handles both sized errors and dereferences to a dynamic error.
pub trait AsDynError<'a> {
    fn as_dyn_error(&self) -> &(dyn core::error::Error + 'a);
}
impl<'a, T: core::error::Error + 'a> AsDynError<'a> for T {
    fn as_dyn_error(&self) -> &(dyn core::error::Error + 'a) {
        self
    }
}
macro_rules! dynamic_error {
    ($($bound:path),*) => {
        impl<'a> AsDynError<'a> for dyn core::error::Error + 'a $(+ $bound)* {
            fn as_dyn_error(&self) -> &(dyn core::error::Error + 'a) { self }
        }
    };
}
dynamic_error!();
dynamic_error!(Send);
dynamic_error!(Send, Sync);
dynamic_error!(Send, Sync, std::panic::UnwindSafe);

// Like thiserror, bare `{field}` also displays paths without requiring `.display()`.
pub trait AsDisplay<'a> {
    type Target: core::fmt::Display;
    fn as_display(&'a self) -> Self::Target;
}
impl<'a, T: core::fmt::Display + ?Sized + 'a> AsDisplay<'a> for &T {
    type Target = &'a T;
    fn as_display(&'a self) -> Self::Target {
        *self
    }
}
impl<'a> AsDisplay<'a> for std::path::Path {
    type Target = std::path::Display<'a>;
    fn as_display(&'a self) -> Self::Target {
        self.display()
    }
}
impl<'a> AsDisplay<'a> for std::path::PathBuf {
    type Target = std::path::Display<'a>;
    fn as_display(&'a self) -> Self::Target {
        self.display()
    }
}

// Match ergonomics binds &T; pointer formatting must observe T, not that extra borrow.
pub struct Pointer<'a, T: ?Sized>(pub &'a T);
impl<T: core::fmt::Pointer + ?Sized> core::fmt::Pointer for Pointer<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Pointer::fmt(self.0, f)
    }
}

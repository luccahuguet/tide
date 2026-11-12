//! Oracle-only `#[trusted_pure]` shim (03 RUN-4, UNSAFE-1). It returns its item
//! unchanged; the admission driver owns its meaning. Production `#[pure]` comes
//! from tide-macros through the shared std prelude.

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn trusted_pure(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

//! Oracle-only shims for the toolchain attributes neither the D18 spike's
//! macros nor the E1 trial's cover. Each returns its item unchanged, so rustc
//! checks the item as written; the checker owns what the attribute means
//! (`#[pure]`: 03 §3.2, AFREE-1; `#[trusted_pure]`: 03 RUN-4, UNSAFE-1).

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn pure(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn trusted_pure(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

//! SubVT procedural macros.
#![warn(clippy::disallowed_types)]
mod diff;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// See `diff.rs` for details.
#[proc_macro_derive(Diff, attributes(diff_key))]
pub fn derive_diff(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    diff::derive_diff(input).into()
}

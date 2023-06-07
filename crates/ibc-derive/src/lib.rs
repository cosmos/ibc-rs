#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![allow(non_snake_case)]
mod client_state;
mod utils;

use client_state::client_state_derive_impl;
use proc_macro::TokenStream as RawTokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ClientState, attributes(generics, mock))]
pub fn client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let output = client_state_derive_impl(ast);

    RawTokenStream::from(output)
}

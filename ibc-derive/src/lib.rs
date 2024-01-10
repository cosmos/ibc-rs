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
mod consensus_state;
mod utils;

use client_state::client_state_derive_impl;
use consensus_state::consensus_state_derive_impl;
use proc_macro::TokenStream as RawTokenStream;
use syn::{parse_macro_input, DeriveInput};
use utils::Imports;

#[proc_macro_derive(IbcClientState, attributes(validation, execution))]
pub fn ibc_client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let imports = Imports::new_ibc();

    let output = client_state_derive_impl(ast, &imports);

    RawTokenStream::from(output)
}

#[proc_macro_derive(IbcCoreClientState, attributes(validation, execution))]
pub fn ibc_core_client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let imports = Imports::new_ibc_core();

    let output = client_state_derive_impl(ast, &imports);

    RawTokenStream::from(output)
}

#[proc_macro_derive(IbcConsensusState)]
pub fn ibc_consensus_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let imports = Imports::new_ibc();

    let output = consensus_state_derive_impl(ast, &imports);

    RawTokenStream::from(output)
}

#[proc_macro_derive(IbcCoreConsensusState)]
pub fn ibc_core_consensus_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let imports = Imports::new_ibc_core();

    let output = consensus_state_derive_impl(ast, &imports);

    RawTokenStream::from(output)
}

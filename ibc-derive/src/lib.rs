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
use utils::{Imports, SupportedCrate};

#[proc_macro_derive(IbcClientState, attributes(validation, execution))]
pub fn ibc_client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    generate_client_state_derive(input, SupportedCrate::Ibc)
}

#[proc_macro_derive(IbcCoreClientState, attributes(validation, execution))]
pub fn ibc_core_client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    generate_client_state_derive(input, SupportedCrate::IbcCore)
}

fn generate_client_state_derive(input: RawTokenStream, source: SupportedCrate) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let imports = Imports::new(source);

    let output = client_state_derive_impl(ast, &imports);

    RawTokenStream::from(output)
}

#[proc_macro_derive(IbcConsensusState)]
pub fn ibc_consensus_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    generate_consensus_state_derive(input, SupportedCrate::Ibc)
}

#[proc_macro_derive(IbcCoreConsensusState)]
pub fn ibc_core_consensus_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    generate_consensus_state_derive(input, SupportedCrate::IbcCore)
}

fn generate_consensus_state_derive(
    input: RawTokenStream,
    source: SupportedCrate,
) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let imports = Imports::new(source);

    let output = consensus_state_derive_impl(ast, &imports);

    RawTokenStream::from(output)
}

#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![allow(non_snake_case)]

mod traits;
mod utils;

use darling::FromDeriveInput;
use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::traits::{
    client_state_base::impl_ClientStateBase, client_state_initializer::impl_ClientStateInitializer,
};

#[derive(FromDeriveInput)]
#[darling(attributes(host))]
pub(crate) struct Opts {
    consensus_state: syn::ExprPath,
}

#[proc_macro_derive(ClientState, attributes(host))]
pub fn client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let opts = match Opts::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(e) => panic!(
            "{} must be annotated with #[host(consensus_state = <your ConsensusState enum>)]: {e:?}",
            ast.ident
        ),
    };

    let output = derive_impl(ast, opts);

    RawTokenStream::from(output)
}

fn derive_impl(ast: DeriveInput, opts: Opts) -> TokenStream {
    let enum_name = ast.ident;
    let enum_variants = match ast.data {
        syn::Data::Enum(enum_data) => enum_data.variants,
        _ => panic!("ClientState only supports enums"),
    };

    let ClientStateBase_impl_block = impl_ClientStateBase(&enum_name, &enum_variants);
    let ClientStateInitializer_impl_block =
        impl_ClientStateInitializer(&enum_name, &enum_variants, &opts);

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate ibc as _ibc;

            #ClientStateBase_impl_block
            #ClientStateInitializer_impl_block
        };
    }
}

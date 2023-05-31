#![allow(non_snake_case)]

mod traits;
mod utils;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::traits::client_state_base::impl_ClientStateBase;

#[proc_macro_derive(ClientState)]
pub fn client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let output = derive_impl(ast);

    RawTokenStream::from(output)
}

fn derive_impl(ast: DeriveInput) -> TokenStream {
    let enum_name = ast.ident;
    let enum_variants = match ast.data {
        syn::Data::Enum(enum_data) => enum_data.variants,
        _ => panic!("ClientState only supports enums"),
    };

    let ClientStateBase_impl_block = impl_ClientStateBase(&enum_name, &enum_variants);

    quote! {
        #ClientStateBase_impl_block
    }
}

#![allow(non_snake_case)]

mod utils;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, punctuated::Iter, DeriveInput, Variant};
use utils::Imports;

use crate::utils::get_enum_variant_type_path;

#[proc_macro_derive(ClientState)]
pub fn client_state_macro_derive(input: RawTokenStream) -> RawTokenStream {
    let ast: DeriveInput = parse_macro_input!(input);

    let output = derive_impl(ast);

    RawTokenStream::from(output)
}

#[allow(non_snake_case)]
fn derive_impl(ast: DeriveInput) -> TokenStream {
    let enum_name = ast.ident;
    let enum_variants = match ast.data {
        syn::Data::Enum(enum_data) => enum_data.variants,
        _ => panic!("ClientState only supports enums"),
    };

    // TODO: Have a `ClientStateBaseTokens` struct with this but for all methods.
    let client_type_impl = client_type(&enum_name, enum_variants.iter());
    let validate_proof_height_impl = validate_proof_height(&enum_name, enum_variants.iter());

    // FIXME: what if the user renames the `ibc` package?
    // We also can't currently use in ibc crate's test, since we need to import as `crate::...`
    let ClientStateBase = Imports::ClientStateBase();
    let ClientType = Imports::ClientType();
    let ClientError = Imports::ClientError();
    let Height = Imports::Height();

    // TODO: Make this in a function ClientStateBaseTokens -> TokenStream,
    // which implements that trait
    quote! {
        impl #ClientStateBase for #enum_name {
            fn client_type(&self) -> #ClientType {
                match self {
                    #(#client_type_impl),*
                }
            }
            fn validate_proof_height(&self, proof_height: #Height) -> Result<(), #ClientError> {
                match self {
                    #(#validate_proof_height_impl),*
                }
            }
        }

    }
}

fn client_type(enum_name: &Ident, enum_variants: Iter<Variant>) -> Vec<TokenStream> {
    let ClientStateBase = Imports::ClientStateBase();

    enum_variants
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_type_name = get_enum_variant_type_path(variant);
            quote! {
                #enum_name::#variant_name(cs) => <#variant_type_name as #ClientStateBase>::client_type(cs)
            }
        })
        .collect()
}

fn validate_proof_height(enum_name: &Ident, enum_variants: Iter<Variant>) -> Vec<TokenStream> {
    let ClientStateBase = Imports::ClientStateBase();

    enum_variants
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_type_name = get_enum_variant_type_path(variant);
            quote! {
                #enum_name::#variant_name(cs) => <#variant_type_name as #ClientStateBase>::validate_proof_height(cs, proof_height)
            }
        })
        .collect()
}

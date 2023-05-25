mod utils;

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, punctuated::Iter, DeriveInput, Variant};

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
    // Note: include `validate_proof_height` to see how to include methods with params.
    let client_type_impl = derive_client_type_impl(&enum_name, enum_variants.iter());

    // FIXME: what if the user renames the `ibc` package?
    // We also can't currently use in ibc crate's test, since we need to import as `crate::...`
    let ClientStateBase = quote! {::ibc::core::ics02_client::client_state::ClientStateBase};
    let ClientType = quote! {::ibc::core::ics02_client::client_type::ClientType};

    // TODO: Make this in a function ClientStateBaseTokens -> TokenStream,
    // which implements that trait
    quote! {
        impl #ClientStateBase for #enum_name {
            fn client_type(&self) -> #ClientType {
                match self {
                    #(#client_type_impl),*
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn derive_client_type_impl(enum_name: &Ident, enum_variants: Iter<Variant>) -> Vec<TokenStream> {
    let ClientStateBase = quote! {::ibc::core::ics02_client::client_state::ClientStateBase};

    enum_variants
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_type_name = get_enum_variant_type_path(&variant);
            quote! {
                #enum_name::#variant_name(cs) => <#variant_type_name as #ClientStateBase>::client_type(cs)
            }
        })
        .collect()
}

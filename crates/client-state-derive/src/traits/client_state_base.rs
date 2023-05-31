use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    punctuated::{Iter, Punctuated},
    token::Comma,
    Variant,
};

use crate::utils::{get_enum_variant_type_path, Imports};

pub fn impl_ClientStateBase(
    enum_name: &Ident,
    enum_variants: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let client_type_impl = client_type(enum_name, enum_variants.iter());
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

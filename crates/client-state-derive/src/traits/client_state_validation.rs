use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    punctuated::{Iter, Punctuated},
    token::Comma,
    Variant,
};

use crate::{
    utils::{get_enum_variant_type_path, Imports},
    Opts,
};

pub(crate) fn impl_ClientStateValidation(
    client_state_enum_name: &Ident,
    enum_variants: &Punctuated<Variant, Comma>,
    opts: &Opts,
) -> TokenStream {
    let verify_client_message_impl = delegate_call_in_match(
        client_state_enum_name,
        enum_variants.iter(),
        opts,
        quote! { verify_client_message(cs, ctx, client_id, client_message, update_kind) },
    );

    let check_for_misbehaviour_impl = delegate_call_in_match(
        client_state_enum_name,
        enum_variants.iter(),
        opts,
        quote! { check_for_misbehaviour(cs, ctx, client_id, client_message, update_kind) },
    );

    let HostClientState = client_state_enum_name;
    let ClientValidationContext = &opts.client_validation_context;

    let Any = Imports::Any();
    let ClientId = Imports::ClientId();
    let ClientError = Imports::ClientError();
    let ClientStateValidation = Imports::ClientStateValidation();
    let UpdateKind = Imports::UpdateKind();

    quote! {
        impl #ClientStateValidation<#ClientValidationContext> for #HostClientState {
            fn verify_client_message(
                &self,
                ctx: &#ClientValidationContext,
                client_id: &#ClientId,
                client_message: #Any,
                update_kind: &#UpdateKind,
            ) -> core::result::Result<(), #ClientError> {
                match self {
                    #(#verify_client_message_impl),*
                }
            }

            fn check_for_misbehaviour(
                &self,
                ctx: &#ClientValidationContext,
                client_id: &#ClientId,
                client_message: #Any,
                update_kind: &#UpdateKind,
            ) -> core::result::Result<bool, #ClientError> {
                match self {
                    #(#check_for_misbehaviour_impl),*
                }
            }
        }

    }
}

fn delegate_call_in_match(
    enum_name: &Ident,
    enum_variants: Iter<'_, Variant>,
    opts: &Opts,
    fn_call: TokenStream,
) -> Vec<TokenStream> {
    let ClientStateValidation = Imports::ClientStateValidation();

    enum_variants
        .map(|variant| {
            let HostClientState = enum_name;
            let Tendermint = &variant.ident;
            let TmClientState = get_enum_variant_type_path(variant);
            let ClientValidationContext = &opts.client_validation_context;

            // Note: We use `HostClientState` and `Tendermint`, etc as *variable names*. They're
            // only meant to improve readability of the `quote`; it's not literally what's generated!
            quote! {
                #HostClientState::#Tendermint(cs) => <#TmClientState as #ClientStateValidation<#ClientValidationContext>>::#fn_call
            }
        })
        .collect()
}

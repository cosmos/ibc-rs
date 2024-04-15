use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::{Iter, Punctuated};
use syn::token::Comma;
use syn::Variant;

use crate::client_state::Opts;
use crate::utils::{get_enum_variant_type_path, Imports};

pub(crate) fn impl_ClientStateValidation(
    client_state_enum_name: &Ident,
    enum_variants: &Punctuated<Variant, Comma>,
    opts: &Opts,
    imports: &Imports,
) -> TokenStream {
    let verify_client_message_impl = delegate_call_in_match(
        client_state_enum_name,
        enum_variants.iter(),
        opts,
        quote! { verify_client_message(cs, ctx, client_id, client_message) },
        imports,
    );

    let check_for_misbehaviour_impl = delegate_call_in_match(
        client_state_enum_name,
        enum_variants.iter(),
        opts,
        quote! { check_for_misbehaviour(cs, ctx, client_id, client_message) },
        imports,
    );

    let status_impl = delegate_call_in_match(
        client_state_enum_name,
        enum_variants.iter(),
        opts,
        quote! { status(cs, ctx, client_id) },
        imports,
    );

    let check_substitute_impl = delegate_call_in_match(
        client_state_enum_name,
        enum_variants.iter(),
        opts,
        quote! { check_substitute(cs, ctx, substitute_client_state) },
        imports,
    );

    // The imports we need for the generated code.
    let Any = imports.any();
    let ClientId = imports.client_id();
    let ClientError = imports.client_error();
    let ClientStateValidation = imports.client_state_validation();
    let Status = imports.status();

    // The types we need for the generated code.
    let HostClientState = client_state_enum_name;
    let V = opts.client_validation_context.clone().into_token_stream();

    // The `impl` block quote based on whether the context includes generics.
    let Impl = opts.client_validation_context.impl_ts();

    // The `Where` clause quote based on whether the generics within the context
    // include trait bounds
    let Where = opts.client_validation_context.where_clause_ts();

    quote! {
        #Impl #ClientStateValidation<#V> for #HostClientState #Where {
            fn verify_client_message(
                &self,
                ctx: &#V,
                client_id: &#ClientId,
                client_message: #Any,
            ) -> core::result::Result<(), #ClientError> {
                match self {
                    #(#verify_client_message_impl),*
                }
            }

            fn check_for_misbehaviour(
                &self,
                ctx: &#V,
                client_id: &#ClientId,
                client_message: #Any,
            ) -> core::result::Result<bool, #ClientError> {
                match self {
                    #(#check_for_misbehaviour_impl),*
                }
            }

            fn status(
                &self,
                ctx: &#V,
                client_id: &#ClientId,
            ) -> core::result::Result<#Status, #ClientError> {
                match self {
                    #(#status_impl),*
                }
            }

            fn check_substitute(
                &self,
                ctx: &#V,
                substitute_client_state: #Any,
            ) -> core::result::Result<(), #ClientError> {
                match self {
                    #(#check_substitute_impl),*
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
    imports: &Imports,
) -> Vec<TokenStream> {
    let ClientStateValidation = imports.client_state_validation();

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

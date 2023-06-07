use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    punctuated::{Iter, Punctuated},
    token::Comma,
    Variant,
};

use crate::{
    client_state::Opts,
    utils::{get_enum_variant_type_path, Imports},
};

pub(crate) fn impl_ClientStateInitializer(
    client_state_enum_name: &Ident,
    enum_variants: &Punctuated<Variant, Comma>,
    opts: &Opts,
) -> TokenStream {
    let initialise_impl =
        delegate_call_in_match(client_state_enum_name, enum_variants.iter(), opts);

    let HostClientState = client_state_enum_name;
    let HostConsensusState = &opts.any_consensus_state;

    let Any = Imports::Any();
    let ClientError = Imports::ClientError();
    let ClientStateInitializer = Imports::ClientStateInitializer();

    quote! {
        impl #ClientStateInitializer<#HostConsensusState> for #HostClientState {

            fn initialise(&self, consensus_state: #Any) -> core::result::Result<#HostConsensusState, #ClientError> {
                match self {
                    #(#initialise_impl),*
                }
            }
        }
    }
}

fn delegate_call_in_match(
    enum_name: &Ident,
    enum_variants: Iter<'_, Variant>,
    opts: &Opts,
) -> Vec<TokenStream> {
    let ClientStateInitializer = Imports::ClientStateInitializer();

    enum_variants
        .map(|variant| {
            let HostClientState = enum_name;
            let Tendermint = &variant.ident;
            let TmClientState = get_enum_variant_type_path(variant);
            let AnyConsensusState = &opts.any_consensus_state;

            // Note: We use `HostClientState` and `Tendermint`, etc as *variable names*. They're
            // only meant to improve readability of the `quote`; it's not literally what's generated!
            quote! {
                #HostClientState::#Tendermint(cs) => <#TmClientState as #ClientStateInitializer<#AnyConsensusState>>::initialise(cs, consensus_state)
            }
        })
        .collect()
}

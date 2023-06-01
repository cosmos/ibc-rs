use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    punctuated::{Iter, Punctuated},
    token::Comma,
    Variant,
};

use crate::utils::{get_enum_variant_type_path, Imports};

pub(crate) fn impl_ClientStateBase(
    enum_name: &Ident,
    enum_variants: &Punctuated<Variant, Comma>,
) -> TokenStream {
    let client_type_impl =
        delegate_call_in_match(enum_name, enum_variants.iter(), quote! {client_type(cs)});
    let latest_height_impl =
        delegate_call_in_match(enum_name, enum_variants.iter(), quote! {latest_height(cs)});
    let validate_proof_height_impl = delegate_call_in_match(
        enum_name,
        enum_variants.iter(),
        quote! {validate_proof_height(cs, proof_height)},
    );
    let confirm_not_frozen_impl = delegate_call_in_match(
        enum_name,
        enum_variants.iter(),
        quote! {confirm_not_frozen(cs)},
    );
    let expired_impl = delegate_call_in_match(
        enum_name,
        enum_variants.iter(),
        quote! {expired(cs, elapsed)},
    );
    let verify_upgrade_client_impl = delegate_call_in_match(
        enum_name,
        enum_variants.iter(),
        quote! {verify_upgrade_client(cs, upgraded_client_state, upgraded_consensus_state, proof_upgrade_client, proof_upgrade_consensus_state, root)},
    );
    let verify_membership_impl = delegate_call_in_match(
        enum_name,
        enum_variants.iter(),
        quote! {verify_membership(cs, prefix, proof, root, path, value)},
    );
    let verify_non_membership_impl = delegate_call_in_match(
        enum_name,
        enum_variants.iter(),
        quote! {verify_non_membership(cs, prefix, proof, root, path)},
    );

    let Any = Imports::Any();
    let CommitmentRoot = Imports::CommitmentRoot();
    let CommitmentPrefix = Imports::CommitmentPrefix();
    let CommitmentProofBytes = Imports::CommitmentProofBytes();
    let ClientStateBase = Imports::ClientStateBase();
    let ClientType = Imports::ClientType();
    let ClientError = Imports::ClientError();
    let Height = Imports::Height();
    let MerkleProof = Imports::MerkleProof();
    let Path = Imports::Path();

    quote! {
        impl #ClientStateBase for #enum_name {
            fn client_type(&self) -> #ClientType {
                match self {
                    #(#client_type_impl),*
                }
            }

            fn latest_height(&self) -> #Height {
                match self {
                    #(#latest_height_impl),*
                }
            }

            fn validate_proof_height(&self, proof_height: #Height) -> Result<(), #ClientError> {
                match self {
                    #(#validate_proof_height_impl),*
                }
            }

            fn confirm_not_frozen(&self) -> Result<(), #ClientError> {
                match self {
                    #(#confirm_not_frozen_impl),*
                }
            }

            fn expired(&self, elapsed: core::time::Duration) -> bool {
                match self {
                    #(#expired_impl),*
                }
            }

            fn verify_upgrade_client(
                &self,
                upgraded_client_state: #Any,
                upgraded_consensus_state: #Any,
                proof_upgrade_client: #MerkleProof,
                proof_upgrade_consensus_state: #MerkleProof,
                root: &#CommitmentRoot,
            ) -> Result<(), #ClientError> {
                match self {
                    #(#verify_upgrade_client_impl),*
                }
            }

            fn verify_membership(
                &self,
                prefix: &#CommitmentPrefix,
                proof: &#CommitmentProofBytes,
                root: &#CommitmentRoot,
                path: #Path,
                value: Vec<u8>,
            ) -> Result<(), #ClientError> {
                match self {
                    #(#verify_membership_impl),*
                }
            }

            fn verify_non_membership(
                &self,
                prefix: &#CommitmentPrefix,
                proof: &#CommitmentProofBytes,
                root: &#CommitmentRoot,
                path: #Path,
            ) -> Result<(), #ClientError> {
                match self {
                    #(#verify_non_membership_impl),*
                }
            }
        }

    }
}

///
/// Generates the per-enum variant function call delegation token streams.
///
/// enum_name:     The user's enum identifier (e.g. `HostClientState`)
/// enum_variants: An iterator of all enum variants (e.g. `[HostClientState::Tendermint, HostClientState::Mock]`)
/// fn_call:       The tokens for the function call. Fully-qualified syntax is assumed, where the name for `self`
///                  is `cs` (e.g. `client_type(cs)`).
///
/// For example,
///
/// ```ignore
/// impl ClientStateBase for HostClientState {
///   fn client_type(&self) -> ClientType {
///     match self {
///       //  BEGIN code generated
///
///       // 1st TokenStream returned
///       HostClientState::Tendermint(cs) => <TmClientState as ClientStateBase>::client_type(cs),
///       // 2nd TokenStream returned
///       HostClientState::Mock(cs) => <MockClientState as ClientStateBase>::client_type(cs),
///
///       //  END code generated
///     }
///   }
/// }
/// ```
///
fn delegate_call_in_match(
    enum_name: &Ident,
    enum_variants: Iter<'_, Variant>,
    fn_call: TokenStream,
) -> Vec<TokenStream> {
    let ClientStateBase = Imports::ClientStateBase();

    enum_variants
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_type_name = get_enum_variant_type_path(variant);

            quote! {
                #enum_name::#variant_name(cs) => <#variant_type_name as #ClientStateBase>::#fn_call
            }
        })
        .collect()
}

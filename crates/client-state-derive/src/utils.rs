use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Variant};

/// Encodes the ibc-rs types that will be used in the macro
/// 
/// Note: we use `_ibc` as our top-level crate, due to the
/// `extern crate ibc as _ibc;` statement we inject.
pub struct Imports;

impl Imports {
    pub fn CommitmentRoot() -> TokenStream {
        quote! {_ibc::core::ics23_commitment::commitment::CommitmentRoot}
    }
    
    pub fn ClientStateBase() -> TokenStream {
        quote! {_ibc::core::ics02_client::client_state::ClientStateBase}
    }

    pub fn ClientType() -> TokenStream {
        quote! {_ibc::core::ics02_client::client_type::ClientType}
    }

    pub fn ClientError() -> TokenStream {
        quote! {_ibc::core::ics02_client::error::ClientError}
    }

    pub fn Height() -> TokenStream {
        quote! {_ibc::Height}
    }

    pub fn Any() -> TokenStream {
        quote!{ibc_proto::google::protobuf::Any}
    }

    pub fn MerkleProof() -> TokenStream {
        quote! {_ibc::core::ics23_commitment::merkle::MerkleProof}
    }
}

/// Retrieves the field of a given enum variant. Outputs an error message if the enum variant
/// is in the wrong format (i.e. isn't an unnamed enum, or contains more than one field).
///
/// For example, given
/// ```rust
/// #[derive(ClientState)]
/// enum HostClientState {
///     Tendermint(TmClientState),
/// }
/// ```
/// when acting on the `Tendermint` variant, this will return `TmClientState`.
///
pub fn get_enum_variant_type_path(enum_variant: &Variant) -> &Path {
    let variant_name = &enum_variant.ident;
    let variant_unnamed_fields = match &enum_variant.fields {
            syn::Fields::Unnamed(fields) => fields,
            _ => panic!("\"{variant_name}\" variant must be unnamed, such as `{variant_name}({variant_name}ClientState)`")
        };

    if variant_unnamed_fields.unnamed.iter().len() != 1 {
        panic!("\"{variant_name}\" must contain exactly one field, such as `{variant_name}({variant_name}ClientState)`");
    }

    // A representation of the variant's field (e.g. `TmClientState`). We must dig into
    // the field to get the `TmClientState` path.
    let unnamed_field = variant_unnamed_fields.unnamed.first().unwrap();

    match &unnamed_field.ty {
        syn::Type::Path(path) => &path.path,
        _ => {
            panic!("Invalid enum variant {variant_name} field. Please use an explicit, named type.")
        }
    }
}

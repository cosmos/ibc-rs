use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Variant};

/// The IBC crates that we already support in the derive macro
pub enum SupportedCrate {
    Ibc,
    IbcCore,
}

/// Encodes the ibc-rs types that will be used in the macro
///
/// Note: we use `ibc` or `ibc-core` as our top-level crate, due to the `extern
/// crate ibc as ibc;` statement we inject.
pub struct Imports {
    prefix: TokenStream,
}

impl Imports {
    pub fn new(crate_name: SupportedCrate) -> Self {
        let prefix = match crate_name {
            SupportedCrate::Ibc => quote! {::ibc::core},
            SupportedCrate::IbcCore => quote! {::ibc_core},
        };

        Self { prefix }
    }

    pub fn prefix(&self) -> &TokenStream {
        &self.prefix
    }

    pub fn commitment_root(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::commitment_types::commitment::CommitmentRoot}
    }

    pub fn commitment_prefix(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::commitment_types::commitment::CommitmentPrefix}
    }

    pub fn commitment_proof_bytes(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::commitment_types::commitment::CommitmentProofBytes}
    }

    pub fn path(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::host::types::path::Path}
    }

    pub fn consensus_state(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::client::context::consensus_state::ConsensusState}
    }

    pub fn client_state_common(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::client::context::client_state::ClientStateCommon}
    }

    pub fn client_state_validation(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::client::context::client_state::ClientStateValidation}
    }

    pub fn client_state_execution(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::client::context::client_state::ClientStateExecution}
    }

    pub fn client_id(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::host::types::identifiers::ClientId}
    }

    pub fn client_type(&self) -> TokenStream {
        let Prefix = self.prefix();
        quote! {#Prefix::host::types::identifiers::ClientType}
    }

    pub fn client_error(&self) -> TokenStream {
        let prefix = self.prefix();
        quote! {#prefix::client::types::error::ClientError}
    }

    pub fn height(&self) -> TokenStream {
        let prefix = self.prefix();
        quote! {#prefix::client::types::Height}
    }

    pub fn any(&self) -> TokenStream {
        let prefix = self.prefix();
        quote! {#prefix::primitives::proto::Any}
    }

    pub fn timestamp(&self) -> TokenStream {
        let prefix = self.prefix();
        quote! {#prefix::primitives::Timestamp}
    }

    pub fn status(&self) -> TokenStream {
        let prefix = self.prefix();
        quote! {#prefix::client::types::Status}
    }
}

/// Retrieves the field of a given enum variant. Outputs an error message if the enum variant
/// is in the wrong format (i.e. isn't an unnamed enum, or contains more than one field).
///
/// For example, given
/// ```ignore
///
/// #[derive(IbcClientState)]
/// enum HostClientState {
///     Tendermint(TmClientState),
/// }
/// ```
/// when acting on the `Tendermint` variant, this will return `TmClientState`.
///
pub fn get_enum_variant_type_path(enum_variant: &Variant) -> &Path {
    let variant_name = &enum_variant.ident;
    let syn::Fields::Unnamed(variant_unnamed_fields) = &enum_variant.fields else {
        panic!("\"{variant_name}\" variant must be unnamed, such as `{variant_name}({variant_name}ClientState)`")
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

mod traits;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use traits::{
    client_state_common::impl_ClientStateCommon, client_state_execution::impl_ClientStateExecution,
    client_state_validation::impl_ClientStateValidation,
};

#[derive(FromDeriveInput)]
#[darling(attributes(generics))]
pub(crate) struct Opts {
    #[darling(rename = "ClientValidationContext")]
    client_validation_context: syn::ExprPath,
    #[darling(rename = "ClientExecutionContext")]
    client_execution_context: syn::ExprPath,
}

pub fn client_state_derive_impl(ast: DeriveInput) -> TokenStream {
    let opts = match Opts::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(e) => panic!(
            "{} must be annotated with #[generics(ClientValidationContext = <your ClientValidationContext>, ClientExecutionContext: <your ClientExecutionContext>)]: {e}",
            ast.ident
        ),
    };

    let enum_name = &ast.ident;
    let enum_variants = match ast.data {
        syn::Data::Enum(ref enum_data) => &enum_data.variants,
        _ => panic!("ClientState only supports enums"),
    };

    let ClientStateCommon_impl_block = impl_ClientStateCommon(enum_name, enum_variants);
    let ClientStateValidation_impl_block =
        impl_ClientStateValidation(enum_name, enum_variants, &opts);
    let ClientStateExecution_impl_block =
        impl_ClientStateExecution(enum_name, enum_variants, &opts);

    let maybe_extern_crate_stmt = if is_mock(&ast) {
        // Note: we must add this statement when in "mock mode"
        // (i.e. in ibc-rs itself) because we don't have `ibc` as a dependency,
        // so we need to define the `ibc` symbol to mean "the `self` crate".
        quote! {extern crate self as ibc;}
    } else {
        quote! {}
    };

    quote! {
        #maybe_extern_crate_stmt

        #ClientStateCommon_impl_block
        #ClientStateValidation_impl_block
        #ClientStateExecution_impl_block
    }
}

/// We are in "mock mode" (i.e. within ibc-rs crate itself) if the user added
/// a #[mock] attribute
fn is_mock(ast: &DeriveInput) -> bool {
    for attr in &ast.attrs {
        let path = match attr.meta {
            syn::Meta::Path(ref path) => path,
            _ => continue,
        };

        for path_segment in path.segments.iter() {
            if path_segment.ident == "mock" {
                return true;
            }
        }
    }

    false
}

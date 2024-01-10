mod traits;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use traits::client_state_common::impl_ClientStateCommon;
use traits::client_state_execution::impl_ClientStateExecution;
use traits::client_state_validation::impl_ClientStateValidation;

use crate::utils::Imports;

const MISSING_ATTR: &str = "must be annotated with #[validation(<your ClientValidationContext>) and #[execution(<your ClientExecutionContext>)]";
const MISSING_VALIDATION_ATTR: &str = "missing #[validation(<your ClientValidationContext>)]";
const MISSING_EXECUTION_ATTR: &str = "missing #[execution(<your ClientExecutionContext>)]";
const INVALID_ATTR: &str = "invalid attribute annotation!";

pub(crate) struct Opts {
    client_validation_context: syn::TypePath,
    client_execution_context: syn::TypePath,
}

impl Opts {
    fn from_derive_input(ast: &DeriveInput) -> Result<Self, syn::Error> {
        let mut client_validation_context = None;
        let mut client_execution_context = None;

        if ast.attrs.is_empty() {
            return Err(syn::Error::new_spanned(ast, MISSING_ATTR));
        }

        for attr in ast.attrs.iter() {
            match attr.meta {
                syn::Meta::List(ref meta_list) => {
                    let type_path: syn::TypePath = syn::parse2(meta_list.tokens.clone())?;

                    match meta_list.path.require_ident() {
                        Ok(ident) => match ident.to_string().as_str() {
                            "validation" => client_validation_context = Some(type_path),
                            "execution" => client_execution_context = Some(type_path),
                            _ => {
                                return Err(syn::Error::new_spanned(&meta_list.path, INVALID_ATTR))
                            }
                        },
                        Err(e) => return Err(syn::Error::new_spanned(attr, e)),
                    }
                }
                _ => continue,
            }
        }

        let client_validation_context = client_validation_context
            .ok_or_else(|| syn::Error::new_spanned(ast, MISSING_VALIDATION_ATTR))?;
        let client_execution_context = client_execution_context
            .ok_or_else(|| syn::Error::new_spanned(ast, MISSING_EXECUTION_ATTR))?;

        Ok(Opts {
            client_validation_context,
            client_execution_context,
        })
    }
}

pub fn client_state_derive_impl(ast: DeriveInput, imports: &Imports) -> TokenStream {
    let opts = match Opts::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(e) => panic!("{e}"),
    };

    let enum_name = &ast.ident;
    let enum_variants = match ast.data {
        syn::Data::Enum(ref enum_data) => &enum_data.variants,
        _ => panic!("ClientState only supports enums"),
    };

    let ClientStateCommon_impl_block = impl_ClientStateCommon(enum_name, enum_variants, imports);
    let ClientStateValidation_impl_block =
        impl_ClientStateValidation(enum_name, enum_variants, &opts, imports);
    let ClientStateExecution_impl_block =
        impl_ClientStateExecution(enum_name, enum_variants, &opts, imports);

    quote! {
        #ClientStateCommon_impl_block
        #ClientStateValidation_impl_block
        #ClientStateExecution_impl_block
    }
}

mod traits;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Error, GenericArgument, Ident, WherePredicate};
use traits::client_state_common::impl_ClientStateCommon;
use traits::client_state_execution::impl_ClientStateExecution;
use traits::client_state_validation::impl_ClientStateValidation;

use crate::utils::Imports;

const MISSING_ATTR: &str = "must be annotated with #[validation(<your ClientValidationContext>) and #[execution(<your ClientExecutionContext>)]";
const MISSING_VALIDATION_ATTR: &str = "missing #[validation(<your ClientValidationContext>)]";
const MISSING_EXECUTION_ATTR: &str = "missing #[execution(<your ClientExecutionContext>)]";
const INVALID_ATTR: &str = "invalid attribute annotation";
const INVALID_ARGS: &str = "invalid context argument";

#[derive(Clone)]
pub(crate) struct ClientCtx {
    ident: Ident,
    generics: Vec<GenericArgument>,
    predicates: Vec<WherePredicate>,
}

impl ClientCtx {
    fn new(ident: Ident, generics: Vec<GenericArgument>, predicates: Vec<WherePredicate>) -> Self {
        Self {
            ident,
            generics,
            predicates,
        }
    }

    /// Returns the `impl` quote block for the given context type, used for
    /// implementing ClientValidation/ExecutionContext on the given enum.
    fn impl_ts(&self) -> TokenStream {
        let gens = self.generics.clone();

        quote! { impl<#(#gens),*> }
    }

    /// Returns the `where` clause quote block for the given context type, used
    /// for implementing ClientValidation/ExecutionContext on the given enum.
    fn where_clause_ts(&self) -> TokenStream {
        let predicates = self.predicates.clone();

        quote! { where #(#predicates),* }
    }
}

impl ToTokens for ClientCtx {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;

        let generics = self.generics.iter().map(|g| g.to_token_stream());

        tokens.extend(quote! { #ident<#(#generics),*> });
    }
}

pub(crate) struct Opts {
    client_validation_context: ClientCtx,
    client_execution_context: ClientCtx,
}

impl Opts {
    /// Returns the `Opts` struct from the given `DeriveInput` AST.
    fn from_derive_input(ast: &DeriveInput) -> Result<Self, Error> {
        let mut client_validation_context = None;
        let mut client_execution_context = None;

        if ast.attrs.is_empty() {
            return Err(Error::new_spanned(ast, MISSING_ATTR));
        }

        for attr in &ast.attrs {
            if let syn::Meta::List(meta_list) = &attr.meta {
                let path: syn::Path = syn::parse2(meta_list.tokens.clone())?;

                let path_segment = match path.segments.last() {
                    Some(segment) => segment.clone(),
                    None => return Err(Error::new_spanned(&meta_list.path, INVALID_ARGS)),
                };

                let meta_ident = match meta_list.path.require_ident() {
                    Ok(ident) => ident.to_string(),
                    Err(e) => return Err(Error::new_spanned(attr, e)),
                };

                let (gens, ps) = split_for_impl(path_segment.arguments)?;

                let ctx = ClientCtx::new(path_segment.ident.clone(), gens, ps);

                match meta_ident.as_str() {
                    "validation" => client_validation_context = Some(ctx),
                    "execution" => client_execution_context = Some(ctx),
                    _ => return Err(Error::new_spanned(&meta_list.path, INVALID_ATTR)),
                };
            }
        }

        let client_validation_context = client_validation_context
            .ok_or_else(|| Error::new_spanned(ast, MISSING_VALIDATION_ATTR))?;
        let client_execution_context = client_execution_context
            .ok_or_else(|| Error::new_spanned(ast, MISSING_EXECUTION_ATTR))?;

        Ok(Self {
            client_validation_context,
            client_execution_context,
        })
    }
}

fn split_for_impl(
    args: syn::PathArguments,
) -> Result<(Vec<GenericArgument>, Vec<WherePredicate>), Error> {
    let mut generics = vec![];
    let mut predicates = vec![];

    if let syn::PathArguments::AngleBracketed(gen) = args {
        for arg in gen.args {
            match arg.clone() {
                GenericArgument::Type(_) | GenericArgument::Lifetime(_) => {
                    generics.push(arg);
                }
                GenericArgument::Constraint(c) => {
                    let ident = c.ident.into_token_stream();

                    let gen = syn::parse2(ident.into_token_stream())?;

                    generics.push(gen);

                    let gen_type_param: syn::TypeParam =
                        syn::parse2(arg.clone().into_token_stream())?;

                    predicates.push(syn::parse2(gen_type_param.into_token_stream())?);
                }
                _ => return Err(Error::new_spanned(arg, INVALID_ARGS)),
            };
        }
    }

    Ok((generics, predicates))
}

pub fn client_state_derive_impl(ast: DeriveInput, imports: &Imports) -> TokenStream {
    let opts = match Opts::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(e) => panic!("{e}"),
    };

    let enum_name = &ast.ident;
    let enum_variants = match &ast.data {
        syn::Data::Enum(enum_data) => &enum_data.variants,
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

use quote::quote;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{parse_macro_input, DataStruct, DeriveInput, Ident as SynIdent};

#[proc_macro_attribute]
pub fn abci_tag(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as SynIdent);

    // Parse the input tokens into a syntax tree
    let item = parse_macro_input!(item as DeriveInput);

    let attrs = if item.attrs.len() == 1 {
        let value = TokenStream::from(item.attrs[0].clone().tokens).to_string();
        let value = value.trim_start_matches('(').trim_end_matches(')');
        let attrs = value
            .split(",")
            .map(|value| Ident::new(value.trim(), item.ident.span()))
            .collect::<Vec<_>>();
        attrs
    } else {
        Vec::<Ident>::new()
    };

    // get the struct name
    let name = item.ident.clone();
    let data = item.data.clone();
    let fields = if let syn::Data::Struct(DataStruct {
        fields: syn::Fields::Named(value),
        ..
    }) = data
    {
        value
            .named
            .into_iter()
            .map(|value| value)
            .collect::<Vec<_>>()
    } else {
        Vec::<syn::Field>::new()
    };

    let field_name = fields[0].ident.clone().unwrap();
    let ty = fields[0].ty.clone();

    let output = quote! {

        #[derive(#(#attrs,)*)]
        pub struct #name {
          pub #field_name: #ty,
        }

        impl From<#name> for tendermint::abci::EventAttribute {
          fn from(attr: #name) -> Self {
            EventAttribute {
              key: #args.parse().unwrap(),
              value: attr.#field_name.as_str().parse().unwrap(),
              index: false,
            }
          }
        }

    };

    output.into()
}

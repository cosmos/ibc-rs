use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ClientState)]
pub fn derive(input: RawTokenStream) -> RawTokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let output = derive_impl(input);

    RawTokenStream::from(output)
}

fn derive_impl(_input: DeriveInput) -> TokenStream {
    quote! {
        pub fn hello() {
            std::println!("hello, world!");
        }
    }
}

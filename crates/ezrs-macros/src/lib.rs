//! Procedural macros for ezrs runtime setup and async tests.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Runs an async main function on Tokio without requiring users to depend on Tokio directly.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let original_name = input.sig.ident.clone();
    let user_name = syn::Ident::new("__ezrs_user_main", original_name.span());
    input.sig.ident = user_name.clone();

    let expanded = quote! {
        #input

        fn #original_name() -> ::ezrs::Result<()> {
            let runtime = ::ezrs::__private::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to build ezrs Tokio runtime");

            runtime.block_on(#user_name())
        }
    };

    expanded.into()
}

/// Runs an async test on Tokio.
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let expanded = quote! {
        #[::ezrs::__private::tokio::test]
        #input
    };

    expanded.into()
}

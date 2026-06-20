use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn isolate(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = syn::parse_macro_input!(input as syn::ItemFn);
    let attrs = &f.attrs;
    let vis = &f.vis;
    let sig = &f.sig;
    let block = &f.block;

    quote! {
        #(#attrs)*
        #vis #sig {
            ::isolation::namespaced(
                env!("CARGO_TARGET_TMPDIR"),
                || async move #block,
            )
        }
    }
    .into()
}

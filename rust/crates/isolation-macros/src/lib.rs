use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn isolate(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = syn::parse_macro_input!(input as syn::ItemFn);
    let attrs = f.attrs;
    let vis = f.vis;
    let mut sig = f.sig;
    let block = f.block;
    let is_async = sig.asyncness.take().is_some();

    if is_async {
        quote! {
            #(#attrs)*
            #vis #sig {
                ::isolation::namespaced(
                    ::tempfile::tempdir().unwrap().keep(),
                    || {
                        let ex = smol::Executor::new();
                        ex.run(async move #block);
                    },
                )
            }
        }
        .into()
    } else {
        quote! {
            #(#attrs)*
            #vis #sig {
                ::isolation::namespaced(
                    ::tempfile::tempdir().unwrap().keep(),
                    || #block,
                )
            }
        }
        .into()
    }
}

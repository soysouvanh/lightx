extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

const _CHANGELOG_VALIDATION: &str = include_str!("../../lightx/CHANGELOG.md");

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.asyncness.is_none() {
        let err = syn::Error::new_spanned(
            &input_fn.sig,
            "lightx_macro::test requires the function to be strictly `async fn`",
        );
        return err.to_compile_error().into();
    }

    let mut has_context = false;
    let mut ctx_ident = syn::Ident::new("ctx", proc_macro2::Span::call_site());
    let mut ctx_path = quote::quote! { crate::RequestContext };

    for input in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input
            && let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                ctx_ident = pat_ident.ident.clone();
                has_context = true;
                if let syn::Type::Reference(type_ref) = &*pat_type.ty
                    && let syn::Type::Path(type_path) = &*type_ref.elem {
                        let path = &type_path.path;
                        ctx_path = quote::quote! { #path };
                    }
                break;
            }
    }

    let vis = &input_fn.vis;
    let ident = &input_fn.sig.ident;
    let output = &input_fn.sig.output;
    let block = &input_fn.block;

    let attr_tokens: proc_macro2::TokenStream = _attr.into();
    let tokio_attr = if attr_tokens.is_empty() {
        quote::quote! { #[tokio::test] }
    } else {
        quote::quote! { #[tokio::test(#attr_tokens)] }
    };

    let expanded = if has_context {
        quote! {
            #tokio_attr
            #vis async fn #ident() #output {
                // 1. Initialize the pristine multidatabase sandbox context
                #[allow(unused_variables, unused_mut)]
                let mut #ctx_ident = match #ctx_path::new_sandbox_context().await {
                    Ok(c) => c,
                    Err(e) => panic!("lightx_macro::test - Failed to initialize sandbox environment: {:?}", e),
                };

                // 2. Execute the user's test logic in isolation
                let test_result = async {
                    #block
                }.await;

                // 3. Inconditional explict ROLLBACK across all databases bounds
                if let Err(e) = #ctx_ident.rollback_all_sandbox_tx().await {
                    panic!("lightx_macro::test - CRITICAL: Sandbox integrity compromised; Rollback Failed: {:?}", e);
                } else {
                    println!("lightx_macro::test - Secure SANDBOX Rollback successful.");
                }

                test_result
            }
        }
    } else {
        quote! {
            #tokio_attr
            #vis async fn #ident() #output {
                let test_result = async {
                    #block
                }.await;
                test_result
            }
        }
    };

    expanded.into()
}

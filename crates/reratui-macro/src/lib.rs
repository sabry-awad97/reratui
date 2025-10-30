//! Reratui Macro - Procedural macros for component definition and RSX syntax
//!
//! This crate provides the `#[component]` attribute macro and `rsx!` macro
//! for declarative UI definition.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Component attribute macro
///
/// Transforms a function into a Reratui component.
/// This is a placeholder implementation that simply wraps the function.
///
/// # Example
/// ```ignore
/// #[component(MyComponent)]
/// fn my_component() -> Element {
///     rsx! { <Block /> }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;

    // Generate the component function
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// RSX macro for declarative UI syntax
///
/// This is a placeholder implementation that returns an empty Element.
/// A full implementation would parse JSX-like syntax and generate VNode structures.
///
/// # Example
/// ```ignore
/// rsx! {
///     <Block title="Hello">
///         <Paragraph>"World"</Paragraph>
///     </Block>
/// }
/// ```
#[proc_macro]
pub fn rsx(_input: TokenStream) -> TokenStream {
    // Placeholder: return an empty Element
    let expanded = quote! {
        reratui::core::Element::new()
    };

    TokenStream::from(expanded)
}

/// Derive macro for Props (placeholder)
#[proc_macro_derive(Props)]
pub fn derive_props(_input: TokenStream) -> TokenStream {
    // Placeholder: no-op for now
    TokenStream::new()
}

/// Main attribute macro for Reratui applications
///
/// Wraps the main function with tokio runtime setup.
/// This is equivalent to `#[tokio::main]` but provides a Reratui-specific entry point.
///
/// # Example
/// ```ignore
/// #[reratui::main]
/// async fn main() -> Result<()> {
///     reratui::render(|| {
///         rsx! { <Counter /> }
///     }).await;
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;

    // Generate the main function with tokio runtime
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name() -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>> {
            ::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
                .block_on(async {
                    #fn_block
                })
        }
    };

    TokenStream::from(expanded)
}

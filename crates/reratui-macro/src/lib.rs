//! Reratui Macro - Procedural macros for component definition and RSX syntax
//!
//! This crate provides the `#[component]` attribute macro and `rsx!` macro
//! for declarative UI definition.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

mod component;
mod rsx;

/// Attribute macro for defining components.
///
/// This macro transforms a function into a component that can be used
/// in RSX expressions.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component_impl(attr, item)
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
pub fn rsx(input: TokenStream) -> TokenStream {
    rsx::rsx_impl(input)
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

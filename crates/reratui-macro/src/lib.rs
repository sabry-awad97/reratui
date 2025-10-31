//! Reratui Macro - Procedural macros for component definition and RSX syntax
//!
//! This crate provides the `#[component]` attribute macro and `rsx!` macro
//! for declarative UI definition.

use proc_macro::TokenStream;

mod component;
mod hook_validator;
mod props;
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

/// Derive macro for component props.
///
/// This macro generates the necessary trait implementations for a struct
/// to be used as component props.
#[proc_macro_derive(Props)]
pub fn derive_props(input: TokenStream) -> TokenStream {
    props::derive_props_impl(input)
}

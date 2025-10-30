//! Code generation for component macro
//!
//! This module implements the code generation phase of component macro processing,
//! creating the appropriate Rust code based on the analyzed component information.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, ItemFn};

use crate::component::error::{ComponentError, ComponentResult};
use crate::component::types::{
    CodeGenConfig, ComponentInfo, ComponentType, GeneratedComponentMetadata,
};

/// Code generator for component functions
///
/// This struct implements the Single Responsibility Principle by focusing
/// solely on code generation. It follows the Open/Closed Principle by
/// being easily extensible for new component patterns through the strategy pattern.
pub struct ComponentCodeGenerator {
    config: CodeGenConfig,
}

impl ComponentCodeGenerator {
    /// Create a new code generator with default configuration
    pub fn new() -> Self {
        Self {
            config: CodeGenConfig::default(),
        }
    }

    /// Create a new code generator with custom configuration
    pub fn with_config(config: CodeGenConfig) -> Self {
        Self { config }
    }

    /// Generate component code based on the analyzed component information
    ///
    /// This method delegates to specialized generators based on the component type,
    /// implementing the Strategy Pattern for different code generation approaches.
    ///
    /// # Arguments
    ///
    /// * `input` - The original function definition
    /// * `component_info` - The analyzed component information
    ///
    /// # Returns
    ///
    /// A `TokenStream` containing the generated component code, or a
    /// `ComponentError` if generation fails.
    pub fn generate(
        &self,
        input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        match &component_info.component_type {
            ComponentType::PropsBased { .. } => {
                self.generate_props_based_component(input, component_info)
            }
            ComponentType::DirectParams { .. } => {
                self.generate_direct_params_component(input, component_info)
            }
            ComponentType::NoParams => self.generate_no_params_component(input, component_info),
        }
    }

    /// Generate code for props-based components
    fn generate_props_based_component(
        &self,
        input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        let generator = PropBasedGenerator::new(&self.config);
        let code = generator.generate(input, component_info)?;

        // Generate metadata for debugging and documentation
        let metadata = GeneratedComponentMetadata::props_based(
            component_info.name.clone(),
            component_info.props_struct_name(),
            component_info.component_struct_name(),
        );

        // Add documentation if enabled
        if self.config.generate_docs {
            let docs = common::generate_docs(component_info, &metadata);
            // Add additional documentation based on component characteristics
            let param_docs = if component_info.has_parameters() {
                quote! {
                    #[doc = "This component accepts parameters."]
                }
            } else {
                quote! {
                    #[doc = "This component does not accept parameters."]
                }
            };
            Ok(quote! {
                #docs
                #param_docs
                #code
            })
        } else {
            Ok(code)
        }
    }

    /// Generate code for direct parameter components
    fn generate_direct_params_component(
        &self,
        input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        let generator = DirectParamsGenerator::new(&self.config);
        let code = generator.generate(input, component_info)?;

        // Generate metadata for debugging and documentation
        let parameter_names = if let Some(params) = component_info.direct_parameters() {
            params.iter().map(|p| p.name.to_string()).collect()
        } else {
            Vec::new()
        };

        let metadata = GeneratedComponentMetadata::direct_params(
            component_info.name.clone(),
            component_info.props_struct_name(),
            component_info.component_struct_name(),
            parameter_names,
        );

        // Add documentation if enabled
        if self.config.generate_docs {
            let docs = common::generate_docs(component_info, &metadata);
            Ok(quote! {
                #docs
                #code
            })
        } else {
            Ok(code)
        }
    }

    /// Generate code for no-parameter components
    fn generate_no_params_component(
        &self,
        input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        let generator = NoParamsGenerator::new(&self.config);
        let code = generator.generate(input, component_info)?;

        // Generate metadata for debugging and documentation
        let metadata = GeneratedComponentMetadata::no_params(
            component_info.name.clone(),
            component_info.props_struct_name(),
            component_info.component_struct_name(),
        );

        // Add documentation if enabled
        if self.config.generate_docs {
            let docs = common::generate_docs(component_info, &metadata);
            Ok(quote! {
                #docs
                #code
            })
        } else {
            Ok(code)
        }
    }
}

impl Default for ComponentCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for component code generators
///
/// This trait implements the Interface Segregation Principle by providing
/// a focused interface for code generation. Different generators can implement
/// this trait to provide specialized generation logic.
trait ComponentGenerator {
    /// Generate component code
    fn generate(
        &self,
        input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream>;
}

/// Common utilities for code generation
///
/// This module provides shared functionality that follows the DRY principle
/// and implements the Dependency Inversion Principle by abstracting common patterns.
pub mod common {
    use super::*;

    /// Generate common component struct implementations
    pub fn generate_component_struct_impl(
        component_info: &ComponentInfo,
        props_struct_name: &syn::Ident,
    ) -> TokenStream {
        let component_struct_name = component_info.component_struct_name();
        let fn_generics = &component_info.generics;
        let (impl_generics, ty_generics, where_clause) = fn_generics.split_for_impl();

        quote! {
            impl #impl_generics Default for #component_struct_name #ty_generics #where_clause {
                fn default() -> Self {
                    Self {
                        props: Default::default(),
                    }
                }
            }

            impl #impl_generics #component_struct_name #ty_generics #where_clause {
                pub fn new(props: #props_struct_name) -> Self {
                    Self { props }
                }

                pub fn with_children(mut self, children: Vec<Element>) -> Self {
                    self.props.set_children(children);
                    self
                }
            }
        }
    }

    /// Generate the Component trait implementation
    pub fn generate_component_trait_impl(component_info: &ComponentInfo) -> TokenStream {
        let component_struct_name = component_info.component_struct_name();
        let fn_name = &component_info.name;
        let fn_generics = &component_info.generics;
        let (impl_generics, ty_generics, where_clause) = fn_generics.split_for_impl();

        quote! {
            impl #impl_generics Component for #component_struct_name #ty_generics #where_clause {
                fn render(&self, area: Rect, buffer: &mut Buffer) {
                    // Provide the component area via context
                    let _area_context = reratui::hooks::context::use_context_provider(|| {
                        reratui::hooks::area::ComponentArea(area)
                    });

                    // Call the component function
                    let element = #fn_name(&self.props);

                    // Render the element
                    element.render(area, buffer);
                }
            }
        }
    }

    /// Generate type alias for the component
    pub fn generate_type_alias(component_info: &ComponentInfo) -> TokenStream {
        let fn_name = &component_info.name;
        let component_struct_name = component_info.component_struct_name();
        let fn_vis = &component_info.visibility;

        quote! {
            #fn_vis type #fn_name = #component_struct_name;
        }
    }

    /// Generate ComponentProps trait implementation
    pub fn generate_component_props_impl(
        props_struct_name: &syn::Ident,
        generics: &Generics,
    ) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        quote! {
            impl #impl_generics ComponentProps for #props_struct_name #ty_generics #where_clause {
                fn get_children(&self) -> Vec<Element> {
                    self.children.clone()
                }

                fn set_children(&mut self, children: Vec<Element>) {
                    self.children = children;
                }
            }
        }
    }

    /// Generate documentation for generated code
    pub fn generate_docs(
        component_info: &ComponentInfo,
        metadata: &GeneratedComponentMetadata,
    ) -> TokenStream {
        let fn_name = &metadata.function_name;
        let component_type_desc = match &component_info.component_type {
            ComponentType::PropsBased { .. } => "props-based",
            ComponentType::DirectParams { .. } => "direct parameters",
            ComponentType::NoParams => "no parameters",
        };

        let doc_comment = format!(
            "Generated component struct for `{}` ({} component)\n\nGenerated methods: {}",
            fn_name,
            component_type_desc,
            metadata.generated_methods.join(", ")
        );

        let children_support_doc = if metadata.supports_children {
            "This component supports children elements."
        } else {
            "This component does not support children elements."
        };

        // Use metadata fields for additional documentation
        let struct_info = if let Some(props_name) = &metadata.props_struct_name {
            format!(
                "Props struct: {}, Component struct: {}",
                props_name, metadata.component_struct_name
            )
        } else {
            format!("Component struct: {}", metadata.component_struct_name)
        };

        quote! {
            #[doc = #doc_comment]
            #[doc = #children_support_doc]
            #[doc = #struct_info]
        }
    }
}

/// Generator for props-based components
struct PropBasedGenerator<'a> {
    config: &'a CodeGenConfig,
}

impl<'a> PropBasedGenerator<'a> {
    fn new(config: &'a CodeGenConfig) -> Self {
        Self { config }
    }
}

impl<'a> ComponentGenerator for PropBasedGenerator<'a> {
    fn generate(
        &self,
        _input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        // Extract props information using utility method
        let (props_type, props_param_name) = component_info.props_info().ok_or_else(|| {
            ComponentError::internal_error(
                "PropBasedGenerator called with non-props-based component",
                "generate",
            )
        })?;

        let fn_name = &component_info.name;
        let fn_vis = &component_info.visibility;
        let fn_block = &component_info.block;
        let fn_generics = &component_info.generics;
        let return_type = &component_info.return_type;
        let component_struct_name = component_info.component_struct_name();
        let (_impl_generics, _ty_generics, where_clause) = fn_generics.split_for_impl();

        // Use configuration to determine derives
        // Be conservative with Debug derive since VNode doesn't implement Debug
        let derives = if self.config.custom_derives.is_empty() {
            quote! { #[derive(Clone)] }
        } else {
            let derive_list = self
                .config
                .custom_derives
                .iter()
                .map(|d| syn::Ident::new(d, proc_macro2::Span::call_site()));
            quote! { #[derive(#(#derive_list),*)] }
        };

        // Generate the complete component code using common utilities
        let props_struct_name = &component_info.props_struct_name();
        let component_struct_impl =
            common::generate_component_struct_impl(component_info, props_struct_name);
        let component_trait_impl = common::generate_component_trait_impl(component_info);
        let type_alias = common::generate_type_alias(component_info);

        // Add debug information if enabled
        let debug_info = if self.config.debug_info {
            let debug_msg = format!("Props-based component: {}", fn_name);
            quote! {
                #[doc = #debug_msg]
                #[doc = "Generated with debug information enabled"]
            }
        } else {
            quote! {}
        };

        let expanded = quote! {
            #debug_info

            // Keep the original function for direct usage
            #[allow(non_snake_case)]
            #fn_vis fn #fn_name #fn_generics(#props_param_name: &#props_type) -> #return_type #where_clause #fn_block

            // Generate a component struct that wraps the function
            #derives
            #fn_vis struct #component_struct_name #fn_generics #where_clause {
                props: #props_type,
            }

            // Use common implementations
            #component_struct_impl
            #component_trait_impl
            #type_alias
        };

        Ok(expanded)
    }
}

/// Generator for direct parameter components
struct DirectParamsGenerator<'a> {
    config: &'a CodeGenConfig,
}

impl<'a> DirectParamsGenerator<'a> {
    fn new(config: &'a CodeGenConfig) -> Self {
        Self { config }
    }
}

impl<'a> ComponentGenerator for DirectParamsGenerator<'a> {
    fn generate(
        &self,
        _input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        // Extract parameters information
        let parameters = match &component_info.component_type {
            ComponentType::DirectParams { parameters } => parameters,
            _ => {
                return Err(ComponentError::internal_error(
                    "DirectParamsGenerator called with non-direct-params component",
                    "generate",
                ));
            }
        };

        let fn_name = &component_info.name;
        let fn_vis = &component_info.visibility;
        let fn_block = &component_info.block;
        let fn_generics = &component_info.generics;
        let return_type = &component_info.return_type;
        let props_struct_name = component_info.props_struct_name();
        let component_struct_name = component_info.component_struct_name();
        let original_fn_name = component_info.original_function_name();
        let (impl_generics, ty_generics, where_clause) = fn_generics.split_for_impl();

        // Use configuration to determine derives
        // Be conservative with Debug derive since VNode doesn't implement Debug
        let derives = if self.config.custom_derives.is_empty() {
            quote! { #[derive(Clone)] }
        } else {
            let derive_list = self
                .config
                .custom_derives
                .iter()
                .map(|d| syn::Ident::new(d, proc_macro2::Span::call_site()));
            quote! { #[derive(#(#derive_list),*)] }
        };

        // Generate field declarations for props struct
        let prop_fields = parameters.iter().map(|param| {
            let name = &param.name;
            let param_type = &param.param_type;
            quote! { pub #name: #param_type }
        });

        // Generate default values for props struct
        let default_fields = parameters.iter().map(|param| {
            let name = &param.name;
            quote! { #name: Default::default() }
        });

        // Generate builder methods for props struct
        let builder_methods = parameters.iter().map(|param| {
            let name = &param.name;
            let param_type = &param.param_type;
            quote! {
                pub fn #name(mut self, #name: #param_type) -> Self {
                    self.#name = #name;
                    self
                }
            }
        });

        // Generate parameter list for function call
        let param_names = parameters.iter().map(|param| &param.name);
        let param_list = quote! { #(props.#param_names.clone()),* };

        // Generate original function parameters
        let original_params = parameters.iter().map(|param| {
            let name = &param.name;
            let param_type = &param.param_type;
            quote! { #name: #param_type }
        });

        // Generate common implementations using utility functions
        let component_props_impl =
            common::generate_component_props_impl(&props_struct_name, fn_generics);
        let component_struct_impl =
            common::generate_component_struct_impl(component_info, &props_struct_name);
        let component_trait_impl = common::generate_component_trait_impl(component_info);
        let type_alias = common::generate_type_alias(component_info);

        let expanded = quote! {
            // Generate props struct
            #derives
            #fn_vis struct #props_struct_name #fn_generics #where_clause {
                #(#prop_fields,)*
                pub children: Vec<Element>,
            }

            impl #impl_generics Default for #props_struct_name #ty_generics #where_clause {
                fn default() -> Self {
                    Self {
                        #(#default_fields,)*
                        children: Vec::new(),
                    }
                }
            }

            impl #impl_generics #props_struct_name #ty_generics #where_clause {
                #(#builder_methods)*

                pub fn with_children(mut self, children: Vec<Element>) -> Self {
                    self.children = children;
                    self
                }
            }

            // Use common ComponentProps implementation
            #component_props_impl

            // Keep the original function for direct usage with a different name
            #[allow(non_snake_case)]
            fn #original_fn_name #fn_generics(#(#original_params),*) -> #return_type #where_clause #fn_block

            // Generate the main component function that takes props
            #[allow(non_snake_case)]
            #fn_vis fn #fn_name #fn_generics(props: &#props_struct_name) -> #return_type #where_clause {
                #original_fn_name(#param_list)
            }

            // Generate a component struct that wraps the function
            #[derive(Clone)]
            #fn_vis struct #component_struct_name #fn_generics #where_clause {
                props: #props_struct_name,
            }

            // Use common implementations
            #component_struct_impl
            #component_trait_impl
            #type_alias
        };

        Ok(expanded)
    }
}

/// Generator for no-parameter components
struct NoParamsGenerator<'a> {
    config: &'a CodeGenConfig,
}

impl<'a> NoParamsGenerator<'a> {
    fn new(config: &'a CodeGenConfig) -> Self {
        Self { config }
    }
}

impl<'a> ComponentGenerator for NoParamsGenerator<'a> {
    fn generate(
        &self,
        _input: &ItemFn,
        component_info: &ComponentInfo,
    ) -> ComponentResult<TokenStream> {
        // Verify this is a no-params component
        match &component_info.component_type {
            ComponentType::NoParams => {}
            _ => {
                return Err(ComponentError::internal_error(
                    "NoParamsGenerator called with non-no-params component",
                    "generate",
                ));
            }
        };

        let fn_name = &component_info.name;
        let fn_vis = &component_info.visibility;
        let fn_block = &component_info.block;
        let fn_generics = &component_info.generics;
        let return_type = &component_info.return_type;
        let props_struct_name = component_info.props_struct_name();
        let component_struct_name = component_info.component_struct_name();
        let original_fn_name = component_info.original_function_name();
        let (impl_generics, ty_generics, where_clause) = fn_generics.split_for_impl();

        // Use configuration to determine derives
        // Be conservative with Debug derive since VNode doesn't implement Debug
        let derives = if self.config.custom_derives.is_empty() {
            quote! { #[derive(Clone, Default)] }
        } else {
            let mut derive_list: Vec<_> = self
                .config
                .custom_derives
                .iter()
                .map(|d| syn::Ident::new(d, proc_macro2::Span::call_site()))
                .collect();
            // Always include Default for no-params components
            if !self.config.custom_derives.contains(&"Default".to_string()) {
                derive_list.push(syn::Ident::new("Default", proc_macro2::Span::call_site()));
            }
            quote! { #[derive(#(#derive_list),*)] }
        };

        // Generate common implementations using utility functions
        let component_props_impl =
            common::generate_component_props_impl(&props_struct_name, fn_generics);
        let component_struct_impl =
            common::generate_component_struct_impl(component_info, &props_struct_name);
        let component_trait_impl = common::generate_component_trait_impl(component_info);
        let type_alias = common::generate_type_alias(component_info);

        let expanded = quote! {
            // Generate empty props struct
            #derives
            #fn_vis struct #props_struct_name #fn_generics #where_clause {
                pub children: Vec<Element>,
            }

            impl #impl_generics #props_struct_name #ty_generics #where_clause {
                pub fn with_children(mut self, children: Vec<Element>) -> Self {
                    self.children = children;
                    self
                }
            }

            // Use common ComponentProps implementation
            #component_props_impl

            // Keep the original function for direct usage with a different name
            #[allow(non_snake_case)]
            fn #original_fn_name #fn_generics() -> #return_type #where_clause #fn_block

            // Generate the main component function that takes props
            #[allow(non_snake_case)]
            #fn_vis fn #fn_name #fn_generics(_props: &#props_struct_name) -> #return_type #where_clause {
                #original_fn_name()
            }

            // Generate a component struct that wraps the function
            #[derive(Clone)]
            #fn_vis struct #component_struct_name #fn_generics #where_clause {
                props: #props_struct_name,
            }

            // Use common implementations
            #component_struct_impl
            #component_trait_impl
            #type_alias
        };

        Ok(expanded)
    }
}

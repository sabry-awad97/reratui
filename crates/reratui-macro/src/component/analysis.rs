//! Component analysis logic
//!
//! This module implements the analysis phase of component macro processing,
//! determining the component type and extracting relevant metadata for code generation.

use syn::{FnArg, ItemFn, Pat, PatType, ReturnType, Type};

use crate::component::error::{ComponentError, ComponentResult, IntoComponentError};
use crate::component::types::{ComponentInfo, ComponentParameter, ComponentType};

/// Analyzer for component function definitions
///
/// This struct implements the Single Responsibility Principle by focusing
/// solely on analyzing component functions and extracting metadata.
/// It follows the Open/Closed Principle by being easily extensible for
/// new component patterns.
pub struct ComponentAnalyzer;

impl ComponentAnalyzer {
    /// Create a new component analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze a component function and extract metadata
    ///
    /// This method determines the component type and extracts all necessary
    /// information for code generation. It delegates to specialized analyzers
    /// for more focused analysis based on the component pattern.
    ///
    /// # Arguments
    ///
    /// * `input` - The validated function definition to analyze
    ///
    /// # Returns
    ///
    /// A `ComponentInfo` struct containing all extracted metadata, or a
    /// `ComponentError` if analysis fails.
    pub fn analyze(&self, input: &ItemFn) -> ComponentResult<ComponentInfo> {
        // Use specialized analyzers based on parameter count and type
        let param_count = utils::count_parameters(input);

        match param_count {
            0 => {
                // Use specialized no-params analyzer
                specialized::NoParamsAnalyzer::analyze(input)
            }
            1 => {
                // Check if it's props-based or single direct parameter
                if let Some(FnArg::Typed(PatType { ty, .. })) = input.sig.inputs.first() {
                    if utils::is_reference_type(ty.as_ref()) {
                        specialized::PropBasedAnalyzer::analyze(input)
                    } else {
                        specialized::DirectParamsAnalyzer::analyze(input)
                    }
                } else {
                    Err(ComponentError::invalid_parameters(
                        &input.sig,
                        "Invalid parameter type",
                        Some("Component parameters must be typed parameters"),
                    ))
                }
            }
            _ => {
                // Multiple parameters - must be direct parameters
                specialized::DirectParamsAnalyzer::analyze(input)
            }
        }
    }

    /// Determine the type of component based on function signature
    fn determine_component_type(&self, input: &ItemFn) -> ComponentResult<ComponentType> {
        let param_count = utils::count_parameters(input);

        match param_count {
            0 => Ok(ComponentType::NoParams),
            1 => {
                // Could be props-based or single direct parameter
                let param = input.sig.inputs.first().unwrap();
                if self.is_props_based_parameter(param)? {
                    self.extract_props_based_info(param)
                } else {
                    self.extract_single_direct_param(param)
                }
            }
            _ => {
                // Multiple parameters - must be direct parameters
                self.extract_direct_parameters(input)
            }
        }
    }

    /// Check if a parameter indicates a props-based component
    fn is_props_based_parameter(&self, param: &FnArg) -> ComponentResult<bool> {
        match param {
            FnArg::Typed(PatType { ty, .. }) => {
                // Props-based components have reference parameters
                Ok(utils::is_reference_type(ty.as_ref()))
            }
            FnArg::Receiver(_) => Err(ComponentError::invalid_parameters(
                param,
                "Component functions cannot have self parameters",
                Some("Use standalone functions for components"),
            )),
        }
    }

    /// Extract props-based component information
    fn extract_props_based_info(&self, param: &FnArg) -> ComponentResult<ComponentType> {
        match param {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // Extract parameter name using utility
                let param_name = utils::extract_identifier(pat.as_ref())
                    .ok_or_else(|| {
                        ComponentError::invalid_parameters(
                            pat,
                            "Props parameter must be a simple identifier",
                            Some("Use a simple name like 'props: &MyProps'"),
                        )
                    })?
                    .clone();

                // Extract inner type from reference using utility
                let props_type = utils::extract_reference_inner(ty.as_ref())
                    .ok_or_else(|| {
                        ComponentError::invalid_parameters(
                            ty,
                            "Props parameter must be a reference type",
                            Some("Use &PropsStruct instead of PropsStruct"),
                        )
                    })?
                    .clone();

                Ok(ComponentType::PropsBased {
                    props_type: Box::new(props_type),
                    props_param_name: param_name,
                })
            }
            _ => unreachable!("Already validated as typed parameter"),
        }
    }

    /// Extract single direct parameter information
    fn extract_single_direct_param(&self, param: &FnArg) -> ComponentResult<ComponentType> {
        let component_param = self.extract_component_parameter(param)?;
        Ok(ComponentType::DirectParams {
            parameters: vec![component_param],
        })
    }

    /// Extract direct parameters information
    fn extract_direct_parameters(&self, input: &ItemFn) -> ComponentResult<ComponentType> {
        let mut parameters = Vec::new();

        for param in &input.sig.inputs {
            let component_param = self.extract_component_parameter(param)?;
            parameters.push(component_param);
        }

        Ok(ComponentType::DirectParams { parameters })
    }

    /// Extract a single component parameter
    fn extract_component_parameter(&self, param: &FnArg) -> ComponentResult<ComponentParameter> {
        match param {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // Use utility function to extract identifier
                let param_name = utils::extract_identifier(pat.as_ref())
                    .ok_or_else(|| {
                        ComponentError::invalid_parameters(
                            pat,
                            "Parameter must be a simple identifier",
                            Some("Use simple parameter names like 'name: String'"),
                        )
                    })?
                    .clone();

                Ok(ComponentParameter::new(param_name, ty.as_ref().clone()))
            }
            FnArg::Receiver(_) => Err(ComponentError::invalid_parameters(
                param,
                "Component functions cannot have self parameters",
                Some("Use standalone functions for components"),
            )),
        }
    }

    /// Extract the return type from the function signature
    fn extract_return_type(&self, input: &ItemFn) -> ComponentResult<Type> {
        match &input.sig.output {
            ReturnType::Type(_, return_type) => {
                // Use IntoComponentError trait for potential parsing errors
                self.validate_return_type_syntax(return_type.as_ref())
                    .into_component_error()?;
                Ok(return_type.as_ref().clone())
            }
            ReturnType::Default => Err(ComponentError::invalid_return_type(
                &input.sig,
                "Component function must have an explicit return type",
                "Element, VNode",
            )),
        }
    }

    /// Validate return type syntax (demonstrates IntoComponentError usage)
    fn validate_return_type_syntax(&self, return_type: &Type) -> syn::Result<()> {
        // This is a demonstration of where IntoComponentError would be useful
        // In a real scenario, this might involve complex type parsing
        match return_type {
            Type::Path(_) => Ok(()),
            Type::Reference(_) => Ok(()),
            Type::Tuple(_) => Ok(()),
            _ => {
                // Create a syn error that can be converted using IntoComponentError
                Err(syn::Error::new_spanned(
                    return_type,
                    "Unsupported return type syntax for component function",
                ))
            }
        }
    }
}

impl Default for ComponentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized analyzers for different component patterns
pub mod specialized {
    use super::*;

    /// Analyzer specifically for props-based components
    pub struct PropBasedAnalyzer;

    impl PropBasedAnalyzer {
        /// Analyze a props-based component
        pub fn analyze(input: &ItemFn) -> ComponentResult<ComponentInfo> {
            let analyzer = ComponentAnalyzer::new();

            // Use the main analyzer's helper methods
            let component_type = analyzer.determine_component_type(input)?;
            let return_type = analyzer.extract_return_type(input)?;

            // Verify it's actually props-based
            match &component_type {
                ComponentType::PropsBased { .. } => {
                    // Create component info
                    let component_info = ComponentInfo::new(
                        component_type,
                        input.sig.ident.clone(),
                        input.vis.clone(),
                        input.block.as_ref().clone(),
                        input.sig.generics.clone(),
                        return_type,
                    );

                    Ok(component_info)
                }
                _ => Err(ComponentError::internal_error(
                    "Expected props-based component but got different type",
                    "PropBasedAnalyzer::analyze",
                )),
            }
        }
    }

    /// Analyzer specifically for direct parameter components
    pub struct DirectParamsAnalyzer;

    impl DirectParamsAnalyzer {
        /// Analyze a direct parameter component
        pub fn analyze(input: &ItemFn) -> ComponentResult<ComponentInfo> {
            let analyzer = ComponentAnalyzer::new();

            // Use the main analyzer's helper methods
            let component_type = analyzer.determine_component_type(input)?;
            let return_type = analyzer.extract_return_type(input)?;

            // Verify it's actually direct parameters
            match &component_type {
                ComponentType::DirectParams { .. } => {
                    // Create component info
                    let component_info = ComponentInfo::new(
                        component_type,
                        input.sig.ident.clone(),
                        input.vis.clone(),
                        input.block.as_ref().clone(),
                        input.sig.generics.clone(),
                        return_type,
                    );

                    Ok(component_info)
                }
                _ => Err(ComponentError::internal_error(
                    "Expected direct parameters component but got different type",
                    "DirectParamsAnalyzer::analyze",
                )),
            }
        }
    }

    /// Analyzer specifically for no-parameter components
    pub struct NoParamsAnalyzer;

    impl NoParamsAnalyzer {
        /// Analyze a no-parameter component
        pub fn analyze(input: &ItemFn) -> ComponentResult<ComponentInfo> {
            let analyzer = ComponentAnalyzer::new();

            // Use the main analyzer's helper methods
            let component_type = analyzer.determine_component_type(input)?;
            let return_type = analyzer.extract_return_type(input)?;

            // Verify it's actually no parameters
            match &component_type {
                ComponentType::NoParams => {
                    // Create component info
                    let component_info = ComponentInfo::new(
                        component_type,
                        input.sig.ident.clone(),
                        input.vis.clone(),
                        input.block.as_ref().clone(),
                        input.sig.generics.clone(),
                        return_type,
                    );

                    Ok(component_info)
                }
                _ => Err(ComponentError::internal_error(
                    "Expected no-parameter component but got different type",
                    "NoParamsAnalyzer::analyze",
                )),
            }
        }
    }
}

/// Utility functions for component analysis
pub mod utils {
    use super::*;

    /// Check if a type is a reference type
    pub fn is_reference_type(ty: &Type) -> bool {
        matches!(ty, Type::Reference(_))
    }

    /// Extract the inner type from a reference type
    pub fn extract_reference_inner(ty: &Type) -> Option<&Type> {
        match ty {
            Type::Reference(type_ref) => Some(type_ref.elem.as_ref()),
            _ => None,
        }
    }

    /// Check if a pattern is a simple identifier
    pub fn is_simple_identifier(pat: &Pat) -> bool {
        matches!(pat, Pat::Ident(_))
    }

    /// Extract identifier from a simple pattern
    pub fn extract_identifier(pat: &Pat) -> Option<&syn::Ident> {
        match pat {
            Pat::Ident(pat_ident) => Some(&pat_ident.ident),
            _ => None,
        }
    }

    /// Count the number of parameters in a function
    pub fn count_parameters(input: &ItemFn) -> usize {
        input.sig.inputs.len()
    }

    /// Check if a function has generics
    pub fn has_generics(input: &ItemFn) -> bool {
        !input.sig.generics.params.is_empty()
    }
}

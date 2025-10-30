//! Validation logic for component functions
//!
//! This module implements comprehensive validation for component function definitions,
//! ensuring they meet the requirements for code generation and providing helpful
//! error messages when they don't.

use syn::{FnArg, ItemFn, Pat, PatType, ReturnType};

use crate::component::error::{ComponentError, ComponentResult, messages};
use crate::component::types::ValidationConfig;

/// Validator for component function definitions
///
/// This struct implements the Single Responsibility Principle by focusing
/// solely on validation logic. It can be easily extended with new validation
/// rules without modifying existing code.
pub struct ComponentValidator {
    config: ValidationConfig,
}

impl ComponentValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    /// Create a new validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate a component function definition
    ///
    /// This method performs comprehensive validation including:
    /// - Function signature validation
    /// - Return type validation
    /// - Parameter validation
    /// - Generics validation
    ///
    /// # Arguments
    ///
    /// * `input` - The function definition to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, or a `ComponentError` with detailed
    /// information about what went wrong and how to fix it.
    pub fn validate(&self, input: &ItemFn) -> ComponentResult<()> {
        // Validate function signature
        self.validate_signature(input)?;

        // Validate return type
        self.validate_return_type(input)?;

        // Validate parameters
        self.validate_parameters(input)?;

        // Validate generics (if present)
        self.validate_generics(input)?;

        // Perform specialized validation based on component type
        self.validate_component_type_specific(input)?;

        Ok(())
    }

    /// Perform component-type-specific validation
    fn validate_component_type_specific(&self, input: &ItemFn) -> ComponentResult<()> {
        // Use utility function to count parameters
        let param_count = crate::component::analysis::utils::count_parameters(input);

        match param_count {
            0 => {
                // Use specialized no-params validator
                specialized::NoParamsValidator::validate(input)
            }
            1 => {
                // Check if it's props-based or single direct parameter
                if let Some(FnArg::Typed(PatType { ty, .. })) = input.sig.inputs.first() {
                    // Use utility function to check reference type
                    if crate::component::analysis::utils::is_reference_type(ty.as_ref()) {
                        specialized::PropBasedValidator::validate(input)
                    } else {
                        specialized::DirectParamsValidator::validate(input)
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
                specialized::DirectParamsValidator::validate(input)
            }
        }
    }

    /// Validate the function signature
    fn validate_signature(&self, input: &ItemFn) -> ComponentResult<()> {
        // Check for async functions
        if input.sig.asyncness.is_some() && !self.config.allow_async {
            return Err(ComponentError::invalid_signature(
                &input.sig,
                messages::signature::ASYNC_NOT_SUPPORTED,
                Some("Remove the 'async' keyword from the component function"),
            ));
        }

        // Check for unsafe functions
        if input.sig.unsafety.is_some() && !self.config.allow_unsafe {
            return Err(ComponentError::invalid_signature(
                &input.sig,
                messages::signature::UNSAFE_NOT_SUPPORTED,
                Some("Remove the 'unsafe' keyword from the component function"),
            ));
        }

        // Check for explicit return type if required
        if self.config.require_explicit_return && matches!(input.sig.output, ReturnType::Default) {
            return Err(ComponentError::invalid_return_type(
                &input.sig,
                messages::signature::MISSING_RETURN_TYPE,
                "Element, VNode",
            ));
        }

        Ok(())
    }

    /// Validate the return type
    fn validate_return_type(&self, input: &ItemFn) -> ComponentResult<()> {
        match &input.sig.output {
            ReturnType::Type(_, return_type) => {
                // For now, we accept any explicit return type
                // In the future, we could add more specific validation
                // to ensure it's Element, VNode, etc.
                self.validate_return_type_compatibility(return_type)?;
            }
            ReturnType::Default => {
                if self.config.require_explicit_return {
                    return Err(ComponentError::invalid_return_type(
                        &input.sig,
                        messages::signature::MISSING_RETURN_TYPE,
                        messages::signature::INVALID_RETURN_TYPE,
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate return type compatibility
    fn validate_return_type_compatibility(&self, _return_type: &syn::Type) -> ComponentResult<()> {
        // TODO: Add specific validation for known return types
        // For now, we accept any type and let the compiler catch issues
        Ok(())
    }

    /// Validate function parameters
    fn validate_parameters(&self, input: &ItemFn) -> ComponentResult<()> {
        // Use utility function to count parameters
        let param_count = crate::component::analysis::utils::count_parameters(input);

        // Check parameter count for direct parameter components
        if param_count > 1 && param_count > self.config.max_direct_params {
            return Err(ComponentError::invalid_parameters(
                &input.sig,
                format!(
                    "Too many parameters ({}). Maximum allowed: {}",
                    param_count, self.config.max_direct_params
                ),
                Some("Consider using a props struct for components with many parameters"),
            ));
        }

        // Validate each parameter
        for param in &input.sig.inputs {
            self.validate_parameter(param)?;
        }

        Ok(())
    }

    /// Validate a single parameter
    fn validate_parameter(&self, param: &FnArg) -> ComponentResult<()> {
        match param {
            FnArg::Receiver(_) => Err(ComponentError::invalid_parameters(
                param,
                messages::parameters::SELF_PARAMETER,
                Some("Component functions should be standalone functions, not methods"),
            )),
            FnArg::Typed(typed_param) => {
                // Validate parameter pattern
                self.validate_parameter_pattern(&typed_param.pat)?;

                // Validate parameter type
                self.validate_parameter_type(&typed_param.ty)?;

                Ok(())
            }
        }
    }

    /// Validate parameter pattern (must be simple identifier)
    fn validate_parameter_pattern(&self, pattern: &Pat) -> ComponentResult<()> {
        match pattern {
            Pat::Ident(_) => Ok(()),
            _ => Err(ComponentError::invalid_parameters(
                pattern,
                messages::parameters::INVALID_PATTERN,
                Some("Use simple parameter names like 'name: String' instead of destructuring"),
            )),
        }
    }

    /// Validate parameter type
    fn validate_parameter_type(&self, _param_type: &syn::Type) -> ComponentResult<()> {
        // TODO: Add specific validation for parameter types
        // For now, we accept any type and let the compiler catch issues
        Ok(())
    }

    /// Validate generics usage
    fn validate_generics(&self, input: &ItemFn) -> ComponentResult<()> {
        // Use utility function to check if component has generics
        if !crate::component::analysis::utils::has_generics(input) {
            return Ok(()); // No generics to validate
        }

        let generics = &input.sig.generics;

        // Check for complex generic bounds
        for param in &generics.params {
            match param {
                syn::GenericParam::Type(type_param) => {
                    if !type_param.bounds.is_empty() {
                        // For now, we allow bounded type parameters but warn about complexity
                        // In the future, we might want to be more restrictive
                    }
                }
                syn::GenericParam::Lifetime(lifetime_param) => {
                    // Lifetime parameters are allowed but have limited support
                    // Warn about potential limitations
                    if lifetime_param.bounds.len() > 2 {
                        return Err(ComponentError::unsupported_generics(
                            lifetime_param,
                            "Complex lifetime bounds may not be fully supported",
                            messages::generics::LIFETIME_PARAMETERS,
                        ));
                    }
                }
                syn::GenericParam::Const(_) => {
                    // Const generics are allowed
                }
            }
        }

        // Check where clause complexity
        if let Some(where_clause) = &generics.where_clause
            && where_clause.predicates.len() > 5
        {
            return Err(ComponentError::unsupported_generics(
                where_clause,
                "Complex where clauses may not be fully supported",
                messages::generics::COMPLEX_BOUNDS,
            ));
        }

        Ok(())
    }
}

impl Default for ComponentValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized validators for different component types
pub mod specialized {
    use super::*;

    /// Validator specifically for props-based components
    pub struct PropBasedValidator;

    impl PropBasedValidator {
        /// Validate a props-based component
        pub fn validate(input: &ItemFn) -> ComponentResult<()> {
            // Additional validation specific to props-based components
            if input.sig.inputs.len() != 1 {
                return Err(ComponentError::invalid_parameters(
                    &input.sig,
                    "Props-based components must have exactly one parameter",
                    Some(messages::parameters::REFERENCE_PARAMETER_SUGGESTION),
                ));
            }

            // Validate that the parameter is a reference using utility function
            if let Some(FnArg::Typed(PatType { ty, .. })) = input.sig.inputs.first()
                && !crate::component::analysis::utils::is_reference_type(ty.as_ref())
            {
                return Err(ComponentError::invalid_parameters(
                    ty,
                    "Props parameter must be a reference type",
                    Some("Use &PropsStruct instead of PropsStruct"),
                ));
            }

            Ok(())
        }
    }

    /// Validator specifically for direct parameter components
    pub struct DirectParamsValidator;

    impl DirectParamsValidator {
        /// Validate a direct parameter component
        pub fn validate(input: &ItemFn) -> ComponentResult<()> {
            // Additional validation specific to direct parameter components
            if input.sig.inputs.is_empty() {
                return Err(ComponentError::invalid_parameters(
                    &input.sig,
                    "Direct parameter components must have at least one parameter",
                    Some(messages::parameters::DIRECT_PARAMETER_SUGGESTION),
                ));
            }

            // Validate that all parameters are simple typed parameters
            for param in &input.sig.inputs {
                match param {
                    FnArg::Receiver(_) => {
                        return Err(ComponentError::invalid_parameters(
                            param,
                            messages::parameters::SELF_PARAMETER,
                            Some("Component functions should be standalone functions, not methods"),
                        ));
                    }
                    FnArg::Typed(PatType { pat, .. }) => {
                        // Use utility function to check if pattern is simple identifier
                        if !crate::component::analysis::utils::is_simple_identifier(pat.as_ref()) {
                            return Err(ComponentError::invalid_parameters(
                                pat,
                                messages::parameters::INVALID_PATTERN,
                                Some(
                                    "Use simple parameter names like 'name: String' instead of destructuring",
                                ),
                            ));
                        }
                    }
                }
            }

            Ok(())
        }
    }

    /// Validator specifically for no-parameter components
    pub struct NoParamsValidator;

    impl NoParamsValidator {
        /// Validate a no-parameter component
        pub fn validate(input: &ItemFn) -> ComponentResult<()> {
            // Additional validation specific to no-parameter components
            if !input.sig.inputs.is_empty() {
                return Err(ComponentError::invalid_parameters(
                    &input.sig,
                    "No-parameter components must not have any parameters",
                    Some("Remove all parameters or use a different component pattern"),
                ));
            }

            Ok(())
        }
    }
}

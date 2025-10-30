//! Component macro implementation with SOLID principles and robust error handling
//!
//! This module provides a clean, extensible architecture for generating component code
//! from function definitions. It follows SOLID principles and provides comprehensive
//! error handling with actionable feedback.

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

// Import submodules for clean separation of concerns
mod analysis;
mod codegen;
mod error;
mod types;
mod validation;

use analysis::ComponentAnalyzer;
use codegen::ComponentCodeGenerator;
use error::ComponentResult;
use types::{CodeGenConfig, ValidationConfig};
use validation::ComponentValidator;

/// Main entry point for the component macro
///
/// This function orchestrates the component generation process by:
/// 1. Parsing and validating the input function
/// 2. Analyzing the component type and parameters
/// 3. Generating appropriate code based on the component pattern
///
/// # Arguments
///
/// * `_attr` - Macro attributes (currently unused but reserved for future extensions)
/// * `item` - The function definition to transform into a component
///
/// # Returns
///
/// A `TokenStream` containing the generated component code, or compilation errors
/// if the input is invalid.
///
/// # Examples
///
/// ```ignore
/// #[component]
/// fn MyComponent(name: String, age: u32) -> Element {
///     rsx! { <div>{format!("Hello {}, age {}", name, age)}</div> }
/// }
/// ```
pub fn component_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function with comprehensive error handling
    let input = parse_macro_input!(item as ItemFn);

    // Process the component through our pipeline
    match process_component(input) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error().into(),
    }
}

/// Process a component function through the complete pipeline
///
/// This function implements the main processing pipeline:
/// 1. Validation - Ensure the function is valid for component generation
/// 2. Hook Validation - Ensure hooks follow the Rules of Hooks
/// 3. Analysis - Determine the component type and extract metadata
/// 4. Code Generation - Generate the appropriate component code
///
/// # Arguments
///
/// * `input` - The parsed function definition
///
/// # Returns
///
/// A `Result` containing either the generated `TokenStream` or a `ComponentError`
fn process_component(mut input: ItemFn) -> ComponentResult<TokenStream> {
    // Step 1: Validate the input function with enhanced configuration
    let validation_config = ValidationConfig::default();

    let validator = ComponentValidator::with_config(validation_config);
    validator.validate(&input)?;

    // Step 2: Validate hook usage (Rules of Hooks)
    let hook_validator = crate::hook_validator::HookValidator::new();
    if let Err(errors) = hook_validator.validate(&mut input.block) {
        // Return the first error (they all have descriptive messages)
        let first_error = errors.into_iter().next().unwrap();
        return Err(error::ComponentError::InvalidSyntax(
            first_error.to_string(),
        ));
    }

    // Step 3: Analyze the component type and extract metadata
    let analyzer = ComponentAnalyzer::new();
    let component_info = analyzer.analyze(&input)?;

    // Step 4: Generate the appropriate component code with enhanced configuration
    let mut config = CodeGenConfig::default();

    // Enable debug info for complex components
    if component_info.has_generics() || component_info.has_parameters() {
        config.debug_info = true;
    }

    // Enable children support for all components
    config.children_support = true;

    // Note: We avoid Debug derive since VNode doesn't implement Debug
    // This could be enabled in the future when VNode supports Debug

    let code_generator = ComponentCodeGenerator::with_config(config);
    let generated_code = code_generator.generate(&input, &component_info)?;

    Ok(generated_code.into())
}

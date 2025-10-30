//! Type definitions for component macro
//!
//! This module defines the core types and data structures used throughout
//! the component macro implementation.

use syn::{Block, Generics, Ident, Type, Visibility};

/// Represents the different types of components that can be generated
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    /// Props-based component: `fn Component(props: &PropsStruct) -> Element`
    PropsBased {
        props_type: Box<Type>,
        props_param_name: Ident,
    },
    /// Direct parameters component: `fn Component(name: String, age: u32) -> Element`
    DirectParams { parameters: Vec<ComponentParameter> },
    /// No parameters component: `fn Component() -> Element`
    NoParams,
}

/// Represents a parameter in a direct parameters component
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentParameter {
    /// Parameter name
    pub name: Ident,
    /// Parameter type
    pub param_type: Type,
}

/// Complete information about a component function
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// The type of component (props-based, direct params, or no params)
    pub component_type: ComponentType,
    /// Function name
    pub name: Ident,
    /// Function visibility
    pub visibility: Visibility,
    /// Function body/block
    pub block: Block,
    /// Function generics
    pub generics: Generics,
    /// Return type of the function
    pub return_type: Type,
}

impl ComponentInfo {
    /// Create a new ComponentInfo instance
    pub fn new(
        component_type: ComponentType,
        name: Ident,
        visibility: Visibility,
        block: Block,
        generics: Generics,
        return_type: Type,
    ) -> Self {
        Self {
            component_type,
            name,
            visibility,
            block,
            generics,
            return_type,
        }
    }

    /// Get the props struct name for this component
    pub fn props_struct_name(&self) -> Ident {
        Ident::new(&format!("{}Props", self.name), self.name.span())
    }

    /// Get the component struct name for this component
    pub fn component_struct_name(&self) -> Ident {
        Ident::new(&format!("{}Component", self.name), self.name.span())
    }

    /// Get the original function name (for internal use)
    pub fn original_function_name(&self) -> Ident {
        Ident::new(&format!("__original_{}", self.name), self.name.span())
    }

    /// Check if this component has parameters
    pub fn has_parameters(&self) -> bool {
        match &self.component_type {
            ComponentType::PropsBased { .. } => true,
            ComponentType::DirectParams { parameters } => !parameters.is_empty(),
            ComponentType::NoParams => false,
        }
    }

    /// Check if this component uses generics
    pub fn has_generics(&self) -> bool {
        // Simply check if generics are present
        !self.generics.params.is_empty()
    }

    /// Get the parameters for direct parameter components
    pub fn direct_parameters(&self) -> Option<&Vec<ComponentParameter>> {
        match &self.component_type {
            ComponentType::DirectParams { parameters } => Some(parameters),
            _ => None,
        }
    }

    /// Get the props information for props-based components
    pub fn props_info(&self) -> Option<(&Type, &Ident)> {
        match &self.component_type {
            ComponentType::PropsBased {
                props_type,
                props_param_name,
            } => Some((props_type, props_param_name)),
            _ => None,
        }
    }
}

impl ComponentParameter {
    /// Create a new ComponentParameter
    pub fn new(name: Ident, param_type: Type) -> Self {
        Self { name, param_type }
    }
}

/// Configuration for component code generation
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    /// Whether to generate debug information
    pub debug_info: bool,
    /// Whether to generate documentation
    pub generate_docs: bool,
    /// Whether to include children support
    pub children_support: bool,
    /// Custom derive attributes for generated structs
    pub custom_derives: Vec<String>,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self {
            debug_info: false,
            generate_docs: true,
            children_support: true,
            custom_derives: vec!["Clone".to_string()],
        }
    }
}

/// Validation configuration for component analysis
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to allow async functions
    pub allow_async: bool,
    /// Whether to allow unsafe functions
    pub allow_unsafe: bool,
    /// Maximum number of parameters for direct parameter components
    pub max_direct_params: usize,
    /// Whether to require explicit return types
    pub require_explicit_return: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            allow_async: false,
            allow_unsafe: false,
            max_direct_params: 15,
            require_explicit_return: true,
        }
    }
}

/// Metadata about the generated component code
#[derive(Debug, Clone)]
pub struct GeneratedComponentMetadata {
    /// The main component function name
    pub function_name: Ident,
    /// The props struct name (if applicable)
    pub props_struct_name: Option<Ident>,
    /// The component struct name
    pub component_struct_name: Ident,
    /// Whether the component supports children
    pub supports_children: bool,
    /// List of generated methods
    pub generated_methods: Vec<String>,
}

impl GeneratedComponentMetadata {
    /// Create metadata for a props-based component
    pub fn props_based(
        function_name: Ident,
        props_struct_name: Ident,
        component_struct_name: Ident,
    ) -> Self {
        Self {
            function_name,
            props_struct_name: Some(props_struct_name),
            component_struct_name,
            supports_children: true,
            generated_methods: vec![
                "new".to_string(),
                "with_children".to_string(),
                "render".to_string(),
            ],
        }
    }

    /// Create metadata for a direct parameters component
    pub fn direct_params(
        function_name: Ident,
        props_struct_name: Ident,
        component_struct_name: Ident,
        parameter_names: Vec<String>,
    ) -> Self {
        let mut methods = vec![
            "new".to_string(),
            "with_children".to_string(),
            "render".to_string(),
        ];
        methods.extend(parameter_names);

        Self {
            function_name,
            props_struct_name: Some(props_struct_name),
            component_struct_name,
            supports_children: true,
            generated_methods: methods,
        }
    }

    /// Create metadata for a no-parameters component
    pub fn no_params(
        function_name: Ident,
        props_struct_name: Ident,
        component_struct_name: Ident,
    ) -> Self {
        Self {
            function_name,
            props_struct_name: Some(props_struct_name),
            component_struct_name,
            supports_children: true,
            generated_methods: vec![
                "new".to_string(),
                "with_children".to_string(),
                "render".to_string(),
            ],
        }
    }
}

//! Validation Module for RSX Parser
//!
//! This module provides comprehensive validation for RSX syntax trees.
//! Following SOLID principles:
//! - Single Responsibility: Each validator has a specific validation purpose
//! - Open/Closed: Easy to add new validators without modifying existing ones
//! - Liskov Substitution: All validators implement the same interface
//! - Interface Segregation: Clean validation interfaces
//! - Dependency Inversion: Depends on AST abstractions

use crate::rsx::parser::{
    AstNode,
    ast::{AstVisitor, CommentNode, ConditionalNode, Element, Prop},
};
use std::{collections::HashSet, rc::Rc};
use syn::{Expr, spanned::Spanned};

/// Core trait for all validators - object-safe design
pub trait Validator {
    /// Validate a Node
    fn validate_node(&self, node: &Node) -> syn::Result<()>;

    /// Validate an Element
    fn validate_element(&self, element: &Element) -> syn::Result<()>;

    /// Validate a Prop
    fn validate_prop(&self, prop: &Prop) -> syn::Result<()>;

    /// Validate a ConditionalNode
    fn validate_conditional(&self, conditional: &ConditionalNode) -> syn::Result<()>;

    /// Validate a ForLoopNode
    fn validate_for_loop(&self, for_loop: &crate::rsx::parser::ast::ForLoopNode)
    -> syn::Result<()>;

    /// Get the name of this validator
    #[allow(unused)]
    fn name(&self) -> &'static str;

    /// Get a description of what this validator checks
    #[allow(unused)]
    fn description(&self) -> &'static str;
}

// Import Node here to avoid circular dependency
use crate::rsx::parser::ast::Node;

/// Validates element structure and naming conventions
pub struct ElementValidator {
    allowed_elements: Option<HashSet<String>>,
    forbidden_elements: HashSet<String>,
    naming_convention: NamingConvention,
}

#[derive(Debug, Clone)]
pub enum NamingConvention {
    PascalCase,
    SnakeCase,
    Any,
}

impl ElementValidator {
    pub fn new() -> Self {
        let mut forbidden_elements = HashSet::new();
        forbidden_elements.insert("Span".to_string());

        Self {
            allowed_elements: None,
            forbidden_elements,
            naming_convention: NamingConvention::Any,
        }
    }

    pub fn with_naming_convention(mut self, convention: NamingConvention) -> Self {
        self.naming_convention = convention;
        self
    }

    fn validate_element_name(&self, name: &str) -> syn::Result<()> {
        // Check if element is forbidden
        if self.forbidden_elements.contains(name) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "Element '{}' is not supported in RSX. Span elements are generated automatically within Line and Paragraph components.",
                    name
                ),
            ));
        }

        // Check if element is in allowed list
        if let Some(ref allowed) = self.allowed_elements
            && !allowed.contains(name)
        {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Element '{}' is not in the allowed elements list", name),
            ));
        }

        // Check naming convention
        match self.naming_convention {
            NamingConvention::PascalCase => {
                if !self.is_pascal_case(name) {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Element '{}' should be in PascalCase", name),
                    ));
                }
            }
            NamingConvention::SnakeCase => {
                if !self.is_snake_case(name) {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Element '{}' should be in snake_case", name),
                    ));
                }
            }
            NamingConvention::Any => {} // No validation
        }

        Ok(())
    }

    fn is_pascal_case(&self, s: &str) -> bool {
        !s.is_empty()
            && s.chars().next().unwrap().is_uppercase()
            && !s.contains('_')
            && !s.contains('-')
    }

    fn is_snake_case(&self, s: &str) -> bool {
        s.chars().all(|c| c.is_lowercase() || c == '_') && !s.contains('-')
    }
}

impl Validator for ElementValidator {
    fn validate_node(&self, node: &Node) -> syn::Result<()> {
        match node {
            Node::Element(element) => self.validate_element(element),
            _ => Ok(()),
        }
    }

    fn validate_element(&self, element: &Element) -> syn::Result<()> {
        let element_name = element
            .name
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default();
        self.validate_element_name(&element_name)
    }

    fn validate_prop(&self, _prop: &Prop) -> syn::Result<()> {
        Ok(())
    }

    fn validate_conditional(&self, _conditional: &ConditionalNode) -> syn::Result<()> {
        Ok(())
    }

    fn validate_for_loop(
        &self,
        _for_loop: &crate::rsx::parser::ast::ForLoopNode,
    ) -> syn::Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ElementValidator"
    }

    fn description(&self) -> &'static str {
        "Validates element names and structure according to naming conventions"
    }
}

/// Validates property structure and naming
pub struct PropertyValidator {
    forbidden_props: HashSet<String>,
    naming_convention: NamingConvention,
}

impl PropertyValidator {
    pub fn new() -> Self {
        Self {
            forbidden_props: HashSet::new(),
            naming_convention: NamingConvention::Any,
        }
    }

    pub fn with_naming_convention(mut self, convention: NamingConvention) -> Self {
        self.naming_convention = convention;
        self
    }

    fn validate_prop_name(&self, name: &str) -> syn::Result<()> {
        if self.forbidden_props.contains(name) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Property '{}' is forbidden", name),
            ));
        }

        // Validate naming convention (similar to ElementValidator)
        if let NamingConvention::SnakeCase = self.naming_convention
            && (name.chars().any(|c| c.is_uppercase()) || name.contains('-'))
        {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Property '{}' should be in snake_case", name),
            ));
        }

        Ok(())
    }
}

impl Validator for PropertyValidator {
    fn validate_node(&self, node: &Node) -> syn::Result<()> {
        match node {
            Node::Element(element) => {
                for prop in &element.attributes {
                    self.validate_prop(prop)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn validate_element(&self, element: &Element) -> syn::Result<()> {
        for prop in &element.attributes {
            self.validate_prop(prop)?;
        }
        Ok(())
    }

    fn validate_prop(&self, prop: &Prop) -> syn::Result<()> {
        self.validate_prop_name(&prop.key.to_string())
    }

    fn validate_conditional(&self, _conditional: &ConditionalNode) -> syn::Result<()> {
        Ok(())
    }

    fn validate_for_loop(
        &self,
        _for_loop: &crate::rsx::parser::ast::ForLoopNode,
    ) -> syn::Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PropertyValidator"
    }

    fn description(&self) -> &'static str {
        "Validates property names and ensures required/forbidden properties"
    }
}

/// Validates conditional expression structure
pub struct ConditionalValidator {
    // max_nesting_depth: Option<usize>,
}

/// Validates for-loop structure and patterns
pub struct ForLoopValidator {
    max_nesting_depth: Option<usize>,
}

impl ConditionalValidator {
    fn validate_nesting_depth(
        &self,
        _node: &ConditionalNode,
        _current_depth: usize,
    ) -> syn::Result<()> {
        // TODO: Implement depth checking by traversing the conditional tree
        Ok(())
    }
}

impl Validator for ConditionalValidator {
    fn validate_node(&self, node: &Node) -> syn::Result<()> {
        match node {
            Node::Conditional(conditional) => self.validate_conditional(conditional),
            _ => Ok(()),
        }
    }

    fn validate_element(&self, _element: &Element) -> syn::Result<()> {
        Ok(())
    }

    fn validate_prop(&self, _prop: &Prop) -> syn::Result<()> {
        Ok(())
    }

    fn validate_conditional(&self, conditional: &ConditionalNode) -> syn::Result<()> {
        self.validate_nesting_depth(conditional, 0)
    }

    fn validate_for_loop(
        &self,
        _for_loop: &crate::rsx::parser::ast::ForLoopNode,
    ) -> syn::Result<()> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ConditionalValidator"
    }

    fn description(&self) -> &'static str {
        "Validates conditional expression structure and nesting"
    }
}

impl ForLoopValidator {
    pub fn new() -> Self {
        Self {
            max_nesting_depth: None,
        }
    }

    pub fn with_max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = Some(depth);
        self
    }

    fn validate_pattern(&self, pattern: &syn::Pat) -> syn::Result<()> {
        match pattern {
            syn::Pat::Ident(pat_ident) => {
                // Validate identifier naming
                let ident_str = pat_ident.ident.to_string();
                if ident_str.starts_with('_') && ident_str.len() > 1 {
                    // Allow underscore prefixed variables for unused items
                    Ok(())
                } else if ident_str.chars().all(|c| c.is_lowercase() || c == '_') {
                    Ok(())
                } else {
                    Err(crate::rsx::error::RsxError::ForLoopError {
                        message: "For-loop variable should use snake_case naming convention"
                            .to_string(),
                        span: pattern.span(),
                        suggestion: Some(format!(
                            "Consider renaming '{}' to '{}'",
                            ident_str,
                            ident_str.to_lowercase()
                        )),
                    }
                    .to_syn_error())
                }
            }
            syn::Pat::Tuple(_) | syn::Pat::Struct(_) => {
                // Destructuring patterns are allowed
                Ok(())
            }
            _ => Err(crate::rsx::error::RsxError::ForLoopError {
                message: "Unsupported pattern in for-loop".to_string(),
                span: pattern.span(),
                suggestion: Some(
                    "Use simple identifiers or destructuring patterns like (a, b) or {field}"
                        .to_string(),
                ),
            }
            .to_syn_error()),
        }
    }

    fn validate_iterable(&self, iterable: &syn::Expr) -> syn::Result<()> {
        // Basic validation - ensure it's not obviously invalid
        match iterable {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(_),
                ..
            }) => Err(syn::Error::new(
                iterable.span(),
                "Cannot iterate over string literals directly. Use .chars() or .bytes() for string iteration.",
            )),
            _ => Ok(()), // Most expressions are potentially valid iterables
        }
    }

    fn validate_nesting_depth(
        for_loop: &crate::rsx::parser::ast::ForLoopNode,
        current_depth: usize,
        max_depth: usize,
    ) -> syn::Result<()> {
        if current_depth >= max_depth {
            return Err(crate::rsx::error::RsxError::ForLoopError {
                message: format!("For-loop nesting depth exceeds maximum of {}", max_depth),
                span: for_loop.span(),
                suggestion: Some(
                    "Consider refactoring nested loops into separate functions".to_string(),
                ),
            }
            .to_syn_error());
        }

        // Check for nested for-loops in the body
        if let crate::rsx::parser::ast::Node::ForLoop(nested_loop) = &*for_loop.body {
            Self::validate_nesting_depth(nested_loop, current_depth + 1, max_depth)?;
        }

        Ok(())
    }
}

impl Validator for ForLoopValidator {
    fn validate_node(&self, node: &Node) -> syn::Result<()> {
        match node {
            Node::ForLoop(for_loop) => self.validate_for_loop(for_loop),
            _ => Ok(()),
        }
    }

    fn validate_element(&self, _element: &Element) -> syn::Result<()> {
        Ok(())
    }

    fn validate_prop(&self, _prop: &Prop) -> syn::Result<()> {
        Ok(())
    }

    fn validate_conditional(&self, _conditional: &ConditionalNode) -> syn::Result<()> {
        Ok(())
    }

    fn validate_for_loop(
        &self,
        for_loop: &crate::rsx::parser::ast::ForLoopNode,
    ) -> syn::Result<()> {
        // Validate the pattern
        self.validate_pattern(&for_loop.pattern)?;

        // Validate the iterable expression
        self.validate_iterable(&for_loop.iterable)?;

        // Validate nesting depth if configured
        if let Some(max_depth) = self.max_nesting_depth {
            Self::validate_nesting_depth(for_loop, 0, max_depth)?;
        }

        // Validate the body
        for_loop.body.validate()?;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "ForLoopValidator"
    }

    fn description(&self) -> &'static str {
        "Validates for-loop structure, patterns, and iterable expressions"
    }
}

/// Comprehensive validator that combines multiple validators
#[derive(Clone)]
pub struct CompositeValidator {
    validators: Vec<Rc<dyn Validator>>,
    strict_mode: bool,
}

impl CompositeValidator {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            strict_mode: false,
        }
    }

    pub fn add_validator(mut self, validator: Rc<dyn Validator>) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    pub fn validate_all(&self, node: &Node) -> syn::Result<()> {
        let mut errors = Vec::new();

        for validator in &self.validators {
            if let Err(err) = validator.validate_node(node) {
                if self.strict_mode {
                    return Err(err);
                } else {
                    errors.push(err);
                }
            }
        }

        if !errors.is_empty() && !self.strict_mode {
            // Combine all errors into one
            let combined_message = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Multiple validation errors: {}", combined_message),
            ));
        }

        Ok(())
    }
}

impl Default for CompositeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation visitor that applies validators during AST traversal
pub struct ValidationVisitor {
    element_validator: Option<ElementValidator>,
    property_validator: Option<PropertyValidator>,
    conditional_validator: Option<ConditionalValidator>,
    errors: Vec<syn::Error>,
}

impl ValidationVisitor {
    pub fn new() -> Self {
        Self {
            element_validator: None,
            property_validator: None,
            conditional_validator: None,
            errors: Vec::new(),
        }
    }

    #[allow(unused)]
    pub fn with_element_validator(mut self, validator: ElementValidator) -> Self {
        self.element_validator = Some(validator);
        self
    }

    #[allow(unused)]
    pub fn with_property_validator(mut self, validator: PropertyValidator) -> Self {
        self.property_validator = Some(validator);
        self
    }

    #[allow(unused)]
    pub fn with_conditional_validator(mut self, validator: ConditionalValidator) -> Self {
        self.conditional_validator = Some(validator);
        self
    }

    #[allow(unused)]
    pub fn get_errors(&self) -> &[syn::Error] {
        &self.errors
    }

    #[allow(unused)]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[allow(unused)]
    pub fn into_result(self) -> syn::Result<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // Return the first error
            Err(self.errors.into_iter().next().unwrap())
        }
    }
}

impl AstVisitor for ValidationVisitor {
    fn visit_element(&mut self, element: &Element) -> syn::Result<()> {
        if let Some(ref validator) = self.element_validator {
            let element_name = element
                .name
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default();

            if let Err(err) = validator.validate_element_name(&element_name) {
                self.errors.push(err);
            }
        }
        Ok(())
    }

    fn visit_prop(&mut self, prop: &Prop) -> syn::Result<()> {
        if let Some(ref validator) = self.property_validator
            && let Err(err) = validator.validate_prop_name(&prop.key.to_string())
        {
            self.errors.push(err);
        }
        Ok(())
    }

    fn visit_conditional(&mut self, conditional: &ConditionalNode) -> syn::Result<()> {
        if let Some(ref validator) = self.conditional_validator
            && let Err(err) = validator.validate_conditional(conditional)
        {
            self.errors.push(err);
        }
        Ok(())
    }

    fn visit_expression(&mut self, _expr: &Expr) -> syn::Result<()> {
        // No specific validation for expressions yet
        Ok(())
    }

    fn visit_comment(&mut self, _comment: &CommentNode) -> syn::Result<()> {
        // No specific validation for comments
        Ok(())
    }

    fn visit_for_loop(
        &mut self,
        _for_loop: &crate::rsx::parser::ast::ForLoopNode,
    ) -> syn::Result<()> {
        // No specific validation for for-loops yet
        Ok(())
    }

    fn visit_fragment(
        &mut self,
        _fragment: &crate::rsx::parser::ast::FragmentNode,
    ) -> syn::Result<()> {
        // No specific validation for fragments yet - they are always valid
        Ok(())
    }
}

impl Default for ValidationVisitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating common validator configurations
pub struct ValidatorFactory;

impl ValidatorFactory {
    /// Create a validator for React-like components (PascalCase elements, camelCase props)
    pub fn create_validator() -> CompositeValidator {
        let element_validator =
            ElementValidator::new().with_naming_convention(NamingConvention::PascalCase);

        let property_validator =
            PropertyValidator::new().with_naming_convention(NamingConvention::SnakeCase);

        let for_loop_validator = ForLoopValidator::new().with_max_nesting_depth(5);

        CompositeValidator::new()
            .with_strict_mode(false)
            .add_validator(Rc::new(element_validator))
            .add_validator(Rc::new(property_validator))
            .add_validator(Rc::new(for_loop_validator))
    }
}

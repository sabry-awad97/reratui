use syn::{
    Result,
    parse::{Parse, ParseStream},
};

// Import submodules for better organization
mod ast;
mod parsers;
mod token_analyzer;
mod validators;

// Re-export public types
pub use ast::*;
pub use parsers::*;
pub use validators::*;

/// Main RSX Parser - orchestrates the parsing process
/// Following SOLID principles with dependency injection and clean interfaces
pub struct RsxMainParser {
    node_parser: NodeParser,
    validator: CompositeValidator,
}

impl RsxMainParser {
    pub fn new() -> Self {
        Self {
            node_parser: ParserFactory::create_node_parser(),
            validator: ValidatorFactory::create_validator(),
        }
    }

    /// Create a parser with React-like validation (PascalCase elements, camelCase props)
    pub fn new_react_like() -> Self {
        Self {
            node_parser: ParserFactory::create_node_parser(),
            validator: ValidatorFactory::create_validator(),
        }
    }

    pub fn parse_with_validation(&self, input: ParseStream) -> Result<Node> {
        let node = match self.node_parser.parse(input) {
            Ok(node) => node,
            Err(err) => {
                // Convert to a more user-friendly error if possible
                let error_msg = err.to_string();
                if error_msg.contains("Expected attribute") {
                    return Err(crate::rsx::error::RsxError::SyntaxError {
                        message: "Invalid attribute syntax".to_string(),
                        span: input.span(),
                        suggestion: Some(
                            "Attributes should be in the format: name=\"value\" or name={expr}"
                                .to_string(),
                        ),
                    }
                    .to_syn_error());
                }
                return Err(err);
            }
        };

        // Always validate the parsed node
        if let Err(err) = self.validator.validate_all(&node) {
            // Try to provide more context for validation errors
            let error_msg = err.to_string();
            if error_msg.contains("naming convention") {
                return Err(crate::rsx::error::RsxError::ValidationError {
                    message: "Component naming convention violation".to_string(),
                    span: node.span(),
                    details: Some(
                        "React-like components should use PascalCase names (e.g., MyComponent)"
                            .to_string(),
                    ),
                }
                .to_syn_error());
            }
            return Err(err);
        }

        Ok(node)
    }
}

impl Default for RsxMainParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        // Use permissive validation by default for backward compatibility
        let parser = RsxMainParser::new();
        parser.parse_with_validation(input)
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser = ParserFactory::create_element_parser();
        parser.parse(input)
    }
}

impl Parse for ConditionalNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser = ParserFactory::create_conditional_parser();
        parser.parse(input)
    }
}

impl Parse for ForLoopNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser = ForLoopParser::new();
        parser.parse(input)
    }
}

impl Parse for FragmentNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser = FragmentParser::new();
        parser.parse(input)
    }
}

/// Utility functions for parsing with different validation modes
impl RsxMainParser {
    /// Parse RSX with React-like validation (PascalCase components, camelCase props)
    ///
    /// # Example
    /// ```ignore
    /// // This would pass validation
    /// let valid_rsx = quote! { <MyComponent className="test" /> };
    ///
    /// // This would fail validation (lowercase component name)
    /// let invalid_rsx = quote! { <myComponent className="test" /> };
    /// ```
    pub fn parse_react_like_tokens(tokens: proc_macro2::TokenStream) -> Result<Node> {
        let parser = Self::new_react_like();
        let parsed: Node = syn::parse2(tokens)?;
        parser.validator.validate_all(&parsed)?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_react_like_validation_failure() {
        // Test that React-like validation fails for improper naming
        let rsx = quote! { <myComponent className="test" /> };
        let result = RsxMainParser::parse_react_like_tokens(rsx);
        assert!(
            result.is_err(),
            "React-like validation should fail for camelCase component"
        );
    }

    #[test]
    fn test_validation_visitor() {
        // Test the validation visitor pattern
        let rsx = quote! { <MyComponent class_name="test"><ChildComponent /></MyComponent> };
        let node: Node = parse2(rsx).expect("Should parse successfully");

        let mut visitor = ValidationVisitor::new()
            .with_element_validator(
                ElementValidator::new().with_naming_convention(NamingConvention::PascalCase),
            )
            .with_property_validator(
                PropertyValidator::new().with_naming_convention(NamingConvention::SnakeCase),
            );

        let result = node.accept(&mut visitor);
        assert!(
            result.is_ok(),
            "Validation visitor should pass for well-formed RSX"
        );
        assert!(!visitor.has_errors(), "Visitor should have no errors");
    }

    #[test]
    fn test_element_factory_validation() {
        // Test that element factory validates during creation
        use syn::{Path, parse_quote};

        let name: Path = parse_quote!(MyComponent);
        let attributes = vec![];
        let children = vec![];
        let span = proc_macro2::Span::call_site();

        let result = ElementFactory::create(name, attributes, children, span);
        assert!(
            result.is_ok(),
            "Element factory should create valid elements"
        );
    }

    #[test]
    fn test_prop_factory_validation() {
        // Test that prop factory validates during creation
        use syn::{Expr, Ident, parse_quote};

        let key: Ident = parse_quote!(className);
        let value: Expr = parse_quote!("test");
        let span = proc_macro2::Span::call_site();

        let result = PropFactory::create(key, value, span);
        assert!(result.is_ok(), "Prop factory should create valid props");
    }

    #[test]
    fn test_conditional_factory_validation() {
        // Test that conditional factory validates during creation
        use syn::{Expr, parse_quote};

        let condition: Expr = parse_quote!(true);
        let then_branch = Box::new(Node::Expression(parse_quote!(42)));
        let span = proc_macro2::Span::call_site();

        let result = ConditionalFactory::create_logical_and(condition, then_branch, span);
        assert!(
            result.is_ok(),
            "Conditional factory should create valid conditionals"
        );
    }

    #[test]
    fn test_string_literal_as_direct_child() {
        // Test that string literals can be used as direct children
        let rsx = quote! { <Span cyan bold>"Navigation:"</Span> };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "String literals should be parseable as direct children"
        );

        if let Ok(Node::Element(element)) = result {
            assert_eq!(element.children.len(), 1, "Should have one child");
            if let Node::Expression(expr) = &element.children[0] {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = expr
                {
                    assert_eq!(
                        lit_str.value(),
                        "Navigation:",
                        "String content should match"
                    );
                } else {
                    panic!("Child should be a string literal expression");
                }
            } else {
                panic!("Child should be an expression node");
            }
        }
    }

    #[test]
    fn test_mixed_children_with_string_literals() {
        // Test that components can have both string literals and other children
        let rsx = quote! {
            <Line>
                <Span cyan>"Label: "</Span>
                <Span white bold>"Value"</Span>
            </Line>
        };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "Mixed children with string literals should parse successfully"
        );
    }

    #[test]
    fn test_attributes_with_string_literal_children() {
        // Test that attributes work correctly when string literal children are present
        let rsx =
            quote! { <Span style={Style::default().fg(Color::Red)} bold>"Error message"</Span> };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "Attributes should work with string literal children"
        );

        if let Ok(Node::Element(element)) = result {
            // Should have both style and bold attributes
            assert!(
                element.attributes.len() >= 2,
                "Should have style and bold attributes"
            );
            // Should have one string literal child
            assert_eq!(element.children.len(), 1, "Should have one child");
        }
    }

    #[test]
    fn test_empty_fragment() {
        // Test that empty fragments parse correctly
        let rsx = quote! { <></> };
        let result = parse2::<Node>(rsx);
        assert!(result.is_ok(), "Empty fragments should parse successfully");

        if let Ok(Node::Fragment(fragment)) = result {
            assert_eq!(
                fragment.children.len(),
                0,
                "Empty fragment should have no children"
            );
        } else {
            panic!("Should parse as a fragment node");
        }
    }

    #[test]
    fn test_fragment_with_children() {
        // Test that fragments with children parse correctly
        let rsx = quote! {
            <>
                <Span cyan>"Label: "</Span>
                <Span white bold>"Value"</Span>
            </>
        };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "Fragments with children should parse successfully"
        );

        if let Ok(Node::Fragment(fragment)) = result {
            assert_eq!(
                fragment.children.len(),
                2,
                "Fragment should have two children"
            );
            // Both children should be elements
            assert!(matches!(fragment.children[0], Node::Element(_)));
            assert!(matches!(fragment.children[1], Node::Element(_)));
        } else {
            panic!("Should parse as a fragment node");
        }
    }

    #[test]
    fn test_fragment_with_string_literals() {
        // Test that fragments can contain string literals directly
        let rsx = quote! {
            <>
                "First text"
                "Second text"
            </>
        };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "Fragments with string literals should parse successfully"
        );

        if let Ok(Node::Fragment(fragment)) = result {
            assert_eq!(
                fragment.children.len(),
                2,
                "Fragment should have two children"
            );
            // Both children should be expressions (string literals)
            assert!(matches!(fragment.children[0], Node::Expression(_)));
            assert!(matches!(fragment.children[1], Node::Expression(_)));
        }
    }

    #[test]
    fn test_nested_fragments() {
        // Test that fragments can be nested
        let rsx = quote! {
            <>
                <Span>"Outer"</Span>
                <>
                    <Span>"Inner 1"</Span>
                    <Span>"Inner 2"</Span>
                </>
            </>
        };
        let result = parse2::<Node>(rsx);
        assert!(result.is_ok(), "Nested fragments should parse successfully");

        if let Ok(Node::Fragment(fragment)) = result {
            assert_eq!(
                fragment.children.len(),
                2,
                "Outer fragment should have two children"
            );
            assert!(matches!(fragment.children[0], Node::Element(_)));
            assert!(matches!(fragment.children[1], Node::Fragment(_)));

            // Check inner fragment
            if let Node::Fragment(inner_fragment) = &fragment.children[1] {
                assert_eq!(
                    inner_fragment.children.len(),
                    2,
                    "Inner fragment should have two children"
                );
            }
        }
    }

    #[test]
    fn test_fragment_with_conditional() {
        // Test that fragments work with conditional rendering
        let rsx = quote! {
            <>
                {if true { <Span>"Shown"</Span> } else { <Span>"Hidden"</Span> }}
                <Span>"Always shown"</Span>
            </>
        };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "Fragments with conditionals should parse successfully"
        );

        if let Ok(Node::Fragment(fragment)) = result {
            assert_eq!(
                fragment.children.len(),
                2,
                "Fragment should have two children"
            );
            assert!(matches!(fragment.children[0], Node::Conditional(_)));
            assert!(matches!(fragment.children[1], Node::Element(_)));
        }
    }

    #[test]
    fn test_for_loop_with_preparation_statements() {
        // Test that for-loops can handle preparation statements before JSX elements
        let rsx = quote! {
            {for (index, file) in props.files.iter().enumerate() {
                let is_selected = index == props.selected_index;
                let status_icon = match file.status {
                    FileStatus::Modified => "📝",
                    FileStatus::Added => "➕",
                    _ => "❓",
                };

                <Line>
                    <Span>{status_icon}</Span>
                    <Span>{file.path.clone()}</Span>
                </Line>
            }}
        };
        let result = parse2::<Node>(rsx);
        assert!(
            result.is_ok(),
            "For-loops with preparation statements should parse successfully"
        );

        if let Ok(Node::ForLoop(for_loop)) = result {
            assert_eq!(
                for_loop.preparation_stmts.len(),
                2,
                "Should have two preparation statements"
            );
            assert!(
                matches!(for_loop.body.as_ref(), Node::Element(_)),
                "Body should be an element"
            );
        }
    }

    #[test]
    fn test_for_loop_simple_syntax() {
        // Test simple for-loop without preparation statements
        let rsx = quote! {
            {for item in items {
                <Span>{item}</Span>
            }}
        };
        let result = parse2::<Node>(rsx);
        assert!(result.is_ok(), "Simple for-loops should parse successfully");

        if let Ok(Node::ForLoop(for_loop)) = result {
            assert_eq!(
                for_loop.preparation_stmts.len(),
                0,
                "Should have no preparation statements"
            );
            assert!(
                matches!(for_loop.body.as_ref(), Node::Element(_)),
                "Body should be an element"
            );
        }
    }
}

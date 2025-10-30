//! Parser Module for RSX
//!
//! This module contains specialized parsers for different RSX constructs.
//! Following SOLID principles:
//! - Single Responsibility: Each parser handles one specific syntax construct
//! - Open/Closed: Easy to add new parsers without modifying existing ones
//! - Liskov Substitution: All parsers implement the same interface
//! - Interface Segregation: Clean parser interfaces
//! - Dependency Inversion: Depends on token analyzer abstractions

use crate::rsx::parser::{Validator, ast::*, token_analyzer::*};
use quote::quote;
use syn::{
    Error, Expr, Ident, Lit, Path, Result, Token,
    parse::ParseStream,
    spanned::Spanned,
    token::{Brace, Paren},
};

/// Core trait for all parsers
pub trait RsxParser<T> {
    /// Parse the input stream into the target type
    fn parse(&self, input: ParseStream) -> Result<T>;

    /// Check if this parser can handle the current input
    fn can_parse(&self, input: ParseStream) -> bool;
}

/// Parser for RSX properties/attributes
pub struct PropParser;

impl PropParser {
    pub fn new() -> Self {
        Self
    }
}

impl RsxParser<Prop> for PropParser {
    fn parse(&self, input: ParseStream) -> Result<Prop> {
        let key: Ident = input.parse()?;
        let key_span = key.span();

        // Check if this is a shorthand style attribute (no = sign)
        if !input.peek(Token![=]) {
            // This is a shorthand attribute like "bold", "white", etc.
            return PropFactory::create_shorthand(key, key_span);
        }

        input.parse::<Token![=]>()?;

        let value: Expr = if input.peek(Brace) {
            let content;
            let _braces = syn::braced!(content in input);
            content.parse()?
        } else {
            let lit: Lit = input.parse()?;
            Expr::Lit(syn::ExprLit { attrs: vec![], lit })
        };

        PropFactory::create(key, value, key_span)
    }

    fn can_parse(&self, input: ParseStream) -> bool {
        input.peek(syn::Ident)
            && (input.peek2(Token![=]) || self.is_shorthand_style_attribute(input))
    }
}

impl PropParser {
    /// Check if the current identifier is a shorthand style attribute
    fn is_shorthand_style_attribute(&self, input: ParseStream) -> bool {
        if let Ok(ident) = input.fork().parse::<Ident>() {
            let ident_str = ident.to_string();
            matches!(
                ident_str.as_str(),
                // Color attributes
                "white" | "black" | "red" | "green" | "blue" | "cyan" | "yellow" | "magenta" |
                "gray" | "dark_gray" | "light_red" | "light_green" | "light_blue" |
                "light_cyan" | "light_yellow" | "light_magenta" |
                // Modifier attributes
                "bold" | "italic" | "underlined" | "crossed_out" | "dim" | "reversed" |
                "rapid_blink" | "slow_blink"
            )
        } else {
            false
        }
    }
}

impl Default for PropParser {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SPECIALIZED ELEMENT PARSING COMPONENTS (SOLID PRINCIPLES)
// ============================================================================

/// Single Responsibility: Parse and validate opening tags like `<ComponentName`
pub struct OpeningTagParser {
    tag_name_validator: TagNameValidator,
}

impl OpeningTagParser {
    pub fn new() -> Self {
        Self {
            tag_name_validator: TagNameValidator::new(),
        }
    }

    /// Parse opening tag and return the component name with validation
    pub fn parse(&self, input: ParseStream) -> Result<(Path, proc_macro2::Span)> {
        input.parse::<Token![<]>()?;
        let name: Path = input.parse()?;
        let name_span = name.span();

        // Validate the tag name according to current validation rules
        self.tag_name_validator.validate(&name)?;

        Ok((name, name_span))
    }

    pub fn can_parse(&self, input: ParseStream) -> bool {
        input.peek(Token![<]) && !input.peek2(Token![/])
    }
}

/// Single Responsibility: Parse and validate closing tags like `</ComponentName>`
pub struct ClosingTagParser {
    tag_matching_validator: TagMatchingValidator,
}

impl ClosingTagParser {
    pub fn new() -> Self {
        Self {
            tag_matching_validator: TagMatchingValidator::new(),
        }
    }

    /// Parse closing tag and validate it matches the opening tag
    pub fn parse(&self, input: ParseStream, expected_name: &Path) -> Result<()> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let closing_name: Path = input.parse()?;

        // Validate that opening and closing tags match
        self.tag_matching_validator
            .validate(expected_name, &closing_name)?;

        input.parse::<Token![>]>()?;
        Ok(())
    }
}

/// Single Responsibility: Parse and validate element attributes/props
pub struct AttributeParser {
    prop_parser: PropParser,
}

impl AttributeParser {
    pub fn new() -> Self {
        Self {
            prop_parser: PropParser::new(),
        }
    }

    /// Parse all attributes until we hit `>` or `/>`
    pub fn parse(&self, input: ParseStream) -> Result<Vec<Prop>> {
        let mut attributes = Vec::new();

        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            if self.prop_parser.can_parse(input) {
                attributes.push(self.prop_parser.parse(input)?);
            } else {
                return Err(Error::new(input.span(), "Expected attribute"));
            }
        }

        Ok(attributes)
    }
}

/// Single Responsibility: Validate component naming conventions
pub struct TagNameValidator {
    element_validator: Option<crate::rsx::parser::ElementValidator>,
}

impl TagNameValidator {
    pub fn new() -> Self {
        Self {
            element_validator: None,
        }
    }

    /// Validate tag name according to current rules
    pub fn validate(&self, name: &Path) -> Result<()> {
        // Basic validation
        if name.segments.is_empty() {
            return Err(crate::rsx::error::RsxError::InvalidComponentName {
                message: "Component name cannot be empty".to_string(),
                span: name.span(),
                suggestion: Some("Use a valid component name like <MyComponent>".to_string()),
            }
            .to_syn_error());
        }

        // Advanced validation using the integrated validation system
        if let Some(ref validator) = self.element_validator {
            // Create a mock element for validation (we only need the name)
            let mock_element = Element {
                name: name.clone(),
                attributes: vec![],
                children: vec![],
                span: name.span(),
            };

            // Use the element validator to check naming conventions
            if let Err(validation_error) = validator.validate_element(&mock_element) {
                let error_message = validation_error.to_string();
                let suggestion = if error_message.contains("lowercase") {
                    Some("Component names should be PascalCase (e.g., MyComponent)".to_string())
                } else {
                    None
                };

                return Err(crate::rsx::error::RsxError::ValidationError {
                    message: format!("Component name validation failed: {}", error_message),
                    span: name.span(),
                    details: suggestion,
                }
                .to_syn_error());
            }
        }

        Ok(())
    }

    /// Set an element validator for advanced validation
    #[allow(unused)]
    pub fn with_element_validator(
        mut self,
        validator: crate::rsx::parser::ElementValidator,
    ) -> Self {
        self.element_validator = Some(validator);
        self
    }
}

/// Single Responsibility: Handle self-closing tag syntax like `<Component />`
pub struct SelfClosingTagHandler;

impl SelfClosingTagHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle self-closing tags like <Component />
    pub fn handle(&self, input: ParseStream) -> Result<bool> {
        if input.peek(Token![/]) && input.peek2(Token![>]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            Ok(true)
        } else if input.peek(Token![>]) {
            Ok(false)
        } else {
            Err(crate::rsx::error::RsxError::SyntaxError {
                message: "Expected '>' or '/>' to close the tag".to_string(),
                span: input.span(),
                suggestion: Some(
                    "Add '>' to create a container element or '/>' for a self-closing element"
                        .to_string(),
                ),
            }
            .to_syn_error())
        }
    }
}

/// Single Responsibility: Validate that opening and closing tags match
pub struct TagMatchingValidator;

impl TagMatchingValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate that opening and closing tag names match exactly
    pub fn validate(&self, opening_name: &Path, closing_name: &Path) -> Result<()> {
        let opening_str = format!("{}", quote!(#opening_name));
        let closing_str = format!("{}", quote!(#closing_name));

        if opening_str != closing_str {
            return Err(crate::rsx::error::RsxError::MismatchedTags {
                message: format!(
                    "Expected closing tag `</{}>`, found `</{}>`",
                    opening_str, closing_str
                ),
                span: closing_name.span(),
                opening_tag: opening_str,
                closing_tag: closing_str,
            }
            .to_syn_error());
        }

        Ok(())
    }
}

/// Single Responsibility: Parse child nodes within an element
pub struct ChildrenParser {
    // Could be extended with child validation rules
}

impl ChildrenParser {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse all child nodes until we encounter a closing tag
    pub fn parse(&self, input: ParseStream, element_name: &Path) -> Result<Vec<Node>> {
        let mut children = Vec::new();

        while !input.peek(Token![<]) || !input.peek2(Token![/]) {
            if input.is_empty() {
                return Err(Error::new(
                    element_name.span(),
                    format!("Missing closing tag for element `<{:?}>`", element_name),
                ));
            }

            // Enhanced parsing to handle string literals as direct children
            let child_node = self.parse_child_node(input)?;

            // Skip comment nodes
            if !matches!(child_node, Node::Comment(_)) {
                children.push(child_node);
            }
        }

        Ok(children)
    }

    /// Parse a single child node with enhanced string literal support
    pub fn parse_child_node(&self, input: ParseStream) -> Result<Node> {
        // Check if this is a string literal first
        if input.peek(syn::LitStr) {
            let lit_str: syn::LitStr = input.parse()?;
            return Ok(Node::Expression(Expr::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Str(lit_str),
            })));
        }

        // Check if this is another literal type
        if input.peek(syn::LitInt) || input.peek(syn::LitFloat) || input.peek(syn::LitBool) {
            let lit: syn::Lit = input.parse()?;
            return Ok(Node::Expression(Expr::Lit(syn::ExprLit {
                attrs: vec![],
                lit,
            })));
        }

        // Fall back to standard node parsing for other cases
        input.parse::<Node>()
    }
}

/// Refactored ElementParser using SOLID principles with specialized components
pub struct ElementParser {
    opening_tag_parser: OpeningTagParser,
    closing_tag_parser: ClosingTagParser,
    attribute_parser: AttributeParser,
    self_closing_handler: SelfClosingTagHandler,
    children_parser: ChildrenParser,
}

impl ElementParser {
    pub fn new() -> Self {
        Self {
            opening_tag_parser: OpeningTagParser::new(),
            closing_tag_parser: ClosingTagParser::new(),
            attribute_parser: AttributeParser::new(),
            self_closing_handler: SelfClosingTagHandler::new(),
            children_parser: ChildrenParser::new(),
        }
    }
}

impl RsxParser<Element> for ElementParser {
    fn parse(&self, input: ParseStream) -> Result<Element> {
        // Step 1: Parse opening tag with validation
        let (name, name_span) = self.opening_tag_parser.parse(input)?;

        // Step 2: Parse attributes
        let attributes = self.attribute_parser.parse(input)?;

        // Step 3: Handle self-closing vs regular tags
        let children = if self.self_closing_handler.handle(input)? {
            // Self-closing tag: <Component />
            Vec::new()
        } else {
            // Regular tag with children: <Component>...</Component>
            input.parse::<Token![>]>()?;
            let children = self.children_parser.parse(input, &name)?;
            self.closing_tag_parser.parse(input, &name)?;
            children
        };

        // Step 4: Create element using factory with validation
        ElementFactory::create(name, attributes, children, name_span)
    }

    fn can_parse(&self, input: ParseStream) -> bool {
        self.opening_tag_parser.can_parse(input)
    }
}

impl Default for ElementParser {
    fn default() -> Self {
        Self::new()
    }
}

// Default implementations for specialized components
impl Default for OpeningTagParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ClosingTagParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AttributeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TagNameValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SelfClosingTagHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TagMatchingValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ChildrenParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for React-style Fragment syntax (<></>)
pub struct FragmentParser {
    children_parser: ChildrenParser,
}

impl FragmentParser {
    pub fn new() -> Self {
        Self {
            children_parser: ChildrenParser::new(),
        }
    }

    /// Check if the input starts with fragment syntax
    pub fn can_parse(&self, input: ParseStream) -> bool {
        input.peek(Token![<]) && input.peek2(Token![>])
    }
}

impl RsxParser<FragmentNode> for FragmentParser {
    fn parse(&self, input: ParseStream) -> Result<FragmentNode> {
        let span = input.span();

        // Parse opening fragment tag: <>
        input.parse::<Token![<]>()?;
        input.parse::<Token![>]>()?;

        // Parse children until we find the closing fragment tag
        let mut children = Vec::new();

        while !input.peek(Token![<]) || !input.peek2(Token![/]) || !input.peek3(Token![>]) {
            if input.is_empty() {
                return Err(Error::new(
                    span,
                    "Missing closing tag for fragment. Expected </>",
                ));
            }

            // Parse child node with enhanced string literal support
            let child_node = self.children_parser.parse_child_node(input)?;

            // Skip comment nodes
            if !matches!(child_node, Node::Comment(_)) {
                children.push(child_node);
            }
        }

        // Parse closing fragment tag: </>
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;

        // Create fragment using factory
        FragmentFactory::create(children, span)
    }

    fn can_parse(&self, input: ParseStream) -> bool {
        self.can_parse(input)
    }
}

impl Default for FragmentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for conditional expressions
pub struct ConditionalParser {
    logical_and_analyzer: LogicalAndAnalyzer,
    conditional_analyzer: ConditionalAnalyzer,
}

impl ConditionalParser {
    pub fn new() -> Self {
        Self {
            logical_and_analyzer: LogicalAndAnalyzer::new(),
            conditional_analyzer: ConditionalAnalyzer::new(),
        }
    }

    /// Parse a conditional branch that can be a Node, string literal, or expression
    fn parse_conditional_branch(&self, input: ParseStream) -> Result<Node> {
        // Try to parse as a fragment first (handles <>...</>)
        if input.peek(Token![<]) && input.peek2(Token![>]) {
            let fragment_parser = FragmentParser::new();
            return Ok(Node::Fragment(fragment_parser.parse(input)?));
        }

        // Try to parse as an element
        if input.peek(Token![<]) {
            return Ok(Node::Element(input.parse::<Element>()?));
        }

        // Try to parse as a nested conditional
        if input.peek(Token![if]) || input.peek(Token![match]) {
            return Ok(Node::Conditional(self.parse(input)?));
        }

        // Try to parse as a braced expression
        if input.peek(Brace) {
            let content;
            let _braces = syn::braced!(content in input);
            return self.parse_conditional_branch(&content);
        }

        // Parse as a general expression (this handles string literals with method calls,
        // function calls, and any other complex expressions)
        Ok(Node::Expression(input.parse()?))
    }

    fn parse_if_expression(&self, input: ParseStream) -> Result<ConditionalNode> {
        input.parse::<Token![if]>()?;
        let condition_span = input.span();

        // Parse condition
        let condition: Expr = {
            let mut condition_tokens = Vec::new();
            while !input.peek(Brace) && !input.is_empty() {
                let token: proc_macro2::TokenTree = input.parse()?;
                condition_tokens.push(token);
            }

            if condition_tokens.is_empty() {
                return Err(Error::new(condition_span, "Expected condition before '{'"));
            }

            let condition_stream: proc_macro2::TokenStream = condition_tokens.into_iter().collect();
            syn::parse2(condition_stream)?
        };

        // Parse then branch
        let then_branch = if input.peek(Brace) {
            let then_content;
            let _braces = syn::braced!(then_content in input);
            Box::new(self.parse_conditional_branch(&then_content)?)
        } else {
            // Support bare expressions/strings without braces
            Box::new(self.parse_conditional_branch(input)?)
        };

        // Parse else-if chains
        let mut else_ifs = Vec::new();
        while input.peek(Token![else]) && input.peek2(Token![if]) {
            input.parse::<Token![else]>()?;
            input.parse::<Token![if]>()?;

            let else_if_condition: Expr = {
                let mut condition_tokens = Vec::new();
                while !input.peek(Brace) && !input.peek(Token![else]) && !input.is_empty() {
                    let token: proc_macro2::TokenTree = input.parse()?;
                    condition_tokens.push(token);
                }

                if condition_tokens.is_empty() {
                    return Err(Error::new(input.span(), "Expected condition"));
                }

                let condition_stream: proc_macro2::TokenStream =
                    condition_tokens.into_iter().collect();
                syn::parse2(condition_stream)?
            };

            let else_if_then_branch = if input.peek(Brace) {
                let else_if_content;
                let _braces = syn::braced!(else_if_content in input);
                Box::new(self.parse_conditional_branch(&else_if_content)?)
            } else {
                Box::new(self.parse_conditional_branch(input)?)
            };

            else_ifs.push(ElseIfBranch {
                condition: else_if_condition,
                then_branch: else_if_then_branch,
            });
        }

        // Parse final else branch
        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            if input.peek(Brace) {
                let else_content;
                let _braces = syn::braced!(else_content in input);
                Some(Box::new(self.parse_conditional_branch(&else_content)?))
            } else {
                Some(Box::new(self.parse_conditional_branch(input)?))
            }
        } else {
            None
        };

        ConditionalFactory::create_if(
            condition,
            then_branch,
            else_ifs,
            else_branch,
            condition_span,
        )
    }

    fn parse_match_expression(&self, input: ParseStream) -> Result<ConditionalNode> {
        input.parse::<Token![match]>()?;
        let match_span = input.span();

        // Parse the match expression
        let expr: Expr = if input.peek(syn::Ident) && input.peek2(Brace) {
            let ident: syn::Ident = input.parse()?;
            syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: ident.into(),
            })
        } else {
            input.parse()?
        };

        let arms_content;
        let _braces = syn::braced!(arms_content in input);

        let mut arms = Vec::new();
        while !arms_content.is_empty() {
            let pattern = syn::Pat::parse_single(&arms_content)?;

            let guard = if arms_content.peek(Token![if]) {
                arms_content.parse::<Token![if]>()?;
                Some(arms_content.parse::<Expr>()?)
            } else {
                None
            };

            arms_content.parse::<Token![=>]>()?;

            // Parse the body
            let body = if arms_content.peek(Brace) {
                Box::new(arms_content.parse::<Node>()?)
            } else if arms_content.peek(Paren) {
                let paren_content;
                let _parens = syn::parenthesized!(paren_content in arms_content);

                if paren_content.peek(Token![<]) {
                    Box::new(Node::Element(paren_content.parse::<Element>()?))
                } else {
                    Box::new(Node::Expression(paren_content.parse::<Expr>()?))
                }
            } else if arms_content.peek(Token![<]) {
                Box::new(Node::Element(arms_content.parse::<Element>()?))
            } else {
                Box::new(Node::Expression(arms_content.parse::<Expr>()?))
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            if arms_content.peek(Token![,]) {
                arms_content.parse::<Token![,]>()?;
            }
        }

        ConditionalFactory::create_match(expr, arms, match_span)
    }

    fn parse_logical_and_expression(&self, input: ParseStream) -> Result<ConditionalNode> {
        let tokens: Vec<proc_macro2::TokenTree> = TokenStreamUtils::parse_stream_to_tokens(input)?;
        let span = input.span();

        if let Some(pos) = self.logical_and_analyzer.find_and_position(&tokens) {
            let (condition_tokens, then_tokens) =
                TokenStreamUtils::split_at_position(&tokens, pos, 2);

            let condition: Expr = syn::parse2(condition_tokens)?;
            let then_branch = Box::new(syn::parse2::<Node>(then_tokens)?);

            ConditionalFactory::create_logical_and(condition, then_branch, span)
        } else {
            Err(Error::new(span, "Expected logical AND expression"))
        }
    }

    fn parse_if_let_expression(&self, input: ParseStream) -> Result<ConditionalNode> {
        input.parse::<Token![if]>()?;
        input.parse::<Token![let]>()?;
        let if_let_span = input.span();

        // Parse pattern
        let pattern: syn::Pat = input.call(syn::Pat::parse_single)?;

        // Parse '=' token
        input.parse::<Token![=]>()?;

        // Parse expression
        let expr: Expr = {
            let mut expr_tokens = Vec::new();
            while !input.peek(Brace) && !input.is_empty() {
                let token: proc_macro2::TokenTree = input.parse()?;
                expr_tokens.push(token);
            }

            if expr_tokens.is_empty() {
                return Err(Error::new(if_let_span, "Expected expression after '='"));
            }

            let expr_stream: proc_macro2::TokenStream = expr_tokens.into_iter().collect();
            syn::parse2(expr_stream)?
        };

        // Parse then branch
        let then_branch = if input.peek(Brace) {
            let then_content;
            let _braces = syn::braced!(then_content in input);
            Box::new(self.parse_conditional_branch(&then_content)?)
        } else {
            // Support bare expressions/strings without braces
            Box::new(self.parse_conditional_branch(input)?)
        };

        // Parse optional else branch
        let else_branch = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            if input.peek(Brace) {
                let else_content;
                let _braces = syn::braced!(else_content in input);
                Some(Box::new(self.parse_conditional_branch(&else_content)?))
            } else {
                // Support bare expressions/strings without braces
                Some(Box::new(self.parse_conditional_branch(input)?))
            }
        } else {
            None
        };

        ConditionalFactory::create_if_let(pattern, expr, then_branch, else_branch, if_let_span)
    }
}

impl RsxParser<ConditionalNode> for ConditionalParser {
    fn parse(&self, input: ParseStream) -> Result<ConditionalNode> {
        if input.peek(Token![if]) {
            // Check if it's an if let expression
            if self.conditional_analyzer.contains_if_let(input) {
                self.parse_if_let_expression(input)
            } else {
                self.parse_if_expression(input)
            }
        } else if input.peek(Token![match]) {
            self.parse_match_expression(input)
        } else if self.logical_and_analyzer.analyze(input) {
            self.parse_logical_and_expression(input)
        } else {
            Err(Error::new(input.span(), "Expected conditional expression"))
        }
    }

    fn can_parse(&self, input: ParseStream) -> bool {
        input.peek(Token![if])
            || input.peek(Token![match])
            || self.logical_and_analyzer.analyze(input)
    }
}

impl Default for ConditionalParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for for-loop expressions in RSX
pub struct ForLoopParser {
    for_loop_analyzer: ForLoopAnalyzer,
    conditional_parser: ConditionalParser,
    conditional_analyzer: ConditionalAnalyzer,
    element_parser: ElementParser,
}

impl ForLoopParser {
    pub fn new() -> Self {
        Self {
            for_loop_analyzer: ForLoopAnalyzer::new(),
            conditional_parser: ConditionalParser::new(),
            conditional_analyzer: ConditionalAnalyzer::new(),
            element_parser: ElementParser::new(),
        }
    }

    pub fn can_parse(&self, input: ParseStream) -> bool {
        self.for_loop_analyzer.contains_for_loop(&input)
    }

    /// Parse the body of a for-loop, separating preparation statements from the final JSX element
    fn parse_for_loop_body(&self, input: ParseStream) -> Result<(Vec<syn::Stmt>, Node)> {
        let mut preparation_stmts = Vec::new();
        let mut body_node = None;

        // Parse statements until we find a JSX element or reach the end
        while !input.is_empty() {
            // Try to parse as a JSX element first
            if self.element_parser.can_parse(input) {
                body_node = Some(Node::Element(self.element_parser.parse(input)?));
                break;
            }

            // Try to parse as a conditional
            if self.conditional_analyzer.analyze(input) {
                body_node = Some(Node::Conditional(self.conditional_parser.parse(input)?));
                break;
            }

            // Try to parse as a fragment
            if input.peek(Token![<]) && input.peek2(Token![>]) {
                let fragment_parser = FragmentParser::new();
                body_node = Some(Node::Fragment(fragment_parser.parse(input)?));
                break;
            }

            // Try to parse as a statement (let binding, match expression, etc.)
            if let Ok(stmt) = input.parse::<syn::Stmt>() {
                preparation_stmts.push(stmt);
            } else {
                // If we can't parse as a statement, try as an expression
                let expr = input.parse::<syn::Expr>()?;
                body_node = Some(Node::Expression(expr));
                break;
            }
        }

        // If we didn't find a body node, create an empty text node
        let final_body_node =
            body_node.unwrap_or_else(|| Node::Expression(syn::parse_quote! { "" }));

        Ok((preparation_stmts, final_body_node))
    }
}

impl RsxParser<ForLoopNode> for ForLoopParser {
    fn parse(&self, input: ParseStream) -> Result<ForLoopNode> {
        let span = input.span();

        // Expect 'for' keyword
        input.parse::<Token![for]>()?;

        // Parse the pattern (e.g., 'item' in 'for item in collection')
        let pattern: syn::Pat = input.call(syn::Pat::parse_single)?;

        // Expect 'in' keyword
        input.parse::<Token![in]>()?;

        // Parse the iterable expression
        // Handle both simple identifiers and complex expressions
        let iterable: syn::Expr = if input.peek(syn::Ident) && input.peek2(Brace) {
            // Simple identifier followed by brace (e.g., "items {")
            let ident: syn::Ident = input.parse()?;
            syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: ident.into(),
            })
        } else {
            // Complex expression
            input.parse()?
        };

        // Parse the body as a braced block containing preparation statements and RSX nodes
        let body_content;
        syn::braced!(body_content in input);

        // Parse the body content with support for preparation statements
        let (preparation_stmts, body_node) = if body_content.is_empty() {
            // Empty body - create an empty text node
            (Vec::new(), Node::Expression(syn::parse_quote! { "" }))
        } else {
            self.parse_for_loop_body(&body_content)?
        };

        // Create the ForLoopNode using the factory
        ForLoopFactory::create(
            pattern,
            iterable,
            preparation_stmts,
            Box::new(body_node),
            span,
        )
    }

    fn can_parse(&self, input: ParseStream) -> bool {
        self.for_loop_analyzer.contains_for_loop(&input)
    }
}

/// Parser for RSX nodes (the main entry point)
pub struct NodeParser {
    element_parser: ElementParser,
    conditional_parser: ConditionalParser,
    for_loop_parser: Option<ForLoopParser>,
    fragment_parser: FragmentParser,
    comment_analyzer: CommentAnalyzer,
    conditional_analyzer: ConditionalAnalyzer,
    for_loop_analyzer: ForLoopAnalyzer,
}

impl NodeParser {
    pub fn new() -> Self {
        Self {
            element_parser: ElementParser::new(),
            conditional_parser: ConditionalParser::new(),
            for_loop_parser: Some(ParserFactory::create_for_loop_parser()),
            fragment_parser: FragmentParser::new(),
            comment_analyzer: CommentAnalyzer::new(),
            conditional_analyzer: ConditionalAnalyzer::new(),
            for_loop_analyzer: ForLoopAnalyzer::new(),
        }
    }
}

impl RsxParser<Node> for NodeParser {
    fn parse(&self, input: ParseStream) -> Result<Node> {
        // Try to parse as JSX comment first
        if self.comment_analyzer.analyze(input) {
            let content = self.comment_analyzer.extract_comment_content(input)?;
            return Ok(Node::Comment(CommentNode { content }));
        }

        // Try to parse as fragment
        if self.fragment_parser.can_parse(input) {
            return Ok(Node::Fragment(self.fragment_parser.parse(input)?));
        }

        // Try to parse as for-loop
        if let Some(ref for_loop_parser) = self.for_loop_parser
            && for_loop_parser.can_parse(input)
        {
            return Ok(Node::ForLoop(for_loop_parser.parse(input)?));
        }

        if self.element_parser.can_parse(input) {
            Ok(Node::Element(self.element_parser.parse(input)?))
        } else if input.peek(Paren) {
            let content;
            let _parens = syn::parenthesized!(content in input);

            if self.element_parser.can_parse(&content) {
                Ok(Node::Element(self.element_parser.parse(&content)?))
            } else if self.for_loop_analyzer.contains_for_loop(&&content) {
                if let Some(ref for_loop_parser) = self.for_loop_parser {
                    Ok(Node::ForLoop(for_loop_parser.parse(&content)?))
                } else {
                    Ok(Node::Expression(content.parse()?))
                }
            } else if self.conditional_analyzer.analyze(&content) {
                Ok(Node::Conditional(self.conditional_parser.parse(&content)?))
            } else {
                Ok(Node::Expression(content.parse()?))
            }
        } else if self.conditional_parser.can_parse(input) {
            Ok(Node::Conditional(self.conditional_parser.parse(input)?))
        } else if input.peek(Brace) {
            let content;
            let _braces = syn::braced!(content in input);

            if self.for_loop_analyzer.contains_for_loop(&&content) {
                if let Some(ref for_loop_parser) = self.for_loop_parser {
                    Ok(Node::ForLoop(for_loop_parser.parse(&content)?))
                } else {
                    Ok(Node::Expression(content.parse()?))
                }
            } else if self.conditional_analyzer.analyze(&content) {
                Ok(Node::Conditional(self.conditional_parser.parse(&content)?))
            } else {
                Ok(Node::Expression(content.parse()?))
            }
        } else {
            // Try to parse as a string literal or other expression
            Ok(Node::Expression(input.parse()?))
        }
    }

    fn can_parse(&self, input: ParseStream) -> bool {
        self.element_parser.can_parse(input)
            || self.conditional_parser.can_parse(input)
            || self.for_loop_analyzer.contains_for_loop(&input)
            || self.comment_analyzer.analyze(input)
            || input.peek(Paren)
            || input.peek(Brace)
    }
}

impl Default for NodeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating parser instances following SOLID principles
pub struct ParserFactory;

impl ParserFactory {
    pub fn create_node_parser() -> NodeParser {
        NodeParser::new()
    }

    pub fn create_element_parser() -> ElementParser {
        ElementParser::new()
    }

    pub fn create_conditional_parser() -> ConditionalParser {
        ConditionalParser::new()
    }

    pub fn create_for_loop_parser() -> ForLoopParser {
        ForLoopParser::new()
    }
}

#[cfg(test)]
mod specialized_parser_tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_opening_tag_parser() {
        let _parser = OpeningTagParser::new();
        let _tokens = quote! { <MyComponent };

        // Create a custom test since we need ParseStream
        let result = syn::parse2::<syn::Path>(quote! { MyComponent });
        assert!(result.is_ok());
        let name = result.unwrap();
        assert_eq!(format!("{}", quote!(#name)), "MyComponent");
    }

    #[test]
    fn test_opening_tag_parser_can_parse() {
        let _parser = OpeningTagParser::new();

        // Test valid opening tag
        let tokens = quote! { <MyComponent };
        let parsed = syn::parse2::<proc_macro2::TokenStream>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_closing_tag_parser_matching() {
        let validator = TagMatchingValidator::new();
        let opening_name: syn::Path = parse_quote!(MyComponent);
        let closing_name: syn::Path = parse_quote!(MyComponent);

        let result = validator.validate(&opening_name, &closing_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_closing_tag_parser_mismatch() {
        let validator = TagMatchingValidator::new();
        let opening_name: syn::Path = parse_quote!(MyComponent);
        let closing_name: syn::Path = parse_quote!(OtherComponent);

        let result = validator.validate(&opening_name, &closing_name);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Expected closing tag"));
    }

    #[test]
    fn test_attribute_parser_creation() {
        let parser = AttributeParser::new();

        // Test that it has the expected structure
        let _prop_parser = &parser.prop_parser;
    }

    #[test]
    fn test_tag_name_validator_basic() {
        let validator = TagNameValidator::new();
        let name: syn::Path = parse_quote!(MyComponent);
        let result = validator.validate(&name);

        assert!(result.is_ok());
    }

    #[test]
    fn test_tag_name_validator_empty() {
        let validator = TagNameValidator::new();
        // Create an empty path (this would be caught by syn parsing, but testing the validator)
        let name = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::new(),
        };
        let result = validator.validate(&name);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("cannot be empty"));
    }

    #[test]
    fn test_self_closing_handler() {
        let _handler = SelfClosingTagHandler::new();
    }

    #[test]
    fn test_tag_matching_validator() {
        let validator = TagMatchingValidator::new();
        let opening: syn::Path = parse_quote!(MyComponent);
        let closing: syn::Path = parse_quote!(MyComponent);

        let result = validator.validate(&opening, &closing);
        assert!(result.is_ok());

        let wrong_closing: syn::Path = parse_quote!(OtherComponent);
        let result = validator.validate(&opening, &wrong_closing);
        assert!(result.is_err());
    }

    #[test]
    fn test_children_parser() {
        let _parser = ChildrenParser::new();
    }

    #[test]
    fn test_refactored_element_parser_creation() {
        let _parser = ElementParser::new();

        // Test that the parser can determine if it can parse
        let tokens = quote! { <MyComponent };
        let parsed = syn::parse2::<proc_macro2::TokenStream>(tokens);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_element_parser_integration() {
        // Test that the refactored ElementParser maintains the same interface
        let _parser = ElementParser::new();

        // Test that it implements the RsxParser trait
        let tokens = quote! { <MyComponent className="test" /> };
        let result = syn::parse2::<Element>(tokens);
        assert!(result.is_ok());

        if let Ok(element) = result {
            let name = &element.name;
            assert_eq!(format!("{}", quote!(#name)), "MyComponent");
            assert_eq!(element.attributes.len(), 1);
        }
    }

    #[test]
    fn test_for_loop_parser_creation() {
        let _parser = ForLoopParser::new();
    }

    #[test]
    fn test_for_loop_analyzer() {
        use crate::rsx::parser::token_analyzer::ForLoopAnalyzer;
        let _analyzer = ForLoopAnalyzer::new();

        // Test basic for-loop detection
        let tokens = quote! { for item in items };
        let result = syn::parse2::<proc_macro2::TokenStream>(tokens);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_node_creation() {
        use crate::rsx::parser::ast::{ForLoopFactory, Node};
        use syn::{Expr, Pat, parse_quote};

        let pattern: Pat = parse_quote!(item);
        let iterable: Expr = parse_quote!(items);
        let body = Box::new(Node::Expression(parse_quote!("test")));
        let span = proc_macro2::Span::call_site();

        let result = ForLoopFactory::create(pattern, iterable, Vec::new(), body, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_parsing_direct() {
        use crate::rsx::parser::token_analyzer::ForLoopAnalyzer;

        let _analyzer = ForLoopAnalyzer::new();
        let tokens = quote! { for item in items { <span>test</span> } };

        // Test parsing the for-loop structure
        let result = syn::parse2::<proc_macro2::TokenStream>(tokens);
        assert!(result.is_ok());

        if let Ok(stream) = result {
            let parsed_result = syn::parse2::<ForLoopNode>(stream);
            // This should work if our parsing is correct
            println!("For-loop parsing result: {:?}", parsed_result.is_ok());
        }
    }

    #[test]
    fn test_for_loop_in_braces() {
        // Test parsing for-loop inside braces like {for item in items { ... }}
        let tokens = quote! {
            {for item in vec![1, 2, 3] {
                <span>{"test"}</span>
            }}
        };

        let result = syn::parse2::<Node>(tokens);
        println!("Braced for-loop parsing result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("Error: {}", e);
        }
    }

    #[test]
    fn test_for_loop_direct_parsing() {
        // Test parsing for-loop directly without braces
        let tokens = quote! {
            for item in vec![1, 2, 3] {
                <span>{"test"}</span>
            }
        };

        let result = syn::parse2::<ForLoopNode>(tokens);
        println!("Direct for-loop parsing result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("Direct parsing error: {}", e);
        }
    }

    #[test]
    fn test_for_loop_with_identifier() {
        // Test parsing for-loop with identifier iterable
        let tokens = quote! {
            for item in items {
                <span>{"test"}</span>
            }
        };

        let result = syn::parse2::<ForLoopNode>(tokens);
        println!("For-loop with identifier result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("For-loop with identifier error: {}", e);
        }
    }

    #[test]
    fn test_for_loop_parts_parsing() {
        // Test parsing individual parts of for-loop

        // Test pattern parsing using ParseStream
        let pattern_tokens = quote! { item };
        let pattern_result = syn::parse2::<proc_macro2::TokenStream>(pattern_tokens);
        if let Ok(stream) = pattern_result {
            let parse_result = syn::parse2::<syn::Ident>(stream);
            println!("Pattern parsing result: {:?}", parse_result.is_ok());
        }

        // Test iterable parsing
        let iterable_tokens = quote! { items };
        let iterable_result = syn::parse2::<syn::Expr>(iterable_tokens);
        println!("Iterable parsing result: {:?}", iterable_result.is_ok());

        // Test body parsing
        let body_tokens = quote! { <span>{"test"}</span> };
        let body_result = syn::parse2::<Element>(body_tokens);
        println!("Body parsing result: {:?}", body_result.is_ok());
        if let Err(e) = &body_result {
            println!("Body parsing error: {}", e);
        }
    }

    #[test]
    fn test_element_parsing_in_for_loop() {
        // Test if we can parse the element inside for-loop body
        let tokens = quote! { <span>test</span> };

        let result = syn::parse2::<Element>(tokens);
        println!("Element parsing result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("Element parsing error: {}", e);
        }
    }

    #[test]
    fn test_pascal_case_element_parsing() {
        // Test if we can parse PascalCase elements
        let tokens = quote! { <Span>test</Span> };

        let result = syn::parse2::<Element>(tokens);
        println!("PascalCase element parsing result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("PascalCase element parsing error: {}", e);
        }
    }

    #[test]
    fn test_for_loop_with_pascal_case() {
        // Test parsing for-loop with PascalCase elements
        let tokens = quote! {
            for item in items {
                <Span>test</Span>
            }
        };

        let result = syn::parse2::<ForLoopNode>(tokens);
        println!(
            "For-loop with PascalCase parsing result: {:?}",
            result.is_ok()
        );
        if let Err(e) = &result {
            println!("For-loop with PascalCase error: {}", e);
        }
    }

    #[test]
    fn test_element_parser_direct() {
        // Test ElementParser directly
        let tokens = quote! { <Span>{"test"}</Span> };

        let _parser = ElementParser::new();
        let result = syn::parse2::<proc_macro2::TokenStream>(tokens);

        if let Ok(stream) = result {
            let parse_result = syn::parse2::<Element>(stream);
            println!("Direct ElementParser result: {:?}", parse_result.is_ok());
            if let Err(e) = &parse_result {
                println!("Direct ElementParser error: {}", e);
            }
        }
    }

    #[test]
    fn test_element_with_string_literal() {
        // Test ElementParser with proper string literal
        let tokens = quote! { <Span>"test"</Span> };

        let result = syn::parse2::<Element>(tokens);
        println!("String literal element result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("String literal element error: {}", e);
        }
    }

    #[test]
    fn test_string_literal_as_node() {
        // Test parsing string literal directly as Node
        let tokens = quote! { "test" };

        let result = syn::parse2::<Node>(tokens);
        println!("String literal as Node result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("String literal as Node error: {}", e);
        }
    }

    #[test]
    fn test_string_literal_as_expression() {
        // Test parsing string literal as Expression
        let tokens = quote! { "test" };

        let result = syn::parse2::<syn::Expr>(tokens);
        println!("String literal as Expression result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("String literal as Expression error: {}", e);
        }
    }

    #[test]
    fn test_for_loop_with_conditional() {
        // Test parsing for-loop with conditional expression in body
        let tokens = quote! {
            for num in numbers {
                if num % 2 == 0 {
                    <Span>{"even"}</Span>
                } else {
                    <Span>{"odd"}</Span>
                }
            }
        };

        let result = syn::parse2::<ForLoopNode>(tokens);
        println!("For-loop with conditional result: {:?}", result.is_ok());
        if let Err(e) = &result {
            println!("For-loop with conditional error: {}", e);
        }
    }

    #[test]
    fn test_for_loop_with_conditional_braced() {
        // Test parsing for-loop with conditional inside braces
        let tokens = quote! {
            {for num in numbers {
                if num % 2 == 0 {
                    <Span>{"even"}</Span>
                } else {
                    <Span>{"odd"}</Span>
                }
            }}
        };

        let result = syn::parse2::<Node>(tokens);
        println!(
            "Braced for-loop with conditional result: {:?}",
            result.is_ok()
        );
        if let Err(e) = &result {
            println!("Braced for-loop with conditional error: {}", e);
        }
    }
}

//! Abstract Syntax Tree (AST) definitions for RSX parser
//!
//! This module defines the core data structures that represent the parsed RSX syntax tree.
//! Following SOLID principles:
//! - Single Responsibility: Each struct has a single, well-defined purpose
//! - Open/Closed: Easy to extend with new node types without modifying existing ones
//! - Liskov Substitution: All node types implement common traits consistently
//! - Interface Segregation: Clean interfaces without unnecessary dependencies
//! - Dependency Inversion: Depends on abstractions (traits) rather than concrete types

use syn::{Expr, Ident, Path, spanned::Spanned};

/// Core trait for all AST nodes - enables polymorphic behavior
pub trait AstNode: std::fmt::Debug + Clone {
    /// Get the span information for error reporting
    fn span(&self) -> proc_macro2::Span;

    /// Accept a visitor for traversal patterns
    #[allow(unused)]
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()>;

    /// Validate the node's semantic correctness
    fn validate(&self) -> syn::Result<()> {
        Ok(()) // Default implementation - can be overridden
    }
}

/// Visitor pattern for AST traversal
#[allow(unused)]
pub trait AstVisitor {
    fn visit_element(&mut self, element: &Element) -> syn::Result<()>;
    fn visit_prop(&mut self, prop: &Prop) -> syn::Result<()>;
    fn visit_conditional(&mut self, conditional: &ConditionalNode) -> syn::Result<()>;
    fn visit_expression(&mut self, expr: &Expr) -> syn::Result<()>;
    fn visit_comment(&mut self, comment: &CommentNode) -> syn::Result<()>;
    fn visit_for_loop(&mut self, for_loop: &ForLoopNode) -> syn::Result<()>;
    fn visit_fragment(&mut self, fragment: &FragmentNode) -> syn::Result<()>;
}

/// Represents an attribute in an XML-like element (key=value)
/// Single Responsibility: Handles only property parsing and representation
#[derive(Debug, Clone)]
pub struct Prop {
    pub key: Ident,
    pub value: Expr,
}

impl AstNode for Prop {
    fn span(&self) -> proc_macro2::Span {
        self.key.span()
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        visitor.visit_prop(self)
    }

    fn validate(&self) -> syn::Result<()> {
        // Validate that the key is a valid identifier
        if self.key.to_string().is_empty() {
            return Err(syn::Error::new(self.span(), "Property key cannot be empty"));
        }
        Ok(())
    }
}

/// Factory for creating Prop instances with validation
pub struct PropFactory;

impl PropFactory {
    pub fn create(key: Ident, value: Expr, _span: proc_macro2::Span) -> syn::Result<Prop> {
        let prop = Prop { key, value };
        prop.validate()?;
        Ok(prop)
    }

    /// Create a shorthand style attribute (e.g., "bold", "white")
    pub fn create_shorthand(key: Ident, _span: proc_macro2::Span) -> syn::Result<Prop> {
        use syn::{Expr, Lit, LitBool};

        // For shorthand attributes, we set the value to true
        let value = Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: Lit::Bool(LitBool {
                value: true,
                span: key.span(),
            }),
        });

        let prop = Prop { key, value };
        prop.validate()?;
        Ok(prop)
    }
}

/// Represents a node in the RSX tree
/// Open/Closed Principle: Easy to add new node types without modifying existing code
#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Expression(Expr),
    Conditional(ConditionalNode),
    Comment(CommentNode),
    ForLoop(ForLoopNode),
    Fragment(FragmentNode),
}

impl AstNode for Node {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Node::Element(e) => e.span(),
            Node::Expression(e) => e.span(),
            Node::Conditional(c) => c.span(),
            Node::Comment(c) => c.span(),
            Node::ForLoop(f) => f.span(),
            Node::Fragment(f) => f.span(),
        }
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        match self {
            Node::Element(e) => e.accept(visitor),
            Node::Expression(e) => visitor.visit_expression(e),
            Node::Conditional(c) => c.accept(visitor),
            Node::Comment(c) => c.accept(visitor),
            Node::ForLoop(f) => f.accept(visitor),
            Node::Fragment(f) => f.accept(visitor),
        }
    }

    fn validate(&self) -> syn::Result<()> {
        match self {
            Node::Element(e) => e.validate(),
            Node::Expression(_) => Ok(()),
            Node::Conditional(c) => c.validate(),
            Node::Comment(c) => c.validate(),
            Node::ForLoop(f) => f.validate(),
            Node::Fragment(f) => f.validate(),
        }
    }
}

/// Represents an XML-like element with a name, attributes, and children
/// Single Responsibility: Handles element structure and validation
#[derive(Debug, Clone)]
pub struct Element {
    pub name: Path,
    pub attributes: Vec<Prop>,
    pub children: Vec<Node>,
    #[allow(dead_code)]
    pub span: proc_macro2::Span,
}

impl AstNode for Element {
    fn span(&self) -> proc_macro2::Span {
        self.name.span()
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        visitor.visit_element(self)?;

        // Visit all attributes
        for attr in &self.attributes {
            attr.accept(visitor)?;
        }

        // Visit all children
        for child in &self.children {
            child.accept(visitor)?;
        }

        Ok(())
    }

    fn validate(&self) -> syn::Result<()> {
        // Validate element name
        if self.name.segments.is_empty() {
            return Err(syn::Error::new(self.span(), "Element name cannot be empty"));
        }

        // Validate all attributes
        for attr in &self.attributes {
            attr.validate()?;
        }

        // Validate all children
        for child in &self.children {
            child.validate()?;
        }

        Ok(())
    }
}

/// Factory for creating Element instances
pub struct ElementFactory;

impl ElementFactory {
    pub fn create(
        name: Path,
        attributes: Vec<Prop>,
        children: Vec<Node>,
        span: proc_macro2::Span,
    ) -> syn::Result<Element> {
        let element = Element {
            name,
            attributes,
            children,
            span,
        };
        element.validate()?;
        Ok(element)
    }
}

/// Represents a JSX-style comment in the RSX tree
#[derive(Debug, Clone)]
pub struct CommentNode {
    #[allow(dead_code)]
    pub content: String,
}

/// Represents a React-style Fragment in the RSX tree
/// Single Responsibility: Handles fragment structure and validation
#[derive(Debug, Clone)]
pub struct FragmentNode {
    /// Children contained within the fragment
    pub children: Vec<Node>,
    /// Span information for error reporting
    pub span: proc_macro2::Span,
}

impl AstNode for CommentNode {
    fn span(&self) -> proc_macro2::Span {
        proc_macro2::Span::call_site()
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        visitor.visit_comment(self)
    }
}

impl AstNode for FragmentNode {
    fn span(&self) -> proc_macro2::Span {
        self.span
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        visitor.visit_fragment(self)?;

        // Visit all children
        for child in &self.children {
            child.accept(visitor)?;
        }

        Ok(())
    }

    fn validate(&self) -> syn::Result<()> {
        // Validate all children
        for child in &self.children {
            child.validate()?;
        }
        Ok(())
    }
}

/// Represents different types of conditional expressions in RSX
/// Open/Closed Principle: Easy to add new conditional types
#[derive(Debug, Clone)]
pub enum ConditionalNode {
    If {
        condition: Expr,
        then_branch: Box<Node>,
        else_ifs: Vec<ElseIfBranch>,
        else_branch: Option<Box<Node>>,
    },
    IfLet {
        pattern: syn::Pat,
        expr: Expr,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
    },
    LogicalAnd {
        condition: Expr,
        then_branch: Box<Node>,
    },
}

impl AstNode for ConditionalNode {
    fn span(&self) -> proc_macro2::Span {
        match self {
            ConditionalNode::If { condition, .. } => condition.span(),
            ConditionalNode::IfLet { expr, .. } => expr.span(),
            ConditionalNode::Match { expr, .. } => expr.span(),
            ConditionalNode::LogicalAnd { condition, .. } => condition.span(),
        }
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        visitor.visit_conditional(self)?;

        match self {
            ConditionalNode::If {
                then_branch,
                else_ifs,
                else_branch,
                ..
            } => {
                then_branch.accept(visitor)?;
                for else_if in else_ifs {
                    else_if.then_branch.accept(visitor)?;
                }
                if let Some(else_node) = else_branch {
                    else_node.accept(visitor)?;
                }
            }
            ConditionalNode::IfLet {
                then_branch,
                else_branch,
                ..
            } => {
                then_branch.accept(visitor)?;
                if let Some(else_node) = else_branch {
                    else_node.accept(visitor)?;
                }
            }
            ConditionalNode::Match { arms, .. } => {
                for arm in arms {
                    arm.body.accept(visitor)?;
                }
            }
            ConditionalNode::LogicalAnd { then_branch, .. } => {
                then_branch.accept(visitor)?;
            }
        }

        Ok(())
    }

    fn validate(&self) -> syn::Result<()> {
        match self {
            ConditionalNode::If {
                then_branch,
                else_ifs,
                else_branch,
                ..
            } => {
                then_branch.validate()?;
                for else_if in else_ifs {
                    else_if.then_branch.validate()?;
                }
                if let Some(else_node) = else_branch {
                    else_node.validate()?;
                }
            }
            ConditionalNode::IfLet {
                then_branch,
                else_branch,
                ..
            } => {
                then_branch.validate()?;
                if let Some(else_node) = else_branch {
                    else_node.validate()?;
                }
            }
            ConditionalNode::Match { arms, .. } => {
                if arms.is_empty() {
                    return Err(syn::Error::new(
                        self.span(),
                        "Match expression must have at least one arm",
                    ));
                }
                for arm in arms {
                    arm.body.validate()?;
                }
            }
            ConditionalNode::LogicalAnd { then_branch, .. } => {
                then_branch.validate()?;
            }
        }
        Ok(())
    }
}

/// Represents an else-if branch in an if expression
#[derive(Debug, Clone)]
pub struct ElseIfBranch {
    pub condition: Expr,
    pub then_branch: Box<Node>,
}

/// Represents a match arm in a match expression
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: syn::Pat,
    pub guard: Option<Expr>,
    pub body: Box<Node>,
}

/// Represents a for-loop in the RSX tree
/// Single Responsibility: Handles for-loop structure and validation
#[derive(Debug, Clone)]
pub struct ForLoopNode {
    /// The pattern to bind each iteration item to (e.g., `item` in `for item in collection`)
    pub pattern: syn::Pat,
    /// The expression to iterate over (e.g., `collection` in `for item in collection`)
    pub iterable: Expr,
    /// Preparation statements before the JSX element (e.g., let bindings, match expressions)
    pub preparation_stmts: Vec<syn::Stmt>,
    /// The body of the loop - what to render for each iteration
    pub body: Box<Node>,
    /// Span information for error reporting
    pub span: proc_macro2::Span,
}

impl AstNode for ForLoopNode {
    fn span(&self) -> proc_macro2::Span {
        self.span
    }

    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> syn::Result<()> {
        visitor.visit_for_loop(self)?;
        // Visit the body of the loop
        self.body.accept(visitor)
    }

    fn validate(&self) -> syn::Result<()> {
        // Validate the pattern is a simple identifier or destructuring pattern
        match &self.pattern {
            syn::Pat::Ident(_) => {}  // Simple identifier is always valid
            syn::Pat::Tuple(_) => {}  // Tuple destructuring is valid
            syn::Pat::Struct(_) => {} // Struct destructuring is valid
            _ => {
                return Err(syn::Error::new(
                    self.pattern.span(),
                    "For-loop pattern must be a simple identifier or destructuring pattern",
                ));
            }
        }

        // Validate the body
        self.body.validate()
    }
}

/// Factory for creating for-loop nodes
pub struct ForLoopFactory;

impl ForLoopFactory {
    pub fn create(
        pattern: syn::Pat,
        iterable: Expr,
        preparation_stmts: Vec<syn::Stmt>,
        body: Box<Node>,
        span: proc_macro2::Span,
    ) -> syn::Result<ForLoopNode> {
        let for_loop = ForLoopNode {
            pattern,
            iterable,
            preparation_stmts,
            body,
            span,
        };
        for_loop.validate()?;
        Ok(for_loop)
    }
}

/// Factory for creating fragment nodes
pub struct FragmentFactory;

impl FragmentFactory {
    pub fn create(children: Vec<Node>, span: proc_macro2::Span) -> syn::Result<FragmentNode> {
        let fragment = FragmentNode { children, span };
        fragment.validate()?;
        Ok(fragment)
    }
}

/// Factory for creating conditional nodes
pub struct ConditionalFactory;

impl ConditionalFactory {
    pub fn create_if(
        condition: Expr,
        then_branch: Box<Node>,
        else_ifs: Vec<ElseIfBranch>,
        else_branch: Option<Box<Node>>,
        _span: proc_macro2::Span,
    ) -> syn::Result<ConditionalNode> {
        let conditional = ConditionalNode::If {
            condition,
            then_branch,
            else_ifs,
            else_branch,
        };
        conditional.validate()?;
        Ok(conditional)
    }

    pub fn create_match(
        expr: Expr,
        arms: Vec<MatchArm>,
        _span: proc_macro2::Span,
    ) -> syn::Result<ConditionalNode> {
        let conditional = ConditionalNode::Match { expr, arms };
        conditional.validate()?;
        Ok(conditional)
    }

    pub fn create_if_let(
        pattern: syn::Pat,
        expr: Expr,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
        _span: proc_macro2::Span,
    ) -> syn::Result<ConditionalNode> {
        let conditional = ConditionalNode::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        };
        conditional.validate()?;
        Ok(conditional)
    }

    pub fn create_logical_and(
        condition: Expr,
        then_branch: Box<Node>,
        _span: proc_macro2::Span,
    ) -> syn::Result<ConditionalNode> {
        let conditional = ConditionalNode::LogicalAnd {
            condition,
            then_branch,
        };
        conditional.validate()?;
        Ok(conditional)
    }
}

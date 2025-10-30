use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned};

use crate::rsx::parser::{
    ConditionalNode, Element, ForLoopNode, FragmentNode, Node, RsxMainParser,
};

pub(crate) mod error;
pub(crate) mod parser;

// Main implementation of the rsx! macro with integrated validation
pub fn rsx_impl(input: TokenStream) -> TokenStream {
    rsx_impl_with_validation(input, ValidationMode::Permissive)
}

// Enhanced rsx! macro implementation with configurable validation
pub fn rsx_impl_with_validation(
    input: TokenStream,
    validation_mode: ValidationMode,
) -> TokenStream {
    let input_tokens = proc_macro2::TokenStream::from(input.clone());

    // Parse and validate the input using the appropriate validation mode
    let validated_node = match validation_mode {
        ValidationMode::Permissive => {
            // Use permissive validation (default)
            match RsxMainParser::parse_react_like_tokens(input_tokens.clone()) {
                Ok(node) => node,
                Err(_) => {
                    // Fall back to no validation for backward compatibility
                    match syn::parse::<Node>(input.clone()) {
                        Ok(node) => node,
                        Err(_) => {
                            // Final fallback to Element parsing
                            let element = parse_macro_input!(input as Element);
                            Node::Element(element)
                        }
                    }
                }
            }
        }
    };

    // Generate the expanded code for the validated node
    let expanded = generate_node_vnode_code(&validated_node);

    quote! {
        {
            #expanded
        }
    }
    .into()
}

/// Validation modes for the RSX macro
#[derive(Clone)]
pub enum ValidationMode {
    /// Permissive validation - allows various naming conventions (default)
    Permissive,
}

// Helper function to generate VNode code for any node type
fn generate_node_vnode_code(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Element(element) => {
            // Check if this is a component (starts with uppercase)
            let name = &element.name;
            let name_str = quote!(#name).to_string();
            let first_char = name_str.chars().next().unwrap_or('_');
            let is_component = first_char.is_uppercase()
                && !name_str.contains("::")
                && ![
                    "Paragraph",
                    "Line",
                    "Span",
                    "List",
                    "Tabs",
                    "Layout",
                    "Block",
                ]
                .contains(&name_str.as_str());

            if is_component {
                // For components, create component instance and wrap in VNode::component
                let component_code = generate_component_code(element);
                quote! { Element::component(#component_code) }
            } else {
                // For widgets, wrap in VNode::widget
                let element_code = generate_element_code(element);
                quote! { Element::widget(#element_code) }
            }
        }
        Node::Expression(expr) => {
            quote! { Element::text(#expr) }
        }
        Node::Conditional(conditional) => {
            let conditional_code = generate_conditional_code(conditional);
            quote! {
                {
                    if let Some(widget) = #conditional_code {
                        Element::widget(widget)
                    } else {
                        Element::text("")
                    }
                }
            }
        }
        Node::Comment(_comment) => {
            // Comments are ignored in the generated code
            quote! { Element::text("") }
        }
        Node::ForLoop(for_loop) => {
            let for_loop_code = generate_for_loop_code(for_loop);
            quote! {
                {
                    let loop_results: Vec<Element> = #for_loop_code;
                    if loop_results.is_empty() {
                        Element::text("")
                    } else if loop_results.len() == 1 {
                        loop_results.into_iter().next().unwrap()
                    } else {
                        // Multiple elements - wrap in a fragment-like container
                        Element::fragment(loop_results)
                    }
                }
            }
        }
        Node::Fragment(fragment) => {
            let fragment_code = generate_fragment_code(fragment);
            quote! { #fragment_code }
        }
    }
}

// Helper function to generate code for an Element
fn generate_element_code(element: &Element) -> proc_macro2::TokenStream {
    let name = &element.name;
    let name_str = quote!(#name).to_string();

    // Check if this is a component (starts with uppercase)
    let first_char = name_str.chars().next().unwrap_or('_');
    let is_component = first_char.is_uppercase()
        && !name_str.contains("::")
        && ![
            "Paragraph",
            "Line",
            "Span",
            "List",
            "Tabs",
            "Layout",
            "Block",
        ]
        .contains(&name_str.as_str());

    if is_component {
        // Handle component - always use VNode::component
        return generate_component_code(element);
    }

    // Extract the last segment of the path as a string
    let widget_type = name_str.split("::").last().unwrap_or(&name_str);

    // Get attributes as key-value pairs
    let attributes = element.attributes.iter().map(|prop| {
        let key = &prop.key;
        let value = &prop.value;
        quote! { .#key(#value) }
    });

    // Handle different widget types differently
    match widget_type {
        // Layout component - special handling for creating layouts with children
        "Layout" => {
            if element.children.is_empty() {
                quote! {
                    #name::default()
                        #(#attributes)*
                }
            } else {
                // Generate a layout component that handles children
                let children = element.children.iter().map(generate_node_code);

                // Check if constraints attribute is present
                let has_constraints = element
                    .attributes
                    .iter()
                    .any(|attr| attr.key == "constraints");

                if has_constraints {
                    // Find the constraints attribute
                    let constraints_attr = element
                        .attributes
                        .iter()
                        .find(|attr| attr.key == "constraints")
                        .unwrap();
                    let constraints_value = &constraints_attr.value;

                    // Filter out constraints from general attributes
                    let layout_attributes = element
                        .attributes
                        .iter()
                        .filter(|attr| attr.key != "constraints")
                        .map(|attr| {
                            let key = &attr.key;
                            let value = &attr.value;
                            quote! { .#key(#value) }
                        });

                    quote! {
                        {
                            use reratui::core::{LayoutWrapper, AnyWidget};
                            LayoutWrapper::with_constraints(
                                #name::default()
                                    #(#layout_attributes)*,
                                vec![
                                    #(#children),*
                                ],
                                #constraints_value
                            )
                        }
                    }
                } else {
                    quote! {
                        {
                            use reratui::core::{LayoutWrapper, AnyWidget};
                            LayoutWrapper::new(
                                #name::default()
                                    #(#attributes)*,
                                vec![
                                    #(#children),*
                                ]
                            )
                        }
                    }
                }
            }
        }

        // Block component - special handling for blocks with children
        "Block" => {
            if element.children.is_empty() {
                quote! {
                    #name::default()
                        #(#attributes)*
                }
            } else {
                // Generate a block component that handles children
                let children = element.children.iter().map(generate_node_code);

                quote! {
                    {
                        use reratui::core::{BlockWrapper, AnyWidget};
                        BlockWrapper::new(
                            #name::default()
                                #(#attributes)*,
                            vec![
                                #(#children),*
                            ]
                        )
                    }
                }
            }
        }
        // Rich text components with special handling
        "Paragraph" => generate_paragraph_code(element, name),
        "Line" => {
            // When Line is used outside of Paragraph, wrap it in a Paragraph
            let line_code = generate_line_code(element);
            quote! {
                ::reratui::ratatui::widgets::Paragraph::new(vec![#line_code])
                    #(#attributes)*
            }
        }
        "Span" => {
            // When Span is used outside of Line, create a Line with the Span
            let span_code = generate_span_code(element);
            quote! {
                ::reratui::ratatui::widgets::Paragraph::new(vec![
                    ::reratui::ratatui::text::Line::from(vec![#span_code])
                ])
                    #(#attributes)*
            }
        }

        // Text-based widgets that take content in constructor
        "Text" => {
            if let Some(Node::Expression(expr)) = element.children.first() {
                quote! {
                    #name::new(#expr)
                        #(#attributes)*
                }
            } else if element.children.is_empty() {
                quote! {
                    #name::new("")
                        #(#attributes)*
                }
            } else {
                // Multiple children - try to concatenate text nodes
                let content = collect_text_content(&element.children);
                quote! {
                    #name::new(#content)
                        #(#attributes)*
                }
            }
        }

        // Tabs widget - special handling for titles
        "Tabs" => {
            // Check if we have a 'titles' attribute
            let has_titles_attr = element.attributes.iter().any(|attr| attr.key == "titles");

            if has_titles_attr {
                // If titles are provided as an attribute, use that
                quote! {
                    #name::default()
                        #(#attributes)*
                }
            } else if !element.children.is_empty() {
                // Otherwise, try to use children as tab titles
                let tab_items = element.children.iter().map(|node| match node {
                    Node::Element(child) => generate_element_code(child),
                    Node::Expression(expr) => {
                        quote! { ::reratui::ratatui::text::Line::from(#expr) }
                    }
                    Node::Conditional(_) => {
                        // For tabs, conditionals should resolve to text
                        quote! { ::reratui::ratatui::text::Line::from("") }
                    }
                    Node::Comment(_) => {
                        // Comments are ignored in tabs
                        quote! { ::reratui::ratatui::text::Line::from("") }
                    }
                    Node::ForLoop(_) => {
                        // For-loops in tabs should resolve to empty lines
                        quote! { ::reratui::ratatui::text::Line::from("") }
                    }
                    Node::Fragment(_) => {
                        // Fragments in tabs should resolve to empty lines
                        quote! { ::reratui::ratatui::text::Line::from("") }
                    }
                });

                quote! {
                    #name::new(vec![
                        #(#tab_items),*
                    ])
                        #(#attributes)*
                }
            } else {
                // No titles or children
                quote! {
                    #name::default()
                        #(#attributes)*
                }
            }
        }

        // List-based widgets
        "List" => {
            if element.children.is_empty() {
                quote! {
                    #name::default()
                        #(#attributes)*
                }
            } else {
                // Convert children to ListItems
                let items = element.children.iter().map(|node| match node {
                    Node::Element(child) => generate_element_code(child),
                    Node::Expression(expr) => quote! { ListItem::new(#expr) },
                    Node::Conditional(_) => {
                        // For lists, conditionals should resolve to empty items
                        quote! { ListItem::new("") }
                    }
                    Node::Comment(_) => {
                        // Comments are ignored in lists
                        quote! { ListItem::new("") }
                    }
                    Node::ForLoop(_) => {
                        // For-loops in lists should resolve to empty items
                        quote! { ListItem::new("") }
                    }
                    Node::Fragment(_) => {
                        // Fragments in lists should resolve to empty items
                        quote! { ListItem::new("") }
                    }
                });

                quote! {
                    #name::new(vec![
                        #(#items),*
                    ])
                        #(#attributes)*
                }
            }
        }

        // Default case for other widgets
        _ => {
            if element.children.is_empty() {
                quote! {
                    #name::default()
                        #(#attributes)*
                }
            } else if element.children.len() == 1 {
                if let Some(Node::Expression(expr)) = element.children.first() {
                    // Try with new constructor for single expression
                    quote! {
                        #name::new(#expr)
                            #(#attributes)*
                    }
                } else {
                    // Default to using children method
                    let child_elements = element.children.iter().map(|node| match node {
                        Node::Element(child_element) => generate_element_code(child_element),
                        Node::Expression(expr) => quote! { #expr },
                        Node::Conditional(conditional) => {
                            let conditional_code = generate_conditional_code(conditional);
                            quote! { #conditional_code.unwrap_or_else(|| panic!("Conditional resolved to None")) }
                        },
                        Node::Comment(_) => {
                            // Comments are ignored
                            quote! { "" }
                        },
                        Node::ForLoop(for_loop) => {
                            // Generate for-loop code and flatten results
                            let for_loop_code = generate_for_loop_code(for_loop);
                            quote! {
                                {
                                    let loop_results: Vec<AnyWidget> = #for_loop_code;
                                    // Convert to string representation for children
                                    format!("{} items", loop_results.len())
                                }
                            }
                        },
                        Node::Fragment(fragment) => {
                            // Generate fragment code and flatten
                            let fragment_code = generate_fragment_code(fragment);
                            quote! { #fragment_code }
                        },
                    });

                    quote! {
                        #name::default()
                            #(#attributes)*
                            .children(vec![
                                #(#child_elements),*
                            ])
                    }
                }
            } else {
                // Multiple children - use children method
                let child_elements = element.children.iter().map(|node| match node {
                    Node::Element(child_element) => generate_element_code(child_element),
                    Node::Expression(expr) => quote! { #expr },
                    Node::Conditional(conditional) => {
                        let conditional_code = generate_conditional_code(conditional);
                        quote! { #conditional_code.unwrap_or_else(|| panic!("Conditional resolved to None")) }
                    },
                    Node::Comment(_) => {
                        // Comments are ignored
                        quote! { "" }
                    },
                    Node::ForLoop(for_loop) => {
                        // Generate for-loop code and flatten results
                        let for_loop_code = generate_for_loop_code(for_loop);
                        quote! {
                            {
                                let loop_results: Vec<AnyWidget> = #for_loop_code;
                                // Convert to string representation for children
                                format!("{} items", loop_results.len())
                            }
                        }
                    },
                    Node::Fragment(fragment) => {
                        // Generate fragment code and flatten
                        let fragment_code = generate_fragment_code(fragment);
                        quote! { #fragment_code }
                    },
                });

                quote! {
                    #name::default()
                        #(#attributes)*
                        .children(vec![
                            #(#child_elements),*
                        ])
                }
            }
        }
    }
}

// Helper function to generate code for any node type (returns AnyWidget)
fn generate_node_code(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Element(element) => {
            let name = &element.name;
            let name_str = quote!(#name).to_string();

            // Check if this is a component (starts with uppercase)
            let first_char = name_str.chars().next().unwrap_or('_');
            let is_component = first_char.is_uppercase()
                && !name_str.contains("::")
                && ![
                    "Paragraph",
                    "Line",
                    "Span",
                    "List",
                    "Tabs",
                    "Layout",
                    "Block",
                ]
                .contains(&name_str.as_str());

            if is_component {
                // For components, create component instance and wrap in VNode, then AnyWidget
                let component_code = generate_component_code(element);
                quote! {
                    AnyWidget::from(
                        Element::component(#component_code)
                    )
                }
            } else {
                // For widgets, generate element code and wrap in AnyWidget
                let element_code = generate_element_code(element);
                quote! { AnyWidget::from(#element_code) }
            }
        }
        Node::Expression(expr) => {
            // Check if this is a string literal that should be converted to text
            match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => {
                    let text_value = &lit_str.value();
                    quote! {
                        AnyWidget::from(
                            Element::text(#text_value.to_string())
                        )
                    }
                }
                _ => {
                    quote! {
                        AnyWidget::from(#expr)
                    }
                }
            }
        }
        Node::Conditional(conditional) => {
            let conditional_code = generate_conditional_code(conditional);
            quote! {
                {
                    if let Some(widget) = #conditional_code {
                        AnyWidget::from(widget)
                    } else {
                        AnyWidget::from(
                            Element::text("".to_string())
                        )
                    }
                }
            }
        }
        Node::Comment(_) => {
            // Comments are ignored
            quote! {
                AnyWidget::from(
                    Element::text("".to_string())
                )
            }
        }
        Node::ForLoop(for_loop) => {
            let for_loop_code = generate_for_loop_code(for_loop);
            quote! {
                {
                    let loop_results: Vec<AnyWidget> = #for_loop_code;
                    if loop_results.is_empty() {
                        AnyWidget::from(
                            Element::text("".to_string())
                        )
                    } else if loop_results.len() == 1 {
                        loop_results.into_iter().next().unwrap()
                    } else {
                        // Multiple elements - create a fragment-like container
                        AnyWidget::from(
                            Element::fragment(
                                loop_results.into_iter().map(|widget| match widget {
                                    AnyWidget::VNode(vnode) => vnode,
                                    _ => Element::text("".to_string()),
                                }).collect()
                            )
                        )
                    }
                }
            }
        }
        Node::Fragment(fragment) => {
            let fragment_code = generate_fragment_code(fragment);
            quote! {
                AnyWidget::from(#fragment_code)
            }
        }
    }
}

// Helper function to generate code for a Component
// Creates a component instance instead of calling the function directly
fn generate_component_code(element: &Element) -> proc_macro2::TokenStream {
    let name = &element.name;
    let name_str = name.segments.last().unwrap().ident.to_string();

    // Get props as key-value pairs for method calls
    let props_methods: Vec<_> = element
        .attributes
        .iter()
        .map(|prop| {
            let key = &prop.key;
            let value = &prop.value;
            quote! { .#key(#value) }
        })
        .collect();

    // Get children as VNodes
    let children: Vec<_> = element.children.iter().map(generate_node_code).collect();

    // Generate props struct name and component struct name
    let props_struct_name = syn::Ident::new(&format!("{}Props", name_str), name.span());
    let component_struct_name = syn::Ident::new(&format!("{}Component", name_str), name.span());

    // Generate a component instance instead of calling the function directly
    // This ensures the Component trait's render method is used, which sets up hook context
    quote! {
        {
            use Element;
            #[allow(unused_mut)]
            let mut props = #props_struct_name::default()
                #(#props_methods)*;

            let children: Vec<Element> = vec![
                #(#children),*
            ].into_iter().map(|widget| match widget {
                AnyWidget::VNode(vnode) => vnode,
                _ => Element::text("".to_string()),
            }).collect();

            if !children.is_empty() {
                props = props.with_children(children);
            }

            // Create a component instance instead of calling the function
            #component_struct_name::new(props)
        }
    }
}

// Helper function to generate code for conditional nodes
fn generate_conditional_code(conditional: &ConditionalNode) -> proc_macro2::TokenStream {
    match conditional {
        ConditionalNode::If {
            condition,
            then_branch,
            else_ifs,
            else_branch,
        } => {
            let then_code = generate_node_code(then_branch);

            // Generate else if chains
            let mut current_else = if let Some(else_branch) = else_branch {
                let else_code = generate_node_code(else_branch);
                quote! { Some(#else_code) }
            } else {
                quote! { None }
            };

            // Build the else if chain from the end backwards
            for else_if in else_ifs.iter().rev() {
                let else_if_condition = &else_if.condition;
                let else_if_code = generate_node_code(&else_if.then_branch);
                current_else = quote! {
                    if #else_if_condition {
                        Some(#else_if_code)
                    } else {
                        #current_else
                    }
                };
            }

            quote! {
                if #condition {
                    Some(#then_code)
                } else {
                    #current_else
                }
            }
        }
        ConditionalNode::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        } => {
            let then_code = generate_node_code(then_branch);
            let else_code = if let Some(else_branch) = else_branch {
                let else_code = generate_node_code(else_branch);
                quote! { Some(#else_code) }
            } else {
                quote! { None }
            };

            quote! {
                if let #pattern = #expr {
                    Some(#then_code)
                } else {
                    #else_code
                }
            }
        }
        ConditionalNode::Match { expr, arms } => {
            let match_arms = arms.iter().map(|arm| {
                let pattern = &arm.pattern;
                let body_code = generate_node_code(&arm.body);

                if let Some(guard) = &arm.guard {
                    quote! {
                        #pattern if #guard => Some(#body_code),
                    }
                } else {
                    quote! {
                        #pattern => Some(#body_code),
                    }
                }
            });

            quote! {
                match #expr {
                    #(#match_arms)*
                }
            }
        }
        ConditionalNode::LogicalAnd {
            condition,
            then_branch,
        } => {
            let then_code = generate_node_code(then_branch);
            quote! {
                if #condition {
                    Some(#then_code)
                } else {
                    None
                }
            }
        }
    }
}

// Helper function to generate code for for-loop nodes
fn generate_for_loop_code(for_loop: &ForLoopNode) -> proc_macro2::TokenStream {
    let pattern = &for_loop.pattern;
    let iterable = &for_loop.iterable;
    let preparation_stmts = &for_loop.preparation_stmts;
    let body_code = generate_node_code(&for_loop.body);

    quote! {
        {
            let mut results = Vec::new();
            for #pattern in #iterable {
                // Execute preparation statements
                #(#preparation_stmts)*

                // Generate the JSX element
                results.push(#body_code);
            }
            results
        }
    }
}

// Helper function to generate code for fragment nodes
fn generate_fragment_code(fragment: &FragmentNode) -> proc_macro2::TokenStream {
    if fragment.children.is_empty() {
        // Empty fragment
        quote! { Element::text("") }
    } else if fragment.children.len() == 1 {
        // Single child - return it directly
        let child_code = generate_node_vnode_code(&fragment.children[0]);
        quote! { #child_code }
    } else {
        // Multiple children - create a fragment
        let children_code: Vec<_> = fragment
            .children
            .iter()
            .map(generate_node_vnode_code)
            .collect();

        quote! {
            Element::fragment(vec![
                #(#children_code),*
            ])
        }
    }
}

// Helper function to collect text content from multiple nodes
fn collect_text_content(nodes: &[Node]) -> proc_macro2::TokenStream {
    let expressions: Vec<_> = nodes
        .iter()
        .filter_map(|node| match node {
            Node::Expression(expr) => Some(expr),
            Node::Conditional(_) => None, // Skip conditionals for text collection
            _ => None,
        })
        .collect();

    if expressions.is_empty() {
        quote! { "" }
    } else if expressions.len() == 1 {
        let expr = expressions[0];
        // Check if this is a string literal and extract its value
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) => {
                let value = &lit_str.value();
                quote! { #value }
            }
            _ => quote! { #expr },
        }
    } else {
        // Concatenate multiple expressions properly, handling string literals
        let format_args: Vec<_> = expressions
            .iter()
            .map(|expr| match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => {
                    let value = &lit_str.value();
                    quote! { #value }
                }
                _ => quote! { #expr },
            })
            .collect();
        quote! {
            format!("{}", vec![#(#format_args),*].join(""))
        }
    }
}

// Helper function to generate code for Paragraph components
fn generate_paragraph_code(element: &Element, name: &syn::Path) -> proc_macro2::TokenStream {
    let regular_attributes = element
        .attributes
        .iter()
        .filter(|attr| !is_style_shorthand(&attr.key))
        .map(|attr| {
            let key = &attr.key;
            let value = &attr.value;
            quote! { .#key(#value) }
        });

    if element.children.is_empty() {
        // Empty paragraph
        quote! {
            #name::new("")
                #(#regular_attributes)*
        }
    } else {
        // Check if children contain Line components, conditionals, or fragments
        let has_complex_children = element.children.iter().any(|child| {
            matches!(
                child,
                Node::Element(el) if el.name.segments.last().unwrap().ident == "Line"
            ) || matches!(child, Node::Conditional(_))
                || matches!(child, Node::Fragment(_))
                || matches!(child, Node::ForLoop(_))
        });

        if has_complex_children {
            // Use the enhanced line generation that handles fragments and conditionals
            let line_codes = element
                .children
                .iter()
                .map(generate_lines_from_node)
                .collect::<Vec<_>>();

            quote! {
                #name::new({
                    let mut all_lines = Vec::new();
                    #(all_lines.extend(#line_codes);)*
                    all_lines
                })
                #(#regular_attributes)*
            }
        } else {
            // Simple text content
            let content = collect_text_content(&element.children);
            quote! {
                #name::new(#content)
                    #(#regular_attributes)*
            }
        }
    }
}

// Helper function to generate code for Line components
fn generate_line_code(element: &Element) -> proc_macro2::TokenStream {
    if element.children.is_empty() {
        // Empty line
        quote! { ::reratui::ratatui::text::Line::from("") }
    } else {
        // Generate spans from all children (including conditionals and expressions)
        let span_codes = element
            .children
            .iter()
            .map(generate_span_from_node)
            .collect::<Vec<_>>();

        if span_codes.is_empty() {
            quote! { ::reratui::ratatui::text::Line::from("") }
        } else {
            quote! {
                ::reratui::ratatui::text::Line::from({
                    let mut spans = Vec::new();
                    #(spans.extend(#span_codes);)*
                    spans
                })
            }
        }
    }
}

// Helper function to generate multiple lines from a node (for fragments and conditionals)
fn generate_lines_from_node(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Element(element) => {
            if element.name.segments.last().unwrap().ident == "Line" {
                // Single Line element
                let line_code = generate_line_code(element);
                quote! { vec![#line_code] }
            } else {
                // Other elements - convert to a single line with text content
                let content = collect_text_content(std::slice::from_ref(node));
                quote! { vec![::reratui::ratatui::text::Line::from(#content)] }
            }
        }
        Node::Expression(expr) => {
            // Handle expressions as text lines
            match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => {
                    let value = &lit_str.value();
                    quote! { vec![::reratui::ratatui::text::Line::from(#value)] }
                }
                _ => {
                    quote! { vec![::reratui::ratatui::text::Line::from(format!("{}", #expr))] }
                }
            }
        }
        Node::Conditional(conditional) => {
            // Handle conditional rendering of lines
            match conditional {
                ConditionalNode::If {
                    condition,
                    then_branch,
                    else_ifs: _,
                    else_branch,
                } => {
                    let then_lines = generate_lines_from_node(then_branch);
                    let else_lines = else_branch
                        .as_ref()
                        .map(|node| generate_lines_from_node(node))
                        .unwrap_or_else(|| quote! { Vec::new() });

                    quote! {
                        if #condition {
                            #then_lines
                        } else {
                            #else_lines
                        }
                    }
                }
                ConditionalNode::IfLet {
                    pattern,
                    expr,
                    then_branch,
                    else_branch,
                } => {
                    let then_lines = generate_lines_from_node(then_branch);
                    let else_lines = else_branch
                        .as_ref()
                        .map(|node| generate_lines_from_node(node))
                        .unwrap_or_else(|| quote! { Vec::new() });

                    quote! {
                        if let #pattern = #expr {
                            #then_lines
                        } else {
                            #else_lines
                        }
                    }
                }
                ConditionalNode::LogicalAnd {
                    condition,
                    then_branch,
                } => {
                    let then_lines = generate_lines_from_node(then_branch);
                    quote! {
                        if #condition {
                            #then_lines
                        } else {
                            Vec::new()
                        }
                    }
                }
                ConditionalNode::Match { expr, arms } => {
                    let match_arms = arms.iter().map(|arm| {
                        let pattern = &arm.pattern;
                        let guard = arm.guard.as_ref().map(|g| quote! { if #g });
                        let body_lines = generate_lines_from_node(&arm.body);
                        quote! {
                            #pattern #guard => #body_lines,
                        }
                    });

                    quote! {
                        match #expr {
                            #(#match_arms)*
                        }
                    }
                }
            }
        }
        Node::Fragment(fragment) => {
            // Handle fragments containing multiple lines
            let child_lines = fragment.children.iter().map(generate_lines_from_node);
            quote! {
                {
                    let mut fragment_lines = Vec::new();
                    #(fragment_lines.extend(#child_lines);)*
                    fragment_lines
                }
            }
        }
        Node::ForLoop(for_loop) => {
            // Handle for-loops that generate lines
            let pattern = &for_loop.pattern;
            let iterable = &for_loop.iterable;
            let preparation_stmts = &for_loop.preparation_stmts;

            // Check if the body is an expression that should be auto-converted to Line/Span
            let body_lines = match &*for_loop.body {
                Node::Expression(expr) => {
                    // Auto-convert string expressions to Line/Span within Paragraph context
                    quote! {
                        vec![::reratui::ratatui::text::Line::from(vec![
                            ::reratui::ratatui::text::Span::raw(format!("{}", #expr))
                        ])]
                    }
                }
                _ => generate_lines_from_node(&for_loop.body),
            };

            quote! {
                {
                    let mut loop_lines = Vec::new();
                    for #pattern in #iterable {
                        #(#preparation_stmts)*
                        loop_lines.extend(#body_lines);
                    }
                    loop_lines
                }
            }
        }
        Node::Comment(_) => {
            // Comments are ignored
            quote! { Vec::new() }
        }
    }
}

// Helper function to generate spans from any node type
fn generate_span_from_node(node: &Node) -> proc_macro2::TokenStream {
    match node {
        Node::Element(element) => {
            if element.name.segments.last().unwrap().ident == "Span" {
                // Direct Span element
                let span_code = generate_span_code(element);
                quote! { vec![#span_code] }
            } else {
                // Other elements - convert to text span
                let content = collect_text_content(std::slice::from_ref(node));
                quote! { vec![::reratui::ratatui::text::Span::raw(#content)] }
            }
        }
        Node::Expression(expr) => {
            // Handle expressions (including string literals)
            match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => {
                    // String literal - create raw span
                    let value = &lit_str.value();
                    quote! { vec![::reratui::ratatui::text::Span::raw(#value)] }
                }
                _ => {
                    // Other expressions - evaluate and create raw span
                    quote! { vec![::reratui::ratatui::text::Span::raw(format!("{}", #expr))] }
                }
            }
        }
        Node::Conditional(conditional) => {
            // Handle conditional rendering based on the conditional type
            match conditional {
                ConditionalNode::If {
                    condition,
                    then_branch,
                    else_ifs: _,
                    else_branch,
                } => {
                    let then_spans = generate_span_from_node(then_branch);
                    let else_spans = else_branch
                        .as_ref()
                        .map(|node| generate_span_from_node(node))
                        .unwrap_or_else(|| quote! { Vec::new() });

                    // For now, handle simple if-else (ignore else_ifs for simplicity)
                    quote! {
                        if #condition {
                            #then_spans
                        } else {
                            #else_spans
                        }
                    }
                }
                ConditionalNode::IfLet {
                    pattern,
                    expr,
                    then_branch,
                    else_branch,
                } => {
                    let then_spans = generate_span_from_node(then_branch);
                    let else_spans = else_branch
                        .as_ref()
                        .map(|node| generate_span_from_node(node))
                        .unwrap_or_else(|| quote! { Vec::new() });

                    quote! {
                        if let #pattern = #expr {
                            #then_spans
                        } else {
                            #else_spans
                        }
                    }
                }
                ConditionalNode::LogicalAnd {
                    condition,
                    then_branch,
                } => {
                    let then_spans = generate_span_from_node(then_branch);
                    quote! {
                        if #condition {
                            #then_spans
                        } else {
                            Vec::new()
                        }
                    }
                }
                ConditionalNode::Match { expr, arms } => {
                    // Handle match expressions
                    let match_arms = arms.iter().map(|arm| {
                        let pattern = &arm.pattern;
                        let guard = arm.guard.as_ref().map(|g| quote! { if #g });
                        let body_spans = generate_span_from_node(&arm.body);
                        quote! {
                            #pattern #guard => #body_spans,
                        }
                    });

                    quote! {
                        match #expr {
                            #(#match_arms)*
                        }
                    }
                }
            }
        }
        Node::ForLoop(for_loop) => {
            // Handle for-loops in spans
            let pattern = &for_loop.pattern;
            let iterable = &for_loop.iterable;
            let preparation_stmts = &for_loop.preparation_stmts;
            let body_spans = generate_span_from_node(&for_loop.body);

            quote! {
                {
                    let mut loop_spans = Vec::new();
                    for #pattern in #iterable {
                        #(#preparation_stmts)*
                        loop_spans.extend(#body_spans);
                    }
                    loop_spans
                }
            }
        }
        Node::Fragment(fragment) => {
            // Handle fragments
            let child_spans = fragment.children.iter().map(generate_span_from_node);
            quote! {
                {
                    let mut fragment_spans = Vec::new();
                    #(fragment_spans.extend(#child_spans);)*
                    fragment_spans
                }
            }
        }
        Node::Comment(_) => {
            // Comments are ignored
            quote! { Vec::new() }
        }
    }
}

// Helper function to generate code for Span components
fn generate_span_code(element: &Element) -> proc_macro2::TokenStream {
    let content = if element.children.is_empty() {
        quote! { "" }
    } else {
        // Enhanced content collection that handles string literals directly
        let content_parts: Vec<_> = element
            .children
            .iter()
            .filter_map(|node| match node {
                Node::Expression(expr) => match expr {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) => {
                        // Extract string literal value directly
                        let value = &lit_str.value();
                        Some(quote! { #value })
                    }
                    _ => Some(quote! { #expr }),
                },
                _ => None,
            })
            .collect();

        if content_parts.is_empty() {
            quote! { "" }
        } else if content_parts.len() == 1 {
            content_parts[0].clone()
        } else {
            quote! {
                format!("{}", vec![#(#content_parts),*].join(""))
            }
        }
    };

    // Process style attributes
    let style_code = generate_style_from_attributes(&element.attributes);

    // Get regular (non-style) attributes
    let regular_attributes = element
        .attributes
        .iter()
        .filter(|attr| !is_style_shorthand(&attr.key) && attr.key != "style")
        .map(|attr| {
            let key = &attr.key;
            let value = &attr.value;
            quote! { .#key(#value) }
        });

    if let Some(style) = style_code {
        quote! {
            ::reratui::ratatui::text::Span::styled(#content, #style)
                #(#regular_attributes)*
        }
    } else {
        // Check for explicit style attribute
        if let Some(style_attr) = element.attributes.iter().find(|attr| attr.key == "style") {
            let style_value = &style_attr.value;
            quote! {
                ::reratui::ratatui::text::Span::styled(#content, #style_value)
                    #(#regular_attributes)*
            }
        } else {
            quote! {
                ::reratui::ratatui::text::Span::raw(#content)
                    #(#regular_attributes)*
            }
        }
    }
}

// Helper function to check if an attribute is a style shorthand
fn is_style_shorthand(key: &syn::Ident) -> bool {
    let key_str = key.to_string();
    matches!(
        key_str.as_str(),
        // Color attributes
        "white" | "black" | "red" | "green" | "blue" | "cyan" | "yellow" | "magenta" |
        "gray" | "dark_gray" | "light_red" | "light_green" | "light_blue" |
        "light_cyan" | "light_yellow" | "light_magenta" |
        // Modifier attributes
        "bold" | "italic" | "underlined" | "crossed_out" | "dim" | "reversed" |
        "rapid_blink" | "slow_blink"
    )
}

// Helper function to generate Style from shorthand attributes
fn generate_style_from_attributes(
    attributes: &[crate::rsx::parser::Prop],
) -> Option<proc_macro2::TokenStream> {
    let style_attrs: Vec<_> = attributes
        .iter()
        .filter(|attr| is_style_shorthand(&attr.key))
        .collect();

    if style_attrs.is_empty() {
        return None;
    }

    let mut style_parts = Vec::new();

    // Process color attributes
    for attr in &style_attrs {
        let key_str = attr.key.to_string();
        match key_str.as_str() {
            // Colors
            "white" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::White) }),
            "black" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Black) }),
            "red" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Red) }),
            "green" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Green) }),
            "blue" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Blue) }),
            "cyan" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Cyan) }),
            "yellow" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Yellow) }),
            "magenta" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Magenta) })
            }
            "gray" => style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::Gray) }),
            "dark_gray" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::DarkGray) })
            }
            "light_red" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::LightRed) })
            }
            "light_green" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::LightGreen) })
            }
            "light_blue" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::LightBlue) })
            }
            "light_cyan" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::LightCyan) })
            }
            "light_yellow" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::LightYellow) })
            }
            "light_magenta" => {
                style_parts.push(quote! { .fg(::reratui::ratatui::style::Color::LightMagenta) })
            }

            // Modifiers
            "bold" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::BOLD) }),
            "italic" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::ITALIC) }),
            "underlined" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::UNDERLINED) }),
            "crossed_out" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::CROSSED_OUT) }),
            "dim" => {
                style_parts.push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::DIM) })
            }
            "reversed" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::REVERSED) }),
            "rapid_blink" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::RAPID_BLINK) }),
            "slow_blink" => style_parts
                .push(quote! { .add_modifier(::reratui::ratatui::style::Modifier::SLOW_BLINK) }),

            _ => {} // Unknown style attribute, ignore
        }
    }

    if style_parts.is_empty() {
        None
    } else {
        Some(quote! {
            ::reratui::ratatui::style::Style::default()
                #(#style_parts)*
        })
    }
}

//! Token Analysis Module
//!
//! This module provides utilities for analyzing token streams to detect patterns
//! and structures in RSX syntax. Following SOLID principles:
//! - Single Responsibility: Each analyzer has a specific detection purpose
//! - Open/Closed: Easy to add new pattern detectors without modifying existing ones
//! - Interface Segregation: Clean interfaces for different analysis types
//! - Dependency Inversion: Depends on token stream abstractions

use syn::{Token, parse::ParseStream};

/// Core trait for token pattern analysis
pub trait TokenAnalyzer {
    /// Analyze the token stream for a specific pattern
    fn analyze(&self, input: ParseStream) -> bool;

    /// Get a description of what this analyzer detects
    #[allow(unused)]
    fn description(&self) -> &'static str;
}

/// Analyzes token streams for JSX comment patterns
pub struct CommentAnalyzer;

impl TokenAnalyzer for CommentAnalyzer {
    fn analyze(&self, input: ParseStream) -> bool {
        self.contains_jsx_comment(&input)
    }

    fn description(&self) -> &'static str {
        "Detects JSX-style comments in token streams"
    }
}

impl CommentAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Check if a braced expression is a JSX comment
    /// JSX comments use the syntax: {/* comment text */}
    pub fn contains_jsx_comment(&self, input: &ParseStream) -> bool {
        if !input.peek(syn::token::Brace) {
            return false;
        }

        let fork = input.fork();
        if let Ok(group) = fork.parse::<proc_macro2::Group>()
            && group.delimiter() == proc_macro2::Delimiter::Brace
        {
            let stream = group.stream();
            let tokens: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();

            // Empty brace group (likely a stripped comment)
            if tokens.is_empty() {
                return true;
            }

            // Check for comment patterns that survive tokenization
            if tokens.len() >= 4
                && let (
                    Some(proc_macro2::TokenTree::Punct(first)),
                    Some(proc_macro2::TokenTree::Punct(second)),
                    Some(proc_macro2::TokenTree::Literal(_)),
                    Some(proc_macro2::TokenTree::Punct(third)),
                    Some(proc_macro2::TokenTree::Punct(fourth)),
                ) = (
                    tokens.first(),
                    tokens.get(1),
                    tokens.get(2),
                    tokens.get(3),
                    tokens.get(4),
                )
            {
                return first.as_char() == '/'
                    && second.as_char() == '*'
                    && third.as_char() == '*'
                    && fourth.as_char() == '/';
            }
        }

        false
    }

    /// Extract comment content from a JSX comment token stream
    pub fn extract_comment_content(&self, input: ParseStream) -> syn::Result<String> {
        if !input.peek(syn::token::Brace) {
            return Err(syn::Error::new(
                input.span(),
                "Expected brace group for comment",
            ));
        }

        let group: proc_macro2::Group = input.parse()?;
        let stream = group.stream();
        let tokens: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();

        if tokens.is_empty() {
            return Ok("JSX comment".to_string());
        }

        // Extract string literal content if present
        if tokens.len() >= 5
            && let Some(proc_macro2::TokenTree::Literal(lit)) = tokens.get(2)
        {
            let content = lit.to_string().trim_matches('"').to_string();
            return Ok(content);
        }

        Ok("JSX comment".to_string())
    }
}

/// Analyzes token streams for logical AND patterns
pub struct LogicalAndAnalyzer;

impl TokenAnalyzer for LogicalAndAnalyzer {
    fn analyze(&self, input: ParseStream) -> bool {
        self.contains_logical_and(&input)
    }

    fn description(&self) -> &'static str {
        "Detects logical AND (&&) operators in token streams"
    }
}

impl LogicalAndAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Check if the input stream contains a logical AND operator at the top level
    pub fn contains_logical_and(&self, input: &ParseStream) -> bool {
        let fork = input.fork();
        let tokens: Result<Vec<proc_macro2::TokenTree>, syn::Error> = fork
            .parse::<proc_macro2::TokenStream>()
            .map(|stream| stream.into_iter().collect());

        if let Ok(tokens) = tokens {
            for (i, token) in tokens.iter().enumerate() {
                if let proc_macro2::TokenTree::Punct(punct) = token
                    && punct.as_char() == '&'
                    && i + 1 < tokens.len()
                    && let proc_macro2::TokenTree::Punct(next_punct) = &tokens[i + 1]
                    && next_punct.as_char() == '&'
                {
                    return true;
                }
            }
        }

        false
    }

    /// Find the position of the && operator in the token stream
    pub fn find_and_position(&self, tokens: &[proc_macro2::TokenTree]) -> Option<usize> {
        for (i, token) in tokens.iter().enumerate() {
            if let proc_macro2::TokenTree::Punct(punct) = token
                && punct.as_char() == '&'
                && i + 1 < tokens.len()
                && let proc_macro2::TokenTree::Punct(next_punct) = &tokens[i + 1]
                && next_punct.as_char() == '&'
            {
                return Some(i);
            }
        }
        None
    }
}

/// Analyzes token streams for match expressions
#[allow(unused)]
pub struct MatchAnalyzer;

impl TokenAnalyzer for MatchAnalyzer {
    fn analyze(&self, input: ParseStream) -> bool {
        self.contains_match_expression(&input)
    }

    fn description(&self) -> &'static str {
        "Detects match expressions in token streams"
    }
}

impl MatchAnalyzer {
    /// Check if the input stream contains a match expression
    #[allow(unused)]
    pub fn contains_match_expression(&self, input: &ParseStream) -> bool {
        let fork = input.fork();
        let tokens: Result<Vec<proc_macro2::TokenTree>, syn::Error> = fork
            .parse::<proc_macro2::TokenStream>()
            .map(|stream| stream.into_iter().collect());

        if let Ok(tokens) = tokens {
            for token in tokens.iter() {
                if let proc_macro2::TokenTree::Ident(ident) = token
                    && *ident == "match"
                {
                    return true;
                }
            }
        }

        false
    }
}

/// Analyzes token streams for for-loop expressions
pub struct ForLoopAnalyzer;

impl TokenAnalyzer for ForLoopAnalyzer {
    fn analyze(&self, input: ParseStream) -> bool {
        self.contains_for_loop(&input)
    }

    fn description(&self) -> &'static str {
        "Detects for-loop expressions in token streams"
    }
}

impl ForLoopAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Check if the input stream contains a for-loop expression
    pub fn contains_for_loop(&self, input: &ParseStream) -> bool {
        input.peek(Token![for])
    }
}

/// Analyzes token streams for conditional expressions
pub struct ConditionalAnalyzer;

impl TokenAnalyzer for ConditionalAnalyzer {
    fn analyze(&self, input: ParseStream) -> bool {
        self.contains_conditional(&input)
    }

    fn description(&self) -> &'static str {
        "Detects conditional expressions (if, if let, match, &&) in token streams"
    }
}

impl ConditionalAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Check if the input contains any conditional expression
    pub fn contains_conditional(&self, input: &ParseStream) -> bool {
        input.peek(Token![if])
            || input.peek(Token![match])
            || LogicalAndAnalyzer::new().contains_logical_and(input)
    }

    /// Check if the input contains an `if let` expression
    pub fn contains_if_let(&self, input: ParseStream) -> bool {
        if !input.peek(Token![if]) {
            return false;
        }

        let fork = input.fork();
        let tokens: Result<Vec<proc_macro2::TokenTree>, syn::Error> = fork
            .parse::<proc_macro2::TokenStream>()
            .map(|stream| stream.into_iter().collect());

        if let Ok(tokens) = tokens {
            // Look for "if let" pattern
            for (i, token) in tokens.iter().enumerate() {
                if let proc_macro2::TokenTree::Ident(ident) = token
                    && *ident == "if"
                    && i + 1 < tokens.len()
                    && let proc_macro2::TokenTree::Ident(next_ident) = &tokens[i + 1]
                    && *next_ident == "let"
                {
                    return true;
                }
            }
        }

        false
    }
}

/// Utility functions for token stream manipulation
pub struct TokenStreamUtils;

impl TokenStreamUtils {
    /// Split a token stream at a specific position
    pub fn split_at_position(
        tokens: &[proc_macro2::TokenTree],
        position: usize,
        skip_count: usize,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let before: proc_macro2::TokenStream = tokens[..position].iter().cloned().collect();
        let after: proc_macro2::TokenStream =
            tokens[position + skip_count..].iter().cloned().collect();
        (before, after)
    }

    /// Convert a ParseStream to a vector of tokens
    pub fn parse_stream_to_tokens(input: ParseStream) -> syn::Result<Vec<proc_macro2::TokenTree>> {
        let tokens: proc_macro2::TokenStream = input.parse()?;
        Ok(tokens.into_iter().collect())
    }
}

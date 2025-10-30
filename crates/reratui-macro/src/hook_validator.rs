//! Hook Validation - Ensures hooks follow the Rules of Hooks
//!
//! Filename: hook_validator.rs
//! Folder: /crates/reratui-macro/src/
//!
//! This module validates that hooks are called correctly according to the Rules of Hooks:
//! 1. Only call hooks at the top level (not inside conditionals, loops, or nested functions)
//! 2. Only call hooks from component functions
//! 3. Hooks must be called in the same order on every render
//!
//! # Architecture
//!
//! - Uses syn's VisitMut to traverse the AST
//! - Tracks branching context (if/else, loops, closures, match)
//! - Emits compile errors when hooks are called in invalid positions
//! - Similar to Yew's hook validation approach

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprCall, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprWhile};

/// Validator that ensures hooks follow the Rules of Hooks
pub struct HookValidator {
    /// Tracks if we're inside a branch (if/loop/closure/match)
    branch_depth: usize,
    /// Errors found during validation
    errors: Vec<syn::Error>,
}

impl HookValidator {
    /// Create a new hook validator
    pub fn new() -> Self {
        Self {
            branch_depth: 0,
            errors: Vec::new(),
        }
    }

    /// Check if we're currently inside a branch
    fn is_branched(&self) -> bool {
        self.branch_depth > 0
    }

    /// Execute a function within a branched context
    fn with_branch<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        self.branch_depth += 1;
        let result = f(self);
        self.branch_depth -= 1;
        result
    }

    /// Check if a function call is a hook (starts with "use_")
    fn is_hook_call(func: &Expr) -> Option<&syn::Path> {
        if let Expr::Path(path_expr) = func
            && let Some(segment) = path_expr.path.segments.last()
            && segment.ident.to_string().starts_with("use_")
        {
            return Some(&path_expr.path);
        }
        None
    }

    /// Validate a hook call and emit error if in invalid position
    fn validate_hook_call(&mut self, path: &syn::Path, span: Span) {
        if self.is_branched() {
            let hook_name = path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_else(|| "hook".to_string());

            let error = syn::Error::new(
                span,
                format!(
                    "Hook `{}` cannot be called inside a conditional, loop, or nested function.\n\
                     \n\
                     Rules of Hooks:\n\
                     1. Only call hooks at the top level of your component\n\
                     2. Don't call hooks inside loops, conditions, or nested functions\n\
                     3. Hooks must be called in the same order every render\n\
                     \n\
                     Help: Move this hook call to the top level of your component function.",
                    hook_name
                ),
            );
            self.errors.push(error);
        }
    }

    /// Validate a component function body
    pub fn validate(mut self, body: &mut syn::Block) -> Result<(), Vec<syn::Error>> {
        self.visit_block_mut(body);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

impl VisitMut for HookValidator {
    /// Visit function calls to detect hook calls
    fn visit_expr_call_mut(&mut self, node: &mut ExprCall) {
        // Check if this is a hook call
        if let Some(path) = Self::is_hook_call(&node.func) {
            self.validate_hook_call(path, node.span());
        }

        // Continue visiting children
        visit_mut::visit_expr_call_mut(self, node);
    }

    /// Visit closures - hooks cannot be called inside closures
    fn visit_expr_closure_mut(&mut self, node: &mut ExprClosure) {
        self.with_branch(|validator| {
            visit_mut::visit_expr_closure_mut(validator, node);
        });
    }

    /// Visit if expressions - hooks cannot be called inside conditionals
    fn visit_expr_if_mut(&mut self, node: &mut ExprIf) {
        // Visit attributes and condition at current level
        for attr in &mut node.attrs {
            visit_mut::visit_attribute_mut(self, attr);
        }
        visit_mut::visit_expr_mut(self, &mut node.cond);

        // Visit then branch in branched context
        self.with_branch(|validator| {
            visit_mut::visit_block_mut(validator, &mut node.then_branch);
        });

        // Visit else branch in branched context
        if let Some((_, else_expr)) = &mut node.else_branch {
            self.with_branch(|validator| {
                visit_mut::visit_expr_mut(validator, else_expr);
            });
        }
    }

    /// Visit match expressions - hooks cannot be called inside match arms
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        // Visit attributes and match expression at current level
        for attr in &mut node.attrs {
            visit_mut::visit_attribute_mut(self, attr);
        }
        visit_mut::visit_expr_mut(self, &mut node.expr);

        // Visit arms in branched context
        self.with_branch(|validator| {
            for arm in &mut node.arms {
                visit_mut::visit_arm_mut(validator, arm);
            }
        });
    }

    /// Visit for loops - hooks cannot be called inside loops
    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        // Visit attributes, label, pattern, and iterable at current level
        for attr in &mut node.attrs {
            visit_mut::visit_attribute_mut(self, attr);
        }
        if let Some(label) = &mut node.label {
            visit_mut::visit_label_mut(self, label);
        }
        visit_mut::visit_pat_mut(self, &mut node.pat);
        visit_mut::visit_expr_mut(self, &mut node.expr);

        // Visit loop body in branched context
        self.with_branch(|validator| {
            visit_mut::visit_block_mut(validator, &mut node.body);
        });
    }

    /// Visit while loops - hooks cannot be called inside loops
    fn visit_expr_while_mut(&mut self, node: &mut ExprWhile) {
        // Visit attributes, label, and condition at current level
        for attr in &mut node.attrs {
            visit_mut::visit_attribute_mut(self, attr);
        }
        if let Some(label) = &mut node.label {
            visit_mut::visit_label_mut(self, label);
        }
        visit_mut::visit_expr_mut(self, &mut node.cond);

        // Visit loop body in branched context
        self.with_branch(|validator| {
            visit_mut::visit_block_mut(validator, &mut node.body);
        });
    }

    /// Visit loop expressions - hooks cannot be called inside loops
    fn visit_expr_loop_mut(&mut self, node: &mut ExprLoop) {
        self.with_branch(|validator| {
            visit_mut::visit_expr_loop_mut(validator, node);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_valid_hook_at_top_level() {
        let mut body: syn::Block = parse_quote! {
            {
                let state = use_state(|| 0);
                let effect = use_effect(|| {}, ());
            }
        };

        let validator = HookValidator::new();
        let result = validator.validate(&mut body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_hook_in_if() {
        let mut body: syn::Block = parse_quote! {
            {
                if condition {
                    let state = use_state(|| 0);
                }
            }
        };

        let validator = HookValidator::new();
        let result = validator.validate(&mut body);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("cannot be called inside"));
    }

    #[test]
    fn test_invalid_hook_in_loop() {
        let mut body: syn::Block = parse_quote! {
            {
                for i in 0..10 {
                    let state = use_state(|| i);
                }
            }
        };

        let validator = HookValidator::new();
        let result = validator.validate(&mut body);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hook_in_closure() {
        let mut body: syn::Block = parse_quote! {
            {
                let callback = || {
                    let state = use_state(|| 0);
                };
            }
        };

        let validator = HookValidator::new();
        let result = validator.validate(&mut body);
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_in_effect_closure_is_invalid() {
        let mut body: syn::Block = parse_quote! {
            {
                use_effect(|| {
                    let state = use_state(|| 0);
                }, ());
            }
        };

        let validator = HookValidator::new();
        let result = validator.validate(&mut body);
        // This should be invalid - hooks in effect closures
        assert!(result.is_err());
    }
}

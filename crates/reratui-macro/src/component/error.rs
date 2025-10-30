//! Error handling for component macro
//!
//! This module provides comprehensive error handling with descriptive messages
//! and actionable feedback for developers using the component macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

/// Result type for component macro operations
pub type ComponentResult<T> = Result<T, ComponentError>;

/// Comprehensive error types for component macro failures
#[derive(Debug)]
pub enum ComponentError {
    /// Invalid function signature
    InvalidSignature {
        message: String,
        span: proc_macro2::Span,
        suggestion: Option<String>,
    },
    /// Invalid return type
    InvalidReturnType {
        message: String,
        span: proc_macro2::Span,
        expected: String,
    },
    /// Invalid parameter structure
    InvalidParameters {
        message: String,
        span: proc_macro2::Span,
        suggestion: Option<String>,
    },
    /// Unsupported generics usage
    UnsupportedGenerics {
        message: String,
        span: proc_macro2::Span,
        limitation: String,
    },
    /// Invalid syntax (e.g., hook validation errors)
    InvalidSyntax(String),
    /// Internal macro error
    InternalError { message: String, context: String },
}

impl ComponentError {
    /// Create an invalid signature error with helpful suggestions
    pub fn invalid_signature<T: Spanned>(
        item: &T,
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::InvalidSignature {
            message: message.into(),
            span: item.span(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create an invalid return type error
    pub fn invalid_return_type<T: Spanned>(
        item: &T,
        message: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self::InvalidReturnType {
            message: message.into(),
            span: item.span(),
            expected: expected.into(),
        }
    }

    /// Create an invalid parameters error
    pub fn invalid_parameters<T: Spanned>(
        item: &T,
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::InvalidParameters {
            message: message.into(),
            span: item.span(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create an unsupported generics error
    pub fn unsupported_generics<T: Spanned>(
        item: &T,
        message: impl Into<String>,
        limitation: impl Into<String>,
    ) -> Self {
        Self::UnsupportedGenerics {
            message: message.into(),
            span: item.span(),
            limitation: limitation.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            context: context.into(),
        }
    }

    /// Convert the error to a compile error token stream
    pub fn to_compile_error(&self) -> TokenStream {
        match self {
            ComponentError::InvalidSignature {
                message,
                span,
                suggestion,
            } => {
                let error_msg = if let Some(suggestion) = suggestion {
                    format!("{}\n\nSuggestion: {}", message, suggestion)
                } else {
                    message.clone()
                };

                quote::quote_spanned! { *span =>
                    compile_error!(#error_msg);
                }
            }
            ComponentError::InvalidReturnType {
                message,
                span,
                expected,
            } => {
                let error_msg = format!("{}\n\nExpected: {}", message, expected);

                quote::quote_spanned! { *span =>
                    compile_error!(#error_msg);
                }
            }
            ComponentError::InvalidParameters {
                message,
                span,
                suggestion,
            } => {
                let error_msg = if let Some(suggestion) = suggestion {
                    format!("{}\n\nSuggestion: {}", message, suggestion)
                } else {
                    message.clone()
                };

                quote::quote_spanned! { *span =>
                    compile_error!(#error_msg);
                }
            }
            ComponentError::UnsupportedGenerics {
                message,
                span,
                limitation,
            } => {
                let error_msg = format!("{}\n\nLimitation: {}", message, limitation);

                quote::quote_spanned! { *span =>
                    compile_error!(#error_msg);
                }
            }
            ComponentError::InvalidSyntax(message) => {
                quote! {
                    compile_error!(#message);
                }
            }
            ComponentError::InternalError { message, context } => {
                let error_msg = format!(
                    "Internal component macro error: {}\nContext: {}\n\nPlease report this as a bug.",
                    message, context
                );

                quote! {
                    compile_error!(#error_msg);
                }
            }
        }
    }
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentError::InvalidSignature { message, .. } => {
                write!(f, "Invalid component signature: {}", message)
            }
            ComponentError::InvalidReturnType {
                message, expected, ..
            } => {
                write!(
                    f,
                    "Invalid return type: {}. Expected: {}",
                    message, expected
                )
            }
            ComponentError::InvalidParameters { message, .. } => {
                write!(f, "Invalid parameters: {}", message)
            }
            ComponentError::UnsupportedGenerics {
                message,
                limitation,
                ..
            } => {
                write!(
                    f,
                    "Unsupported generics: {}. Limitation: {}",
                    message, limitation
                )
            }
            ComponentError::InvalidSyntax(message) => {
                write!(f, "Invalid syntax: {}", message)
            }
            ComponentError::InternalError { message, context } => {
                write!(f, "Internal error: {} (Context: {})", message, context)
            }
        }
    }
}

impl std::error::Error for ComponentError {}

/// Helper trait for converting syn errors to component errors
pub trait IntoComponentError<T> {
    fn into_component_error(self) -> ComponentResult<T>;
}

impl<T> IntoComponentError<T> for syn::Result<T> {
    fn into_component_error(self) -> ComponentResult<T> {
        self.map_err(|syn_error| {
            ComponentError::internal_error(syn_error.to_string(), "Failed to parse syntax tree")
        })
    }
}

/// Predefined error messages for common scenarios
pub mod messages {
    /// Error messages for function signatures
    pub mod signature {
        pub const MISSING_RETURN_TYPE: &str =
            "Component function must have an explicit return type";

        pub const INVALID_RETURN_TYPE: &str =
            "Component function must return Element, VNode, or a compatible type";

        pub const ASYNC_NOT_SUPPORTED: &str =
            "Async component functions are not currently supported";

        pub const UNSAFE_NOT_SUPPORTED: &str = "Unsafe component functions are not supported";
    }

    /// Error messages for parameters
    pub mod parameters {
        pub const INVALID_PATTERN: &str =
            "Component parameters must be simple identifiers (e.g., `name: String`)";

        pub const SELF_PARAMETER: &str = "Component functions cannot have `self` parameters";

        pub const REFERENCE_PARAMETER_SUGGESTION: &str =
            "Use `props: &PropsStruct` for props-based components";

        pub const DIRECT_PARAMETER_SUGGESTION: &str =
            "Use direct parameters like `name: String, age: u32` for direct parameter components";
    }

    /// Error messages for generics
    pub mod generics {
        pub const COMPLEX_BOUNDS: &str =
            "Complex generic bounds are not fully supported in component functions";

        pub const LIFETIME_PARAMETERS: &str =
            "Lifetime parameters in component functions have limited support";
    }
}

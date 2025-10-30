/// Comprehensive error type for RSX macro processing
#[derive(Debug, thiserror::Error)]
pub enum RsxError {
    #[error("Invalid component name: {message}")]
    InvalidComponentName {
        message: String,
        span: proc_macro2::Span,
        suggestion: Option<String>,
    },

    #[error("Invalid attribute: {message}")]
    #[allow(unused)]
    InvalidAttribute {
        message: String,
        span: proc_macro2::Span,
        suggestion: Option<String>,
    },

    #[error("Mismatched tags: {message}")]
    MismatchedTags {
        message: String,
        span: proc_macro2::Span,
        opening_tag: String,
        closing_tag: String,
    },

    #[error("Syntax error: {message}")]
    SyntaxError {
        message: String,
        span: proc_macro2::Span,
        suggestion: Option<String>,
    },

    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        span: proc_macro2::Span,
        details: Option<String>,
    },

    #[error("Internal error: {message}")]
    #[allow(unused)]
    InternalError { message: String, context: String },

    #[error("For-loop error: {message}")]
    ForLoopError {
        message: String,
        span: proc_macro2::Span,
        suggestion: Option<String>,
    },
}

impl RsxError {
    /// Convert to a syn::Error with rich formatting
    pub fn to_syn_error(&self) -> syn::Error {
        match self {
            RsxError::InvalidComponentName {
                message,
                span,
                suggestion,
            } => {
                let error_msg = if let Some(suggestion) = suggestion {
                    format!(
                        "Invalid component name: {}\n\nSuggestion: {}",
                        message, suggestion
                    )
                } else {
                    format!("Invalid component name: {}", message)
                };
                syn::Error::new(*span, error_msg)
            }
            RsxError::InvalidAttribute {
                message,
                span,
                suggestion,
            } => {
                let error_msg = if let Some(suggestion) = suggestion {
                    format!(
                        "Invalid attribute: {}\n\nSuggestion: {}",
                        message, suggestion
                    )
                } else {
                    format!("Invalid attribute: {}", message)
                };
                syn::Error::new(*span, error_msg)
            }
            RsxError::MismatchedTags {
                message,
                span,
                opening_tag,
                closing_tag,
            } => {
                let error_msg = format!(
                    "Mismatched tags: {}\n\nOpening tag: <{}>\nClosing tag: </{}>\n\nTags must match exactly.",
                    message, opening_tag, closing_tag
                );
                syn::Error::new(*span, error_msg)
            }
            RsxError::SyntaxError {
                message,
                span,
                suggestion,
            } => {
                let error_msg = if let Some(suggestion) = suggestion {
                    format!("Syntax error: {}\n\nSuggestion: {}", message, suggestion)
                } else {
                    format!("Syntax error: {}", message)
                };
                syn::Error::new(*span, error_msg)
            }
            RsxError::ValidationError {
                message,
                span,
                details,
            } => {
                let error_msg = if let Some(details) = details {
                    format!("Validation error: {}\n\nDetails: {}", message, details)
                } else {
                    format!("Validation error: {}", message)
                };
                syn::Error::new(*span, error_msg)
            }
            RsxError::InternalError { message, context } => {
                let error_msg = format!(
                    "Internal error: {}\nContext: {}\n\nThis is likely a bug in the RSX macro. Please report it.",
                    message, context
                );
                syn::Error::new(proc_macro2::Span::call_site(), error_msg)
            }
            RsxError::ForLoopError {
                message,
                span,
                suggestion,
            } => {
                let error_msg = if let Some(suggestion) = suggestion {
                    format!("For-loop error: {}\n\nSuggestion: {}", message, suggestion)
                } else {
                    format!("For-loop error: {}", message)
                };
                syn::Error::new(*span, error_msg)
            }
        }
    }
}

// Result type for RSX operations
// pub type RsxResult<T> = Result<T, RsxError>;

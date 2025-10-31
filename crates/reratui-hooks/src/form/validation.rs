//! Validation utilities for form fields

use regex::Regex;
use std::sync::{Arc, OnceLock};

/// Validator for form fields
#[derive(Clone)]
pub struct Validator {
    #[allow(clippy::type_complexity)]
    validate_fn: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
}

impl Validator {
    /// Create a custom validator
    pub fn custom<F>(validate_fn: F) -> Self
    where
        F: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            validate_fn: Arc::new(validate_fn),
        }
    }

    /// Validate a value
    pub fn validate(&self, value: &str) -> Option<String> {
        (self.validate_fn)(value)
    }

    /// Required field validator
    pub fn required(message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.trim().is_empty() {
                Some(message.to_string())
            } else {
                None
            }
        })
    }

    /// Minimum length validator
    pub fn min_length(min: usize, message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.len() < min {
                Some(message.to_string())
            } else {
                None
            }
        })
    }

    /// Maximum length validator
    pub fn max_length(max: usize, message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.len() > max {
                Some(message.to_string())
            } else {
                None
            }
        })
    }

    /// Email format validator
    pub fn email(message: &'static str) -> Self {
        static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

        Self::custom(move |value| {
            let regex = EMAIL_REGEX.get_or_init(|| {
                Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
            });

            if value.is_empty() || regex.is_match(value) {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// URL format validator
    pub fn url(message: &'static str) -> Self {
        static URL_REGEX: OnceLock<Regex> = OnceLock::new();

        Self::custom(move |value| {
            let regex =
                URL_REGEX.get_or_init(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());

            if value.is_empty() || regex.is_match(value) {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// Numeric validator
    pub fn numeric(message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.is_empty() || value.parse::<f64>().is_ok() {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// Integer validator
    pub fn integer(message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.is_empty() || value.parse::<i64>().is_ok() {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// Pattern (regex) validator
    pub fn pattern(pattern: &'static str, message: &'static str) -> Self {
        Self::custom(move |value| {
            let regex = Regex::new(pattern).unwrap();
            if value.is_empty() || regex.is_match(value) {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// Minimum value validator (for numeric fields)
    pub fn min(min: f64, message: &'static str) -> Self {
        Self::custom(move |value| {
            if let Ok(num) = value.parse::<f64>() {
                if num < min {
                    Some(message.to_string())
                } else {
                    None
                }
            } else {
                None // Let numeric validator handle non-numeric values
            }
        })
    }

    /// Maximum value validator (for numeric fields)
    pub fn max(max: f64, message: &'static str) -> Self {
        Self::custom(move |value| {
            if let Ok(num) = value.parse::<f64>() {
                if num > max {
                    Some(message.to_string())
                } else {
                    None
                }
            } else {
                None // Let numeric validator handle non-numeric values
            }
        })
    }

    /// Range validator (for numeric fields)
    pub fn range(min: f64, max: f64, message: &'static str) -> Self {
        Self::custom(move |value| {
            if let Ok(num) = value.parse::<f64>() {
                if num < min || num > max {
                    Some(message.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Alphanumeric validator
    pub fn alphanumeric(message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.is_empty() || value.chars().all(|c| c.is_alphanumeric()) {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// Alpha (letters only) validator
    pub fn alpha(message: &'static str) -> Self {
        Self::custom(move |value| {
            if value.is_empty() || value.chars().all(|c| c.is_alphabetic()) {
                None
            } else {
                Some(message.to_string())
            }
        })
    }

    /// Matches another field validator
    pub fn matches(other_value: String, message: &'static str) -> Self {
        Self::custom(move |value| {
            if value == other_value {
                None
            } else {
                Some(message.to_string())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validator() {
        let validator = Validator::required("Field is required");

        assert!(validator.validate("").is_some());
        assert!(validator.validate("   ").is_some());
        assert!(validator.validate("value").is_none());
    }

    #[test]
    fn test_min_length_validator() {
        let validator = Validator::min_length(5, "Must be at least 5 characters");

        assert!(validator.validate("abc").is_some());
        assert!(validator.validate("abcde").is_none());
        assert!(validator.validate("abcdef").is_none());
    }

    #[test]
    fn test_email_validator() {
        let validator = Validator::email("Invalid email");

        assert!(validator.validate("invalid").is_some());
        assert!(validator.validate("test@").is_some());
        assert!(validator.validate("test@example.com").is_none());
        assert!(validator.validate("user.name+tag@example.co.uk").is_none());
    }

    #[test]
    fn test_numeric_validator() {
        let validator = Validator::numeric("Must be a number");

        assert!(validator.validate("abc").is_some());
        assert!(validator.validate("123").is_none());
        assert!(validator.validate("123.45").is_none());
        assert!(validator.validate("-123.45").is_none());
    }

    #[test]
    fn test_range_validator() {
        let validator = Validator::range(0.0, 100.0, "Must be between 0 and 100");

        assert!(validator.validate("-1").is_some());
        assert!(validator.validate("101").is_some());
        assert!(validator.validate("50").is_none());
        assert!(validator.validate("0").is_none());
        assert!(validator.validate("100").is_none());
    }
}

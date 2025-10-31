//! Type definitions for form management

use crate::state::{StateHandle, StateSetter};
use std::{collections::HashMap, sync::Arc};

use super::validation::Validator;

/// Configuration for form initialization
#[derive(Clone)]
pub struct FormConfig {
    /// Initial values for form fields
    pub(crate) initial_values: HashMap<String, String>,

    /// Validators for each field
    pub(crate) validators: HashMap<String, Vec<Validator>>,

    /// Callback when form is submitted
    pub(crate) on_submit: Arc<dyn Fn(HashMap<String, String>) + Send + Sync>,
}

impl FormConfig {
    /// Create a new form configuration builder
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use reratui::prelude::*;
    ///
    /// let config = FormConfig::builder()
    ///     .field("email", "")
    ///     .field("password", "")
    ///     .validate("email", vec![
    ///         Validator::required("Email is required"),
    ///         Validator::email("Invalid email"),
    ///     ])
    ///     .on_submit(|values| {
    ///         println!("Submitted: {:?}", values);
    ///     })
    ///     .build();
    /// ```
    pub fn builder() -> FormConfigBuilder {
        FormConfigBuilder::new()
    }
}

/// Builder for creating FormConfig with a fluent API
///
/// Provides a clean, type-safe way to construct form configurations
/// with method chaining.
pub struct FormConfigBuilder {
    initial_values: HashMap<String, String>,
    validators: HashMap<String, Vec<Validator>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Arc<dyn Fn(HashMap<String, String>) + Send + Sync>>,
}

impl FormConfigBuilder {
    /// Create a new form configuration builder
    pub fn new() -> Self {
        Self {
            initial_values: HashMap::new(),
            validators: HashMap::new(),
            on_submit: None,
        }
    }

    /// Add a field with an initial value
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reratui::prelude::*;
    /// FormConfig::builder()
    ///     .field("username", "john_doe")
    ///     .field("email", "john@example.com");
    /// ```
    pub fn field(mut self, name: impl Into<String>, initial_value: impl Into<String>) -> Self {
        self.initial_values
            .insert(name.into(), initial_value.into());
        self
    }

    /// Add multiple fields at once
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reratui::prelude::*;
    /// # use std::collections::HashMap;
    /// FormConfig::builder()
    ///     .fields(HashMap::from([
    ///         ("username".to_string(), "".to_string()),
    ///         ("email".to_string(), "".to_string()),
    ///     ]));
    /// ```
    pub fn fields(mut self, fields: HashMap<String, String>) -> Self {
        self.initial_values.extend(fields);
        self
    }

    /// Add validators for a specific field
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reratui::prelude::*;
    /// FormConfig::builder()
    ///     .field("email", "")
    ///     .validate("email", vec![
    ///         Validator::required("Email is required"),
    ///         Validator::email("Invalid email format"),
    ///     ]);
    /// ```
    pub fn validate(mut self, field: impl Into<String>, validators: Vec<Validator>) -> Self {
        self.validators.insert(field.into(), validators);
        self
    }

    /// Add a single validator for a field
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reratui::prelude::*;
    /// FormConfig::builder()
    ///     .field("username", "")
    ///     .validator("username", Validator::required("Username is required"))
    ///     .validator("username", Validator::min_length(3, "Min 3 characters"));
    /// ```
    pub fn validator(mut self, field: impl Into<String>, validator: Validator) -> Self {
        let field = field.into();
        self.validators.entry(field).or_default().push(validator);
        self
    }

    /// Set the submit handler
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use reratui::prelude::*;
    /// FormConfig::builder()
    ///     .on_submit(|values| {
    ///         println!("Form submitted with: {:?}", values);
    ///     });
    /// ```
    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: Fn(HashMap<String, String>) + Send + Sync + 'static,
    {
        self.on_submit = Some(Arc::new(handler));
        self
    }

    /// Build the final FormConfig
    ///
    /// # Panics
    ///
    /// Panics if `on_submit` was not set. Use `build_with_default_submit()` if you
    /// want a no-op submit handler.
    pub fn build(self) -> FormConfig {
        FormConfig {
            initial_values: self.initial_values,
            validators: self.validators,
            on_submit: self
                .on_submit
                .expect("on_submit handler must be set. Use build_with_default_submit() for a no-op handler."),
        }
    }

    /// Build the FormConfig with a default no-op submit handler
    ///
    /// Useful for testing or when submit handling is not needed.
    pub fn build_with_default_submit(self) -> FormConfig {
        FormConfig {
            initial_values: self.initial_values,
            validators: self.validators,
            on_submit: self.on_submit.unwrap_or_else(|| Arc::new(|_| {})),
        }
    }
}

impl Default for FormConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for interacting with form state
#[derive(Clone)]
pub struct FormHandle {
    pub(crate) values: StateHandle<HashMap<String, String>>,
    pub(crate) set_values: StateSetter<HashMap<String, String>>,
    pub(crate) errors: StateHandle<HashMap<String, String>>,
    pub(crate) set_errors: StateSetter<HashMap<String, String>>,
    pub(crate) touched: StateHandle<HashMap<String, bool>>,
    pub(crate) set_touched: StateSetter<HashMap<String, bool>>,
    pub(crate) is_submitting: StateHandle<bool>,
    pub(crate) set_is_submitting: StateSetter<bool>,
    pub(crate) is_valid: StateHandle<bool>,
    pub(crate) set_is_valid: StateSetter<bool>,
    pub(crate) validators: HashMap<String, Vec<Validator>>,
    pub(crate) on_submit: Arc<dyn Fn(HashMap<String, String>) + Send + Sync>,
}

impl Default for FormHandle {
    fn default() -> Self {
        Self {
            values: StateHandle::default(),
            set_values: StateSetter::default(),
            errors: StateHandle::default(),
            set_errors: StateSetter::default(),
            touched: StateHandle::default(),
            set_touched: StateSetter::default(),
            is_submitting: StateHandle::default(),
            set_is_submitting: StateSetter::default(),
            is_valid: StateHandle::default(),
            set_is_valid: StateSetter::default(),
            validators: HashMap::new(),
            on_submit: Arc::new(|_| {}),
        }
    }
}

impl FormHandle {
    /// Register a field and get its registration info
    pub fn register(&self, name: &str) -> FieldRegistration {
        FieldRegistration {
            name: name.to_string(),
            value: self.get_value(name).unwrap_or_default(),
            error: self.get_error(name),
            touched: self.is_touched(name),
        }
    }

    /// Get the current value of a field
    pub fn get_value(&self, name: &str) -> Option<String> {
        self.values.get().get(name).cloned()
    }

    /// Set the value of a field
    pub fn set_value(&self, name: &str, value: String) {
        let mut values = self.values.get();
        values.insert(name.to_string(), value.clone());
        self.set_values.set(values);

        // Validate field if it has been touched
        if self.is_touched(name) {
            self.validate_field(name, &value);
        }
    }

    /// Get the error message for a field
    pub fn get_error(&self, name: &str) -> Option<String> {
        self.errors.get().get(name).cloned()
    }

    /// Set the error for a field
    pub fn set_error(&self, name: &str, error: Option<String>) {
        let mut errors = self.errors.get();
        if let Some(err) = error {
            errors.insert(name.to_string(), err);
        } else {
            errors.remove(name);
        }
        self.set_errors.set(errors);
    }

    /// Check if a field has been touched
    pub fn is_touched(&self, name: &str) -> bool {
        self.touched.get().get(name).copied().unwrap_or(false)
    }

    /// Mark a field as touched
    pub fn set_touched(&self, name: &str, is_touched: bool) {
        let mut touched = self.touched.get();
        touched.insert(name.to_string(), is_touched);
        self.set_touched.set(touched);
    }

    /// Validate a specific field
    pub fn validate_field(&self, name: &str, value: &str) -> bool {
        if let Some(validators) = self.validators.get(name) {
            for validator in validators {
                if let Some(error) = validator.validate(value) {
                    self.set_error(name, Some(error));
                    return false;
                }
            }
        }
        self.set_error(name, None);
        true
    }

    /// Validate all fields in the form
    pub fn validate_all(&self) -> bool {
        let values = self.values.get();
        let mut all_valid = true;

        for (name, value) in values.iter() {
            if !self.validate_field(name, value) {
                all_valid = false;
            }
        }

        self.set_is_valid.set(all_valid);
        all_valid
    }

    /// Reset the form to initial values
    pub fn reset(&self, initial_values: HashMap<String, String>) {
        self.set_values.set(initial_values);
        self.set_errors.set(HashMap::new());
        self.set_touched.set(HashMap::new());
        self.set_is_submitting.set(false);
        self.set_is_valid.set(true);
    }

    /// Submit the form
    pub fn submit(&self) {
        // Mark all fields as touched
        let values = self.values.get();
        let mut touched = HashMap::new();
        for name in values.keys() {
            touched.insert(name.clone(), true);
        }
        self.set_touched.set(touched);

        // Validate all fields
        if self.validate_all() {
            self.set_is_submitting.set(true);
            (self.on_submit)(self.values.get());
            self.set_is_submitting.set(false);
        }
    }

    /// Check if the form is currently submitting
    pub fn is_submitting(&self) -> bool {
        self.is_submitting.get()
    }

    /// Check if the form is valid
    pub fn is_valid(&self) -> bool {
        self.is_valid.get()
    }

    /// Get all form values
    pub fn get_values(&self) -> HashMap<String, String> {
        self.values.get()
    }

    /// Get all form values (alias for get_values)
    pub fn get_all_values(&self) -> HashMap<String, String> {
        self.get_values()
    }

    /// Get all errors
    pub fn get_errors(&self) -> HashMap<String, String> {
        self.errors.get()
    }

    /// Check if the form has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.get().is_empty()
    }

    /// Check if any field is dirty (modified from initial value)
    pub fn is_dirty(&self) -> bool {
        !self.touched.get().is_empty()
    }
}

/// Field registration information
#[derive(Debug, Clone)]
pub struct FieldRegistration {
    /// Field name
    pub name: String,

    /// Current field value
    pub value: String,

    /// Current error message, if any
    pub error: Option<String>,

    /// Whether the field has been touched
    pub touched: bool,
}

impl FieldRegistration {
    /// Check if the field has an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the error message
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

//! Tests for form hook

use super::*;
use crate::test_utils::{with_component_id, with_test_isolate};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[test]
fn test_form_initialization() {
    with_test_isolate(|| {
        with_component_id("FormInit", |_| {
            let initial_values = HashMap::from([
                ("email".to_string(), "test@example.com".to_string()),
                ("password".to_string(), "password123".to_string()),
            ]);

            let form = use_form(FormConfig {
                initial_values: initial_values.clone(),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            assert_eq!(
                form.get_value("email"),
                Some("test@example.com".to_string())
            );
            assert_eq!(form.get_value("password"), Some("password123".to_string()));
            assert!(!form.is_submitting());
            assert!(form.is_valid());
            assert!(!form.has_errors());
            assert!(!form.is_dirty());
        });
    });
}

#[test]
fn test_form_set_value() {
    with_test_isolate(|| {
        with_component_id("FormSetValue", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("username".to_string(), "".to_string())]),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            assert_eq!(form.get_value("username"), Some("".to_string()));

            form.set_value("username", "john_doe".to_string());
            assert_eq!(form.get_value("username"), Some("john_doe".to_string()));
        });
    });
}

#[test]
fn test_form_touched_state() {
    with_test_isolate(|| {
        with_component_id("FormTouched", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("email".to_string(), "".to_string())]),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            assert!(!form.is_touched("email"));
            assert!(!form.is_dirty());

            form.set_touched("email", true);
            assert!(form.is_touched("email"));
            assert!(form.is_dirty());
        });
    });
}

#[test]
fn test_form_validation_required() {
    with_test_isolate(|| {
        with_component_id("FormValidationRequired", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("email".to_string(), "".to_string())]),
                validators: HashMap::from([(
                    "email".to_string(),
                    vec![Validator::required("Email is required")],
                )]),
                on_submit: Arc::new(|_| {}),
            });

            // Empty value should fail validation
            assert!(!form.validate_field("email", ""));
            assert_eq!(
                form.get_error("email"),
                Some("Email is required".to_string())
            );

            // Non-empty value should pass
            assert!(form.validate_field("email", "test@example.com"));
            assert_eq!(form.get_error("email"), None);
        });
    });
}

#[test]
fn test_form_validation_email() {
    with_test_isolate(|| {
        with_component_id("FormValidationEmail", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("email".to_string(), "".to_string())]),
                validators: HashMap::from([(
                    "email".to_string(),
                    vec![Validator::email("Invalid email format")],
                )]),
                on_submit: Arc::new(|_| {}),
            });

            // Invalid email should fail
            assert!(!form.validate_field("email", "invalid"));
            assert_eq!(
                form.get_error("email"),
                Some("Invalid email format".to_string())
            );

            // Valid email should pass
            assert!(form.validate_field("email", "test@example.com"));
            assert_eq!(form.get_error("email"), None);
        });
    });
}

#[test]
fn test_form_validation_min_length() {
    with_test_isolate(|| {
        with_component_id("FormValidationMinLength", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("password".to_string(), "".to_string())]),
                validators: HashMap::from([(
                    "password".to_string(),
                    vec![Validator::min_length(
                        8,
                        "Password must be at least 8 characters",
                    )],
                )]),
                on_submit: Arc::new(|_| {}),
            });

            // Too short should fail
            assert!(!form.validate_field("password", "short"));
            assert_eq!(
                form.get_error("password"),
                Some("Password must be at least 8 characters".to_string())
            );

            // Long enough should pass
            assert!(form.validate_field("password", "longenough"));
            assert_eq!(form.get_error("password"), None);
        });
    });
}

#[test]
fn test_form_validation_multiple_validators() {
    with_test_isolate(|| {
        with_component_id("FormValidationMultiple", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("email".to_string(), "".to_string())]),
                validators: HashMap::from([(
                    "email".to_string(),
                    vec![
                        Validator::required("Email is required"),
                        Validator::email("Invalid email format"),
                    ],
                )]),
                on_submit: Arc::new(|_| {}),
            });

            // Empty should fail required validator
            assert!(!form.validate_field("email", ""));
            assert_eq!(
                form.get_error("email"),
                Some("Email is required".to_string())
            );

            // Invalid format should fail email validator
            assert!(!form.validate_field("email", "invalid"));
            assert_eq!(
                form.get_error("email"),
                Some("Invalid email format".to_string())
            );

            // Valid email should pass all validators
            assert!(form.validate_field("email", "test@example.com"));
            assert_eq!(form.get_error("email"), None);
        });
    });
}

#[test]
fn test_form_validate_all() {
    with_test_isolate(|| {
        with_component_id("FormValidateAll", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([
                    ("email".to_string(), "".to_string()),
                    ("password".to_string(), "".to_string()),
                ]),
                validators: HashMap::from([
                    (
                        "email".to_string(),
                        vec![Validator::required("Email is required")],
                    ),
                    (
                        "password".to_string(),
                        vec![Validator::required("Password is required")],
                    ),
                ]),
                on_submit: Arc::new(|_| {}),
            });

            // Both fields empty should fail
            assert!(!form.validate_all());
            assert!(!form.is_valid());
            assert!(form.has_errors());

            // Set one field
            form.set_value("email", "test@example.com".to_string());
            form.set_touched("email", true);
            form.validate_field("email", "test@example.com");

            // Still should fail because password is empty
            assert!(!form.validate_all());

            // Set both fields
            form.set_value("password", "password123".to_string());
            form.set_touched("password", true);
            form.validate_field("password", "password123");

            // Now should pass
            assert!(form.validate_all());
            assert!(form.is_valid());
            assert!(!form.has_errors());
        });
    });
}

#[test]
fn test_form_submit() {
    with_test_isolate(|| {
        with_component_id("FormSubmit", |_| {
            let submitted = Arc::new(AtomicBool::new(false));
            let submitted_clone = submitted.clone();

            let form = use_form(FormConfig {
                initial_values: HashMap::from([
                    ("email".to_string(), "test@example.com".to_string()),
                    ("password".to_string(), "password123".to_string()),
                ]),
                validators: HashMap::from([
                    (
                        "email".to_string(),
                        vec![Validator::required("Email is required")],
                    ),
                    (
                        "password".to_string(),
                        vec![Validator::required("Password is required")],
                    ),
                ]),
                on_submit: Arc::new(move |values| {
                    submitted_clone.store(true, Ordering::SeqCst);
                    assert_eq!(values.get("email"), Some(&"test@example.com".to_string()));
                    assert_eq!(values.get("password"), Some(&"password123".to_string()));
                }),
            });

            // Submit should validate and call on_submit
            form.submit();
            assert!(submitted.load(Ordering::SeqCst));
            assert!(form.is_touched("email"));
            assert!(form.is_touched("password"));
        });
    });
}

#[test]
fn test_form_submit_invalid() {
    with_test_isolate(|| {
        with_component_id("FormSubmitInvalid", |_| {
            let submitted = Arc::new(AtomicBool::new(false));
            let submitted_clone = submitted.clone();

            let form = use_form(FormConfig {
                initial_values: HashMap::from([("email".to_string(), "".to_string())]),
                validators: HashMap::from([(
                    "email".to_string(),
                    vec![Validator::required("Email is required")],
                )]),
                on_submit: Arc::new(move |_| {
                    submitted_clone.store(true, Ordering::SeqCst);
                }),
            });

            // Submit with invalid data should not call on_submit
            form.submit();
            assert!(!submitted.load(Ordering::SeqCst));
            assert!(form.is_touched("email"));
            assert!(form.has_errors());
        });
    });
}

#[test]
fn test_form_reset() {
    with_test_isolate(|| {
        with_component_id("FormReset", |_| {
            let initial_values = HashMap::from([
                ("email".to_string(), "initial@example.com".to_string()),
                ("password".to_string(), "initial123".to_string()),
            ]);

            let form = use_form(FormConfig {
                initial_values: initial_values.clone(),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            // Modify form
            form.set_value("email", "changed@example.com".to_string());
            form.set_touched("email", true);
            form.set_error("email", Some("Some error".to_string()));

            assert_eq!(
                form.get_value("email"),
                Some("changed@example.com".to_string())
            );
            assert!(form.is_touched("email"));
            assert!(form.has_errors());

            // Reset form
            form.reset(initial_values.clone());

            assert_eq!(
                form.get_value("email"),
                Some("initial@example.com".to_string())
            );
            assert!(!form.is_touched("email"));
            assert!(!form.has_errors());
            assert!(!form.is_submitting());
            assert!(form.is_valid());
        });
    });
}

#[test]
fn test_form_register_field() {
    with_test_isolate(|| {
        with_component_id("FormRegister", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("username".to_string(), "john".to_string())]),
                validators: HashMap::from([(
                    "username".to_string(),
                    vec![Validator::required("Username is required")],
                )]),
                on_submit: Arc::new(|_| {}),
            });

            let registration = form.register("username");

            assert_eq!(registration.name, "username");
            assert_eq!(registration.value, "john");
            assert!(!registration.has_error());
            assert!(!registration.touched);
        });
    });
}

#[test]
fn test_field_registration_with_error() {
    let registration = FieldRegistration {
        name: "email".to_string(),
        value: "invalid".to_string(),
        error: Some("Invalid email format".to_string()),
        touched: true,
    };

    assert!(registration.has_error());
    assert_eq!(registration.error_message(), Some("Invalid email format"));
}

#[test]
fn test_field_registration_without_error() {
    let registration = FieldRegistration {
        name: "email".to_string(),
        value: "test@example.com".to_string(),
        error: None,
        touched: false,
    };

    assert!(!registration.has_error());
    assert!(registration.error_message().is_none());
}

#[test]
fn test_form_get_values() {
    with_test_isolate(|| {
        with_component_id("FormGetValues", |_| {
            let initial_values = HashMap::from([
                ("email".to_string(), "test@example.com".to_string()),
                ("password".to_string(), "password123".to_string()),
            ]);

            let form = use_form(FormConfig {
                initial_values: initial_values.clone(),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            let values = form.get_values();
            assert_eq!(values.len(), 2);
            assert_eq!(values.get("email"), Some(&"test@example.com".to_string()));
            assert_eq!(values.get("password"), Some(&"password123".to_string()));
        });
    });
}

#[test]
fn test_form_get_errors() {
    with_test_isolate(|| {
        with_component_id("FormGetErrors", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("email".to_string(), "".to_string())]),
                validators: HashMap::from([(
                    "email".to_string(),
                    vec![Validator::required("Email is required")],
                )]),
                on_submit: Arc::new(|_| {}),
            });

            form.validate_field("email", "");
            let errors = form.get_errors();

            assert_eq!(errors.len(), 1);
            assert_eq!(errors.get("email"), Some(&"Email is required".to_string()));
        });
    });
}

#[test]
fn test_form_provider_and_context() {
    with_test_isolate(|| {
        with_component_id("FormProvider", |_| {
            // Create form with provider
            let form = use_form(FormConfig {
                initial_values: HashMap::from([
                    ("username".to_string(), "john".to_string()),
                    ("email".to_string(), "john@example.com".to_string()),
                ]),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            // Verify provider returns the form
            assert_eq!(form.get_value("username"), Some("john".to_string()));
            assert_eq!(
                form.get_value("email"),
                Some("john@example.com".to_string())
            );
        });

        // Simulate child component accessing context
        with_component_id("FormConsumer", |_| {
            let form = use_form_context();

            // Should have access to the same form state
            assert_eq!(form.get_value("username"), Some("john".to_string()));
            assert_eq!(
                form.get_value("email"),
                Some("john@example.com".to_string())
            );

            // Modify form through context
            form.set_value("username", "jane".to_string());
            assert_eq!(form.get_value("username"), Some("jane".to_string()));
        });
    });
}

#[test]
fn test_form_context_shared_state() {
    with_test_isolate(|| {
        with_component_id("FormProviderShared", |_| {
            let form = use_form(FormConfig {
                initial_values: HashMap::from([("count".to_string(), "0".to_string())]),
                validators: HashMap::new(),
                on_submit: Arc::new(|_| {}),
            });

            form.set_value("count", "42".to_string());
            assert_eq!(form.get_value("count"), Some("42".to_string()));
        });

        // Access from another component
        with_component_id("FormConsumerShared", |_| {
            let form = use_form_context();

            // Should see the updated value
            assert_eq!(form.get_value("count"), Some("42".to_string()));

            // Update again
            form.set_value("count", "100".to_string());
        });

        // Access from yet another component
        with_component_id("FormConsumerShared2", |_| {
            let form = use_form_context();

            // Should see the latest value
            assert_eq!(form.get_value("count"), Some("100".to_string()));
        });
    });
}

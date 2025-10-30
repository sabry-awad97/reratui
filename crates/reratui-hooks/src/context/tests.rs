use crate::{
    context::{use_context, use_context_provider},
    test_utils::with_component_id,
};

#[derive(Clone, Debug, PartialEq)]
struct TestUser {
    name: String,
    role: String,
}

#[derive(Clone, Debug, PartialEq)]
struct TestTheme {
    color: String,
    font: String,
}

#[test]
fn test_context_provider_and_use() {
    // Simulate a component render and provide a context value
    with_component_id("ContextProviderComponent", |_ctx| {
        // Provide a user context
        let user = use_context_provider(|| TestUser {
            name: "Test User".to_string(),
            role: "Admin".to_string(),
        });

        // The returned value should be the same as the provided value
        assert_eq!(
            user,
            TestUser {
                name: "Test User".to_string(),
                role: "Admin".to_string(),
            }
        );

        // Simulate a child component render
        with_component_id("ContextConsumerComponent", |_ctx| {
            // Get the user from context
            let user = use_context::<TestUser>();

            // The user should be the same as the provided value
            assert_eq!(
                user,
                TestUser {
                    name: "Test User".to_string(),
                    role: "Admin".to_string(),
                }
            );
        });
    });
}

#[test]
fn test_multiple_context_types() {
    // Simulate a component render and provide multiple context values
    with_component_id("ContextProviderComponent", |_ctx| {
        // Provide a user context
        let user = use_context_provider(|| TestUser {
            name: "Test User".to_string(),
            role: "Admin".to_string(),
        });

        // Provide a theme context
        let theme = use_context_provider(|| TestTheme {
            color: "Dark".to_string(),
            font: "Sans".to_string(),
        });

        // The returned values should be the same as the provided values
        assert_eq!(
            user,
            TestUser {
                name: "Test User".to_string(),
                role: "Admin".to_string(),
            }
        );
        assert_eq!(
            theme,
            TestTheme {
                color: "Dark".to_string(),
                font: "Sans".to_string(),
            }
        );

        // Simulate a child component render
        with_component_id("ContextConsumerComponent", |_ctx| {
            // Get the user from context
            let user = use_context::<TestUser>();
            // Get the theme from context
            let theme = use_context::<TestTheme>();

            // The values should be the same as the provided values
            assert_eq!(
                user,
                TestUser {
                    name: "Test User".to_string(),
                    role: "Admin".to_string(),
                }
            );
            assert_eq!(
                theme,
                TestTheme {
                    color: "Dark".to_string(),
                    font: "Sans".to_string(),
                }
            );
        });
    });
}

#[test]
fn test_nested_context_providers() {
    // Test that context providers work in a nested component hierarchy
    // This test simulates how the API would be used in practice

    // First, let's test the outer component
    with_component_id("ContextProviderComponent", |_ctx| {
        // Provide an outer theme context
        let outer_theme = use_context_provider(|| TestTheme {
            color: "Light".to_string(),
            font: "Serif".to_string(),
        });

        assert_eq!(
            outer_theme,
            TestTheme {
                color: "Light".to_string(),
                font: "Serif".to_string(),
            }
        );

        // Verify we can read the context we just provided
        let theme = use_context::<TestTheme>();
        assert_eq!(
            theme,
            TestTheme {
                color: "Light".to_string(),
                font: "Serif".to_string(),
            }
        );
    });

    // Now, let's test a child component that overrides the context
    with_component_id("ContextProviderComponent", |_ctx| {
        // Provide a different theme context
        let inner_theme = use_context_provider(|| TestTheme {
            color: "Dark".to_string(),
            font: "Sans".to_string(),
        });

        assert_eq!(
            inner_theme,
            TestTheme {
                color: "Dark".to_string(),
                font: "Sans".to_string(),
            }
        );

        // Verify we can read the context we just provided
        let theme = use_context::<TestTheme>();
        assert_eq!(
            theme,
            TestTheme {
                color: "Dark".to_string(),
                font: "Sans".to_string(),
            }
        );
    });

    // Finally, let's test a grandchild component
    with_component_id("ContextProviderComponent", |_ctx| {
        // Provide yet another theme context
        let grandchild_theme = use_context_provider(|| TestTheme {
            color: "Blue".to_string(),
            font: "Monospace".to_string(),
        });

        assert_eq!(
            grandchild_theme,
            TestTheme {
                color: "Blue".to_string(),
                font: "Monospace".to_string(),
            }
        );

        // Verify we can read the context we just provided
        let theme = use_context::<TestTheme>();
        assert_eq!(
            theme,
            TestTheme {
                color: "Blue".to_string(),
                font: "Monospace".to_string(),
            }
        );
    });
}

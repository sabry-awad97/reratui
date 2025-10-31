//! Form Context Example - React Hook Form Style
//!
//! Demonstrates using form context to share form state across components
//! without prop drilling, similar to React Hook Form's FormProvider.
//!
//! Features:
//! - FormProvider for sharing form state
//! - useFormContext for accessing form in child components
//! - Reusable form field components
//! - Clean component composition
//!
//! Controls:
//! - Tab/Shift+Tab to navigate between fields
//! - Type to enter values
//! - Enter to submit form
//! - Press 'q' to exit

use reratui::prelude::*;

mod form;
mod form_description;
mod form_field;
mod form_input;
mod form_item;
mod form_label;
mod form_message;

use form::*;
use form_description::*;
use form_field::*;
use form_input::*;
use form_item::*;
use form_label::*;
use form_message::*;

/// Main application component
#[component]
fn App() -> Element {
    let frame = use_frame();

    use_keyboard_shortcut(KeyCode::Char('q'), KeyModifiers::NONE, || {
        request_exit();
    });

    // Animated title
    let pulse = ((frame.count as f32 / 10.0).sin() * 0.5 + 0.5) * 255.0;
    let title_color = Color::Rgb((59.0 + pulse * 0.3) as u8, (130.0 + pulse * 0.2) as u8, 246);

    // Create form handle
    let form = use_form(
        FormConfig::builder()
            .field("username", "")
            .field("email", "")
            .field("phone", "")
            .field("password", "")
            .validator("username", Validator::required("Username is required"))
            .validator(
                "username",
                Validator::min_length(3, "Username must be at least 3 characters"),
            )
            .validator("email", Validator::required("Email is required"))
            .validator("email", Validator::email("Invalid email format"))
            .validator("password", Validator::required("Password is required"))
            .validator(
                "password",
                Validator::min_length(8, "Password must be at least 8 characters"),
            )
            .on_submit(|values| {
                println!("✅ Form submitted!: {values:?}");
            })
            .build(),
    );

    rsx! {
        <Block
            title={"📝 Form Context Example"}
            title_style={Style::default().fg(title_color).add_modifier(Modifier::BOLD)}
            borders={Borders::ALL}
            border_style={Style::default().fg(Color::Cyan)}
            style={Style::default().bg(Color::Rgb(17, 17, 27))}
        >
            <Layout
                direction={Direction::Vertical}
                constraints={vec![
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]}
            >
                // Header
                <Block
                    borders={Borders::BOTTOM}
                    border_style={Style::default().fg(Color::DarkGray)}
                    style={Style::default().bg(Color::Rgb(24, 24, 37))}
                >
                    <Paragraph
                        style={Style::default().fg(Color::Gray)}
                        alignment={Alignment::Center}
                    >
                        {"Using Form component with context - No prop drilling!"}
                    </Paragraph>
                </Block>

                // Form component provides context to children
                <Form form={form}>
                    <FormField
                        name={"username"}
                        render={|ctx: FormFieldContext| {
                            rsx! {
                                <FormItem>
                                    <FormLabel text={"Username".to_string()} required={true} />
                                    <Input
                                        value={ctx.value.clone()}
                                        placeholder={"Enter username (min 3 chars)".to_string()}
                                        variant={InputVariant::Outlined}
                                        icon={"👤".to_string()}
                                        focused={ctx.is_focused}
                                        error={ctx.error.is_some() && ctx.touched}
                                    />
                                    <FormFieldMessage />
                                </FormItem>
                            }
                        }}
                    />
                    <FormField
                        name={"email"}
                        render={|ctx: FormFieldContext| {
                            rsx! {
                                <FormItem>
                                    <FormLabel text={"Email".to_string()} required={true} />
                                    <Input
                                        value={ctx.value.clone()}
                                        placeholder={"Enter email address".to_string()}
                                        variant={InputVariant::Outlined}
                                        icon={"📧".to_string()}
                                        focused={ctx.is_focused}
                                        error={ctx.error.is_some() && ctx.touched}
                                    />
                                    <FormDescription text={"We'll never share your email with anyone.".to_string()} />
                                    <FormFieldMessage />
                                </FormItem>
                            }
                        }}
                    />
                    <FormField
                        name={"phone"}
                        render={|ctx: FormFieldContext| {
                            rsx! {
                                <FormItem>
                                    <FormLabel text={"Phone".to_string()} />
                                    <Input
                                        value={ctx.value.clone()}
                                        placeholder={"Enter phone number".to_string()}
                                        variant={InputVariant::Outlined}
                                        icon={"📱".to_string()}
                                        focused={ctx.is_focused}
                                        error={ctx.error.is_some() && ctx.touched}
                                    />
                                    <FormFieldMessage />
                                </FormItem>
                            }
                        }}
                    />
                    <FormField
                        name={"password"}
                        render={|ctx: FormFieldContext| {
                            rsx! {
                                <FormItem>
                                    <FormLabel text={"Password".to_string()} required={true} />
                                    <Input
                                        value={ctx.value.clone()}
                                        placeholder={"Enter password (min 8 chars)".to_string()}
                                        variant={InputVariant::Outlined}
                                        password={true}
                                        icon={"🔒".to_string()}
                                        focused={ctx.is_focused}
                                        error={ctx.error.is_some() && ctx.touched}
                                    />
                                    <FormFieldMessage />
                                </FormItem>
                            }
                        }}
                    />
                </Form>

                // Footer with submit button
                <FormSubmitButton />
            </Layout>
        </Block>
    }
}

/// Submit button component - uses form context
#[component]
fn FormSubmitButton() -> Element {
    // Access form from context
    let form = use_form_context();

    rsx! {
        <Block
            borders={Borders::TOP}
            border_style={Style::default().fg(Color::DarkGray)}
            style={Style::default().bg(Color::Rgb(24, 24, 37))}
        >
            <Paragraph
                style={Style::default()
                    .fg(if form.is_valid() { Color::Green } else { Color::Yellow })
                    .add_modifier(Modifier::BOLD)}
                alignment={Alignment::Center}
            >
                {if form.is_valid() {
                    "✓ Form is valid - Press Enter to submit"
                } else if form.has_errors() {
                    "⚠ Please fix errors before submitting"
                } else {
                    "Fill out all required fields"
                }}
            </Paragraph>
        </Block>
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    render(|| {
        rsx! { <App /> }
    })
    .await?;
    Ok(())
}

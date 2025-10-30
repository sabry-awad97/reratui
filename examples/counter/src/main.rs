//! Counter Example with RSX and use_state Hook
//!
//! A simple counter application demonstrating the Reratui framework with:
//! - RSX syntax for declarative UI
//! - use_state hook for reactive state management
//! - Automatic re-rendering on state changes
//!
//! Press any key to increment the counter.
//! Press Esc or Ctrl+C to exit.

use reratui::prelude::*;

#[component(Counter)]
fn counter() -> Element {
    let (count, set_count) = use_state(|| 0);

    rsx! {
        <Block title="Counter" borders={Borders::ALL}>
            <Paragraph>
                "Count: {count}"
            </Paragraph>
            <Button on_click={move |_| set_count(count + 1)}>
                "Increment"
            </Button>
        </Block>
    }
}

#[reratui::main]
async fn main() -> Result<()> {
    render(|| {
        rsx! { <Counter /> }
    })
    .await;
    Ok(())
}

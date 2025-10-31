use reratui::prelude::*;

mod app;
mod components;
mod hooks;
mod models;
mod theme;
mod utils;

use app::CommandPaletteApp;

/// Entry point for the application
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create app with CommandPaletteApp component
    render(|| CommandPaletteApp::new("✨ Enhanced Command Palette Demo ✨").into()).await?;
    Ok(())
}

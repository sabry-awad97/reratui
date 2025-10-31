use reratui::prelude::*;

mod app;
mod components;
mod hooks;
mod models;
mod theme;
mod utils;

use app::CommandPaletteApp;

/// Entry point for the application
#[reratui::main]
async fn main() -> Result<()> {
    // Create app with CommandPaletteApp component
    render(|| CommandPaletteApp::new("✨ Enhanced Command Palette Demo ✨").into()).await?;
    Ok(())
}

mod hud;

use eframe::egui;
use hud::{HUD_DECORATIONS, HUD_HEIGHT, HUD_TITLE, HUD_TRANSPARENT, HUD_WIDTH};

fn main() -> eframe::Result<()> {
    // Configure the HUD window.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([HUD_WIDTH, HUD_HEIGHT])
            .with_decorations(HUD_DECORATIONS) // Remove the native title bar and borders.
            .with_transparent(HUD_TRANSPARENT) // Allow the desktop to show through.
            .with_always_on_top(), // Keep the HUD above other windows.
        ..Default::default()
    };

    // Start the application.
    eframe::run_native(
        HUD_TITLE,
        options,
        Box::new(|_cc| Ok(Box::new(hud::TaskerApp))),
    )
}

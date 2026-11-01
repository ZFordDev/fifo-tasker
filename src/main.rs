mod dashboard;
mod hud;

use eframe::egui;
use hud::{HUD_DECORATIONS, HUD_HEIGHT, HUD_TITLE, HUD_TRANSPARENT, HUD_WIDTH};

fn main() -> eframe::Result<()> {
    if std::env::args().any(|arg| arg == "--dashboard") {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([dashboard::DASHBOARD_WIDTH, dashboard::DASHBOARD_HEIGHT])
                .with_decorations(dashboard::DASHBOARD_DECORATIONS) // Let the OS wear the frame.
                .with_transparent(dashboard::DASHBOARD_TRANSPARENT),
            ..Default::default()
        };
        return eframe::run_native(
            dashboard::DASHBOARD_TITLE,
            options,
            Box::new(|_cc| Ok(Box::new(dashboard::DashboardApp))),
        );
    }

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

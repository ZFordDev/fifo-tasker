mod config;
mod frame;

use eframe::egui;

use config::{CONTENT_TOP_SPACING, PLACEHOLDER_TEXT};
// Expose only the settings needed to launch the HUD.
pub(crate) use config::{HUD_DECORATIONS, HUD_HEIGHT, HUD_TITLE, HUD_TRANSPARENT, HUD_WIDTH};

pub(crate) struct TaskerApp;

impl eframe::App for TaskerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        frame::show(ui);

        // Temporary content while developing the HUD.
        ui.vertical_centered(|ui| {
            ui.add_space(CONTENT_TOP_SPACING);
            ui.heading(HUD_TITLE);
            ui.label(PLACEHOLDER_TEXT);
        });
    }
}

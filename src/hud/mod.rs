mod config;
mod frame;

use crate::logic::Command;
use eframe::egui;

use config::{CONTENT_TOP_SPACING, PLACEHOLDER_TEXT};
// Expose only the settings needed to launch the HUD.
pub(crate) use config::{HUD_DECORATIONS, HUD_HEIGHT, HUD_TITLE, HUD_TRANSPARENT, HUD_WIDTH};

pub(crate) struct TaskerApp;

impl TaskerApp {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        frame::show(ui);

        // Temporary content while developing the HUD.
        ui.vertical_centered(|ui| {
            ui.add_space(CONTENT_TOP_SPACING);
            ui.heading(HUD_TITLE);
            ui.label(PLACEHOLDER_TEXT);
            if ui
                .button("Close HUD")
                .on_hover_text("Keep FIFO Tasker running in the tray")
                .clicked()
            {
                return Some(Command::CloseHud); // Clocking off the window, not the app.
            }
            // Temporary doorway until the tray takes over.
            ui.button("Open Dashboard")
                .clicked()
                .then_some(Command::OpenDashboard)
        })
        .inner
    }
}

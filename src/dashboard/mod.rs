mod config;
mod frame;

use crate::logic::Command;
use eframe::egui;

use config::{CONTENT_TOP_SPACING, PLACEHOLDER_TEXT};
// Expose only the settings needed to launch the dashboard.
pub(crate) use config::{
    DASHBOARD_DECORATIONS, DASHBOARD_HEIGHT, DASHBOARD_TITLE, DASHBOARD_TRANSPARENT,
    DASHBOARD_WIDTH,
};

pub(crate) struct DashboardApp;

impl DashboardApp {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        frame::show(ui);

        // Temporary content while developing the dashboard.
        ui.vertical_centered(|ui| {
            ui.add_space(CONTENT_TOP_SPACING);
            ui.heading(DASHBOARD_TITLE);
            ui.label(PLACEHOLDER_TEXT);
            // Temporary doorway until the tray takes over.
            ui.button("Open HUD").clicked().then_some(Command::OpenHud)
        })
        .inner
    }
}

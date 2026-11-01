mod config;
mod frame;

use eframe::egui;

use config::{CONTENT_TOP_SPACING, PLACEHOLDER_TEXT};
// Expose only the settings needed to launch the dashboard.
pub(crate) use config::{
    DASHBOARD_DECORATIONS, DASHBOARD_HEIGHT, DASHBOARD_TITLE, DASHBOARD_TRANSPARENT,
    DASHBOARD_WIDTH,
};

pub(crate) struct DashboardApp;

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        frame::show(ui);

        // Temporary content while developing the dashboard.
        ui.vertical_centered(|ui| {
            ui.add_space(CONTENT_TOP_SPACING);
            ui.heading(DASHBOARD_TITLE);
            ui.label(PLACEHOLDER_TEXT);
        });
    }
}

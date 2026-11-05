mod config;
mod frame;

use crate::{
    logic::{Command, tasks::TaskData},
    task_list,
};
use eframe::egui;

use config::CONTENT_TOP_SPACING;
// Expose only the settings needed to launch the dashboard.
pub(crate) use config::{
    DASHBOARD_DECORATIONS, DASHBOARD_HEIGHT, DASHBOARD_TITLE, DASHBOARD_TRANSPARENT,
    DASHBOARD_WIDTH,
};

pub(crate) struct DashboardApp;

impl DashboardApp {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, data: &TaskData) -> Option<Command> {
        frame::show(ui);

        let command = ui
            .vertical_centered(|ui| {
                ui.add_space(CONTENT_TOP_SPACING);
                ui.heading(DASHBOARD_TITLE);
                // Temporary doorway until the tray takes over.
                ui.button("Open HUD").clicked().then_some(Command::OpenHud)
            })
            .inner;
        task_list::show(ui, data);
        command
    }
}

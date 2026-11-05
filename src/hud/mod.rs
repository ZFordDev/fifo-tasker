mod config;
mod frame;

use crate::{
    logic::{Command, tasks::TaskData},
    task_list,
};
use eframe::egui;

use config::CONTENT_TOP_SPACING;
// Expose only the settings needed to launch the HUD.
pub(crate) use config::{HUD_DECORATIONS, HUD_HEIGHT, HUD_TITLE, HUD_TRANSPARENT, HUD_WIDTH};

pub(crate) struct TaskerApp;

impl TaskerApp {
    pub(crate) fn show(&mut self, ui: &mut egui::Ui, data: &TaskData) -> Option<Command> {
        frame::show(ui);

        let command = ui
            .vertical_centered(|ui| {
                ui.add_space(CONTENT_TOP_SPACING);
                ui.heading(HUD_TITLE);
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
            .inner;
        task_list::show(ui, data);
        command
    }
}

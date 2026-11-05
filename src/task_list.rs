use eframe::egui;

use crate::logic::tasks::TaskData;

// Both windows read the same queue. Neither gets to invent its own running order.
pub(crate) fn show(ui: &mut egui::Ui, data: &TaskData) {
    egui::Frame::new().inner_margin(16.0).show(ui, |ui| {
        if let Some(error) = &data.load_error {
            ui.colored_label(ui.visuals().error_fg_color, "Could not load task data.");
            ui.label(error);
            if data.document.tasks().is_empty() {
                return; // A failed load isn't an empty queue we can confidently report.
            }
        }
        if data.document.tasks().is_empty() {
            ui.label("No tasks yet.");
            return;
        }
        ui.label(format!("Tasks: {}", data.document.tasks().len()));
        egui::ScrollArea::vertical()
            .id_salt(("task-list", ui.ctx().viewport_id()))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for task in data.document.tasks() {
                    ui.push_id(&task.id, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(if task.completed { "Done" } else { "Queued" });
                            let title = egui::RichText::new(&task.title);
                            ui.label(if task.completed {
                                title.strikethrough()
                            } else {
                                title
                            });
                        });
                        ui.separator();
                    });
                }
            });
    });
}

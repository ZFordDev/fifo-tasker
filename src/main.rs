// We need to move this next outside main into a HUD folder, but for now it's here to get the HUD working.
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Configure the HUD window.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([340.0, 420.0])
            .with_decorations(false) // Remove the native title bar and borders.
            .with_transparent(true) // Allow the desktop to show through.
            .with_always_on_top(), // Keep the HUD above other windows.
        ..Default::default()
    };

    // Start the application.
    eframe::run_native(
        "FIFO Tasker",
        options,
        Box::new(|_cc| Ok(Box::new(TaskerApp))),
    )
}

struct TaskerApp;

impl eframe::App for TaskerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let rect = ui.max_rect();

        // Draw the slightly lighter, translucent workspace.
        ui.painter().rect_filled(
            rect.shrink(8.0),
            12.0,
            egui::Color32::from_rgba_unmultiplied(28, 37, 52, 200),
        );

        // Draw the dark blue outer border.
        ui.painter().rect_stroke(
            rect.shrink(4.0),
            12.0,
            egui::Stroke::new(
                8.0,
                egui::Color32::from_rgba_unmultiplied(17, 25, 40, 200),
            ),
            egui::StrokeKind::Inside,
        );

        // Grab a straight edge to move the HUD. The corners are saving themselves for resizing.
        let border = rect.shrink(4.0);
        let edges = [
            egui::Rect::from_min_max(
                border.min + egui::vec2(16.0, 0.0),
                border.right_top() + egui::vec2(-16.0, 8.0),
            ),
            egui::Rect::from_min_max(
                border.left_bottom() + egui::vec2(16.0, -8.0),
                border.max - egui::vec2(16.0, 0.0),
            ),
            egui::Rect::from_min_max(
                border.min + egui::vec2(0.0, 16.0),
                border.left_bottom() + egui::vec2(8.0, -16.0),
            ),
            egui::Rect::from_min_max(
                border.right_top() + egui::vec2(-8.0, 16.0),
                border.max - egui::vec2(0.0, 16.0),
            ),
        ];
        for (index, edge) in edges.into_iter().enumerate() {
            if ui
                .interact(edge, ui.scope_id().with(index), egui::Sense::drag())
                .drag_started_by(egui::PointerButton::Primary)
            {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
        }

        // Temporary content while developing the HUD.
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("FIFO Tasker");
            ui.label("Hello, FIFO Tasker!");
        });
    }
}
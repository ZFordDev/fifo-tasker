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

        // Temporary content while developing the HUD.
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("FIFO Tasker");
            ui.label("Hello, FIFO Tasker!");
        });
    }
}
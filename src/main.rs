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

        // Draw a translucent background with rounded corners.
        ui.painter().rect_filled(
            rect.shrink(1.0),
            12.0, // Corner radius.
            egui::Color32::from_rgba_unmultiplied(32, 33, 36, 200), // RGBA colour.
        );

        // Temporary content while developing the HUD.
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("FIFO Tasker");
            ui.label("Hello, FIFO Tasker!");
        });
    }
}
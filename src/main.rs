// We need to move this next outside main into a HUD folder, but for now it's here to get the HUD working.
use eframe::egui;

// HUD appearance and layout.
const HUD_WIDTH: f32 = 340.0;
const HUD_HEIGHT: f32 = 420.0;
const BORDER_WIDTH: f32 = 8.0;
const CORNER_RADIUS: f32 = 12.0;
const RESIZE_CORNER_SIZE: f32 = 16.0;

fn main() -> eframe::Result<()> {
    // Configure the HUD window.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([HUD_WIDTH, HUD_HEIGHT])
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
            rect.shrink(BORDER_WIDTH),
            CORNER_RADIUS,
            egui::Color32::from_rgba_unmultiplied(28, 37, 52, 200),
        );

        // Draw the dark blue outer border.
        ui.painter().rect_stroke(
            rect.shrink(BORDER_WIDTH / 2.0),
            CORNER_RADIUS,
            egui::Stroke::new(
                BORDER_WIDTH,
                egui::Color32::from_rgba_unmultiplied(17, 25, 40, 200),
            ),
            egui::StrokeKind::Inside,
        );

        // Grab a straight edge to move the HUD. The corners are saving themselves for resizing.
        let border = rect.shrink(BORDER_WIDTH / 2.0);
        let edges = [
            // Top edge.
            egui::Rect::from_min_max(
                border.min + egui::vec2(RESIZE_CORNER_SIZE, 0.0),
                border.right_top() + egui::vec2(-RESIZE_CORNER_SIZE, BORDER_WIDTH),
            ),
            // Bottom edge.
            egui::Rect::from_min_max(
                border.left_bottom() + egui::vec2(RESIZE_CORNER_SIZE, -BORDER_WIDTH),
                border.max - egui::vec2(RESIZE_CORNER_SIZE, 0.0),
            ),
            // Left edge.
            egui::Rect::from_min_max(
                border.min + egui::vec2(0.0, RESIZE_CORNER_SIZE),
                border.left_bottom() + egui::vec2(BORDER_WIDTH, -RESIZE_CORNER_SIZE),
            ),
            // Right edge.
            egui::Rect::from_min_max(
                border.right_top() + egui::vec2(-BORDER_WIDTH, RESIZE_CORNER_SIZE),
                border.max - egui::vec2(0.0, RESIZE_CORNER_SIZE),
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

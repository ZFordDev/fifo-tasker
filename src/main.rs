// We need to move this next outside main into a HUD folder, but for now it's here to get the HUD working.
use eframe::egui;

// Configuration: values we choose for the HUD.
// Window size and behavior.
const HUD_WIDTH: f32 = 340.0;
const HUD_HEIGHT: f32 = 420.0;
const HUD_DECORATIONS: bool = false;
const HUD_TRANSPARENT: bool = true;

// Border shape and the space reserved for future resize handles.
const BORDER_WIDTH: f32 = 8.0;
const CORNER_RADIUS: f32 = 12.0;
const RESIZE_CORNER_SIZE: f32 = 16.0;

// Colors in red, green, blue, alpha order. Alpha controls transparency.
const WORKSPACE_RGBA: [u8; 4] = [28, 37, 52, 200];
const BORDER_RGBA: [u8; 4] = [17, 25, 40, 200];

// Window title and temporary workspace content.
const HUD_TITLE: &str = "FIFO Tasker";
const PLACEHOLDER_TEXT: &str = "Hello, FIFO Tasker!";
const CONTENT_TOP_SPACING: f32 = 20.0;

// Derived values: calculated from the configuration above.
const BORDER_INSET: f32 = BORDER_WIDTH / 2.0;

fn main() -> eframe::Result<()> {
    // Configure the HUD window.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([HUD_WIDTH, HUD_HEIGHT])
            .with_decorations(HUD_DECORATIONS) // Remove the native title bar and borders.
            .with_transparent(HUD_TRANSPARENT) // Allow the desktop to show through.
            .with_always_on_top(), // Keep the HUD above other windows.
        ..Default::default()
    };

    // Start the application.
    eframe::run_native(
        HUD_TITLE,
        options,
        Box::new(|_cc| Ok(Box::new(TaskerApp))),
    )
}

struct TaskerApp;

impl eframe::App for TaskerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Runtime geometry: use the actual window size, including after a resize.
        let rect = ui.max_rect();

        // Draw the slightly lighter, translucent workspace.
        let [r, g, b, a] = WORKSPACE_RGBA;
        ui.painter().rect_filled(
            rect.shrink(BORDER_WIDTH),
            CORNER_RADIUS,
            egui::Color32::from_rgba_unmultiplied(r, g, b, a),
        );

        // Draw the dark blue outer border.
        let [r, g, b, a] = BORDER_RGBA;
        ui.painter().rect_stroke(
            rect.shrink(BORDER_INSET),
            CORNER_RADIUS,
            egui::Stroke::new(
                BORDER_WIDTH,
                egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            ),
            egui::StrokeKind::Inside,
        );

        // Grab a straight edge to move the HUD. The corners are saving themselves for resizing.
        let border = rect.shrink(BORDER_INSET);
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
            ui.add_space(CONTENT_TOP_SPACING);
            ui.heading(HUD_TITLE);
            ui.label(PLACEHOLDER_TEXT);
        });
    }
}

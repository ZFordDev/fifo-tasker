use eframe::egui;

use super::config::{
    BORDER_INSET, BORDER_RGBA, BORDER_WIDTH, CORNER_RADIUS, RESIZE_CORNER_SIZE, WORKSPACE_RGBA,
};

// Paint the HUD shell and let its straight edges move the window.
pub(super) fn show(ui: &mut egui::Ui) {
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
}

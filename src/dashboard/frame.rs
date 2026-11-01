use eframe::egui;

use super::config::{BORDER_INSET, BORDER_RGBA, BORDER_WIDTH, CORNER_RADIUS, WORKSPACE_RGBA};

// Paint the workspace; the native frame handles moving and resizing.
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
}

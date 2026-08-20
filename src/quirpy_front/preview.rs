use crate::quirpy_encoder;

const MATRIX_SIZE: usize = 21;

pub fn ui(ui: &mut egui::Ui) {
    ui.heading("Preview");
    ui.separator();

    let matrix = quirpy_encoder::placeholder_matrix(MATRIX_SIZE);

    let available = ui.available_size();
    let side = available.x.min(available.y - 80.0).max(50.0);
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

    let module_size = side / MATRIX_SIZE as f32;
    let module_color = ui.visuals().text_color();

    for (row, cells) in matrix.iter().enumerate() {
        for (col, &on) in cells.iter().enumerate() {
            if !on {
                continue;
            }
            let module_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(col as f32 * module_size, row as f32 * module_size),
                egui::vec2(module_size, module_size),
            );
            painter.rect_filled(module_rect, 0.0, module_color);
        }
    }

    ui.add_space(12.0);

    disabled_button(ui, "Export as...");
}

fn disabled_button(ui: &mut egui::Ui, label: &str) {
    ui.add_enabled(false, egui::Button::new(label))
        .on_disabled_hover_text("Coming soon");
}

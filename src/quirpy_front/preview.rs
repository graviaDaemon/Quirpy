use crate::quirpy_front::generate::{Generator, Output};
use crate::quirpy_front::style::StyleState;

const QUIET_ZONE: usize = 4;
const PAYLOAD_PREVIEW_CHARS: usize = 180;

pub fn ui(ui: &mut egui::Ui, generator: &Generator, style: &StyleState, show_details: bool) {
    ui.heading("Preview");
    ui.separator();

    let reserved = if show_details { 200.0 } else { 80.0 };
    let available = ui.available_size();
    let side = available.x.min(available.y - reserved).max(50.0);
    let (rect, _response) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());

    match generator.output() {
        Some(Output::Symbol { symbol, .. }) => paint(ui, rect, &symbol.matrix, style),
        Some(Output::Invalid(reason)) => frame(ui, rect, Some(reason), style),
        None => frame(ui, rect, None, style),
    }

    ui.add_space(12.0);
    ui.add_enabled(false, egui::Button::new("Export as..."))
        .on_disabled_hover_text("Coming soon");

    if show_details {
        ui.add_space(8.0);
        details(ui, generator.output());
    }
}

fn paint(ui: &egui::Ui, rect: egui::Rect, matrix: &[Vec<bool>], style: &StyleState) {
    let grid = matrix.len() + 2 * QUIET_ZONE;
    // Flooring the module size to whole pixels is what keeps the modules seam-free at arbitrary
    // window sizes; the grid is then centred in whatever space is left over.
    let module = (rect.width().min(rect.height()) / grid as f32)
        .floor()
        .max(1.0);
    let side = module * grid as f32;
    let origin = rect.center() - egui::vec2(side, side) / 2.0;

    let painter = ui.painter_at(rect);
    painter.rect_filled(
        egui::Rect::from_min_size(origin, egui::vec2(side, side)),
        0.0,
        style.light,
    );

    for (row, cells) in matrix.iter().enumerate() {
        for (col, on) in cells.iter().enumerate() {
            if !on {
                continue;
            }
            let min = origin
                + egui::vec2(
                    (col + QUIET_ZONE) as f32 * module,
                    (row + QUIET_ZONE) as f32 * module,
                );
            painter.rect_filled(
                egui::Rect::from_min_size(min, egui::vec2(module, module)),
                0.0,
                style.dark,
            );
        }
    }
}

fn frame(ui: &egui::Ui, rect: egui::Rect, reason: Option<&str>, style: &StyleState) {
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, style.light.gamma_multiply(0.35));
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, ui.visuals().weak_text_color()),
        egui::StrokeKind::Inside,
    );

    if let Some(reason) = reason {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            reason,
            egui::FontId::proportional(13.0),
            ui.visuals().weak_text_color(),
        );
    }
}

fn details(ui: &mut egui::Ui, output: Option<&Output>) {
    egui::CollapsingHeader::new("Details")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("preview_details")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| match output {
                    Some(Output::Symbol { payload, symbol }) => {
                        payload_row(ui, payload);
                        row(ui, "Mode", symbol.mode.to_string());
                        row(
                            ui,
                            "Version",
                            format!("{} ({} × {})", symbol.version, symbol.size(), symbol.size()),
                        );
                        row(ui, "Error correction", symbol.ec.label().to_owned());
                        row(ui, "Mask", symbol.mask.to_string());
                        row(
                            ui,
                            "Dark modules",
                            format!(
                                "{} of {}",
                                symbol.dark_modules(),
                                symbol.size() * symbol.size()
                            ),
                        );
                    }
                    Some(Output::Invalid(reason)) => {
                        ui.label("Payload");
                        ui.colored_label(ui.visuals().error_fg_color, reason);
                        ui.end_row();
                        for label in [
                            "Mode",
                            "Version",
                            "Error correction",
                            "Mask",
                            "Dark modules",
                        ] {
                            row(ui, label, "—".to_owned());
                        }
                    }
                    None => {
                        for label in [
                            "Payload",
                            "Mode",
                            "Version",
                            "Error correction",
                            "Mask",
                            "Dark modules",
                        ] {
                            row(ui, label, "—".to_owned());
                        }
                    }
                });
        });
}

fn row(ui: &mut egui::Ui, label: &str, value: String) {
    ui.label(label);
    ui.label(value);
    ui.end_row();
}

fn payload_row(ui: &mut egui::Ui, payload: &str) {
    ui.label("Payload");
    let shown = if payload.chars().count() > PAYLOAD_PREVIEW_CHARS {
        let head: String = payload.chars().take(PAYLOAD_PREVIEW_CHARS).collect();
        format!("{head}…")
    } else {
        payload.to_owned()
    };
    ui.add(
        egui::Label::new(egui::RichText::new(shown).monospace())
            .wrap()
            .selectable(true),
    )
    .on_hover_text(payload);
    ui.end_row();
}

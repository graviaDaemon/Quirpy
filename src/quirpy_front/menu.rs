use crate::quirpy_front::form::FormState;

pub fn action_new(form: &mut FormState) {
    *form = FormState::default();
    tracing::info!("new project (form reset to default)");
}

pub fn action_exit(ctx: &egui::Context) {
    tracing::info!("exit requested");
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
}

pub fn ui_full(ui: &mut egui::Ui, ctx: &egui::Context, form: &mut FormState, dark_mode: &mut bool) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New").clicked() {
                action_new(form);
                ui.close();
            }

            disabled_item(ui, "Open");
            ui.menu_button("Open Recent", |ui| {
                for i in 1..=5 {
                    disabled_item(ui, &format!("Recent {i}"));
                }
            });

            ui.separator();

            if ui.button("Exit").clicked() {
                action_exit(ctx);
                ui.close();
            }
        });

        ui.menu_button("Edit", |ui| {
            disabled_item(ui, "Undo");
            disabled_item(ui, "Redo");
            ui.separator();
            disabled_item(ui, "Import image");
            ui.separator();
            disabled_item(ui, "Preferences");
        });

        ui.menu_button("Help", |ui| {
            disabled_item(ui, "About");
            disabled_item(ui, "Version");
            disabled_item(ui, "Check for update");
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let toggle_label = if *dark_mode { "☀ Light" } else { "🌙 Dark" };
            if ui.button(toggle_label).clicked() {
                *dark_mode = !*dark_mode;
                tracing::debug!(dark_mode = *dark_mode, "theme toggled");
            }
        });
    });
}

pub fn ui_toolbar(ui: &mut egui::Ui, dark_mode: &mut bool) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let toggle_label = if *dark_mode { "☀ Light" } else { "🌙 Dark" };
        if ui.button(toggle_label).clicked() {
            *dark_mode = !*dark_mode;
            tracing::debug!(dark_mode = *dark_mode, "theme toggled");
        }
    });
}

fn disabled_item(ui: &mut egui::Ui, label: &str) {
    ui.add_enabled(false, egui::Button::new(label))
        .on_disabled_hover_text("Coming soon");
}

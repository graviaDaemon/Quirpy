use std::path::Path;

use crate::quirpy_config::{self, Config, ExportFormat, LogLevel, THEMES, theme_label};
use crate::quirpy_front::app::QuirpyApp;

pub fn ui(app: &mut QuirpyApp, ctx: &egui::Context) {
    if !app.settings_open {
        return;
    }

    let before = app.config.clone();
    let mut close_requested = false;

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("settings"),
        egui::ViewportBuilder::default()
            .with_title("Settings")
            .with_inner_size([520.0, 400.0]),
        |ui, _class| {
            egui::CentralPanel::default().show(ui, |ui| body(ui, &mut app.config));
            close_requested = ui.ctx().input(|input| input.viewport().close_requested());
        },
    );

    if close_requested {
        app.settings_open = false;
    }

    if app.config == before {
        return;
    }

    if app.config.theme != before.theme {
        ctx.set_theme(app.config.theme);
    }
    if app.config.log_level != before.log_level {
        app.logging.set_level(app.config.log_level);
    }
    if let Err(error) = quirpy_config::save(&app.config) {
        tracing::warn!(%error, "could not write configuration");
    }
}

fn body(ui: &mut egui::Ui, config: &mut Config) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        group(ui, "General", |ui| {
            egui::Grid::new("settings_general")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Theme");
                    egui::ComboBox::from_id_salt("settings_theme")
                        .selected_text(theme_label(config.theme))
                        .show_ui(ui, |ui| {
                            for theme in THEMES {
                                ui.selectable_value(&mut config.theme, theme, theme_label(theme));
                            }
                        });
                    ui.end_row();

                    ui.label("QR details");
                    ui.checkbox(
                        &mut config.show_preview_details,
                        "Show details under the preview",
                    );
                    ui.end_row();
                });
        });

        ui.add_space(12.0);

        group(ui, "System", |ui| {
            egui::Grid::new("settings_system")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Default save location");
                    ui.horizontal(|ui| {
                        if ui.button("Browse…").clicked()
                            && let Some(dir) = rfd::FileDialog::new().pick_folder()
                        {
                            tracing::info!(?dir, "default save location set");
                            config.default_save_location = Some(dir);
                        }
                        if ui
                            .add_enabled(
                                config.default_save_location.is_some(),
                                egui::Button::new("Clear"),
                            )
                            .clicked()
                        {
                            config.default_save_location = None;
                        }
                    });
                    ui.end_row();

                    ui.label("");
                    ui.label(location_label(config.default_save_location.as_deref()));
                    ui.end_row();

                    ui.label("Default QR format");
                    egui::ComboBox::from_id_salt("settings_export_format")
                        .selected_text(config.default_export_format.label())
                        .show_ui(ui, |ui| {
                            for format in ExportFormat::ALL {
                                ui.selectable_value(
                                    &mut config.default_export_format,
                                    format,
                                    format.label(),
                                );
                            }
                        });
                    ui.end_row();

                    if config.default_export_format == ExportFormat::Jpg {
                        ui.label("");
                        ui.colored_label(
                            ui.visuals().warn_fg_color,
                            "JPG is lossy — its artifacts land on the edges scanners read, which can reduce scan reliability.",
                        );
                        ui.end_row();
                    }

                    ui.label("Log level");
                    egui::ComboBox::from_id_salt("settings_log_level")
                        .selected_text(config.log_level.label())
                        .show_ui(ui, |ui| {
                            for level in LogLevel::ALL {
                                ui.selectable_value(&mut config.log_level, level, level.label());
                            }
                        });
                    ui.end_row();

                    ui.label("Logging");
                    ui.vertical(|ui| {
                        ui.checkbox(&mut config.log_to_file, "Write logs to file");
                        ui.weak("Applies after restart.");
                        if ui.button("Open log folder").clicked() {
                            open_log_folder();
                        }
                    });
                    ui.end_row();
                });
        });
    });
}

fn group(ui: &mut egui::Ui, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(title).strong());
        ui.separator();
        contents(ui);
    });
}

fn location_label(location: Option<&Path>) -> String {
    location
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_else(|| "(ask every time)".to_owned())
}

fn open_log_folder() {
    let Some(dir) = quirpy_config::log_dir() else {
        tracing::warn!("no platform data directory; there is no log folder to open");
        return;
    };

    if let Err(error) = std::fs::create_dir_all(&dir) {
        tracing::warn!(?dir, %error, "could not create the log folder");
        return;
    }

    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };

    match std::process::Command::new(opener).arg(&dir).spawn() {
        Ok(_) => tracing::debug!(?dir, "opened the log folder"),
        Err(error) => tracing::warn!(?dir, %error, "could not open the log folder"),
    }
}

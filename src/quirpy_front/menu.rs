use std::path::{Path, PathBuf};

#[cfg(not(target_os = "macos"))]
use egui::{Key, KeyboardShortcut, Modifiers};

// macOS drives these from the native menu's own accelerators; see menu_native.rs.
#[cfg(not(target_os = "macos"))]
pub const NEW: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
#[cfg(not(target_os = "macos"))]
pub const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
#[cfg(not(target_os = "macos"))]
pub const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
#[cfg(not(target_os = "macos"))]
pub const SAVE_AS: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::S);
#[cfg(not(target_os = "macos"))]
pub const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);

#[cfg(not(target_os = "macos"))]
pub const REDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);

#[cfg(not(target_os = "macos"))]
pub const PREFERENCES: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Comma);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    New,
    Open,
    OpenRecent(PathBuf),
    Save,
    SaveAs,
    Undo,
    Redo,
    Settings,
    About,
    Version,
    Exit,
}

#[cfg(not(target_os = "macos"))]
pub fn ui_full(
    ui: &mut egui::Ui,
    recents: &[PathBuf],
    can_undo: bool,
    can_redo: bool,
) -> Option<Command> {
    let mut command = None;

    egui::MenuBar::new().ui(ui, |ui| {
        let ctx = ui.ctx().clone();

        ui.menu_button("File", |ui| {
            if item(ui, "New", Some(&ctx.format_shortcut(&NEW)), true).clicked() {
                command = Some(Command::New);
                ui.close();
            }
            if item(ui, "Open…", Some(&ctx.format_shortcut(&OPEN)), true).clicked() {
                command = Some(Command::Open);
                ui.close();
            }
            ui.menu_button("Open Recent", |ui| {
                if item(ui, "Open…", None, true).clicked() {
                    command = Some(Command::Open);
                    ui.close();
                }
                ui.separator();
                if recents.is_empty() {
                    item(ui, "(No recent files)", None, false);
                } else {
                    for path in recents {
                        if item(ui, &recent_label(path), None, true)
                            .on_hover_text(path.to_string_lossy())
                            .clicked()
                        {
                            command = Some(Command::OpenRecent(path.clone()));
                            ui.close();
                        }
                    }
                }
            });

            ui.separator();

            if item(ui, "Save", Some(&ctx.format_shortcut(&SAVE)), true).clicked() {
                command = Some(Command::Save);
                ui.close();
            }
            if item(ui, "Save As…", Some(&ctx.format_shortcut(&SAVE_AS)), true).clicked() {
                command = Some(Command::SaveAs);
                ui.close();
            }

            ui.separator();

            if item(ui, "Exit", None, true).clicked() {
                command = Some(Command::Exit);
                ui.close();
            }
        });

        ui.menu_button("Edit", |ui| {
            if item(ui, "Undo", Some(&ctx.format_shortcut(&UNDO)), can_undo).clicked() {
                command = Some(Command::Undo);
                ui.close();
            }
            if item(ui, "Redo", Some(&ctx.format_shortcut(&REDO)), can_redo).clicked() {
                command = Some(Command::Redo);
                ui.close();
            }
            ui.separator();
            coming_soon(ui, "Import image");
            ui.separator();
            if item(
                ui,
                "Preferences",
                Some(&ctx.format_shortcut(&PREFERENCES)),
                true,
            )
            .clicked()
            {
                command = Some(Command::Settings);
                ui.close();
            }
        });

        ui.menu_button("Help", |ui| {
            if item(ui, "About", None, true).clicked() {
                command = Some(Command::About);
                ui.close();
            }
            if item(ui, "Version", None, true).clicked() {
                command = Some(Command::Version);
                ui.close();
            }
            coming_soon(ui, "Check for update");
        });
    });

    command
}

pub fn recent_label(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

#[cfg(not(target_os = "macos"))]
fn item(ui: &mut egui::Ui, label: &str, shortcut: Option<&str>, enabled: bool) -> egui::Response {
    let mut button = egui::Button::new(label);
    if let Some(shortcut) = shortcut {
        button = button.shortcut_text(shortcut);
    }
    ui.add_enabled(enabled, button)
}

#[cfg(not(target_os = "macos"))]
fn coming_soon(ui: &mut egui::Ui, label: &str) {
    ui.add_enabled(false, egui::Button::new(label))
        .on_disabled_hover_text("Coming soon");
}

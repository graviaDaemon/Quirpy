use std::path::PathBuf;

use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};

use crate::quirpy_config::Config;
use crate::quirpy_front::menu::{self, Command};

const NEW: &str = "file_new";
const OPEN: &str = "file_open";
const SAVE: &str = "file_save";
const SAVE_AS: &str = "file_save_as";
const QUIT: &str = "app_quit";
const UNDO: &str = "edit_undo";
const REDO: &str = "edit_redo";
const RECENT_OPEN: &str = "recent_open";
const SETTINGS: &str = "app_settings";
const PREFERENCES: &str = "edit_preferences";
const APP_ABOUT: &str = "app_about";
const HELP_ABOUT: &str = "help_about";
const VERSION: &str = "help_version";
const RECENT_PREFIX: &str = "recent_";

pub struct NativeMenu {
    _menu_bar: Menu,
    open_recent: Submenu,
    undo_item: MenuItem,
    redo_item: MenuItem,
    recents: Vec<PathBuf>,
}

impl NativeMenu {
    pub fn init(config: &Config) -> Self {
        let menu_bar = Menu::new();

        // macOS promotes the first submenu of the menu bar into the application menu, so this one
        // has to come first — otherwise File is absorbed into it and never renders as its own menu.
        let app_menu = Submenu::with_items(
            "Quirpy",
            true,
            &[
                &MenuItem::with_id(APP_ABOUT, "About Quirpy", true, None),
                &MenuItem::with_id(
                    SETTINGS,
                    "Settings…",
                    true,
                    Some(Accelerator::new(Some(Modifiers::META), Code::Comma)),
                ),
                &PredefinedMenuItem::separator(),
                // A plain item rather than PredefinedMenuItem::quit: the predefined one terminates
                // the process from AppKit, which would walk straight past the unsaved-changes guard.
                &MenuItem::with_id(
                    QUIT,
                    "Quit Quirpy",
                    true,
                    Some(Accelerator::new(Some(Modifiers::META), Code::KeyQ)),
                ),
            ],
        )
        .expect("failed to build application menu");

        let open_recent = Submenu::new("Open Recent", true);

        let file_menu = Submenu::with_items(
            "File",
            true,
            &[
                &MenuItem::with_id(
                    NEW,
                    "New",
                    true,
                    Some(Accelerator::new(Some(Modifiers::META), Code::KeyN)),
                ),
                &MenuItem::with_id(
                    OPEN,
                    "Open…",
                    true,
                    Some(Accelerator::new(Some(Modifiers::META), Code::KeyO)),
                ),
                &open_recent,
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    SAVE,
                    "Save",
                    true,
                    Some(Accelerator::new(Some(Modifiers::META), Code::KeyS)),
                ),
                &MenuItem::with_id(
                    SAVE_AS,
                    "Save As…",
                    true,
                    Some(Accelerator::new(
                        Some(Modifiers::META | Modifiers::SHIFT),
                        Code::KeyS,
                    )),
                ),
            ],
        )
        .expect("failed to build File menu");

        let undo_item = MenuItem::with_id(
            UNDO,
            "Undo",
            false,
            Some(Accelerator::new(Some(Modifiers::META), Code::KeyZ)),
        );
        let redo_item = MenuItem::with_id(
            REDO,
            "Redo",
            false,
            Some(Accelerator::new(
                Some(Modifiers::META | Modifiers::SHIFT),
                Code::KeyZ,
            )),
        );

        let edit_menu = Submenu::with_items(
            "Edit",
            true,
            &[
                &undo_item,
                &redo_item,
                &PredefinedMenuItem::separator(),
                &MenuItem::new("Import image", false, None),
                &PredefinedMenuItem::separator(),
                // ⌘, already belongs to Settings… in the application menu; a second item cannot
                // claim the same accelerator, so this one is click-only.
                &MenuItem::with_id(PREFERENCES, "Preferences", true, None),
            ],
        )
        .expect("failed to build Edit menu");

        let help_menu = Submenu::with_items(
            "Help",
            true,
            &[
                &MenuItem::with_id(HELP_ABOUT, "About", true, None),
                &MenuItem::with_id(VERSION, "Version", true, None),
                &MenuItem::new("Check for update", false, None),
            ],
        )
        .expect("failed to build Help menu");

        menu_bar
            .append_items(&[&app_menu, &file_menu, &edit_menu, &help_menu])
            .expect("failed to attach menus to menu bar");

        menu_bar.init_for_nsapp();

        let mut native = Self {
            _menu_bar: menu_bar,
            open_recent,
            undo_item,
            redo_item,
            recents: Vec::new(),
        };
        native.sync_recents(config);
        native
    }

    pub fn sync_recents(&mut self, config: &Config) {
        self.recents = config.recent_files.clone();

        while self.open_recent.remove_at(0).is_some() {}

        let mut items: Vec<Box<dyn IsMenuItem>> = vec![
            Box::new(MenuItem::with_id(RECENT_OPEN, "Open…", true, None)),
            Box::new(PredefinedMenuItem::separator()),
        ];

        if self.recents.is_empty() {
            items.push(Box::new(MenuItem::new("(No recent files)", false, None)));
        } else {
            for (index, path) in self.recents.iter().enumerate() {
                items.push(Box::new(MenuItem::with_id(
                    format!("{RECENT_PREFIX}{index}"),
                    menu::recent_label(path),
                    true,
                    None,
                )));
            }
        }

        let refs: Vec<&dyn IsMenuItem> = items.iter().map(|item| item.as_ref()).collect();
        if let Err(error) = self.open_recent.append_items(&refs) {
            tracing::warn!(%error, "could not rebuild the Open Recent menu");
        }
    }

    pub fn sync_history(&self, can_undo: bool, can_redo: bool) {
        if self.undo_item.is_enabled() != can_undo {
            self.undo_item.set_enabled(can_undo);
        }
        if self.redo_item.is_enabled() != can_redo {
            self.redo_item.set_enabled(can_redo);
        }
    }

    pub fn poll(&self) -> Vec<Command> {
        let mut commands = Vec::new();

        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            tracing::debug!(id = ?event.id, "native menu event");

            let id = event.id.as_ref();
            let command = match id {
                NEW => Some(Command::New),
                OPEN | RECENT_OPEN => Some(Command::Open),
                SAVE => Some(Command::Save),
                SAVE_AS => Some(Command::SaveAs),
                UNDO => Some(Command::Undo),
                REDO => Some(Command::Redo),
                QUIT => Some(Command::Exit),
                SETTINGS | PREFERENCES => Some(Command::Settings),
                APP_ABOUT | HELP_ABOUT => Some(Command::About),
                VERSION => Some(Command::Version),
                _ => id
                    .strip_prefix(RECENT_PREFIX)
                    .and_then(|slot| slot.parse::<usize>().ok())
                    .and_then(|slot| self.recents.get(slot))
                    .map(|path| Command::OpenRecent(path.clone())),
            };

            if let Some(command) = command {
                commands.push(command);
            }
        }

        commands
    }
}

use crate::quirpy_front::{form::ProjectState, menu};
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct NativeMenu {
    new_id: muda::MenuId,
    exit_id: muda::MenuId,
}

impl NativeMenu {
    pub fn init() -> Self {
        let menu_bar = Menu::new();

        let new_item = MenuItem::new("New", true, None);
        let open_item = MenuItem::new("Open", false, None);
        let recent_items: Vec<MenuItem> = (1..=5)
            .map(|i| MenuItem::new(format!("Recent {i}"), false, None))
            .collect();
        let open_recent = Submenu::with_items(
            "Open Recent",
            true,
            &recent_items
                .iter()
                .map(|item| item as &dyn muda::IsMenuItem)
                .collect::<Vec<_>>(),
        )
        .expect("failed to build Open Recent submenu");
        let exit_item = MenuItem::new("Exit", true, None);

        let file_menu = Submenu::with_items(
            "File",
            true,
            &[
                &new_item,
                &open_item,
                &open_recent,
                &PredefinedMenuItem::separator(),
                &exit_item,
            ],
        )
        .expect("failed to build File menu");

        let undo_item = MenuItem::new("Undo", false, None);
        let redo_item = MenuItem::new("Redo", false, None);
        let import_item = MenuItem::new("Import image", false, None);
        let preferences_item = MenuItem::new("Preferences", false, None);

        let edit_menu = Submenu::with_items(
            "Edit",
            true,
            &[
                &undo_item,
                &redo_item,
                &PredefinedMenuItem::separator(),
                &import_item,
                &PredefinedMenuItem::separator(),
                &preferences_item,
            ],
        )
        .expect("failed to build Edit menu");

        let about_item = MenuItem::new("About", false, None);
        let version_item = MenuItem::new("Version", false, None);
        let update_item = MenuItem::new("Check for update", false, None);

        let help_menu =
            Submenu::with_items("Help", true, &[&about_item, &version_item, &update_item])
                .expect("failed to build Help menu");

        menu_bar
            .append_items(&[&file_menu, &edit_menu, &help_menu])
            .expect("failed to attach menus to menu bar");

        menu_bar.init_for_nsapp();

        Self {
            new_id: new_item.id().clone(),
            exit_id: exit_item.id().clone(),
        }
    }

    pub fn poll(&self, project: &mut ProjectState, ctx: &egui::Context) {
        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            if event.id == self.new_id {
                menu::action_new(project);
            } else if event.id == self.exit_id {
                menu::action_exit(ctx);
            }
        }
    }
}

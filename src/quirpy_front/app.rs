use std::path::PathBuf;

use crate::Logging;
use crate::quirpy_config::Config;
use crate::quirpy_front::{
    about, actions, form::ProjectState, generate::Generator, history::History, menu, preview,
    settings,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    New,
    Open,
    OpenRecent(PathBuf),
    Exit,
}

pub struct ErrorPrompt {
    pub message: String,
    pub open_anyway: Option<PathBuf>,
}

pub struct QuirpyApp {
    pub project: ProjectState,
    pub saved_state: ProjectState,
    pub current_path: Option<PathBuf>,
    pub history: History,
    pub generator: Generator,
    pub config: Config,
    pub pending: Option<PendingAction>,
    pub error: Option<ErrorPrompt>,
    pub force_exit: bool,
    pub recents_dirty: bool,
    pub settings_open: bool,
    pub about_open: bool,
    pub version_open: bool,
    pub logging: Logging,
    title: String,
    #[cfg(target_os = "macos")]
    native_menu: crate::quirpy_front::menu_native::NativeMenu,
}

impl QuirpyApp {
    pub fn new(config: Config, logging: Logging) -> Self {
        let project = ProjectState::default();

        Self {
            saved_state: project.clone(),
            history: History::new(&project),
            generator: Generator::default(),
            project,
            current_path: None,
            pending: None,
            error: None,
            force_exit: false,
            recents_dirty: true,
            settings_open: false,
            about_open: false,
            version_open: false,
            logging,
            title: String::new(),
            #[cfg(target_os = "macos")]
            native_menu: crate::quirpy_front::menu_native::NativeMenu::init(&config),
            config,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.project != self.saved_state
    }

    fn dispatch(&mut self, command: menu::Command, ctx: &egui::Context) {
        match command {
            menu::Command::New => self.guard(PendingAction::New, ctx),
            menu::Command::Open => self.guard(PendingAction::Open, ctx),
            menu::Command::OpenRecent(path) => self.guard(PendingAction::OpenRecent(path), ctx),
            menu::Command::Exit => self.guard(PendingAction::Exit, ctx),
            menu::Command::Save => {
                actions::save(self);
            }
            menu::Command::SaveAs => {
                actions::save_as(self);
            }
            menu::Command::Undo => {
                self.history.undo(&mut self.project);
            }
            menu::Command::Redo => {
                self.history.redo(&mut self.project);
            }
            menu::Command::Settings => self.settings_open = true,
            menu::Command::About => self.about_open = true,
            menu::Command::Version => self.version_open = true,
        }
    }

    fn guard(&mut self, action: PendingAction, ctx: &egui::Context) {
        if self.is_dirty() {
            self.pending = Some(action);
        } else {
            self.run(action, ctx);
        }
    }

    fn run(&mut self, action: PendingAction, ctx: &egui::Context) {
        match action {
            PendingAction::New => actions::new_project(self),
            PendingAction::Open => actions::open(self),
            PendingAction::OpenRecent(path) => actions::open_path(self, &path),
            PendingAction::Exit => {
                tracing::info!("exit requested");
                self.force_exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn shortcuts(&mut self, ctx: &egui::Context) -> Vec<menu::Command> {
        let mut commands = Vec::new();
        ctx.input_mut(|input| {
            for (shortcut, command) in [
                (menu::SAVE_AS, menu::Command::SaveAs),
                (menu::SAVE, menu::Command::Save),
                (menu::NEW, menu::Command::New),
                (menu::OPEN, menu::Command::Open),
                (menu::REDO, menu::Command::Redo),
                (menu::UNDO, menu::Command::Undo),
                (menu::PREFERENCES, menu::Command::Settings),
            ] {
                if input.consume_shortcut(&shortcut) {
                    commands.push(command);
                }
            }
        });
        commands
    }

    fn close_guard(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.viewport().close_requested()) || self.force_exit {
            return;
        }
        if self.is_dirty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(PendingAction::Exit);
        } else {
            self.force_exit = true;
        }
    }

    fn unsaved_changes_modal(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending.clone() else {
            return;
        };

        let name = if self.project.name.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            self.project.name.trim().to_owned()
        };

        let mut choice = None;
        egui::Modal::new(egui::Id::new("unsaved_changes")).show(ctx, |ui| {
            ui.set_max_width(360.0);
            ui.heading("Unsaved changes");
            ui.add_space(4.0);
            ui.label(format!("Save changes to {name}?"));
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    choice = Some(0);
                }
                if ui.button("Discard").clicked() {
                    choice = Some(1);
                }
                if ui.button("Cancel").clicked() {
                    choice = Some(2);
                }
            });
        });

        match choice {
            Some(0) => {
                self.pending = None;
                if actions::save(self) {
                    self.run(pending, ctx);
                }
            }
            Some(1) => {
                self.pending = None;
                self.run(pending, ctx);
            }
            Some(2) => self.pending = None,
            _ => {}
        }
    }

    fn error_modal(&mut self, ctx: &egui::Context) {
        let Some(prompt) = self.error.take() else {
            return;
        };

        let mut dismissed = false;
        let mut open_anyway = false;
        egui::Modal::new(egui::Id::new("project_error")).show(ctx, |ui| {
            ui.set_max_width(420.0);
            ui.heading("Quirpy");
            ui.add_space(4.0);
            ui.label(&prompt.message);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if prompt.open_anyway.is_some() && ui.button("Open anyway").clicked() {
                    open_anyway = true;
                }
                if ui.button("Close").clicked() {
                    dismissed = true;
                }
            });
        });

        match (open_anyway, prompt.open_anyway.clone()) {
            (true, Some(path)) => actions::open_path_ignoring_checksum(self, &path),
            _ if !dismissed => self.error = Some(prompt),
            _ => {}
        }
    }

    fn sync_title(&mut self, ctx: &egui::Context) {
        let title = actions::title(self);
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }
}

impl eframe::App for QuirpyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        let mut commands: Vec<menu::Command> = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if self.recents_dirty {
                self.native_menu.sync_recents(&self.config);
                self.recents_dirty = false;
            }
            self.native_menu
                .sync_history(self.history.can_undo(), self.history.can_redo());
            commands.extend(self.native_menu.poll());
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // On macOS the native menu already owns these accelerators; registering them here as well
        // would fire every action twice.
        #[cfg(not(target_os = "macos"))]
        commands.extend(self.shortcuts(&ctx));

        self.close_guard(&ctx);

        #[cfg(not(target_os = "macos"))]
        egui::Panel::top("top_panel").show(ui, |ui| {
            if let Some(command) = menu::ui_full(
                ui,
                &self.config.recent_files,
                self.history.can_undo(),
                self.history.can_redo(),
            ) {
                commands.push(command);
            }
        });

        let save_clicked = egui::Panel::left("form_panel")
            .default_size(340.0)
            .show(ui, |ui| {
                crate::quirpy_front::form::ui(ui, &mut self.project)
            })
            .inner;
        if save_clicked {
            commands.push(menu::Command::Save);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            preview::ui(
                ui,
                &self.generator,
                &self.project.style,
                self.config.show_preview_details,
            );
        });

        self.unsaved_changes_modal(&ctx);
        self.error_modal(&ctx);
        settings::ui(self, &ctx);
        about::ui(self, &ctx);

        for command in commands {
            self.dispatch(command, &ctx);
        }

        self.generator.tick(&self.project, &ctx);
        self.history.maybe_commit(&self.project, &ctx);
        self.sync_title(&ctx);
    }
}

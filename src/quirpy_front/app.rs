use crate::quirpy_front::{form::FormState, menu, preview};

pub struct QuirpyApp {
    form: FormState,
    dark_mode: bool,
    #[cfg(target_os = "macos")]
    native_menu: crate::quirpy_front::menu_native::NativeMenu,
}

impl QuirpyApp {
    pub fn new() -> Self {
        Self {
            form: FormState::default(),
            dark_mode: true,
            #[cfg(target_os = "macos")]
            native_menu: crate::quirpy_front::menu_native::NativeMenu::init(),
        }
    }
}

impl Default for QuirpyApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for QuirpyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        #[cfg(target_os = "macos")]
        {
            self.native_menu.poll(&mut self.form, &ctx);
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        #[cfg(target_os = "macos")]
        egui::Panel::top("top_panel").show(ui, |ui| {
            menu::ui_toolbar(ui, &mut self.dark_mode);
        });

        #[cfg(not(target_os = "macos"))]
        egui::Panel::top("top_panel").show(ui, |ui| {
            menu::ui_full(ui, &ctx, &mut self.form, &mut self.dark_mode);
        });

        egui::Panel::left("form_panel")
            .default_size(340.0)
            .show(ui, |ui| {
                crate::quirpy_front::form::ui(ui, &mut self.form);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            preview::ui(ui);
        });
    }
}

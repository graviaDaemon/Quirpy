use crate::quirpy_front::app::QuirpyApp;
use crate::version;

const GITHUB: &str = "https://github.com/graviaDaemon/Quirpy";

pub fn ui(app: &mut QuirpyApp, ctx: &egui::Context) {
    window(app, ctx);
    version_modal(app, ctx);
}

fn window(app: &mut QuirpyApp, ctx: &egui::Context) {
    if !app.about_open {
        return;
    }

    let mut close_requested = false;

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("about"),
        egui::ViewportBuilder::default()
            .with_title("About Quirpy")
            .with_inner_size([440.0, 320.0])
            .with_resizable(false),
        |ui, _class| {
            egui::CentralPanel::default().show(ui, body);
            close_requested = ui.ctx().input(|input| input.viewport().close_requested());
        },
    );

    if close_requested {
        app.about_open = false;
    }
}

fn body(ui: &mut egui::Ui) {
    ui.heading("Quirpy");
    ui.add_space(4.0);
    ui.label("Created by Daemonium");
    ui.label(format!("Updated: {}", version::BUILD_DATE));

    ui.add_space(10.0);
    ui.label(
        "An open-source project to generate your own custom QR codes, without subscriptions and \
         without data-harvesting websites. Everything stays on your own machine.",
    );
    ui.add_space(6.0);
    ui.label("Fork it, change what you need, and open a pull request:");
    ui.hyperlink_to(GITHUB, GITHUB);

    ui.add_space(10.0);
    ui.separator();
    ui.label(format!("Version: {}", version::full_version()));
    ui.weak("The suffix is the commit this build came from.");
}

fn version_modal(app: &mut QuirpyApp, ctx: &egui::Context) {
    if !app.version_open {
        return;
    }

    let mut close = false;
    egui::Modal::new(egui::Id::new("version_modal")).show(ctx, |ui| {
        ui.set_max_width(280.0);
        ui.heading("Quirpy");
        ui.add_space(4.0);
        ui.label(version::full_version());
        ui.add_space(12.0);
        if ui.button("Close").clicked() {
            close = true;
        }
    });

    if close {
        app.version_open = false;
    }
}

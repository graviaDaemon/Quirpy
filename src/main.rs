mod quirpy_encoder;
mod quirpy_front;

use quirpy_front::app::QuirpyApp;
use tracing_subscriber::EnvFilter;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Quirpy starting");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Quirpy",
        options,
        Box::new(|_cc| Ok(Box::new(QuirpyApp::new()))),
    )
}

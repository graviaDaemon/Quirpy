mod quirpy_config;
mod quirpy_encoder;
mod quirpy_front;
mod quirpy_payload;
mod quirpy_project;
mod version;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry, reload};

use quirpy_config::{Config, LogLevel};
use quirpy_front::app::QuirpyApp;

// The filter sits directly on the registry so its handle stays simply typed; the file layer is a
// reload slot because the config that decides it can only be read once logging is already up.
type Filtered = Layered<reload::Layer<EnvFilter, Registry>, Registry>;
type FileSlot = Option<Box<dyn Layer<Filtered> + Send + Sync>>;

pub struct Logging {
    filter: reload::Handle<EnvFilter, Registry>,
    file: reload::Handle<FileSlot, Filtered>,
    env_override: bool,
    _file_guard: Option<WorkerGuard>,
}

// The chosen level applies to Quirpy's own logs. Dependencies stay at warn, because wgpu and winit
// alone write hundreds of kilobytes per minute at debug and would bury everything this app says.
fn directive(level: LogLevel) -> String {
    let dependencies = match level {
        LogLevel::Error => "error",
        _ => "warn",
    };
    format!(
        "{dependencies},{}={}",
        env!("CARGO_CRATE_NAME"),
        level.keyword()
    )
}

impl Logging {
    pub fn set_level(&self, level: LogLevel) {
        if let Err(error) = self
            .filter
            .modify(|filter| *filter = EnvFilter::new(directive(level)))
        {
            tracing::warn!(%error, "could not change the log level");
            return;
        }
        tracing::info!(level = level.keyword(), "log level changed");
    }

    // Called once, after the configuration has been read. Toggling file logging later needs a
    // restart; the Settings window says so.
    fn apply(&mut self, config: &Config) {
        if self.env_override {
            tracing::debug!("RUST_LOG is set; ignoring the configured log level");
        } else {
            self.set_level(config.log_level);
        }

        if !config.log_to_file {
            return;
        }

        let Some((layer, guard)) = file_layer() else {
            return;
        };

        match self.file.modify(|slot| *slot = Some(layer)) {
            Ok(()) => self._file_guard = Some(guard),
            Err(error) => tracing::warn!(%error, "could not start writing logs to file"),
        }
    }
}

fn init_logging() -> Logging {
    let from_env = std::env::var("RUST_LOG")
        .ok()
        .filter(|directive| !directive.trim().is_empty());
    let env_override = from_env.is_some();

    let (filter, filter_handle) = reload::Layer::new(
        from_env
            .and_then(|directive| EnvFilter::try_new(directive).ok())
            .unwrap_or_else(|| EnvFilter::new(directive(LogLevel::default()))),
    );
    let (file, file_handle) = reload::Layer::new(FileSlot::None);

    Registry::default()
        .with(filter)
        .with(file)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Logging {
        filter: filter_handle,
        file: file_handle,
        env_override,
        _file_guard: None,
    }
}

fn file_layer() -> Option<(Box<dyn Layer<Filtered> + Send + Sync>, WorkerGuard)> {
    let dir = quirpy_config::log_dir()?;

    if let Err(error) = std::fs::create_dir_all(&dir) {
        tracing::warn!(?dir, %error, "could not create the log directory");
        return None;
    }

    let (writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&dir, "quirpy.log"));
    tracing::info!(?dir, "writing logs to file");

    Some((
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(writer)
            .boxed(),
        guard,
    ))
}

fn main() -> eframe::Result<()> {
    let mut logging = init_logging();
    let config = quirpy_config::load();
    logging.apply(&config);

    tracing::info!(version = %version::full_version(), "Quirpy starting");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Quirpy",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(config.theme);
            Ok(Box::new(QuirpyApp::new(config, logging)))
        }),
    )
}

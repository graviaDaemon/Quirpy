use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use egui::ThemePreference;
use ini::{Ini, Properties};

const MAX_RECENT: usize = 5;
const FILE_NAME: &str = "configuration.qpy";
const RECENT: &str = "recent";
const GENERAL: &str = "general";
const SYSTEM: &str = "system";

const THEME: &str = "theme";
const SAVE_LOCATION: &str = "default_save_location";
const EXPORT_FORMAT: &str = "default_export_format";
const LOG_LEVEL: &str = "log_level";
const LOG_TO_FILE: &str = "log_to_file";
const SHOW_PREVIEW_DETAILS: &str = "show_preview_details";

pub const THEMES: [ThemePreference; 3] = [
    ThemePreference::Light,
    ThemePreference::Dark,
    ThemePreference::System,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExportFormat {
    #[default]
    Png,
    Svg,
    Jpg,
}

impl ExportFormat {
    pub const ALL: [Self; 3] = [Self::Png, Self::Svg, Self::Jpg];

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Svg => "SVG",
            Self::Jpg => "JPG",
        }
    }

    fn keyword(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Jpg => "jpg",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "jpg" | "jpeg" => Some(Self::Jpg),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const ALL: [Self; 5] = [
        Self::Trace,
        Self::Debug,
        Self::Info,
        Self::Warn,
        Self::Error,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Trace => "Trace",
            Self::Debug => "Debug",
            Self::Info => "Info",
            Self::Warn => "Warn",
            Self::Error => "Error",
        }
    }

    pub fn keyword(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

pub fn theme_label(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "Light",
        ThemePreference::Dark => "Dark",
        ThemePreference::System => "System",
    }
}

fn theme_keyword(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
        ThemePreference::System => "system",
    }
}

fn parse_theme(value: &str) -> Option<ThemePreference> {
    match value.to_ascii_lowercase().as_str() {
        "light" => Some(ThemePreference::Light),
        "dark" => Some(ThemePreference::Dark),
        "system" => Some(ThemePreference::System),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub recent_files: Vec<PathBuf>,
    pub theme: ThemePreference,
    pub show_preview_details: bool,
    pub default_save_location: Option<PathBuf>,
    pub default_export_format: ExportFormat,
    pub log_level: LogLevel,
    pub log_to_file: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            recent_files: Vec::new(),
            theme: ThemePreference::default(),
            show_preview_details: true,
            default_save_location: None,
            default_export_format: ExportFormat::default(),
            log_level: LogLevel::default(),
            log_to_file: false,
        }
    }
}

impl Config {
    pub fn push_recent(&mut self, path: &Path) {
        self.recent_files.retain(|existing| existing != path);
        self.recent_files.insert(0, path.to_path_buf());
        self.recent_files.truncate(MAX_RECENT);
    }
}

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "Quirpy")
}

pub fn config_path() -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.config_dir().join(FILE_NAME))
}

pub fn log_dir() -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.data_dir().join("logs"))
}

pub fn load() -> Config {
    let Some(path) = config_path() else {
        tracing::warn!("no platform config directory; using default configuration");
        return Config::default();
    };

    if !path.exists() {
        tracing::debug!(?path, "no configuration file yet; using defaults");
        return Config::default();
    }

    let ini = match Ini::load_from_file(&path) {
        Ok(ini) => ini,
        Err(error) => {
            tracing::warn!(?path, %error, "unreadable configuration; using defaults");
            return Config::default();
        }
    };

    let general = ini.section(Some(GENERAL));
    let system = ini.section(Some(SYSTEM));

    Config {
        recent_files: read_recents(ini.section(Some(RECENT))),
        theme: read(general, THEME, parse_theme, ThemePreference::default()),
        show_preview_details: read(
            general,
            SHOW_PREVIEW_DETAILS,
            |value| value.parse().ok(),
            true,
        ),
        default_save_location: read_save_location(system),
        default_export_format: read(
            system,
            EXPORT_FORMAT,
            ExportFormat::parse,
            ExportFormat::default(),
        ),
        log_level: read(system, LOG_LEVEL, LogLevel::parse, LogLevel::default()),
        log_to_file: read(system, LOG_TO_FILE, |value| value.parse().ok(), false),
    }
}

fn read<T>(
    section: Option<&Properties>,
    key: &str,
    parse: impl Fn(&str) -> Option<T>,
    default: T,
) -> T {
    let Some(value) = section.and_then(|section| section.get(key)) else {
        return default;
    };

    match parse(value.trim()) {
        Some(parsed) => parsed,
        None => {
            tracing::warn!(
                key,
                value,
                "unusable configuration value; using the default"
            );
            default
        }
    }
}

fn read_recents(section: Option<&Properties>) -> Vec<PathBuf> {
    let mut recent_files = Vec::new();
    let Some(section) = section else {
        return recent_files;
    };

    for slot in 1..=MAX_RECENT {
        let Some(value) = section.get(format!("file{slot}")) else {
            continue;
        };
        let candidate = PathBuf::from(value.trim());
        if candidate.as_os_str().is_empty() {
            continue;
        }
        if !candidate.is_file() {
            tracing::debug!(?candidate, "pruning recent file that no longer exists");
            continue;
        }
        if !recent_files.contains(&candidate) {
            recent_files.push(candidate);
        }
    }

    recent_files
}

fn read_save_location(section: Option<&Properties>) -> Option<PathBuf> {
    let value = section.and_then(|section| section.get(SAVE_LOCATION))?;
    let candidate = PathBuf::from(value.trim());
    if candidate.as_os_str().is_empty() {
        return None;
    }
    if !candidate.is_dir() {
        tracing::warn!(?candidate, "default save location is gone; ignoring it");
        return None;
    }
    Some(candidate)
}

pub fn save(config: &Config) -> Result<(), io::Error> {
    let Some(path) = config_path() else {
        return Err(io::Error::other("no platform config directory"));
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut ini = Ini::new();

    ini.with_section(Some(GENERAL))
        .set(THEME, theme_keyword(config.theme))
        .set(
            SHOW_PREVIEW_DETAILS,
            config.show_preview_details.to_string(),
        );

    let mut system = ini.with_section(Some(SYSTEM));
    system
        .set(
            SAVE_LOCATION,
            config
                .default_save_location
                .as_ref()
                .map(|dir| dir.to_string_lossy().into_owned())
                .unwrap_or_default(),
        )
        .set(EXPORT_FORMAT, config.default_export_format.keyword())
        .set(LOG_LEVEL, config.log_level.keyword())
        .set(LOG_TO_FILE, config.log_to_file.to_string());

    let mut recent = ini.with_section(Some(RECENT));
    for (index, file) in config.recent_files.iter().take(MAX_RECENT).enumerate() {
        recent.set(
            format!("file{}", index + 1),
            file.to_string_lossy().as_ref(),
        );
    }

    ini.write_to_file(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_recent_moves_a_repeat_to_the_front() {
        let mut config = Config::default();
        config.push_recent(Path::new("/a.qpy"));
        config.push_recent(Path::new("/b.qpy"));
        config.push_recent(Path::new("/a.qpy"));

        assert_eq!(
            config.recent_files,
            vec![PathBuf::from("/a.qpy"), PathBuf::from("/b.qpy")]
        );
    }

    #[test]
    fn push_recent_caps_the_list() {
        let mut config = Config::default();
        for index in 0..(MAX_RECENT + 3) {
            config.push_recent(Path::new(&format!("/{index}.qpy")));
        }

        assert_eq!(config.recent_files.len(), MAX_RECENT);
        assert_eq!(config.recent_files[0], PathBuf::from("/7.qpy"));
    }

    #[test]
    fn keywords_round_trip() {
        for theme in THEMES {
            assert_eq!(parse_theme(theme_keyword(theme)), Some(theme));
        }
        for format in ExportFormat::ALL {
            assert_eq!(ExportFormat::parse(format.keyword()), Some(format));
        }
        for level in LogLevel::ALL {
            assert_eq!(LogLevel::parse(level.keyword()), Some(level));
        }
    }

    #[test]
    fn nonsense_values_fall_back_to_the_default() {
        let ini = Ini::load_from_str("[general]\ntheme=purple\n[system]\nlog_level=chatty\n")
            .expect("test ini parses");

        assert_eq!(
            read(
                ini.section(Some(GENERAL)),
                THEME,
                parse_theme,
                ThemePreference::default()
            ),
            ThemePreference::System
        );
        assert_eq!(
            read(
                ini.section(Some(SYSTEM)),
                LOG_LEVEL,
                LogLevel::parse,
                LogLevel::default()
            ),
            LogLevel::Info
        );
    }
}

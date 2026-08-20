use std::path::{Path, PathBuf};

use crate::quirpy_config;
use crate::quirpy_front::app::{ErrorPrompt, QuirpyApp};
use crate::quirpy_project::{self, ProjectFileError};

const EXTENSION: &str = "qpy";
const FILTER: &str = "Quirpy project";

pub fn new_project(app: &mut QuirpyApp) {
    app.project = Default::default();
    app.saved_state = app.project.clone();
    app.current_path = None;
    app.history.reset(&app.project);
    tracing::info!("new project (form reset to defaults)");
}

pub fn save(app: &mut QuirpyApp) -> bool {
    let Some(path) = app.current_path.clone() else {
        return save_as(app);
    };
    write(app, &path)
}

pub fn save_as(app: &mut QuirpyApp) -> bool {
    let name = if app.project.name.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        app.project.name.trim().to_owned()
    };

    let mut dialog = rfd::FileDialog::new()
        .add_filter(FILTER, &[EXTENSION])
        .set_file_name(format!("{name}.{EXTENSION}"));
    if let Some(dir) = app.config.default_save_location.as_ref() {
        dialog = dialog.set_directory(dir);
    }

    let Some(path) = dialog.save_file() else {
        return false;
    };

    write(app, with_extension(path).as_path())
}

pub fn open(app: &mut QuirpyApp) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter(FILTER, &[EXTENSION])
        .pick_file()
    else {
        return;
    };
    open_path(app, &path);
}

pub fn open_path(app: &mut QuirpyApp, path: &Path) {
    match quirpy_project::load(path) {
        Ok(project) => adopt(app, project, path),
        Err(ProjectFileError::ChecksumMismatch) => {
            tracing::warn!(?path, "checksum mismatch");
            app.error = Some(ErrorPrompt {
                message: ProjectFileError::ChecksumMismatch.to_string(),
                open_anyway: Some(path.to_path_buf()),
            });
        }
        Err(error) => {
            tracing::warn!(?path, %error, "could not open project");
            app.error = Some(ErrorPrompt {
                message: error.to_string(),
                open_anyway: None,
            });
        }
    }
}

pub fn open_path_ignoring_checksum(app: &mut QuirpyApp, path: &Path) {
    match quirpy_project::load_ignoring_checksum(path) {
        Ok(project) => {
            tracing::warn!(?path, "opened a project with a failing checksum");
            adopt(app, project, path);
        }
        Err(error) => {
            tracing::warn!(?path, %error, "could not open project");
            app.error = Some(ErrorPrompt {
                message: error.to_string(),
                open_anyway: None,
            });
        }
    }
}

pub fn title(app: &QuirpyApp) -> String {
    let name = if app.project.name.trim().is_empty() {
        app.current_path
            .as_deref()
            .and_then(Path::file_stem)
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_owned())
    } else {
        app.project.name.trim().to_owned()
    };

    if app.is_dirty() {
        format!("Quirpy — {name} •")
    } else {
        format!("Quirpy — {name}")
    }
}

fn adopt(app: &mut QuirpyApp, project: crate::quirpy_front::form::ProjectState, path: &Path) {
    app.project = project;
    app.saved_state = app.project.clone();
    app.current_path = Some(path.to_path_buf());
    app.history.reset(&app.project);
    remember(app, path);
    tracing::info!(?path, "project opened");
}

fn write(app: &mut QuirpyApp, path: &Path) -> bool {
    match quirpy_project::save(&app.project, path) {
        Ok(()) => {
            app.saved_state = app.project.clone();
            app.current_path = Some(path.to_path_buf());
            remember(app, path);
            tracing::info!(?path, "project saved");
            true
        }
        Err(error) => {
            tracing::error!(?path, %error, "could not save project");
            app.error = Some(ErrorPrompt {
                message: format!("Could not save this project: {error}"),
                open_anyway: None,
            });
            false
        }
    }
}

fn remember(app: &mut QuirpyApp, path: &Path) {
    app.config.push_recent(path);
    app.recents_dirty = true;
    if let Err(error) = quirpy_config::save(&app.config) {
        tracing::warn!(%error, "could not write configuration");
    }
}

fn with_extension(path: PathBuf) -> PathBuf {
    if path.extension().is_some() {
        path
    } else {
        path.with_extension(EXTENSION)
    }
}

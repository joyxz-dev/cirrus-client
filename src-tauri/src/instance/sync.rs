use std::path::Path;
use tauri::AppHandle;

use crate::error::CirrusError;
use crate::instance::{cirrus_data_dir, instance_dir, Instance};

const DEFAULT_OPTIONS: &str = "version:3\nsound_device:\nsensitivity:0.5\n";

pub fn setup_sync(app: &AppHandle, instance: &Instance) -> Result<(), CirrusError> {
    if instance.sync_options {
        enable_sync(app, &instance.id)
    } else {
        Ok(())
    }
}

pub fn enable_sync(app: &AppHandle, instance_id: &str) -> Result<(), CirrusError> {
    let data_dir = cirrus_data_dir(app)?;
    let global_opts = data_dir.join("settings").join("global_options.txt");
    let inst_dir = instance_dir(app, instance_id)?;
    let inst_opts = inst_dir.join("options.txt");

    std::fs::create_dir_all(global_opts.parent().unwrap())?;
    if !global_opts.exists() {
        std::fs::write(&global_opts, DEFAULT_OPTIONS)?;
    }

    // Remove existing file or symlink before creating the new one
    if inst_opts.exists() || inst_opts.symlink_metadata().is_ok() {
        std::fs::remove_file(&inst_opts)?;
    }

    create_symlink(&global_opts, &inst_opts)?;
    Ok(())
}

pub fn disable_sync(app: &AppHandle, instance_id: &str) -> Result<(), CirrusError> {
    let data_dir = cirrus_data_dir(app)?;
    let global_opts = data_dir.join("settings").join("global_options.txt");
    let inst_dir = instance_dir(app, instance_id)?;
    let inst_opts = inst_dir.join("options.txt");

    // Copy the global file into the instance as a real file
    let content = if global_opts.exists() {
        std::fs::read_to_string(&global_opts)?
    } else {
        DEFAULT_OPTIONS.to_string()
    };

    if inst_opts.symlink_metadata().is_ok() {
        std::fs::remove_file(&inst_opts)?;
    }
    std::fs::write(&inst_opts, content)?;
    Ok(())
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<(), CirrusError> {
    std::os::unix::fs::symlink(target, link).map_err(Into::into)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> Result<(), CirrusError> {
    // Try symlink first (requires Developer Mode or admin)
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fallback: copy the file
            std::fs::copy(target, link)?;
            Ok(())
        }
    }
}

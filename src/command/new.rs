////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line dispatching.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::PositionSelector;
use crate::command::CommonOptions;
use crate::command::Script;
use crate::error::FileError;
use crate::error::FileErrorContext as _;
use crate::palette::Palette;
use crate::setup::Config;
use crate::setup::Settings;

// External library imports.
use log::*;

// Standard library imports.
use std::path::PathBuf;



/// Returns true if a FileError has ErrorKind::AlreadyExists.
pub fn already_exists(e: &FileError) -> bool {
    e.is_io_error_kind(std::io::ErrorKind::AlreadyExists)
}

/// Creates a new palette.
fn create_new_palette(
    path: PathBuf,
    no_history: bool,
    name: Option<String>)
    -> Result<Palette, anyhow::Error>
{
    let mut palette = Palette::new().with_load_path(path);

    if !no_history { palette = palette.with_history(); }
    if let Some(name) = name {
        let _ = palette.inner_mut().assign_name(name, PositionSelector::ALL)?;
    }
    Ok(palette)
}

/// Initializes a new palette file.
pub fn new_palette(
    script_path: Option<PathBuf>,
    path: PathBuf,
    set_active: bool,
    no_history: bool,
    name: Option<String>,
    common: &CommonOptions,
    config: &Config,
    settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    let mut settings_changed = false;
    let mut palette = create_new_palette(path, no_history, name)?;

    if set_active {
        settings.active_palette = palette
            .load_path()
            .map(ToOwned::to_owned);
        settings_changed = true;
    }

    if let Some(script_path) = script_path {
        let script = Script::read_from_path(script_path)?;
        let _ = script.execute(&mut palette, common, config, settings)?;

    }

    if settings_changed {
        let _ = settings.write_to_load_path()?;
    }

    let res = palette.write_to_load_path_if_new();
    if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
        info!("Palette file already exists.");
        debug!("Palette {:?}", palette.load_path());
    } else {
        let _ = res.with_context(|| 
            if let Some(path) = palette.load_path() {
                format!("Error writing palette file: {}", path.display())
            } else {
                format!("Error writing palette file")
            })?;
    }
    Ok(())
}

/// Initializes a new config file.
pub fn new_config(path: PathBuf) -> Result<(), FileError> {
    let new = Config::new().with_load_path(path);

    let res = new.write_to_load_path_if_new();
    if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
        info!("Config file already exists.");
        debug!("Config {:?}", new.load_path());
    } else {
        let _ = res.with_context(|| 
            if let Some(path) = new.load_path() {
                format!("Error writing config file: {}", path.display())
            } else {
                format!("Error writing config file")
            })?;
    }
    Ok(())
}

/// Initializes a new settings file.
pub fn new_settings(path: PathBuf) -> Result<(), FileError> {
    let new = Settings::new().with_load_path(path);

    let res = new.write_to_load_path_if_new();
    if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
        info!("Settings file already exists.");
        debug!("Settings {:?}", new.load_path());
    } else {
        let _ = res.with_context(|| 
            if let Some(path) = new.load_path() {
                format!("Error writing settings file: {}", path.display())
            } else {
                format!("Error writing settings file")
            })?;
    }
    Ok(())
}

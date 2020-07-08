////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line dispatching.
////////////////////////////////////////////////////////////////////////////////
#![allow(unused)] // TODO: Remove this.

// Local imports.
use crate::command::AtmaOptions;
use crate::command::InsertOption;
use crate::command::CommandOption;
use crate::error::PaletteError;
use crate::error::ParseError;
use crate::error::FileError;
use crate::palette::Palette;
use crate::cell::PositionSelector;
use crate::color::Color;
use crate::parse::ParseResultExt as _;
use crate::parse::color;
use crate::Config;
use crate::Settings;

// External library imports.
use log::*;

// Standard library imports.
use std::path::PathBuf;
use std::path::Path;



fn parse_color(text: String) -> Result<Color, ParseError> {
    color(&text[..])
        .finish()
        .map_err(ParseError::from)
}

////////////////////////////////////////////////////////////////////////////////
// dispatch
////////////////////////////////////////////////////////////////////////////////
/// Executes the given `AtmaOptions` on the given `Palette`.
pub fn dispatch(
    mut palette: Palette,
    opts: AtmaOptions,
    config: Config,
    settings: Settings)
    -> Result<(), anyhow::Error>
{
    use CommandOption::*;
    use anyhow::Context as _;

    if opts.common.dry_run {
        // TODO: Implement this.
        println!("Dry run is currently unsupported.");
        return Ok(());
    }

    match opts.command {
        None => unimplemented!(),

        Some(command) => match command {
            New {
                name,
                no_history,
                no_config_file,
                no_settings_file,
                set_active,
            } => new_palette(
                    palette,
                    name,
                    no_history,
                    if no_config_file { None } else { Some(config) },
                    if no_settings_file { None } else { Some(settings) },
                    set_active)
                .with_context(|| "Command 'new' failed"),

            List => unimplemented!(),
            Insert { insert_options } => match insert_options {
                InsertOption::Colors { colors, name, at } => {
                    let colors: Vec<Color> = colors
                        .into_iter()
                        .map(parse_color)
                        .collect::<Result<Vec<_>,_>>()?;

                    let res = palette.insert_colors(&colors[..], name, at);
                    info!("{:?}", palette);
                    res.with_context(|| "Command 'insert' failed")
                },

                InsertOption::Ramp { ..}=> //points, count, interpolate, name, at } => 
                {
                    unimplemented!()
                },
            },
            Delete => unimplemented!(),
            Move => unimplemented!(),
            Set => unimplemented!(),
            Unset => unimplemented!(),
            Undo { count } => {
                let performed = palette.undo(count);
                println!("{} undo operations performed.", performed);
                Ok(())
            },
            Redo { count } => {
                let performed = palette.redo(count);
                println!("{} redo operations performed.", performed);
                Ok(())
            },
            Import => unimplemented!(),
            Export => unimplemented!(),
        },
    }
}


/// Initializes a new palette.
fn new_palette(
    mut palette: Palette,
    name: Option<String>,
    no_history: bool,
    config: Option<Config>,
    mut settings: Option<Settings>,
    set_active: bool)
    -> Result<(), FileError>
{
    use crate::error::FileErrorContext as _;

    fn already_exists(e: &FileError) -> bool {
        e.is_io_error_kind(std::io::ErrorKind::AlreadyExists)
    }

    if !no_history { palette = palette.with_history(); }
    if let Some(name) = name {
        palette.inner_mut().assign_name(name, PositionSelector::ALL);
    }

    if let Some(config) = config {
        let res = config.write_to_load_path();
        if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
            info!("Config file already exists.");
            debug!("Config {:?}", config.load_path());
        } else {
            let _ = res.with_context(|| 
                if let Some(path) = config.load_path() {
                    format!("Error writing config file: {}", path.display())
                } else {
                    format!("Error writing config file")
                })?;
        }
    }

    if let Some(mut settings) = settings {
        if set_active {
            settings.active_palette = palette
                .load_path()
                .map(ToOwned::to_owned);
        }
        
        let res = settings.write_to_load_path();
        if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
            info!("Settings file already exists.");
            debug!("Settings {:?}", settings.load_path());
        } else {
            let _ = res.with_context(|| 
                if let Some(path) = settings.load_path() {
                    format!("Error writing settings file: {}", path.display())
                } else {
                    format!("Error writing settings file")
                })?;
        }
    }

    let res = palette.write_to_load_path();
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

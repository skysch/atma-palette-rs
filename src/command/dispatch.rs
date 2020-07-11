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
use crate::cell::CellRef;
use crate::cell::PositionSelector;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::color::Color;
use crate::command::AtmaOptions;
use crate::command::CommandOption;
use crate::command::InsertOption;
use crate::command::ExportOption;
use crate::Config;
use crate::DEFAULT_PALETTE_PATH;
use crate::error::FileError;
use crate::error::PaletteError;
use crate::error::ParseError;
use crate::palette::Palette;
use crate::parse::color;
use crate::parse::ParseResultExt as _;
use crate::Settings;

// External library imports.
use log::*;
use anyhow::anyhow;

// Standard library imports.
use std::path::PathBuf;
use std::path::Path;

/// Error message returned when no active palette is loaded.
const NO_PALETTE: &'static str = "No active palette loaded.";


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
    mut palette: Option<Palette>,
    opts: AtmaOptions,
    config: Config,
    settings: Settings,
    cur_dir: PathBuf)
    -> Result<(), anyhow::Error>
{
    use CommandOption::*;
    use anyhow::Context as _;

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
                    palette.unwrap_or(Palette::new()
                        .with_load_path(cur_dir.join(DEFAULT_PALETTE_PATH))),
                    name,
                    no_history,
                    if no_config_file { None } else { Some(config) },
                    if no_settings_file { None } else { Some(settings) },
                    set_active)
                .context("Command 'new' failed"),

            List { selection } => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                debug!("Start listing for selection {:?}", selection);
                let selection = selection.unwrap_or(CellSelector::All.into());
                let index_selection = selection.resolve(pal.inner());
                debug!("Start listing for {:?}", index_selection);

                for idx in index_selection {
                    if let Ok(Some(c)) = pal.inner()
                        .color(&CellRef::Index(idx))
                    {
                        println!("{:4X} {:X}", idx, c);
                    }
                }
                Ok(())
            },

            Insert { insert_option } => match insert_option {
                InsertOption::Colors { colors, name, at } => {
                    let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                    let colors: Vec<Color> = colors
                        .into_iter()
                        .map(parse_color)
                        .collect::<Result<Vec<_>,_>>()?;

                    let res = pal.insert_colors(&colors[..], name, at);
                    debug!("{:?}", pal);

                    res.context("Command 'insert' failed")?;
                    pal.write_to_load_path()
                        .map(|_| ())
                        .context("Failed to write palette")
                },

                InsertOption::Ramp { ..} => unimplemented!(),
            },
            Delete => unimplemented!(),
            Move => unimplemented!(),
            Set => unimplemented!(),
            Unset => unimplemented!(),
            Undo { count } => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let performed = pal.undo(count);
                println!("{} undo operations performed.", performed);
                Ok(())
            },
            Redo { count } => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let performed = pal.redo(count);
                println!("{} redo operations performed.", performed);
                Ok(())
            },
            Import => unimplemented!(),
            Export { export_option } => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                match export_option {
                    ExportOption::Png { selection, output } => {
                        write_png(
                            &pal,
                            selection.unwrap_or(CellSelector::All.into()),
                            &cur_dir.clone().join(output))
                    },
                }
            },
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
        let res = config.write_to_load_path_if_new();
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
        
        let res = settings.write_to_load_path_if_new();
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


#[cfg(feature = "png")]
fn write_png<'a>(palette: &Palette, selection: CellSelection<'a>, path: &Path)
    -> Result<(), anyhow::Error>
{
    let mut pal_data = Vec::new();
    let index_selection = selection.resolve(palette.inner());    
    for idx in index_selection {
        if let Ok(Some(c)) = palette.inner().color(&CellRef::Index(idx)) {
            pal_data.extend(&c.rgb_octets());
        }
    }

    let file = std::fs::File::create(path)?;
    let ref mut w = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1, 1);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(pal_data);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&[0])?;
    println!("Palette exported to {}", path.display());
    Ok(())
}

#[cfg(not(feature = "png"))]
fn write_png<'a>(palette: &Palette, selection: CellSelection<'a>, path: &Path)
    -> Result<(), anyhow::Error>
{
    Err(anyhow!("Export using PNG format is unsupported."))
}

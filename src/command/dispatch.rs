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
use crate::cell::CellRef;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::PositionSelector;
use crate::command::AtmaOptions;
use crate::command::CommandOption;
use crate::command::ExportOption;
use crate::command::NewOption;
use crate::command::SetOption;
use crate::Config;
use crate::DEFAULT_CONFIG_PATH;
use crate::error::FileError;
use crate::palette::Palette;
use crate::Settings;
use crate::utility::normalize_path;

// External library imports.
use log::*;
use anyhow::anyhow;

// Standard library imports.
use std::path::PathBuf;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////

/// Error message returned when no active palette is loaded.
const NO_PALETTE: &'static str = "No active palette loaded.";


////////////////////////////////////////////////////////////////////////////////
// dispatch
////////////////////////////////////////////////////////////////////////////////
/// Executes the given `AtmaOptions` on the given `Palette`.
pub fn dispatch(
    palette: Option<Palette>,
    opts: AtmaOptions,
    config: Config,
    mut settings: Settings,
    cur_dir: PathBuf)
    -> Result<(), anyhow::Error>
{
    trace!("Begin command dispatch.");
    use CommandOption::*;
    use anyhow::Context as _;

    match opts.command {

        // New
        ////////////////////////////////////////////////////////////////////////
        New { new_option } => match new_option {
            NewOption::Palette { path, set_active, no_history, name } => {
                new_palette(
                        normalize_path(
                            cur_dir.clone(),
                            path.unwrap_or_else(|| cur_dir
                                .join(&config.default_palette_path))),
                        set_active,
                        no_history,
                        name,
                        &mut settings)
                    .context("Command 'new palette' failed")
            },

            NewOption::Config { path } => new_config(
                    normalize_path(
                        cur_dir.clone(),
                        path.unwrap_or_else(
                            || cur_dir.join(dbg!(DEFAULT_CONFIG_PATH))),
                        ))
                .context("Command 'new config' failed"),

            NewOption::Settings { path } => new_settings(
                    normalize_path(
                        cur_dir.clone(),
                        path.unwrap_or_else(|| cur_dir
                            .join(&config.default_settings_path))))
                .context("Command 'new settings' failed"),
        },

        // List
        ////////////////////////////////////////////////////////////////////////
        List { selection } => {
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

            // TODO: Move all of this to an inner function.
            debug!("Start listing for selection {:?}", selection);
            let selection = selection.unwrap_or(CellSelector::All.into());
            let index_selection = selection.resolve(pal.inner());
            debug!("Start listing for {:?}", index_selection);

            for idx in index_selection {
                if let Ok(Some(c)) = pal.inner()
                    .color(&CellRef::Index(idx))
                {
                    print!("{:4X} {:X}", idx, c);
                    if let Some(pos) = pal.inner()
                        .assigned_position(&CellRef::Index(idx))
                    {
                        print!(" {}", pos);
                    }
                    if let Some(name) = pal.inner()
                        .assigned_name(&CellRef::Index(idx))
                    {
                        print!(" \"{}\"", name);
                    }
                    println!();
                } else {
                    println!("{:4X} invalid color", idx);
                }
            }
            Ok(())
        },

        // Insert
        ////////////////////////////////////////////////////////////////////////
        Insert { exprs, name, at } => {
            let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            if exprs.is_empty() {
                println!("No expressions to insert.");
                return Ok(()); 
            }
            let at = at.unwrap_or(config.default_positioning);
            let cursor_behavior = settings
                .insert_cursor_behavior
                .unwrap_or(config.default_insert_cursor_behavior);

            pal.insert_exprs(&exprs[..], name, at, cursor_behavior)
                .context("insert command failed.")?;

            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette")
        },

        // Delete
        ////////////////////////////////////////////////////////////////////////
        Delete { selection } => match selection {
            Some(selection) => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let cursor_behavior = settings
                    .delete_cursor_behavior
                    .unwrap_or(config.default_delete_cursor_behavior);
                pal.delete_selection(selection, cursor_behavior)
                    .context("delete command failed.")?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },
            None => {
                println!("No cell selection; nothing to delete.");
                Ok(())
            },
        },

        // Move
        ////////////////////////////////////////////////////////////////////////
        Move { selection, to } => match selection {
            Some(selection) => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let to = to.unwrap_or(config.default_positioning);
                let cursor_behavior = settings
                    .move_cursor_behavior
                    .unwrap_or(config.default_move_cursor_behavior);
                pal.move_selection(selection, to, cursor_behavior)?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },
            None => {
                println!("No cell selection; nothing to move.");
                Ok(())
            },
        },

        // Set
        ////////////////////////////////////////////////////////////////////////
        Set { set_option } => match set_option {
            SetOption::Name { position_selector, name } => unimplemented!(),

            SetOption::Cursor { position } => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let _ = pal.set_position_cursor(position);
                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")                
            }

            SetOption::History { history_set_option } => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                pal.set_history(history_set_option);
                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },

            SetOption::ActivePalette { path } => {
                if let Some(path) = path {
                    settings.active_palette = Some(normalize_path(
                        cur_dir,
                        path));
                } else {
                    settings.active_palette = None;
                }

                settings.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write settings file")
            },

            SetOption::DeleteCursorBehavior { cursor_behavior } => {
                settings.delete_cursor_behavior = cursor_behavior;
                settings.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write settings file")
            },

            SetOption::InsertCursorBehavior { cursor_behavior } => {
                settings.insert_cursor_behavior = cursor_behavior;
                settings.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write settings file")
            },

            SetOption::MoveCursorBehavior { cursor_behavior } => {
                settings.move_cursor_behavior = cursor_behavior;
                settings.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write settings file")
            },
        },

        // Undo
        ////////////////////////////////////////////////////////////////////////
        Undo { count } => {
            let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            let count = count.unwrap_or(1);
            if count == 0 {
                println!("0 undo operations performed.");
                return Ok(());
            };
            let performed = pal.undo(count);
            match performed {
                0 => {
                    println!("No undo operations recorded.");
                    return Ok(());
                },
                1 => println!("Undo operation completed."),
                _ => println!("{} undo operations performed.", performed),
            }
            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette")
        },

        // Redo
        ////////////////////////////////////////////////////////////////////////
        Redo { count } => {
            let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            let count = count.unwrap_or(1);
            if count == 0 {
                println!("0 redo operations performed.");
                return Ok(());
            };
            let performed = pal.redo(count);
            match performed {
                0 => {
                    println!("No redo operations recorded.");
                    return Ok(());
                },
                1 => println!("Redo operation completed."),
                _ => println!("{} redo operations performed.", performed),
            }
            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette")
        },

        // Import
        ////////////////////////////////////////////////////////////////////////
        Import => unimplemented!(),

        // Export
        ////////////////////////////////////////////////////////////////////////
        Export { export_option } => {
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            match export_option {
                ExportOption::Png { selection, output } => {
                    write_png(
                        &pal,
                        selection.unwrap_or(CellSelector::All.into()),
                        &cur_dir.clone().join(output))
                },
            }
        },
    }
}


////////////////////////////////////////////////////////////////////////////////
// New command support
////////////////////////////////////////////////////////////////////////////////

/// Returns true if a FileError has ErrorKind::AlreadyExists.
fn already_exists(e: &FileError) -> bool {
    e.is_io_error_kind(std::io::ErrorKind::AlreadyExists)
}

/// Initializes a new palette.
fn new_palette(
    path: PathBuf,
    set_active: bool,
    no_history: bool,
    name: Option<String>,
    settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    use crate::error::FileErrorContext as _;

    let mut palette = Palette::new().with_load_path(path);

    if set_active {
        settings.active_palette = palette
            .load_path()
            .map(ToOwned::to_owned);
        let _ = settings.write_to_load_path()?;
    }

    if !no_history { palette = palette.with_history(); }
    if let Some(name) = name {
        let _ = palette.inner_mut().assign_name(name, PositionSelector::ALL)?;
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
fn new_config(path: PathBuf) -> Result<(), FileError> {
    use crate::error::FileErrorContext as _;

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
fn new_settings(path: PathBuf) -> Result<(), FileError> {
    use crate::error::FileErrorContext as _;

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

////////////////////////////////////////////////////////////////////////////////
// Export command support
////////////////////////////////////////////////////////////////////////////////

#[cfg(not(feature = "png"))]
fn write_png<'a>(palette: &Palette, selection: CellSelection<'a>, path: &Path)
    -> Result<(), anyhow::Error>
{
    Err(anyhow!("Export using PNG format is unsupported."))
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

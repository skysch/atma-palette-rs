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
use crate::cell::CellSelector;
use crate::command::AtmaOptions;
use crate::command::CommandOption;
use crate::command::ExportOption;
use crate::command::NewOption;
use crate::command::SetOption;
use crate::command::new::new_config;
use crate::command::new::new_settings;
use crate::command::new::new_palette;
use crate::command::export_png::write_png;
use crate::Config;
use crate::DEFAULT_CONFIG_PATH;
use crate::palette::Palette;
use crate::Settings;
use crate::utility::normalize_path;

// External library imports.
use log::*;
use anyhow::anyhow;

// Standard library imports.
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
    palette: Option<&mut Palette>,
    opts: AtmaOptions,
    config: &Config,
    settings: &mut Settings,
    cur_dir: Option<&Path>)
    -> Result<(), anyhow::Error>
{
    trace!("Begin command dispatch.");
    use CommandOption::*;
    use anyhow::Context as _;

    match opts.command {

        // New
        ////////////////////////////////////////////////////////////////////////
        New { new_option } => match new_option {
            NewOption::Script {
                script_path,
                path,
                set_active,
                no_history,
                name,
            } => {
                new_palette(
                        Some(script_path),
                        normalize_path(
                            cur_dir
                                .expect("Currend directory not determined")
                                .clone(),
                            path.unwrap_or_else(|| cur_dir
                                .expect("Currend directory not determined")
                                .join(&config.default_palette_path))),
                        set_active,
                        no_history,
                        name,
                        config,
                        settings)
                    .context("Command 'new script' failed")
            },

            NewOption::Palette { path, set_active, no_history, name } => {
                new_palette(
                        None,
                        normalize_path(
                            cur_dir
                                .expect("Currend directory not determined")
                                .clone(),
                            path.unwrap_or_else(|| cur_dir
                                .expect("Currend directory not determined")
                                .join(&config.default_palette_path))),
                        set_active,
                        no_history,
                        name,
                        config,
                        settings)
                    .context("Command 'new palette' failed")
            },

            NewOption::Config { path } => new_config(
                    normalize_path(
                        cur_dir
                            .expect("Currend directory not determined")
                            .clone(),
                        path.unwrap_or_else(|| cur_dir
                                .expect("Currend directory not determined")
                                .join(dbg!(DEFAULT_CONFIG_PATH))),
                        ))
                .context("Command 'new config' failed"),

            NewOption::Settings { path } => new_settings(
                    normalize_path(
                        cur_dir
                            .expect("Currend directory not determined")
                            .clone(),
                        path.unwrap_or_else(|| cur_dir
                            .expect("Currend directory not determined")
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

                    for group in pal.inner()
                        .assigned_groups(&CellRef::Index(idx))?
                    {
                        print!(" \"{}\"", group);
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
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
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
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
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
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
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
            SetOption::Name { position_selector, name } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

                pal.set_name(name, position_selector)?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },

            SetOption::Group { selection, name, remove } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

                pal.set_group(name, selection, remove)?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },

            SetOption::Expr { at, expr } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

                let mut exprs = expr.exprs(pal.inner())?;
                if exprs.len() > 1 {
                    return Err(anyhow!("The `set` command does not support \
                        ramp expressions."));
                }
                
                *pal.cell_mut(&at)?.expr_mut() = exprs.pop()
                    .ok_or(anyhow!("No expression to set."))?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },

            SetOption::Cursor { position } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let _ = pal.set_position_cursor(position);
                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            }

            SetOption::History { history_set_option } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                pal.set_history(history_set_option);
                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },

            SetOption::ActivePalette { path } => {
                if let Some(path) = path {
                    settings.active_palette = Some(normalize_path(
                        cur_dir.expect("Currend directory not determined"),
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
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
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
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
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
                        &cur_dir.expect("Currend directory not determined")
                            .clone()
                            .join(output))
                },
            }
        },
    }
}


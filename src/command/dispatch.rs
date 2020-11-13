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
use crate::cell::CellSelector;
use crate::command::ColorDisplay;
use crate::command::ColorStyle;
use crate::command::CommandOption;
use crate::command::CommonOptions;
use crate::command::export_png::write_png;
use crate::command::ExportOption;
use crate::command::list::list;
use crate::command::new::new_config;
use crate::command::new::new_palette;
use crate::command::new::new_settings;
use crate::command::NewOption;
use crate::command::SetOption;
use crate::command::TextStyle;
use crate::palette::Palette;
use crate::setup::Config;
use crate::setup::DEFAULT_CONFIG_PATH;
use crate::setup::Settings;
use crate::utility::normalize_path;

// External library imports.
use anyhow::anyhow;
use tracing::debug;
use tracing::Level;
use tracing::span;
use tracing::trace;
use tracing::warn;


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
    command: CommandOption,
    common: &CommonOptions,
    config: &Config,
    settings: &mut Settings,
    cur_dir: Option<&Path>)
    -> Result<(), anyhow::Error>
{
    let span = span!(Level::TRACE, "dispatch");
    let _enter = span.enter();

    use CommandOption::*;
    use anyhow::Context as _;

    match command {

        // New
        ////////////////////////////////////////////////////////////////////////
        New { new_option } => match new_option {
            NewOption::Palette {
                path,
                script_path,
                set_active,
                no_history,
                overwrite,
                name,
            } => {
                new_palette(
                        script_path,
                        normalize_path(
                            cur_dir
                                .expect("Current directory not determined")
                                .clone(),
                            path.unwrap_or_else(|| cur_dir
                                .expect("Current directory not determined")
                                .join(&config.default_palette_path))),
                        set_active,
                        no_history,
                        overwrite,
                        name,
                        common,
                        config,
                        settings)
                    .context("Command 'new palette' failed")
            },

            NewOption::Config { path, overwrite } => new_config(
                    normalize_path(
                        cur_dir
                            .expect("Current directory not determined")
                            .clone(),
                        path.unwrap_or_else(|| cur_dir
                                .expect("Current directory not determined")
                                .join(dbg!(DEFAULT_CONFIG_PATH))),
                        ),
                    overwrite)
                .context("Command 'new config' failed"),

            NewOption::Settings { path, overwrite } => new_settings(
                    normalize_path(
                        cur_dir
                            .expect("Current directory not determined")
                            .clone(),
                        path.unwrap_or_else(|| cur_dir
                            .expect("Current directory not determined")
                            .join(&config.default_settings_path))),
                    overwrite)
                .context("Command 'new settings' failed"),
        },

        // List
        ////////////////////////////////////////////////////////////////////////
        List {
            selection,
            mode,
            color_style,
            text_style,
            rule_style,
            line_style,
            gutter_style,
            max_width,
            max_height,
            max_columns,
            no_color,
        } => {
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            let mode = mode.unwrap_or(config.default_list_mode);
            let color_display = if let 
                (Some(ColorStyle::None), Some(TextStyle::None))
                    = (color_style, text_style) 
            {
                warn!("Invalid combination of color-style and text-style.\
                    Using --text-style hex.");
                config.invalid_color_display_fallback
            } else {
                ColorDisplay {
                    color_style: color_style
                        .unwrap_or(config.default_list_color_style),
                    text_style: text_style
                        .unwrap_or(config.default_list_text_style),
                }
            };

            let rule_style = rule_style
                .unwrap_or(config.default_list_rule_style);
            let line_style = line_style
                .unwrap_or(config.default_list_line_style);
            let gutter_style = gutter_style
                .unwrap_or(config.default_list_gutter_style);

            #[cfg(feature = "termsize")]
            let (w, h) = {
                termsize::get()
                    .map(|sz| (sz.cols, sz.rows))
                    .unwrap_or_else(|| {
                        let w = max_width.unwrap_or(120);
                        let h = max_height.unwrap_or(40);
                        (w, h)
                    })
            };
            #[cfg(not(feature = "termsize"))]
            let (w, h) = {
                let w = max_width.unwrap_or(120);
                let h = max_height.unwrap_or(40);
                (w, h)
            };

            list(
                &pal,
                selection,
                mode,
                (w, h),
                color_display,
                rule_style,
                line_style,
                gutter_style,
                no_color,
                max_columns,
                config,
                settings)
        },

        // Insert
        ////////////////////////////////////////////////////////////////////////
        Insert { exprs, name, at } => {
            debug!(" Insert {{ exprs: {:?}, name: {:?}, at: {:?} }}",
                exprs, name, at);
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

            pal.set_modified(true);
            Ok(())
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

                pal.set_modified(true);
                Ok(())
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

                pal.set_modified(true);
                Ok(())
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

                pal.set_modified(true);
                Ok(())
            },

            SetOption::Group { selection, name, remove } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

                pal.set_group(name, selection, remove)?;

                pal.set_modified(true);
                Ok(())
            },

            SetOption::Expr { at, expr } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

                let mut exprs = expr.exprs(pal.inner())?;
                if exprs.len() > 1 {
                    return Err(anyhow!("The `set` command does not support \
                        ramp expressions."));
                }
                let expr = exprs.pop()
                    .ok_or(anyhow!("No expression to set."))?;

                pal.set_expr(at, expr)?;
                pal.set_modified(true);
                Ok(())
            },

            SetOption::Cursor { position } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let _ = pal.set_position_cursor(position);
                pal.set_modified(true);
                Ok(())
            }

            SetOption::History { history_set_option } => {
                let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                pal.set_history_option(history_set_option);
                pal.set_modified(true);
                Ok(())
            },

            SetOption::ActivePalette { path } => {
                if let Some(path) = path {
                    settings.active_palette = Some(normalize_path(
                        cur_dir.expect("Current directory not determined"),
                        path));
                } else {
                    settings.active_palette = None;
                }

                settings.set_modified(true);
                Ok(())
            },

            SetOption::DeleteCursorBehavior { cursor_behavior } => {
                settings.delete_cursor_behavior = cursor_behavior;
                settings.set_modified(true);
                Ok(())
            },

            SetOption::InsertCursorBehavior { cursor_behavior } => {
                settings.insert_cursor_behavior = cursor_behavior;
                settings.set_modified(true);
                Ok(())
            },

            SetOption::MoveCursorBehavior { cursor_behavior } => {
                settings.move_cursor_behavior = cursor_behavior;
                settings.set_modified(true);
                Ok(())
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
                        &cur_dir.expect("Current directory not determined")
                            .clone()
                            .join(output))
                },
            }
        },
    }
}


////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Module for the `list` command.
////////////////////////////////////////////////////////////////////////////////


// Internal module imports.
use crate::palette::Palette;
use crate::command::ColorDisplay;
use crate::setup::Config;
use crate::setup::Settings;
use crate::error::PaletteError;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::CellRef;
use crate::cell::Position;

// External module imports.
use log::*;

/// Prints palette information.
pub fn list<'a>(
    palette: &Palette,
    selection: Option<CellSelection<'a>>,
    _config: &Config,
    _settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    debug!("Start listing for selection {:?}", selection);
    let selection = selection.unwrap_or(CellSelector::All.into());
    let index_selection = selection.resolve(palette.inner());
    debug!("Start listing for {:?}", index_selection);

    for idx in index_selection {
        if let Ok(Some(c)) = palette.inner()
            .color(&CellRef::Index(idx))
        {
            print!("{:4X} ", idx);
            ColorDisplay::Tile.print(c);

            if let Some(pos) = palette.inner()
                .assigned_position(&CellRef::Index(idx))
            {
                print!(" {}", pos);
            }
            if let Some(name) = palette.inner()
                .assigned_name(&CellRef::Index(idx))
            {
                print!(" \"{}\"", name);
            }

            for group in palette.inner()
                .assigned_groups(&CellRef::Index(idx))?
            {
                print!(" \"{}\"", group);
            }
            
        } else {
            print!("{:4X} ", idx);
            ColorDisplay::Tile.print_invalid();

        }
        println!();
    }
    Ok(())
}


/// List palette cells in a grid.
pub fn list_grid<'a>(
    palette: &Palette,
    selection: Option<CellSelection<'a>>,
    size: (u16, u16),
    corner_position: Position,
    color_display: ColorDisplay,
    _config: &Config,
    _settings: &mut Settings)
    -> Result<(), anyhow::Error>
{
    let columns: u16 = (size.0 / color_display.width()) - 1;

    let max_col = corner_position.column.saturating_add(columns);
    let max_line = corner_position.line.saturating_add(size.1 - 2);
    const MAX_SKIP: u16 = 20;
    trace!("{} {}", max_col, max_line);

    let mut line_buf = Vec::with_capacity(columns.into());
    let mut skipped: u16 = 0;
    let mut line = corner_position.line;

    while line < max_line {
        let mut print_line = false;
        for column in corner_position.column..=max_col {
            match palette.inner().color(&CellRef::Position(Position {
                    page: corner_position.page,
                    line,
                    column,
                }))
            {
                Ok(Some(c)) => {
                    line_buf.push(Ok(Some(c)));
                    print_line = true;
                },
                Ok(None)    => line_buf.push(Ok(None)),
                Err(e)      => line_buf.push(Err(e)),
            }
        }

        if print_line {
            if skipped > 0 {
                println!("\t ... {} empty line{}.",
                    skipped,
                    if skipped == 1 { "" } else { "s" });
            }
            skipped = 0;
            match line.checked_add(1) {
                None => break,
                Some(new_line) => { line = new_line; },
            }

            for elem in line_buf.drain(..) {
                match elem {
                    Ok(Some(c)) => color_display.print(c),
                    Err(PaletteError::UndefinedColor { cell_ref, circular }) => {
                        color_display.print_invalid();
                        warn!("{:?} {:?}", cell_ref, circular);
                    },
                    Ok(None)    => color_display.print_empty(),
                    Err(_)      => color_display.print_empty(),
                }
            }
            println!();
        } else {
            match skipped.checked_add(1) {
                None => break,
                Some(new_skipped) if new_skipped > MAX_SKIP => break,
                Some(new_skipped) => { skipped = new_skipped; },
            }
        }
    }
    if skipped > 0 {
        println!("\t ... {}+ empty line{}.",
            skipped,
            if skipped == 1 { "" } else { "s" });
    }
    Ok(())
}

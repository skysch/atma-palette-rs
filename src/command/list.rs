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
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::CellRef;

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
    // TODO: Move all of this to an inner function.
    debug!("Start listing for selection {:?}", selection);
    let selection = selection.unwrap_or(CellSelector::All.into());
    let index_selection = selection.resolve(palette.inner());
    debug!("Start listing for {:?}", index_selection);

    for idx in index_selection {
        if let Ok(Some(c)) = palette.inner()
            .color(&CellRef::Index(idx))
        {
            print!("{:4X}", idx);
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
            println!();
        } else {
            println!("{:4X} invalid color", idx);
        }
    }
    Ok(())
}

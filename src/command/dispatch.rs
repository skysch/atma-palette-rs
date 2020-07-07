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
use crate::Error;
use crate::palette::Palette;
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



fn parse_color(text: String) -> Result<Color, Error> {
    color(&text[..])
        .finish()
        .map_err(Error::from)
}

////////////////////////////////////////////////////////////////////////////////
// dispatch
////////////////////////////////////////////////////////////////////////////////
/// Executes the given `AtmaOptions` on the given `Palette`.
pub fn dispatch(
    palette: Option<Palette>,
    opts: AtmaOptions,
    config: Config,
    settings: Settings)
    -> Result<(), Error>
{
    if opts.common.dry_run {
        // TODO: Implement this.
        println!("Dry run is currently unsupported.");
        return Ok(());
    }

    use CommandOption::*;
    match (palette, opts.command) {
        (_, None) => unimplemented!(),
        (None, Some(_)) => unimplemented!(),

        (Some(mut palette), Some(command)) => match command {
            New {
                name,
                no_history,
                no_config_file,
                no_settings_file,
                set_active,
            } => new_palette(
                name,
                no_history,
                if no_config_file { None } else { Some(config) },
                if no_settings_file { None } else { Some(settings) },
                set_active),

            List => unimplemented!(),
            Insert { insert_options } => match insert_options {
                InsertOption::Colors { colors, name, at } => {
                    let colors: Vec<Color> = colors
                        .into_iter()
                        .map(parse_color)
                        .collect::<Result<Vec<_>,_>>()?;

                    let res = palette.insert_colors(&colors[..], name, at);
                    info!("{:?}", palette);
                    res
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



fn new_palette(
    name: Option<PathBuf>,
    no_history: bool,
    config: Option<Config>,
    settings: Option<Settings>,
    set_active: bool)
    -> Result<(), Error>
{
    let palette = if no_history {
        Palette::new()
    } else {
        Palette::new().with_history()
    };

    

    unimplemented!()
}

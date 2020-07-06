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
use crate::command::AtmaOptions;
use crate::command::InsertOptions;
use crate::command::CommandOptions;
use crate::Error;
use crate::palette::Palette;
use crate::color::Color;
use crate::parse::ParseResultExt as _;
use crate::parse::color;

// External library imports.
use log::*;

fn parse_color(text: String) -> Result<Color, Error> {
    color(&text[..])
        .finish()
        .map_err(Error::from)
}

////////////////////////////////////////////////////////////////////////////////
// dispatch
////////////////////////////////////////////////////////////////////////////////
/// Executes the given `AtmaOptions` on the given `Palette`.
pub fn dispatch(palette: Option<Palette>, opts: AtmaOptions)
    -> Result<(), Error>
{
    trace!("{:?}", opts);
    if opts.common.dry_run {
        // TODO: Implement this.
        println!("Dry run is currently unsupported.");
        return Ok(());
    }

    use CommandOptions::*;
    match (palette, opts.command) {
        (_, None) => unimplemented!(),
        (None, Some(_)) => unimplemented!(),

        (Some(mut palette), Some(command)) => match command {
            List => unimplemented!(),
            Insert { insert_options } => match insert_options {
                InsertOptions::Colors { colors, name, at } => {
                    trace!("{:?}", colors);
                    let colors: Vec<Color> = colors
                        .into_iter()
                        .map(parse_color)
                        .collect::<Result<Vec<_>,_>>()?;
                    trace!("{:?}", colors);

                    let res = palette.insert_colors(&colors[..], name, at);
                    info!("{:?}", palette);
                    res

                },

                InsertOptions::Ramp { ..}=> //points, count, interpolate, name, at } => 
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

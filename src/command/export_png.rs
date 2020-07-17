////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface module.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellSelection;
use crate::cell::CellRef;
use crate::palette::Palette;

// Standard library imports.
use std::path::Path;


/// Writes the palette selection to a PNG file at the given path.
///
/// # Errors
///
/// This function will always fail because the "png" feature is not available.
#[cfg(not(feature = "png"))]
pub fn write_png<'a>(
    palette: &Palette,
    selection: CellSelection<'a>,
    path: &Path)
    -> Result<(), anyhow::Error>
{
    Err(anyhow!("Export using PNG format is unsupported."))
}

/// Writes the palette selection to a PNG file at the given path.
#[cfg(feature = "png")]
pub fn write_png<'a>(
    palette: &Palette,
    selection: CellSelection<'a>,
    path: &Path)
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

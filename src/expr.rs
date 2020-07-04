////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette color expression definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::basic::BasicPalette;
use crate::color::Color;
use crate::color::Rgb;
use crate::cell::CellRef;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

////////////////////////////////////////////////////////////////////////////////
// Expr
////////////////////////////////////////////////////////////////////////////////
/// Atma color expression for defining the behavior of a cell.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum Expr {
    /// An color expression with no color.
    Empty,

    /// A color.
    Color(Color),

    /// Performs an RGB multiply between the colors in the given cells.
    RgbMultiply(CellRef<'static>, CellRef<'static>),
}


impl Expr {
    /// Returns the Expr's color.
    pub fn color(&self, basic: &BasicPalette) -> Option<Color> {
        match self {
            Expr::Empty => None,
            
            Expr::Color(c) => Some(c.clone()),

            Expr::RgbMultiply(a, b) => match (
                basic.color(a).ok()?,
                basic.color(b).ok()?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([ra * rb, ga * gb, ba * bb]);
                    Some(Color::from(rgb))
                },
                _ => None,
            },
        }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Empty
    }
}

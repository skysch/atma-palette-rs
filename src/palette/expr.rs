////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette color expression definitions.
////////////////////////////////////////////////////////////////////////////////
#![allow(variant_size_differences)] // TODO: Remove this.

// Local imports.
use crate::palette::BasicPalette;
use crate::color::Color;
use crate::color::Rgb;
use crate::cell::CellRef;
use crate::error::Error;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::collections::HashSet;

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
    pub fn color(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>)
        -> Result<Option<Color>, Error>
    {
        match self {
            Expr::Empty => Ok(None),
            
            Expr::Color(c) => Ok(Some(c.clone())),

            Expr::RgbMultiply(a, b) => match (
                basic.cycle_detect_color(a, index_list)?,
                basic.cycle_detect_color(b, index_list)?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([ra * rb, ga * gb, ba * bb]);
                    Ok(Some(Color::from(rgb)))
                },
                _ => Ok(None),
            },
        }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Empty
    }
}

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
use crate::error::PaletteError;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::collections::HashSet;

////////////////////////////////////////////////////////////////////////////////
// Interpolate
////////////////////////////////////////////////////////////////////////////////
/// Color interpolation function for ramps.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Interpolate {
    /// Linear interpolation over each RGB channel.
    LinearRgb {
        /// The interpolation factor.
        amount: f32,
    },
    
    /// Cubic interpolation over each RGB channel.
    CubicRgb {
        /// The slope of the start color.
        start_slope: f32,
        /// The slope of the end color.
        end_slope: f32,
        /// The interpolation factor.
        amount: f32,
    },
}

impl Interpolate {
    /// Applies the interpolation function to the given colors.
    fn apply<C, D>(&self, start: C, end: D) -> Color 
        where
            C: Into<Color> + Sized,
            D: Into<Color> + Sized,
    {
        use Interpolate::*;
        match self {
            LinearRgb { amount } => {
                Color::rgb_linear_interpolate(
                        start.into(),
                        end.into(),
                        *amount)
                    .into()
            },

            CubicRgb { start_slope, end_slope, amount } => {
                Color::rgb_cubic_interpolate(
                        start.into(),
                        end.into(),
                        *start_slope,
                        *end_slope,
                        *amount)
                    .into()
            },
        }
    }
}

impl Default for Interpolate {
    fn default() -> Self {
        Interpolate::LinearRgb { amount: 1.0 }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Expr
////////////////////////////////////////////////////////////////////////////////
/// Atma color expression for defining the behavior of a cell.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Expr {
    /// An color expression with no color.
    Empty,

    /// A color.
    Color(Color),

    /// Performs an RGB multiply between the colors in the given cells.
    RgbMultiply(CellRef<'static>, CellRef<'static>, Interpolate),


}

impl Expr {
    /// Returns the Expr's color.
    pub fn color(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>)
        -> Result<Option<Color>, PaletteError>
    {
        match self {
            Expr::Empty => Ok(None),
            
            Expr::Color(c) => Ok(Some(c.clone())),

            Expr::RgbMultiply(a, b, int) => match (
                basic.cycle_detect_color(a, index_list)?,
                basic.cycle_detect_color(b, index_list)?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([ra * rb, ga * gb, ba * bb]);
                    Ok(Some(int.apply(a, rgb)))
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

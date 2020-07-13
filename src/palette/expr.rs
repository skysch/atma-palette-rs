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
use crate::parse::interpolate;
use crate::parse::ParseResultExt as _;
use crate::parse::FailureOwned;

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
    RgbLinear {
        /// The interpolation factor.
        amount: f32,
    },

    /// Cubic interpolation over each RGB channel.
    RgbCubic {
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
        match self {
            Interpolate::RgbLinear { amount } => {
                Color::rgb_linear_interpolate(
                        start.into(),
                        end.into(),
                        *amount)
                    .into()
            },

            Interpolate::RgbCubic { start_slope, end_slope, amount } => {
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

impl std::str::FromStr for Interpolate {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        interpolate(text)
            .end_of_text()
            .finish()
    }
}

impl Default for Interpolate {
    fn default() -> Self {
        Interpolate::RgbLinear { amount: 1.0 }
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

    /// A reference to another cell's color.
    Reference(CellRef<'static>),

    /// Performs an RGB multiply blend between the colors in the given cells.
    RgbMultiply(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB divide blend between the colors in the given cells.
    RgbDivide(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB subtract blend between the colors in the given cells.
    RgbSubtract(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB difference blend between the colors in the given cells.
    RgbDifference(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB screen blend between the colors in the given cells.
    RgbScreen(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB overlay blend between the colors in the given cells.
    RgbOverlay(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB hard light blend between the colors in the given cells.
    RgbHardLight(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB soft light blend between the colors in the given cells.
    RgbSoftLight(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB color dodge blend between the colors in the given cells.
    RgbColorDodge(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB color burn blend between the colors in the given cells.
    RgbColorBurn(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB linear dodge blend between the colors in the given
    /// cells.
    RgbLinearDodge(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB linear burn blend between the colors in the given cells.
    RgbLinearBurn(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB vivid light blend between the colors in the given cells.
    RgbVividLight(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB linear light blend between the colors in the given
    /// cells.
    RgbLinearLight(CellRef<'static>, CellRef<'static>, Interpolate),

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

            Expr::Reference(cell_ref) => basic
                .cycle_detect_color(cell_ref, index_list),
            
            Expr::RgbMultiply(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, multiply),
            Expr::RgbDivide(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, divide),
            Expr::RgbSubtract(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, subtract),
            Expr::RgbDifference(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, difference),
            Expr::RgbScreen(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, screen),
            Expr::RgbOverlay(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, overlay),
            Expr::RgbHardLight(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, hard_light),
            Expr::RgbSoftLight(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, soft_light),
            Expr::RgbColorDodge(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, color_dodge),
            Expr::RgbColorBurn(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, color_burn),
            Expr::RgbLinearDodge(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, linear_dodge),
            Expr::RgbLinearBurn(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, linear_burn),
            Expr::RgbVividLight(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, vivid_light),
            Expr::RgbLinearLight(a, b, int)
                => apply_rgb(basic, index_list, a, b, int, linear_light),
        }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Empty
    }
}


////////////////////////////////////////////////////////////////////////////////
// Component application functions
////////////////////////////////////////////////////////////////////////////////
/// Applies the given component blend to the RGB components of the given colors.
fn apply_rgb<F>(
    basic: &BasicPalette,
    index_list: &mut HashSet<u32>,
    a: &CellRef<'static>,
    b: &CellRef<'static>,
    int: &Interpolate,
    f: F)
    -> Result<Option<Color>, PaletteError>
    where F: Fn(f32, f32) -> f32
{
    match (
        basic.cycle_detect_color(a, index_list)?,
        basic.cycle_detect_color(b, index_list)?)
    {
        (Some(a), Some(b)) => {
            let [ra, ga, ba] = a.rgb_ratios();
            let [rb, gb, bb] = b.rgb_ratios();
            let rgb = Rgb::from([
                (f)(ra, rb),
                (f)(ga, gb),
                (f)(ba, bb),
            ]);
            Ok(Some(int.apply(a, rgb)))
        },
        _ => Ok(None),
    }
}


////////////////////////////////////////////////////////////////////////////////
// Component blend functions.
////////////////////////////////////////////////////////////////////////////////
#[inline]
fn multiply(a: f32, b: f32) -> f32 {
    a * b
}

#[inline]
fn divide(a: f32, b: f32) -> f32 {
    a / b
}

#[inline]
fn subtract(a: f32, b: f32) -> f32 {
    if a - b < 0.0 { 0.0 } else { a - b }
}

#[inline]
fn difference(a: f32, b: f32) -> f32 {
    if a > b { a - b } else { b - a }
}

#[inline]
fn screen(a: f32, b: f32) -> f32 {
    1.0 - (1.0 - a) * (1.0 - b)
}

#[inline]
fn overlay(a: f32, b: f32) -> f32 {
    if a < 0.5 {
        2.0 * a * b
    } else {
        1.0 - 2.0 * (1.0 - a) * (1.0 - b)
    }
}

#[inline]
fn hard_light(a: f32, b: f32) -> f32 {
    if b < 0.5 {
        2.0 * a * b
    } else {
        1.0 - 2.0 * (1.0 - a) * (1.0 - b)
    }
}

#[inline]
fn soft_light(a: f32, b: f32) -> f32 {
    let s = multiply(a, b);
    let e = screen(a, b);

    ((e - s) * a) + s
}

#[inline]
fn color_dodge(a: f32, b: f32) -> f32 {
    b / (1.0 - a)
}

#[inline]
fn linear_dodge(a: f32, b: f32) -> f32 {
    if a + b > 1.0 { 1.0 } else { a + b }
}

#[inline]
fn color_burn(a: f32, b: f32) -> f32 {
    1.0 - (1.0 - a) / b
}

#[inline]
fn linear_burn(a: f32, b: f32) -> f32 {
    a + b - 1.0
}

#[inline]
fn vivid_light(a: f32, b: f32) -> f32 {
    if a > 0.5 { color_dodge(a, b) } else { color_burn(a, b) }
}

#[inline]
fn linear_light(a: f32, b: f32) -> f32 {
    2.0 * a + b - 1.0
}

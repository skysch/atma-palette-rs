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
        match self {
            Interpolate::LinearRgb { amount } => {
                Color::rgb_linear_interpolate(
                        start.into(),
                        end.into(),
                        *amount)
                    .into()
            },

            Interpolate::CubicRgb { start_slope, end_slope, amount } => {
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

    /// Performs an RGB multiply blend between the colors in the given cells.
    RgbMultiply(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB screen blend between the colors in the given cells.
    RgbScreen(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB overlay blend between the colors in the given cells.
    RgbOverlay(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB hard light blend between the colors in the given cells.
    RgbHardLight(CellRef<'static>, CellRef<'static>, Interpolate),

    /// Performs an RGB soft light blend between the colors in the given cells.
    RgbSoftLight(CellRef<'static>, CellRef<'static>, Interpolate),


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
                    let rgb = Rgb::from([
                        multiply(ra, rb),
                        multiply(ga, gb),
                        multiply(ba, bb),
                    ]);
                    Ok(Some(int.apply(a, rgb)))
                },
                _ => Ok(None),
            },

            Expr::RgbScreen(a, b, int) => match (
                basic.cycle_detect_color(a, index_list)?,
                basic.cycle_detect_color(b, index_list)?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([
                        screen(ra, rb),
                        screen(ga, gb),
                        screen(ba, bb),
                    ]);
                    Ok(Some(int.apply(a, rgb)))
                },
                _ => Ok(None),
            },

            Expr::RgbOverlay(a, b, int) => match (
                basic.cycle_detect_color(a, index_list)?,
                basic.cycle_detect_color(b, index_list)?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([
                        overlay(ra, rb),
                        overlay(ga, gb),
                        overlay(ba, bb),
                    ]);
                    Ok(Some(int.apply(a, rgb)))
                },
                _ => Ok(None),
            },

            Expr::RgbHardLight(a, b, int) => match (
                basic.cycle_detect_color(a, index_list)?,
                basic.cycle_detect_color(b, index_list)?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([
                        hard_light(ra, rb),
                        hard_light(ga, gb),
                        hard_light(ba, bb),
                    ]);
                    Ok(Some(int.apply(a, rgb)))
                },
                _ => Ok(None),
            },

            Expr::RgbSoftLight(a, b, int) => match (
                basic.cycle_detect_color(a, index_list)?,
                basic.cycle_detect_color(b, index_list)?)
            {
                (Some(a), Some(b)) => {
                    let [ra, ga, ba] = a.rgb_ratios();
                    let [rb, gb, bb] = b.rgb_ratios();
                    let rgb = Rgb::from([
                        soft_light(ra, rb),
                        soft_light(ga, gb),
                        soft_light(ba, bb),
                    ]);
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




////////////////////////////////////////////////////////////////////////////////
// Component blend functions.
////////////////////////////////////////////////////////////////////////////////
#[inline]
fn multiply(a: f32, b: f32) -> f32 {
    a * b
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
    lerp_f32(multiply(a, b), screen(a, b), a)
}


#[inline]
fn lerp_f32(s: f32, e:f32, a: f32) -> f32 {
    ((e-s) * a) + s
}

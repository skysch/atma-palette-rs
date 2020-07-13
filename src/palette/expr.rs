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
use crate::parse::insert_expr;
use crate::parse::interpolate;
use crate::parse::blend_method;
use crate::parse::color_space;
use crate::parse::ParseResultExt as _;
use crate::parse::FailureOwned;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::collections::HashSet;

////////////////////////////////////////////////////////////////////////////////
// InsertExpr
////////////////////////////////////////////////////////////////////////////////
/// Palette-insertable color expression objects.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum InsertExpr {
    /// Insert a color.
    Color(Color),
    /// Insert a reference to a cell.
    Reference(CellRef<'static>),
    /// Insert a copy of the color from a cell.
    Copy(CellRef<'static>),
    /// Insert a color blend operation.
    Blend(BlendExpr),
    /// Insert an interpolated range of color blend operations.
    Ramp {
        /// The number of colors in the ramp.
        count: u32,
        /// The ramp blend function.
        blend_fn: BlendFunction,
        /// The range of values to interpolate over.
        interpolate: InterpolateRange,
    },
}

impl std::str::FromStr for InsertExpr {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        insert_expr(text)
            .end_of_text()
            .with_new_context(text, text)
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Expr
////////////////////////////////////////////////////////////////////////////////
/// Atma color expression for defining the behavior of a cell.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Expr {
    /// A color expression with no color.
    Empty,
    /// A simple color expression.
    Color(Color),
    /// A reference to another cell.
    Reference(CellRef<'static>),
    /// A color blend expression.
    Blend(BlendExpr),
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

            Expr::Blend(blend_expr) => blend_expr.color(basic, index_list),
        }
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Empty
    }
}

////////////////////////////////////////////////////////////////////////////////
// BlendExpr
////////////////////////////////////////////////////////////////////////////////
/// A color blend function.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct BlendExpr {
    /// the blend function
    pub blend_fn: BlendFunction,
    /// The blend interpolation.
    pub interpolate: Interpolate,
}

impl BlendExpr {
    /// Resolves the source and target references and returns their blended
    /// result.
    pub fn color(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>)
        -> Result<Option<Color>, PaletteError>
    {
        self.blend_fn.apply(basic, index_list, &self.interpolate)
    }
}

/// A color blend function.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct BlendFunction {
    /// The color space in which to apply the blend method.
    pub color_space: ColorSpace,
    /// The blend method.
    pub blend_method: BlendMethod,
    /// The source color of the blend.
    pub source: CellRef<'static>,
    /// The target color of the blend.
    pub target: CellRef<'static>,
}

impl BlendFunction {
    /// Resolves the source and target references and returns their blended
    /// result.
    pub fn apply(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>,
        int: &Interpolate)
        -> Result<Option<Color>, PaletteError>
    {
        match (
            basic.cycle_detect_color(&self.source, index_list)?,
            basic.cycle_detect_color(&self.target, index_list)?)
        {
            (Some(a), Some(b)) => {
                let blend_fn = |a, b| self.blend_method.apply(a, b);
                let res = self
                    .color_space
                    .map_channels_binary(a, b, blend_fn);
                Ok(Some(res))
            },
            _ => Ok(None),
        }
    }
}


/// Color blending method.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum BlendMethod {
    /// Simple alpha blend of color channels.
    Blend,
    /// Mutiply color channels.
    Multiply,
    /// Divide color channels.
    Divide,
    /// Subtract and clamp color channel.
    Subtract,
    /// Subtract smallest channel from largest and clamp.
    Difference,
    /// Multiply inverted channels and invert the result.
    Screen,
    /// Multiply light channel and screen dark channel of the source.
    Overlay,
    /// Multiply light channel and screen dark channel of the target.
    HardLight,
    /// Smoothly interpolate between multiply and screen.
    SoftLight,
    /// Lighten image by dividing target channel by inverted source channel.
    ColorDodge,
    /// Darken image by dividing inverted source channel by target channel and
    /// subtracting from 1.
    ColorBurn,
    /// Apply color dodge or burn based on source channel lightness.
    VividLight,
    /// Lighten image by adding channel.
    LinearDodge,
    /// Darken image by adding channels and subtracting 1.
    LinearBurn,
    /// Apply linear dodge or burn based on source channel lightness.
    LinearLight,
}

impl BlendMethod {
    /// Applies the blend calculation to the given channel values.
    pub fn apply(&self, a: f32, b: f32) -> f32 {
        use BlendMethod::*;
        match self {
            Blend       => b,
            Multiply    => a * b,
            Divide      => a / b,
            Subtract    => if a - b < 0.0 { 0.0 } else { a - b },
            Difference  => if a > b { a - b } else { b - a },
            Screen      => 1.0 - (1.0 - a) * (1.0 - b),
            Overlay     => {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            },
            HardLight   => {
                if b < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            },
            SoftLight   => {
                let s = Multiply.apply(a, b);
                let e = Screen.apply(a, b);
                ((e - s) * a) + s
            },
            ColorDodge  => b / (1.0 - a),
            ColorBurn   => 1.0 - (1.0 - a) / b,
            VividLight  => {
                if a > 0.5 {
                    ColorDodge.apply(a, b)
                } else {
                    ColorBurn.apply(a, b)
                }
            },
            LinearDodge => if a + b > 1.0 { 1.0 } else { a + b },
            LinearBurn  => a + b - 1.0,
            LinearLight => 2.0 * a + b - 1.0,
        }
    }
}

impl Default for BlendMethod {
    fn default() -> Self {
        BlendMethod::Blend
    }
}

impl std::str::FromStr for BlendMethod {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        blend_method(text)
            .end_of_text()
            .with_new_context(text, text)
            .finish()
    }
}

impl std::fmt::Display for BlendMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BlendMethod::Blend       => "blend",
            BlendMethod::Multiply    => "multiply",
            BlendMethod::Divide      => "divide",
            BlendMethod::Subtract    => "subtract",
            BlendMethod::Difference  => "difference",
            BlendMethod::Screen      => "screen",
            BlendMethod::Overlay     => "overlay",
            BlendMethod::HardLight   => "hard_light",
            BlendMethod::SoftLight   => "soft_light",
            BlendMethod::ColorDodge  => "color_dodge",
            BlendMethod::ColorBurn   => "color_burn",
            BlendMethod::VividLight  => "vivid_light",
            BlendMethod::LinearDodge => "linear_dodge",
            BlendMethod::LinearBurn  => "linear_burn",
            BlendMethod::LinearLight => "linear_light",
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// ColorSpace
////////////////////////////////////////////////////////////////////////////////
/// Supported color spaces.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ColorSpace {
    /// RGB color space.
    Rgb,
}

impl ColorSpace {
    /// Applies the given binary closure to the channels of the given colors.
    pub fn map_channels_binary<A, B, F>(&self, a: A, b: B, f: F) -> Color
        where
            A: Into<Color> + Sized,
            B: Into<Color> + Sized,
            F: Fn(f32, f32) -> f32,
    {
        match self {
            ColorSpace::Rgb => {
                let [ra, ga, ba] = a.into().rgb_ratios();
                let [rb, gb, bb] = b.into().rgb_ratios();
                Rgb::from([
                    (f)(ra, rb),
                    (f)(ga, gb),
                    (f)(ba, bb),
                ]).into()
            },
        }
    }
}

impl Default for ColorSpace {
    fn default() -> Self {
        ColorSpace::Rgb
    }
}

impl std::str::FromStr for ColorSpace {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        color_space(text)
            .end_of_text()
            .with_new_context(text, text)
            .finish()
    }
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ColorSpace::Rgb => "RGB",
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// Interpolate
////////////////////////////////////////////////////////////////////////////////
/// Interpolation of colors.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Interpolate {
    /// The color space whose channels to apply the interpolation to.
    pub color_space: ColorSpace,
    /// The interpolate function.
    pub interpolate_fn: InterpolateFunction,
    /// The interpolation amount.
    pub amount: f32,
}

impl Interpolate {
    /// Applies the interpolation to the given colors.
    pub fn apply<A, B>(&self, a: A, b: B) -> Color
        where
            A: Into<Color> + Sized,
            B: Into<Color> + Sized,
    {
        self.interpolate_fn.apply(self.color_space, a, b, self.amount)
    }
}

impl Default for Interpolate {
    fn default() -> Self {
        Interpolate {
            color_space: ColorSpace::default(),
            interpolate_fn: InterpolateFunction::default(),
            amount: 1.0,
        }
    }
}

/// Interpolation range for ramps.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct InterpolateRange {
    /// The color space whose channels to apply the interpolation to.
    pub color_space: ColorSpace,
    /// The interpolate function.
    pub interpolate_fn: InterpolateFunction,
    /// The start point of the range.
    pub start: f32,
    /// The end point of the range.
    pub end: f32,
}

impl InterpolateRange {
    /// Compute the `BlendExpr`s for the ramp, using the given `BlendFunction`.
    pub fn blend_exprs(&self, count: usize, blend_fn: &BlendFunction)
        -> Vec<BlendExpr>
    {
        let mut exprs = Vec::with_capacity(count);
        let inc = (self.end - self.start) / (count as f32 + 2.0);
        let mut amount = inc;

        for _ in 0..count {
            exprs.push(BlendExpr {
                blend_fn: blend_fn.clone(),
                interpolate: Interpolate {
                    color_space: self.color_space,
                    interpolate_fn: self.interpolate_fn,
                    amount,
                },
            });
            amount += inc;
        }
        exprs
    }
}

impl Default for InterpolateRange {
    fn default() -> Self {
        InterpolateRange {
            color_space: ColorSpace::default(),
            interpolate_fn: InterpolateFunction::default(),
            start: 0.0,
            end: 1.0,
        }
    }
}

/// Interpolation function.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum InterpolateFunction {
    /// Linear interpolation.
    Linear,
    /// Cubic interpolation with the given boundary derivatives.
    Cubic(f32, f32),
}

impl InterpolateFunction {
    /// Applies the interpolation function to the given colors.
    pub fn apply<A, B>(&self, color_space: ColorSpace, a: A, b: B, amount: f32)
        -> Color
        where
            A: Into<Color> + Sized,
            B: Into<Color> + Sized,
    {
        use ColorSpace::*;
        use InterpolateFunction::*;

        match (color_space, self) {
            (Rgb, Linear) => Color::rgb_linear_interpolate(
                    a.into(),
                    b.into(),
                    amount)
                .into(),

            (Rgb, Cubic(m0, m1)) => Color::rgb_cubic_interpolate(
                    a.into(),
                    b.into(),
                    *m0,
                    *m1,
                    amount)
                .into()
        }
    }
}

impl Default for InterpolateFunction {
    fn default() -> Self {
        InterpolateFunction::Linear
    }
}

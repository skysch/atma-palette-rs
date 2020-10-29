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
use crate::cell::CellRef;
use crate::color::Color;
use crate::color::Hsv;
use crate::color::Rgb;
use crate::error::PaletteError;
use crate::palette::BasicPalette;
use crate::parse::AstExprMatch as _;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::ast_expr;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use tephra::lexer::Lexer;
use tephra::position::Lf;
use tephra::result::FailureOwned;
use tephra::result::ParseResultExt as _;

// Standard library imports.
use std::collections::HashSet;
use std::convert::TryInto;


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
// InsertExpr
////////////////////////////////////////////////////////////////////////////////
/// Palette-insertable color expression objects.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum InsertExpr {
    /// Insert an interpolated range of color blend operations.
    Ramp(RampExpr),
    /// Insert a color blend operation.
    Blend(BlendExpr),
    /// Insert a color.
    Color(Color),
    /// Insert a copy of the color from a cell.
    Copy(CellRef<'static>),
    /// Insert a reference to a cell.
    Reference(CellRef<'static>),
}

impl InsertExpr {
    /// Returns the color `Expr`s to be inserted.
    pub fn exprs(&self, basic: &BasicPalette)
        -> Result<Vec<Expr>, PaletteError>
    {
        match self {
            InsertExpr::Ramp(ramp_expr) => Ok(ramp_expr.interpolate
                .blend_exprs(ramp_expr.count, &ramp_expr.blend_fn)
                .into_iter()
                .map(Expr::Blend)
                .collect()),
            
            InsertExpr::Blend(blend_expr) => Ok(vec![
                Expr::Blend(blend_expr.clone())
            ]),

            InsertExpr::Color(color) => Ok(vec![
                Expr::Color(color.clone())
            ]),
            
            // TODO: Config to generate default color instead of error?
            InsertExpr::Copy(cell_ref) => Ok(vec![
                Expr::Color(basic.color(cell_ref)?
                    .ok_or_else(|| PaletteError::UndefinedColor {
                        cell_ref: cell_ref.clone(),
                        circular: false,
                    })?)
            ]),

            InsertExpr::Reference(cell_ref) => Ok(vec![
                Expr::Reference(cell_ref.clone())
            ]),
        }
    }
}

impl std::str::FromStr for InsertExpr {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        InsertExpr::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}

////////////////////////////////////////////////////////////////////////////////
// RampExpr
////////////////////////////////////////////////////////////////////////////////
/// A color ramp expression.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct RampExpr {
    /// The number of colors in the ramp.
    pub count: u8,
    /// The ramp blend function.
    pub blend_fn: BlendFunction,
    /// The range of values to interpolate over.
    pub interpolate: InterpolateRange,
}

impl std::str::FromStr for RampExpr {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        RampExpr::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}

////////////////////////////////////////////////////////////////////////////////
// BlendExpr
////////////////////////////////////////////////////////////////////////////////
/// A color blend function.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct BlendExpr {
    /// The blend function.
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

impl std::str::FromStr for BlendExpr {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        BlendExpr::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}


////////////////////////////////////////////////////////////////////////////////
// BlendFunction
////////////////////////////////////////////////////////////////////////////////
/// A color blend function.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum BlendFunction {
    /// A unary blend function.
    Unary(UnaryBlendFunction),
    /// A binary blend function.
    Binary(BinaryBlendFunction),
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
        use BlendFunction::*;


        match self {
            Unary(un_fn)   => un_fn.apply(basic, index_list, int),
            Binary(bin_fn) => bin_fn.apply(basic, index_list, int),
        }
    }
}

impl std::str::FromStr for BlendFunction {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        BlendFunction::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}



////////////////////////////////////////////////////////////////////////////////
// InvalidBlendMethod
////////////////////////////////////////////////////////////////////////////////
/// An invalid blend method was provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidBlendMethod;

impl std::fmt::Display for InvalidBlendMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for InvalidBlendMethod {}


////////////////////////////////////////////////////////////////////////////////
// UnaryBlendFunction
////////////////////////////////////////////////////////////////////////////////
/// A color blend function.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct UnaryBlendFunction {
    /// The blend method.
    pub blend_method: UnaryBlendMethod,
    /// The blend value.
    pub value: f32,
    /// The argument of the blend.
    pub arg: CellRef<'static>,
}

impl UnaryBlendFunction {
    /// Resolves the arg_1 and arg_2 references and returns their blended
    /// result.
    pub fn apply(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>,
        int: &Interpolate)
        -> Result<Option<Color>, PaletteError>
    {
        match basic.cycle_detect_color(&self.arg, index_list)? {
            Some(color) => {
                let blended = self.blend_method.apply(&color, self.value);
                Ok(Some(int.apply(color, blended)))
            },
            _ => Ok(None),
        }
    }
}

impl std::str::FromStr for UnaryBlendFunction {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        UnaryBlendFunction::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}

/// Color blending method for unary blend functions.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum UnaryBlendMethod {
    /// Override the red channel of the source color.
    SetRed,
    /// Override the green channel of the source color.
    SetGreen,
    /// Override the blue channel of the source color.
    SetBlue,

    /// Shift the hue of the source color.
    HueShift,
    /// Override the hue of the source color.
    SetHue,
    /// Saturate the source color.
    Saturate,
    /// Desaturate the source color.
    Desaturate,
    /// Lighten the source color.
    Lighten,
    /// Darken the source color.
    Darken,
}

impl UnaryBlendMethod {
    /// Applies the blend calculation to the given channel values.
    pub fn apply(&self, arg: &Color, value: f32) -> Color {
        use UnaryBlendMethod::*;
        match self {
            SetRed     => {
                let rgb = arg.rgb_ratios();
                Color::from(Rgb::from([value, rgb[1], rgb[2]]))
            },
            SetGreen   => {
                let rgb = arg.rgb_ratios();
                Color::from(Rgb::from([rgb[0], value, rgb[2]]))
            },
            SetBlue    => {
                let rgb = arg.rgb_ratios();
                Color::from(Rgb::from([rgb[0], rgb[1], value]))
            },

            HueShift   => {
                let hsv = arg.hsv_components();
                Color::from(Hsv::from([hsv[0] + value, hsv[1], hsv[2]]))
            },
            SetHue     => {
                let hsv = arg.hsv_components();
                Color::from(Hsv::from([value, hsv[1], hsv[2]]))
            },
            Saturate   => {
                let hsv = arg.hsv_components();
                Color::from(Hsv::from([hsv[0], hsv[1] + value, hsv[2]]))
            },
            Desaturate => {
                let hsv = arg.hsv_components();
                Color::from(Hsv::from([hsv[0], hsv[1] - value, hsv[2]]))
            },
            Lighten    => {
                let hsv = arg.hsv_components();
                Color::from(Hsv::from([hsv[0], hsv[1], hsv[2] + value]))
            },
            Darken     => {
                let hsv = arg.hsv_components();
                Color::from(Hsv::from([hsv[0], hsv[1], hsv[2] - value]))
            },
        }
    }
}

impl std::str::FromStr for UnaryBlendMethod {
    type Err = InvalidBlendMethod;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "set_red"    => Ok(UnaryBlendMethod::SetRed),
            "set_green"  => Ok(UnaryBlendMethod::SetGreen),
            "set_blue"   => Ok(UnaryBlendMethod::SetBlue),
            "hue_shift"  => Ok(UnaryBlendMethod::HueShift),
            "set_hue"    => Ok(UnaryBlendMethod::SetHue),
            "saturate"   => Ok(UnaryBlendMethod::Saturate),
            "desaturate" => Ok(UnaryBlendMethod::Desaturate),
            "lighten"    => Ok(UnaryBlendMethod::Lighten),
            "darken"     => Ok(UnaryBlendMethod::Darken),
            _            => Err(InvalidBlendMethod),
        }
    }
}

impl std::fmt::Display for UnaryBlendMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            UnaryBlendMethod::SetRed     => "set_red",
            UnaryBlendMethod::SetGreen   => "set_green",
            UnaryBlendMethod::SetBlue    => "set_blue",
            UnaryBlendMethod::HueShift   => "hue_shift",
            UnaryBlendMethod::SetHue     => "set_hue",
            UnaryBlendMethod::Saturate   => "saturate",
            UnaryBlendMethod::Desaturate => "desaturate",
            UnaryBlendMethod::Lighten    => "lighten",
            UnaryBlendMethod::Darken     => "darken",
        })
    }
}


////////////////////////////////////////////////////////////////////////////////
// BinaryBlendFunction
////////////////////////////////////////////////////////////////////////////////
/// A color blend function.
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct BinaryBlendFunction {
    /// The color space in which to apply the blend method.
    pub color_space: ColorSpace,
    /// The blend method.
    pub blend_method: BinaryBlendMethod,
    /// The first argument of the blend.
    pub arg_0: CellRef<'static>,
    /// The second argument of the blend.
    pub arg_1: CellRef<'static>,
}

impl BinaryBlendFunction {
    /// Resolves the arg_1 and arg_2 references and returns their blended
    /// result.
    pub fn apply(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>,
        int: &Interpolate)
        -> Result<Option<Color>, PaletteError>
    {
        let mut index_list_2 = index_list.clone();
        match (
            basic.cycle_detect_color(&self.arg_0, index_list)?,
            basic.cycle_detect_color(&self.arg_1, &mut index_list_2)?)
        {
            (Some(a), Some(b)) => {
                let blend_fn = |a, b| self.blend_method.apply(a, b);
                let blended = self
                    .color_space
                    .map_channels_binary(a, b, blend_fn);
                Ok(Some(int.apply(a, blended)))
            },
            _ => Ok(None),
        }
    }
}

impl std::str::FromStr for BinaryBlendFunction {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        BinaryBlendFunction::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}


/// Color blending method for binary bland functions.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum BinaryBlendMethod {
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
    /// Multiply light channel and screen dark channel of the arg_1.
    Overlay,
    /// Multiply light channel and screen dark channel of the arg_2.
    HardLight,
    /// Smoothly interpolate between multiply and screen.
    SoftLight,
    /// Lighten image by dividing arg_2 channel by inverted arg_1 channel.
    ColorDodge,
    /// Darken image by dividing inverted arg_1 channel by arg_2 channel and
    /// subtracting from 1.
    ColorBurn,
    /// Apply color dodge or burn based on arg_1 channel lightness.
    VividLight,
    /// Lighten image by adding channel.
    LinearDodge,
    /// Darken image by adding channels and subtracting 1.
    LinearBurn,
    /// Apply linear dodge or burn based on arg_1 channel lightness.
    LinearLight,
}

impl BinaryBlendMethod {
    /// Applies the blend calculation to the given channel values.
    pub fn apply(&self, a: f32, b: f32) -> f32 {
        use BinaryBlendMethod::*;
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


impl std::str::FromStr for BinaryBlendMethod {
    type Err = InvalidBlendMethod;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "blend"        => Ok(BinaryBlendMethod::Blend),
            "multiply"     => Ok(BinaryBlendMethod::Multiply),
            "divide"       => Ok(BinaryBlendMethod::Divide),
            "subtract"     => Ok(BinaryBlendMethod::Subtract),
            "difference"   => Ok(BinaryBlendMethod::Difference),
            "screen"       => Ok(BinaryBlendMethod::Screen),
            "overlay"      => Ok(BinaryBlendMethod::Overlay),
            "hard_light"   => Ok(BinaryBlendMethod::HardLight),
            "soft_light"   => Ok(BinaryBlendMethod::SoftLight),
            "color_dodge"  => Ok(BinaryBlendMethod::ColorDodge),
            "color_burn"   => Ok(BinaryBlendMethod::ColorBurn),
            "vivid_light"  => Ok(BinaryBlendMethod::VividLight),
            "linear_dodge" => Ok(BinaryBlendMethod::LinearDodge),
            "linear_burn"  => Ok(BinaryBlendMethod::LinearBurn),
            "linear_light" => Ok(BinaryBlendMethod::LinearLight),
            _              => Err(InvalidBlendMethod),
        }
    }
}

impl std::fmt::Display for BinaryBlendMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BinaryBlendMethod::Blend       => "blend",
            BinaryBlendMethod::Multiply    => "multiply",
            BinaryBlendMethod::Divide      => "divide",
            BinaryBlendMethod::Subtract    => "subtract",
            BinaryBlendMethod::Difference  => "difference",
            BinaryBlendMethod::Screen      => "screen",
            BinaryBlendMethod::Overlay     => "overlay",
            BinaryBlendMethod::HardLight   => "hard_light",
            BinaryBlendMethod::SoftLight   => "soft_light",
            BinaryBlendMethod::ColorDodge  => "color_dodge",
            BinaryBlendMethod::ColorBurn   => "color_burn",
            BinaryBlendMethod::VividLight  => "vivid_light",
            BinaryBlendMethod::LinearDodge => "linear_dodge",
            BinaryBlendMethod::LinearBurn  => "linear_burn",
            BinaryBlendMethod::LinearLight => "linear_light",
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
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        ColorSpace::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
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
    /// Validates the interpolation.
    pub fn validate(self) -> Result<Self, PaletteError> {
        if self.amount < 0.0 || self.amount > 1.0 {
            Err(PaletteError::InvalidInputValue {
                msg: format!("interpolate value {} must lie within the \
                    range [0.0, 1.0].", self.amount).into()
            })
        } else {
            Ok(self)
        }
    }

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

impl std::str::FromStr for Interpolate {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        Interpolate::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
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
    /// Validates the interpolation ranges.
    pub fn validate(self) -> Result<Self, PaletteError> {
        if self.start < 0.0 || self.start > 1.0 {
            Err(PaletteError::InvalidInputValue {
                msg: format!("interpolate start value {} must lie within the \
                    range [0.0, 1.0].", self.start).into()
            })
        } else if self.end < 0.0 || self.end > 1.0 {
            Err(PaletteError::InvalidInputValue {
                msg: format!("interpolate end value {} must lie within the \
                    range [0.0, 1.0].", self.end).into()
            })
        } else {
            Ok(self)
        }
    }

    /// Compute the `BlendExpr`s for the ramp, using the given `BinaryBlendFunction`.
    pub fn blend_exprs(&self, count: u8, blend_fn: &BlendFunction)
        -> Vec<BlendExpr>
    {
        let mut exprs = Vec::with_capacity(count.try_into().unwrap());
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

impl std::str::FromStr for InterpolateRange {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        InterpolateRange::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
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

impl std::str::FromStr for InterpolateFunction {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let column_metrics = Lf::with_tab_width(4);
        let mut lexer = Lexer::new(scanner, text, column_metrics);
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        let ast = ast_expr(lexer)
            .finish()?;

        InterpolateFunction::match_expr(ast, column_metrics)
            .map_err(|parse_error| FailureOwned {
                parse_error: parse_error.into_owned(),
                source: None,
            })
    }
}

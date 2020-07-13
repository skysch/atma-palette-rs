////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Ancillary command option inputs.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::cell::CellRef;
use crate::cell::Position;
use crate::color::Color;
use crate::parse::cell_ref;
use crate::parse::color;
use crate::parse::any_literal_map_once;
use crate::parse::FailureOwned;
use crate::parse::float;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::ParseResultExt as _;
use crate::parse::position;


// External library imports.
use serde::Serialize;
use serde::Deserialize;


////////////////////////////////////////////////////////////////////////////////
// ExprTarget
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for the target of an expression.
#[allow(variant_size_differences)]
#[derive(Debug, Clone)]
pub enum ExprTarget {
    /// A Color.
    Color(Color),
    /// A cell reference.
    CellRef(CellRef<'static>),
}

impl std::str::FromStr for ExprTarget {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let color_res = color(text)
            .end_of_text()
            .finish();

        if let Ok(color) = color_res {
            return Ok(ExprTarget::Color(color));
        }

        cell_ref(text)
            .end_of_text()
            .source_for("expected color or cell reference")
            .finish()
            .map(CellRef::into_static)
            .map(ExprTarget::CellRef)
    }
}


////////////////////////////////////////////////////////////////////////////////
// FunctionInput
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for input to a ramp.
#[allow(variant_size_differences)]
#[derive(Debug, Clone)]
pub enum FunctionInput {
    /// A floating point value
    Value(f32),
    /// A color.
    Color(Color),
    /// A cell reference.
    CellRef(CellRef<'static>),
}

impl std::str::FromStr for FunctionInput {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let float_res = float::<f32>("f32")(text)
            .end_of_text()
            .finish();

        if let Ok(float) = float_res {
            return Ok(FunctionInput::Value(float));
        }

        let color_res = color(text)
            .end_of_text()
            .finish();

        if let Ok(color) = color_res {
            return Ok(FunctionInput::Color(color));
        }

        cell_ref(text)
            .end_of_text()
            .source_for("expected value, color, or cell reference")
            .finish()
            .map(CellRef::into_static)
            .map(FunctionInput::CellRef)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Positioning
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for input or move positioning.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum Positioning {
    /// An explicit position.
    Position(Position),
    /// The position of the palette cursor.
    Cursor,
    /// The first open position.
    Open,
    /// No positioning.
    None,
}

impl Positioning {
    /// Returns true if the positioning is `None`.
    pub fn is_none(&self) -> bool {
        match self {
            Positioning::None => true,
            _                 => false,
        }
    }
}

impl std::str::FromStr for Positioning {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Ok(success) = any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("cursor", Positioning::Cursor),
                    ("open",   Positioning::Open),
                    ("none",   Positioning::None),
                ])
            (text)
            .end_of_text()
        {
            return Ok(success.value);
        }

        position(text)
            .end_of_text()
            .source_for("expected 'cursor', 'open', or a position")
            .finish()
            .map(Positioning::Position)
    }
}


////////////////////////////////////////////////////////////////////////////////
// BlendMode
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for blend mode functions.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum BlendMode {
    /// A reference to another cell's color.
    Reference,
    /// Performs an RGB multiply blend between the colors in the given cells.
    RgbMultiply,
    /// Performs an RGB divide blend between the colors in the given cells.
    RgbDivide,
    /// Performs an RGB subtract blend between the colors in the given cells.
    RgbSubtract,
    /// Performs an RGB difference blend between the colors in the given cells.
    RgbDifference,
    /// Performs an RGB screen blend between the colors in the given cells.
    RgbScreen,
    /// Performs an RGB overlay blend between the colors in the given cells.
    RgbOverlay,
    /// Performs an RGB hard light blend between the colors in the given cells.
    RgbHardLight,
    /// Performs an RGB soft light blend between the colors in the given cells.
    RgbSoftLight,
    /// Performs an RGB color dodge blend between the colors in the given cells.
    RgbColorDodge,
    /// Performs an RGB color burn blend between the colors in the given cells.
    RgbColorBurn,
    /// Performs an RGB linear dodge blend between the colors in the given
    /// cells.
    RgbLinearDodge,
    /// Performs an RGB linear burn blend between the colors in the given cells.
    RgbLinearBurn,
    /// Performs an RGB vivid light blend between the colors in the given cells.
    RgbVividLight,
    /// Performs an RGB linear light blend between the colors in the given
    /// cells.
    RgbLinearLight,
}

impl std::str::FromStr for BlendMode {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "blend mode",
                vec![
                    ("reference",    BlendMode::Reference),
                    ("multiply",     BlendMode::RgbMultiply),
                    ("divide",       BlendMode::RgbDivide),
                    ("subtract",     BlendMode::RgbSubtract),
                    ("difference",   BlendMode::RgbDifference),
                    ("screen",       BlendMode::RgbScreen),
                    ("overlay",      BlendMode::RgbOverlay),
                    ("hard_light",   BlendMode::RgbHardLight),
                    ("soft_light",   BlendMode::RgbSoftLight),
                    ("color_dodge",  BlendMode::RgbColorDodge),
                    ("color_burn",   BlendMode::RgbColorBurn),
                    ("linear_dodge", BlendMode::RgbLinearDodge),
                    ("linear_burn",  BlendMode::RgbLinearBurn),
                    ("vivid_light",  BlendMode::RgbVividLight),
                    ("linear_light", BlendMode::RgbLinearLight),
                ])
            (text)
            .end_of_text()
            .with_new_context(text, text)
            .finish()
    }
}

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
use crate::parse::any_literal_map;
use crate::parse::FailureOwned;
use crate::parse::float;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::ParseResultExt as _;
use crate::parse::position;


// External library imports.
use serde::Serialize;
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////

/// Token for positioning by cursor.
pub const POSITIONING_CURSOR: &'static str = "cursor";

/// Token for positioning by open.
pub const POSITIONING_OPEN: &'static str = "open";

/// Token for no positioning.
pub const POSITIONING_NONE: &'static str = "none";

/// Token for reference function.
pub const BLEND_MODE_REFERENCE: &'static str = "reference";

/// Token for multiply blend mode.
pub const BLEND_MODE_MULTIPLY: &'static str = "multiply";

/// Token for divide blend mode.
pub const BLEND_MODE_DIVIDE: &'static str = "divide";

/// Token for subtract blend mode.
pub const BLEND_MODE_SUBTRACT: &'static str = "subtract";

/// Token for difference blend mode.
pub const BLEND_MODE_DIFFERENCE: &'static str = "difference";

/// Token for screen blend mode.
pub const BLEND_MODE_SCREEN: &'static str = "screen";

/// Token for overlay blend mode.
pub const BLEND_MODE_OVERLAY: &'static str = "overlay";

/// Token for hardlight blend mode.
pub const BLEND_MODE_HARDLIGHT: &'static str = "hard_light";

/// Token for softlight blend mode.
pub const BLEND_MODE_SOFTLIGHT: &'static str = "soft_light";

/// Token for colordodge blend mode.
pub const BLEND_MODE_COLORDODGE: &'static str = "color_dodge";

/// Token for colorburn blend mode.
pub const BLEND_MODE_COLORBURN: &'static str = "color_burn";

/// Token for lineardodge blend mode.
pub const BLEND_MODE_LINEARDODGE: &'static str = "linear_dodge";

/// Token for linearburn blend mode.
pub const BLEND_MODE_LINEARBURN: &'static str = "linear_burn";

/// Token for vividlight blend mode.
pub const BLEND_MODE_VIVIDLIGHT: &'static str = "vivid_light";

/// Token for linearlight blend mode.
pub const BLEND_MODE_LINEARLIGHT: &'static str = "linear_light";



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
        if let Ok(positioning) = any_literal_map(
                literal_ignore_ascii_case,
                "",
                vec![
                    (POSITIONING_CURSOR, Positioning::Cursor),
                    (POSITIONING_OPEN,   Positioning::Open),
                    (POSITIONING_NONE,   Positioning::None),
                ])
            (text)
            .end_of_text()
            .finish()
        {
            return Ok(positioning);
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
        any_literal_map(
                literal_ignore_ascii_case,
                "blend mode",
                vec![
                    (BLEND_MODE_REFERENCE,   BlendMode::Reference),
                    (BLEND_MODE_MULTIPLY,    BlendMode::RgbMultiply),
                    (BLEND_MODE_DIVIDE,      BlendMode::RgbDivide),
                    (BLEND_MODE_SUBTRACT,    BlendMode::RgbSubtract),
                    (BLEND_MODE_DIFFERENCE,  BlendMode::RgbDifference),
                    (BLEND_MODE_SCREEN,      BlendMode::RgbScreen),
                    (BLEND_MODE_OVERLAY,     BlendMode::RgbOverlay),
                    (BLEND_MODE_HARDLIGHT,   BlendMode::RgbHardLight),
                    (BLEND_MODE_SOFTLIGHT,   BlendMode::RgbSoftLight),
                    (BLEND_MODE_COLORDODGE,  BlendMode::RgbColorDodge),
                    (BLEND_MODE_COLORBURN,   BlendMode::RgbColorBurn),
                    (BLEND_MODE_LINEARDODGE, BlendMode::RgbLinearDodge),
                    (BLEND_MODE_LINEARBURN,  BlendMode::RgbLinearBurn),
                    (BLEND_MODE_VIVIDLIGHT,  BlendMode::RgbVividLight),
                    (BLEND_MODE_LINEARLIGHT, BlendMode::RgbLinearLight),
                ])
            (text)
            .end_of_text()
            .finish()
    }
}

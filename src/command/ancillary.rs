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
// RampInput
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for input to a ramp.
#[allow(variant_size_differences)]
#[derive(Debug, Clone)]
pub enum RampInput {
    /// A floating point value
    Value(f32),
    /// A color.
    Color(Color),
    /// A cell reference.
    CellRef(CellRef<'static>),
}

impl std::str::FromStr for RampInput {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let float_res = float::<f32>("f32")(text)
            .end_of_text()
            .finish();

        if let Ok(float) = float_res {
            return Ok(RampInput::Value(float));
        }

        let color_res = color(text)
            .end_of_text()
            .finish();

        if let Ok(color) = color_res {
            return Ok(RampInput::Color(color));
        }

        cell_ref(text)
            .end_of_text()
            .source_for("expected value, color, or cell reference")
            .finish()
            .map(CellRef::into_static)
            .map(RampInput::CellRef)
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
}

impl std::str::FromStr for Positioning {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if literal_ignore_ascii_case(POSITIONING_CURSOR)(text)
            .end_of_text()
            .is_ok()
        {
            return Ok(Positioning::Cursor);
        }

        if literal_ignore_ascii_case(POSITIONING_OPEN)(text)
            .end_of_text()
            .is_ok()
        {
            return Ok(Positioning::Open);
        }

        position(text)
            .end_of_text()
            .source_for("expected 'cursor', 'open', or a position")
            .finish()
            .map(Positioning::Position)
    }
}


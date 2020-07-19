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
use crate::color::Color;
use crate::cell::Position;
use crate::parse::any_literal_map_once;
use crate::parse::FailureOwned;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::ParseResultExt as _;
use crate::parse::position;


// External library imports.
use serde::Serialize;
use serde::Deserialize;



////////////////////////////////////////////////////////////////////////////////
// CursorBehavior
////////////////////////////////////////////////////////////////////////////////
/// The behavior of the cursor after an operation is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum CursorBehavior {
    /// Move the cursor to the lowest position in the affected selection.
    MoveToStart,
    /// Move the cursor after the highest position in the affected selection.
    MoveAfterEnd,
    /// Move the cursor to the lowest open position.
    MoveToOpen,
    /// Do not move the cursor.
    RemainInPlace,
}

impl std::str::FromStr for CursorBehavior {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("move_to_start",   CursorBehavior::MoveToStart),
                    ("move_after_end",  CursorBehavior::MoveAfterEnd),
                    ("move_to_open",    CursorBehavior::MoveToOpen),
                    ("remain_in_place", CursorBehavior::RemainInPlace),
                ])
            (text)
            .end_of_text()
            .source_for("expected one of 'move_to_start', 'move_after_end', \
                'move_to_open', or 'remain_in_place'")
            .finish()
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
// HistorySetOption
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for setting palette history.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum HistorySetOption {
    /// Enables the palette history.
    Enable,
    /// Disables and clears the palette history.
    Disable,
    /// Clears the palette history.
    Clear,
}

impl std::str::FromStr for HistorySetOption {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("enable",  HistorySetOption::Enable),
                    ("disable", HistorySetOption::Disable),
                    ("clear",   HistorySetOption::Clear),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'enable', 'disable', or 'clear'.")
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// ListMode
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for list mode.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum ListMode {
    /// Display colors in a grid.
    Grid,
    /// Display one color per line.
    Lines,
    /// Display colors in a line.
    List,
}

impl std::str::FromStr for ListMode {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("grid",  ListMode::Grid),
                    ("lines", ListMode::Lines),
                    ("list",  ListMode::List),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'grid', 'lines', or 'list'.")
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// ListOrder
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for list ordering.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum ListOrder {
    /// Order cells by position.
    Position,
    /// Order cells by index.
    Index,
    /// Order cells by name / group.
    Name,
}

impl std::str::FromStr for ListOrder {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("position", ListOrder::Position),
                    ("index",    ListOrder::Index),
                    ("name",     ListOrder::Name),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'position', 'index', or 'name'.")
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// ColorDisplay
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for color display.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum ColorDisplay {
    /// Display colors using a colored tile.
    Tile,
    /// Display colors using a 6-digit RGB hex code.
    Hex6,
    /// Display colors using a 3-digit RGB hex code.
    Hex3,
    /// Display colors using RGB function notation.
    Rgb,
}

impl ColorDisplay {
    /// Returns the total dedicated width of the color output, including
    /// whitespace.
    pub fn width(&self) -> usize {
        match self {
            ColorDisplay::Tile => 1,
            ColorDisplay::Hex6 => 8,
            ColorDisplay::Hex3 => 5,
            ColorDisplay::Rgb  => 20,
        }
    }

    /// Prints a color using the color display mode.
    pub fn print(&self, color: Color) {
        match self {
            ColorDisplay::Tile => {
                print!(" X");
            }

            ColorDisplay::Hex6 => {
                print!(" {:X}", color);
            },

            ColorDisplay::Hex3 => {
                let hex = color.rgb_hex();
                let r = (0xFF0000 & hex) >> 20;
                let g = (0x00FF00 & hex) >> 12;
                let b = (0x0000FF & hex) >> 4;
                print!(" #{:01X}{:01X}{:01X}", r, g, b);
            }

            ColorDisplay::Rgb  => {
                let [r, g, b] = color.rgb_ratios();
                print!(" rgb({:0.2},{:0.2},{:0.2})", r, g, b);
            },
        }
    }
}

impl std::str::FromStr for ColorDisplay {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("tile",  ColorDisplay::Tile),
                    ("hex_6", ColorDisplay::Hex6),
                    ("hex_3", ColorDisplay::Hex3),
                    ("rgb",   ColorDisplay::Rgb),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'tile', 'hex_6', 'hex_3', or 'rgb'.")
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// ColorMode
////////////////////////////////////////////////////////////////////////////////
/// Option parse result for color display.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub enum ColorMode {
    /// Enable colors.
    Enable,
    /// Disable colors.
    Disable,
}

impl std::str::FromStr for ColorMode {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        any_literal_map_once(
                literal_ignore_ascii_case,
                "",
                vec![
                    ("enable",  ColorMode::Enable),
                    ("disable", ColorMode::Disable),
                ])
            (text)
            .end_of_text()
            .source_for("expected 'enable' or 'disable'.")
            .finish()
    }
}

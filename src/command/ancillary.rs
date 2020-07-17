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

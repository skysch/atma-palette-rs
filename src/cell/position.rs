////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette position and position selectors.
////////////////////////////////////////////////////////////////////////////////

// Local library imports.
use crate::cell::REF_PREFIX_TOKEN;
use crate::cell::REF_POS_SEP_TOKEN;
use crate::cell::REF_ALL_TOKEN;
use crate::parse::position;
use crate::parse::position_selector;
use crate::parse::FailureOwned;
use crate::parse::ParseResultExt as _;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::convert::TryFrom;


/// Error created by attempting to convert a PositionSelector into a Position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PositionCellConversionError;


////////////////////////////////////////////////////////////////////////////////
// Position
////////////////////////////////////////////////////////////////////////////////
/// A reference to a `Cell` in a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct Position {
    /// The page number of the cell.
    pub page: u16,
    /// The line number of the cell.
    pub line: u16,
    /// The column number of the cell.
    pub column: u16,
}

impl Position {
    /// Returns the next position after the given one.
    pub fn succ(&self) -> Position {
        let (column, over) = self.column.overflowing_add(1);
        let (line, over) = self.line.overflowing_add(if over { 1 } else { 0 });
        let page = self.page.checked_add(if over { 1 } else { 0 })
            .expect("position page overflow");

        Position { page, line, column }
    }
}

impl TryFrom<PositionSelector> for Position {
    type Error = PositionCellConversionError;

    fn try_from(selector: PositionSelector) -> Result<Self, Self::Error> {
        match selector {
            PositionSelector {
                page: Some(page),
                line: Some(line),
                column: Some(column),
            } => Ok(Position { page, line, column }),
            _ => Err(PositionCellConversionError),
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}{}{}{}", 
            REF_PREFIX_TOKEN,
            self.page,
            REF_POS_SEP_TOKEN,
            self.line,
            REF_POS_SEP_TOKEN,
            self.column)
    }
}

impl std::str::FromStr for Position {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        position(text)
            .expect_end_of_text()
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// PositionSelector
////////////////////////////////////////////////////////////////////////////////
/// A reference to a `Cell`, page, line, or column combination in a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct PositionSelector {
    /// The page number of the cell, or None if all pages are selected.
    pub page: Option<u16>,
    /// The line number of the cell, or None if all lines are selected.
    pub line: Option<u16>,
    /// The column number of the cell, or None if all columns are selected.
    pub column: Option<u16>,
}

impl PositionSelector {
    /// Returns the PositionSelector which selects all positions.
    pub fn all() -> Self {
        PositionSelector {
            page: None,
            line: None,
            column: None,
        }
    }

    /// Returns the PositionSelector which selects a single line.
    pub fn line(page: u16, line: u16) -> Self {
        PositionSelector {
            page: Some(page),
            line: Some(line),
            column: None,
        }
    }

    /// Returns true if the given position is selected.
    pub fn contains(&self, other: &Position) -> bool {
        self.page.map(|p| p == other.page).unwrap_or(true) &&
        self.line.map(|l| l == other.line).unwrap_or(true) &&
        self.column.map(|c| c == other.column).unwrap_or(true)
    }

    /// Returns the bounds of the selectable positions.
    pub fn bounds(&self) -> (Position, Position) {
        let mut low = Position { page: 0, line: 0, column: 0 };
        let mut high = Position { 
            page: u16::MAX,
            line: u16::MAX,
            column: u16::MAX,
        };

        match (self.page, self.line, self.column) {
            (Some(p), Some(l), Some(c)) => {
                low.page = p; high.page = p;
                low.line = l; high.line = l;
                low.column = c; high.column = c;
            }
            (Some(p), Some(l), None) => {
                low.page = p; high.page = p;
                low.line = l; high.line = l;
            },
            (Some(p), None, Some(c)) => {
                low.page = p; high.page = p;
                low.column = c; high.column = c;
            },
            (None, Some(l), Some(c)) => {
                low.line = l; high.line = l;
                low.column = c; high.column = c;
            },
            (Some(p), None, None) => {
                low.page = p; high.page = p;
            },
            (None, Some(l), None) => {
                low.line = l; high.line = l;
            },
            (None, None, Some(c)) => {
                low.column = c; high.column = c;
            },
            (None, None, None) => (),
        }

        (low, high)
    }
}

impl From<Position> for PositionSelector {
    fn from(pos: Position) -> Self {
        PositionSelector {
            page: Some(pos.page),
            line: Some(pos.line),
            column: Some(pos.column),
        }
    }
}

impl std::fmt::Display for PositionSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", REF_PREFIX_TOKEN)?;
        match self.page {
            Some(page) => write!(f, "{}", page)?,
            None => write!(f, "{}", REF_ALL_TOKEN)?,
        }
        write!(f, "{}", REF_POS_SEP_TOKEN)?;
        match self.line {
            Some(line) => write!(f, "{}", line)?,
            None => write!(f, "{}", REF_ALL_TOKEN)?,
        }
        write!(f, "{}", REF_POS_SEP_TOKEN)?;
        match self.column {
            Some(column) => write!(f, "{}", column),
            None => write!(f, "{}", REF_ALL_TOKEN),
        }
    }
}

impl std::str::FromStr for PositionSelector {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        position_selector(text)
            .expect_end_of_text()
            .map(|suc| suc.value)
            .map_err(|fail| fail.to_owned())
    }
}

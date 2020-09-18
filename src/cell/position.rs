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
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use tephra::result::ParseResultExt as _;
use tephra::lexer::Lexer;
use tephra::position::Lf;

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
    /// The zero position.
    pub const ZERO: Position = Position {
        page: 0,
        line: 0,
        column: 0,
    };

    /// The minimum position value.
    pub const MIN: Position = Position::ZERO;

    /// The maximump position value.
    pub const MAX: Position = Position {
        page: u16::MAX,
        line: u16::MAX,
        column: u16::MAX,
    };

    /// Returns the next position after the given one.
    pub fn succ(&self) -> Position {
        let (column, over) = self.column.overflowing_add(1);
        let (line, over) = self.line.overflowing_add(if over { 1 } else { 0 });
        let page = self.page.checked_add(if over { 1 } else { 0 })
            .expect("position page overflow");

        Position { page, line, column }
    }

    /// Returns the next position after the given one, or None if the position
    /// is MAX.
    pub fn checked_succ(&self) -> Option<Position> {
        let (column, over) = self.column.overflowing_add(1);
        let (line, over) = self.line.overflowing_add(if over { 1 } else { 0 });
        let page = self.page.checked_add(if over { 1 } else { 0 });

        page.map(|page| Position { page, line, column })
    }

    /// Returns the next position after the given one, wrapping to zero if an
    /// overflow occurs.
    pub fn wrapping_succ(&self) -> Position {
        let (column, over) = self.column.overflowing_add(1);
        let (line, over) = self.line.overflowing_add(if over { 1 } else { 0 });
        let page = self.page.checked_add(if over { 1 } else { 0 });

        match page {
            Some(page) => Position { page, line, column },
            None => Position::ZERO,
        }
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
    type Err = PositionParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        
        let scanner = AtmaScanner::new();
        let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        position(lexer)
            .finish()
            .map_err(|_| PositionParseError)
    }
}

/// A parse error occured where a Position was expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionParseError;

impl std::fmt::Display for PositionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PositionParseError {}


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
    /// The PositionSelector which selects all positions.
    pub const ALL: PositionSelector = PositionSelector {
        page: None,
        line: None,
        column: None,
    };
    
    /// Constructs a new PositionSelector with the given page, line, and column.
    pub fn new<P, L, C>(page: P, line: L, column: C) -> Self
        where
            P: Into<Option<u16>>,
            L: Into<Option<u16>>,
            C: Into<Option<u16>>,
    {
        PositionSelector {
            page: page.into(),
            line: line.into(),
            column: column.into(),
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
        let mut low = Position::MIN;
        let mut high = Position::MAX;

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
    type Err = tephra::result::FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}

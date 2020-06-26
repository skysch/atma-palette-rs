////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette cell reference definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::REF_POS_SEP_TOKEN;
use crate::cell::REF_PREFIX_TOKEN;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////
/// A reference to a `Cell` in a palette.
///
/// The lifetime of the CellRef is the lifetime of any names. Most palette
/// interfaces accept a `CellRef` with an arbitrary lifetime
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum CellRef<'name> {
    /// A reference to a cell based on an internal index.
    Index(u32),

    /// A reference to a cell based on an assigned position.
    Position(Position),

    /// A reference to a cell based on an assigned name.
    Name(Cow<'name, str>),

    /// A reference to a cell based on an assigned group and index within that
    /// group.
    Group {
        /// The name of the group.
        group: Cow<'name, str>,
        /// The index of the cell within the group.
        idx: u32,
    },
}

impl<'name> CellRef<'name> {
    /// Converts a `CellRef` to a static lifetime.
    pub fn into_static(self) -> CellRef<'static> {
        use CellRef::*;
        match self {
            Index(idx) => Index(idx),
            Position(position) => Position(position),
            Name(name) => Name(Cow::from(name.into_owned())),
            Group { group, idx } => Group {
                group: Cow::from(group.into_owned()),
                idx,
            },
        }
    }
}

impl<'name> std::fmt::Display for CellRef<'name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CellRef::*;
        match self {
            Index(idx) => write!(f, "{}{}", REF_PREFIX_TOKEN, idx),
            Name(name) => write!(f, "{}", name),
            Position(position) => write!(f, "{}", position),
            Group { group, idx } => write!(f, 
                "{}{}{}", group, REF_PREFIX_TOKEN, idx),
        }
    }
}


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
    pub fn next(&self) -> Position {
        
        let (column, over) = self.column.overflowing_add(1);
        let (line, over) = self.line.overflowing_add(if over { 1 } else { 0 });
        let page = self.page.checked_add(if over { 1 } else { 0 })
            .expect("position page overflow");

        Position { page, line, column }
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


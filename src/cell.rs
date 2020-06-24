////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette cell definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::color::Color;
use crate::expr::Expr;
use crate::parse::REF_POS_SEP_TOKEN;
use crate::parse::REF_PREFIX_TOKEN;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::borrow::Cow;



////////////////////////////////////////////////////////////////////////////////
// Cell
////////////////////////////////////////////////////////////////////////////////
/// A cell holding a color expression.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cell {
    /// The cell's expression.
    expr: Expr,
    #[serde(skip)]
    cached: Option<Color>,
}

impl Cell {
    /// Constructs a new `Cell`.
    pub fn new() -> Self {
        Cell {
            expr: Default::default(),
            cached: None,
        }
    }

    /// Returns a reference to the cell's color expression.
    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns a mut reference to the cell's color expression.
    pub fn expr_mut(&mut self) -> &mut Expr {
        &mut self.expr
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new()
    }
}


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


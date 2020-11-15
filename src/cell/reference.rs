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
use crate::cell::Position;
use crate::cell::REF_PREFIX_TOKEN;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::cell_ref;

// External library imports.
use serde::Serialize;
use serde::Deserialize;
use tephra::lexer::Lexer;
use tephra::position::Lf;
use tephra::result::FailureOwned;
use tephra::result::ParseResultExt as _;

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

impl<'name> From<Position> for CellRef<'name> {
    fn from(pos: Position) -> Self {
        CellRef::Position(pos)
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

impl std::str::FromStr for CellRef<'static> {
    type Err = FailureOwned<Lf>;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        cell_ref(lexer)
            .finish()
            .map(CellRef::into_static)
    }
}

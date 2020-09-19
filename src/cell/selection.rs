////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette cell selections.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellSelector;
use crate::palette::BasicPalette;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::cell_selection;

// External library imports.
use normalize_interval::Selection;
use serde::Deserialize;
use serde::Serialize;
use tephra::lexer::Lexer;
use tephra::position::Lf;
use tephra::result::FailureOwned;
use tephra::result::ParseResultExt as _;

// Standard library imports.
use std::iter::FromIterator;


////////////////////////////////////////////////////////////////////////////////
// CellSelection
////////////////////////////////////////////////////////////////////////////////
/// A reference to a set of `Cell`s in a palette.
///
/// The lifetime of the CellSelector is the lifetime of any names. The same
/// `CellSelection` may be resolved for a palette multiple times yielding
/// different results if the palette is modified intermediately.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize)]
pub struct CellSelection<'name>(Vec<CellSelector<'name>>);

impl<'name> CellSelection<'name> {
    /// Converts a `CellRef` to a static lifetime.
    pub fn into_static(self) -> CellSelection<'static> {
        CellSelection(
            self.0
                .iter()
                .cloned()
                .map(CellSelector::into_static)
                .collect())
    }
    

    /// Resolves the CellSelection into a CellIndexSelection containing all of
    /// the selected and occupied cells for the given palette.
    pub fn resolve(&self, basic: &BasicPalette) -> CellIndexSelection {
        // Do quick check for an all selectors.
        for selector in &self.0[..] {
            if selector.is_all_selector() {
                return CellIndexSelection(
                    CellSelector::All.resolve(basic).into_iter().collect());
            }
        }

        let mut index_selection = CellIndexSelection(Selection::new());
        for selector in &self.0[..] {
            index_selection.insert_all(selector.resolve(basic));
        }
        index_selection
    }

    /// Returns true if the selection is trivially empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator of `CellSelector`s.
    pub fn iter(&self) -> impl Iterator<Item=&CellSelector<'name>> {
        self.0.iter()
    }
}

impl<'name> From<CellSelector<'name>> for CellSelection<'name> {
    fn from(selector: CellSelector<'name>) -> Self {
        CellSelection(vec![selector])
    }
}
impl<'name> From<Vec<CellSelector<'name>>> for CellSelection<'name> {
    fn from(selectors: Vec<CellSelector<'name>>) -> Self {
        CellSelection(selectors)
    }
}

impl<'name> FromIterator<CellSelector<'name>> for CellSelection<'name> {
    fn from_iter<I: IntoIterator<Item=CellSelector<'name>>>(iter: I)
        -> CellSelection<'name> 
    {
        CellSelection(Vec::from_iter(iter))
    }
}

impl<'name> IntoIterator for CellSelection<'name> {
    type Item = CellSelector<'name>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'name> std::fmt::Display for CellSelection<'name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Dedicated syntax for empty selection?
        if self.is_empty() { return Ok(()); }

        let mut iter = self.iter();
        for selector in (&mut iter).take(self.0.len() - 1) {
            write!(f, "{}, ", selector)?;
        }
        write!(f, "{}", iter.next().unwrap())
    }
}

impl std::str::FromStr for CellSelection<'static> {
    type Err = FailureOwned;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // Setup parser.
        let scanner = AtmaScanner::new();
        let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
        lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

        // Perform parse.
        cell_selection(lexer)
            .finish()
            .map(CellSelection::into_static)
    }
}


////////////////////////////////////////////////////////////////////////////////
// CellIndexSelection
////////////////////////////////////////////////////////////////////////////////
/// A resolved `CellSelection`, holding a set of indices for `Cell`s in a
/// palette.
///
/// The lifetime of the CellSelector is the lifetime of any names. The set of
/// `Cell`s referenced is fixed, and edits to the palette may invalidate the
/// selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellIndexSelection(Selection<u32>);

impl CellIndexSelection {
    /// Inserts cell indices into the selection from an iterator. Returns the
    /// number of new indices inserted.
    pub fn insert_all<I>(&mut self, indices: I)
        where I: IntoIterator<Item=u32>
    {
        for idx in indices.into_iter() {
            self.0.union_in_place(idx.into())
        }
    }

    /// Returns an iterator oof cell indexes.
    pub fn iter(&self) -> impl Iterator<Item=u32> + '_ {
        self.0.iter()
    }
}

impl FromIterator<u32> for CellIndexSelection {
    fn from_iter<I: IntoIterator<Item=u32>>(iter: I) -> CellIndexSelection {
        CellIndexSelection(Selection::from_iter(iter))
    }
}

impl IntoIterator for CellIndexSelection {
    type Item = u32;
    type IntoIter = normalize_interval::selection::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

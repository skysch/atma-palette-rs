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
use crate::basic::BasicPalette;
use crate::parse::cell_selection;
use crate::parse::circumfix;
use crate::parse::whitespace;
use crate::parse::maybe;
use crate::parse::Failure;
use crate::parse::FailureOwned;
use crate::parse::ParseResultExt as _;
use crate::error::Error;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::collections::BTreeSet;
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
    /// Parses a CellSelection from the given string.
    pub fn parse(input: &'name str) -> Result<Self, Error> {
        circumfix(
            cell_selection,
            maybe(whitespace))
        (input)
            .expect_end_of_text()
            .map_err(Error::from)
            .map(|success| success.value)
    }

    /// Converts a `CellRef` to a static lifetime.
    pub fn clone_static(&self) -> CellSelection<'static> {
        CellSelection(
            self.0
                .iter()
                .map(|selector| selector.clone().into_static())
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

        let mut index_selection = CellIndexSelection(BTreeSet::new());
        for selector in &self.0[..] {
            let _ = index_selection.insert_all(selector.resolve(basic));
        }
        index_selection
    }

    /// Returns true if the selection is trivially empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Moves all `CellSelector`s in `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    /// Pushes a `CellSelector` into the selection.
    pub fn push(&mut self, selector: CellSelector<'name>) {
        self.0.push(selector);
    }

    /// Returns an iterator of `CellSelector`s.
    pub fn iter(&self) -> impl Iterator<Item=&CellSelector<'name>> {
        self.0.iter()
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
        cell_selection(text)
            .expect_end_of_text()
            .map_err(Failure::to_owned)
            .map(|success| success.value.clone_static())
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
#[derive(Serialize, Deserialize)]
// TODO: Implement this using a data structure which handles unions of disjoint
// intervals.
pub struct CellIndexSelection(BTreeSet<u32>);

impl CellIndexSelection {
    /// Inserts cell indices into the selection from an iterator. Returns the
    /// number of new indices inserted.
    pub fn insert_all<I>(&mut self, indices: I) -> usize 
        where I: IntoIterator<Item=u32>
    {
        let mut count = 0;
        for idx in indices.into_iter() {
            if self.0.insert(idx) { count += 1; }
        }
        count
    }

    /// Returns an iterator oof cell indexes.
    pub fn iter(&self) -> impl Iterator<Item=&u32> {
        self.0.iter()
    }
}

impl FromIterator<u32> for CellIndexSelection {
    fn from_iter<I: IntoIterator<Item=u32>>(iter: I) -> CellIndexSelection {
        CellIndexSelection(BTreeSet::from_iter(iter))
    }
}

impl IntoIterator for CellIndexSelection {
    type Item = u32;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

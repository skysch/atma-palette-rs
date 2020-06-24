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
use crate::cell::Position;
use crate::cell::CellRef;
use crate::parse::REF_ALL_TOKEN;
use crate::parse::REF_POS_SEP_TOKEN;
use crate::parse::REF_PREFIX_TOKEN;
use crate::parse::REF_RANGE_TOKEN;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::convert::TryFrom;
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
pub struct CellIndexSelection(BTreeSet<u32>);

impl CellIndexSelection {
    /// Moves all indices in `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0)
    }

    /// Inserts a cell index into the selection. Returns true if the element is
    /// index is new.
    pub fn insert(&mut self, idx: u32) -> bool {
        self.0.insert(idx)
    }

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

////////////////////////////////////////////////////////////////////////////////
// CellSelector
////////////////////////////////////////////////////////////////////////////////
/// A reference to a `Cell` in a palette.
///
/// The lifetime of the CellSelector is the lifetime of any names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum CellSelector<'name> {
    /// Select all cells.
    All,

    /// Select the cell with the given index.
    Index(u32),
    
    /// Select all cells within the given indices. The bounds are inclusive.
    IndexRange {
        /// The lower bound (inclusive) of the selection.
        low: u32,
        /// The upper bound (inclusive) of the selection.
        high: u32,
    },

    /// Select all cells identified by the given PositionSelector.
    PositionSelector(PositionSelector),

    /// Select all cells within the given positions. The bounds are inclusive.
    PositionRange {
        /// The lower bound (inclusive) of the selection.
        low: Position,
        /// The upper bound (inclusive) of the selection.
        high: Position
    },

    /// Select the cell with the given name.
    Name(Cow<'name, str>),

    /// Select the cell within the given group with the given index.
    Group {
        /// The name of the group.
        group: Cow<'name, str>,
        /// The index of the cell within the group.
        idx: u32,
    },

    /// Select alls cells within the given group within the given indices.
    /// The bounds are inclusive.
    GroupRange {
        /// The name of the group.
        group: Cow<'name, str>,
        /// The lower bound (inclusive) of the selection.
        low: u32,
        /// The upper bound (inclusive) of the selection.
        high: u32,
    },

    /// Select alls cells within the given group.
    GroupAll(Cow<'name, str>),
}

impl<'name> CellSelector<'name> {
    /// Converts a `CellSelector` to a static lifetime.
    pub fn into_static(self) -> CellSelector<'static> {
        use CellSelector::*;
        match self {
            All => All,
            Index(idx) => Index(idx),
            IndexRange { low, high } => IndexRange { low, high },
            PositionSelector(pos_sel) => PositionSelector(pos_sel),
            PositionRange { low, high } => PositionRange { low, high },
            Name(name) => Name(Cow::from(name.into_owned())),
            Group { group, idx } => Group {
                group: Cow::from(group.into_owned()),
                idx,
            },
            GroupRange { group, low, high } => GroupRange {
                group: Cow::from(group.into_owned()),
                low,
                high
            },
            GroupAll(group) => GroupAll(Cow::from(group.into_owned())),
        }
    }

    /// Constructs a `CellSelecto::IndexRange` from its indices.
    pub fn index_range(low: u32, high: u32)
        -> Result<CellSelector<'name>, InvalidCellSelector>
    {
        if low >= high {
            Err(InvalidCellSelector::range_mismatch(
                CellRef::Index(low),
                CellRef::Index(high)))
        } else {
            Ok(CellSelector::IndexRange { low, high })
        }
    }

    /// Constructs a `CellSelecto::PositionRange` from its positions.
    pub fn position_range(low: Position, high: Position)
        -> Result<CellSelector<'name>, InvalidCellSelector>
    {
        if low >= high {
            Err(InvalidCellSelector::range_mismatch(
                CellRef::Position(low),
                CellRef::Position(high)))
        } else {
            Ok(CellSelector::PositionRange { low, high })
        }
    }

    /// Constructs a `CellSelecto::GroupRange` from its group names and indices.
    pub fn group_range(
        group_low: Cow<'name, str>,
        idx_low: u32,
        group_high: Cow<'name, str>,
        idx_high: u32)
        -> Result<CellSelector<'name>, InvalidCellSelector>
    {
        if group_low != group_high {
            Err(InvalidCellSelector::range_mismatch(
                CellRef::Group { group: group_low, idx: idx_low },
                CellRef::Group { group: group_high, idx: idx_high }))

        } else if idx_low >= idx_high {
            Err(InvalidCellSelector::range_order(
                CellRef::Group { group: group_low, idx: idx_low },
                CellRef::Group { group: group_high, idx: idx_high }))

        } else {
            Ok(CellSelector::GroupRange { 
                group: group_low,
                low: idx_low,
                high: idx_high,
            })
        }
    }
}


impl<'name> std::fmt::Display for CellSelector<'name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CellSelector::*;
        match self {
            All => write!(f, "{}", REF_ALL_TOKEN),
            Index(idx) => write!(f, "{}{}", REF_PREFIX_TOKEN, idx),
            IndexRange { low, high } => write!(f, "{}{}{}{}{}",
                REF_PREFIX_TOKEN, low, REF_RANGE_TOKEN, REF_PREFIX_TOKEN, high),
            PositionSelector(pos_sel) => write!(f, "{}", pos_sel),
            PositionRange { low, high } => write!(f, 
                "{}{}{}", low, REF_RANGE_TOKEN, high),
            Name(name) => write!(f, "{}", name),
            Group { group, idx } => write!(f, 
                "{}{}{}", group, REF_PREFIX_TOKEN, idx),
            GroupRange { group, low, high } => write!(f, "{}{}{}{}{}{}{}",
                group, REF_PREFIX_TOKEN, low, REF_RANGE_TOKEN,
                group, REF_PREFIX_TOKEN, high),
            GroupAll(group) => write!(f, 
                "{}{}{}", group, REF_PREFIX_TOKEN, REF_ALL_TOKEN),
        }
    }
}


impl<'name> From<CellRef<'name>> for CellSelector<'name> {
    fn from(cell_ref: CellRef<'name>) -> Self {
        match cell_ref {
            CellRef::Index(idx) => CellSelector::Index(idx),
            CellRef::Position(pos)
                => CellSelector::PositionSelector(pos.into()),
            CellRef::Name(name) => CellSelector::Name(name),
            CellRef::Group { group, idx } => CellSelector::Group { group, idx },
        }
    }
}

impl<'name> TryFrom<(CellRef<'name>, CellRef<'name>)> for CellSelector<'name> {
    type Error = InvalidCellSelector;
    fn try_from((low, high): (CellRef<'name>, CellRef<'name>))
        -> Result<Self, Self::Error>
    {
        match (low, high) {
            (CellRef::Index(low), CellRef::Index(high))
                => CellSelector::index_range(low, high),

            (CellRef::Position(low), CellRef::Position(high))
                => CellSelector::position_range(low, high),

            (CellRef::Group { group: group_low, idx: idx_low },
                    CellRef::Group { group: group_high, idx: idx_high })
                => CellSelector::group_range(
                    group_low,
                    idx_low,
                    group_high,
                    idx_high),

            (low, high) => Err(InvalidCellSelector::range_mismatch(low, high)),
        }
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

impl From<Position> for PositionSelector {
    fn from(pos: Position) -> Self {
        PositionSelector {
            page: Some(pos.page),
            line: Some(pos.line),
            column: Some(pos.column),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// InvalidCellSelector
////////////////////////////////////////////////////////////////////////////////

/// A cell selector range was invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidCellSelector {
    /// A range Cellref with incompatable bounds.
    RangeMismatch {
        /// The range's lower bound.
        low: Cow<'static, str>,
        /// The range's upper bound.
        high: Cow<'static, str>,
    },
    /// A range Cellref with invalid bound ordering.
    RangeOrder {
        /// The range's lower bound.
        low: Cow<'static, str>,
        /// The range's upper bound.
        high: Cow<'static, str>,
    },
}

impl InvalidCellSelector {
    /// Constructs an `InvalidCellSelector::RangeMismatch` from `CellRef`
    /// bounds.
    pub fn range_mismatch<'name>(low: CellRef<'name>, high: CellRef<'name>)
        -> Self
    {
        InvalidCellSelector::RangeMismatch {
            low: format!("{}", low).into(),
            high: format!("{}", high).into(),
        }
    }

    /// Constructs an `InvalidCellSelector::RangeOrder` from `CellRef` bounds.
    pub fn range_order<'name>(low: CellRef<'name>, high: CellRef<'name>)
        -> Self
    {
        InvalidCellSelector::RangeOrder {
            low: format!("{}", low).into(),
            high: format!("{}", high).into(),
        }
    }
}


impl std::fmt::Display for InvalidCellSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InvalidCellSelector::*;
        write!(f, "invalid cell selector: ")?;
        match self {
            RangeMismatch { low, high } => write!(f,
                "range with lower bound '{}' \
                    incompatable with upper bound `{}`",
                low, high),
            RangeOrder { low, high } => write!(f, "range lower bound '{}'\
                exceeds range upper bound '{}'",
                low, high),
        }
    }
}

impl std::error::Error for InvalidCellSelector {}

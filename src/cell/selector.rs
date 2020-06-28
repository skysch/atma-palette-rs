////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette cell selectors for selecting ranges of cells.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::basic::BasicPalette;
use crate::cell::Position;
use crate::cell::CellRef;
use crate::utility::inclusive_range_intersect as intersect;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::borrow::Cow;
use std::convert::TryFrom;


////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////

/// The CellSelector 'all' selection token.
pub const REF_ALL_TOKEN: char = '*';

/// The CellSelector position separator token.
pub const REF_POS_SEP_TOKEN: char = '.';

/// The CellSelector index prefix token.
pub const REF_PREFIX_TOKEN: char = ':';

/// The CellSelector range separator token.
pub const REF_RANGE_TOKEN: char = '-';

/// The CellSelection list separator token.
pub const REF_SEP_TOKEN: char = ',';


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
        if low > high {
            Err(InvalidCellSelector::range_mismatch(
                CellRef::Index(low),
                CellRef::Index(high)))
        } else if low == high {
            Ok(CellSelector::Index(low))
        } else {
            Ok(CellSelector::IndexRange { low, high })
        }
    }

    /// Constructs a `CellSelecto::PositionRange` from its positions.
    pub fn position_range(low: Position, high: Position)
        -> Result<CellSelector<'name>, InvalidCellSelector>
    {
        if low > high {
            Err(InvalidCellSelector::range_mismatch(
                CellRef::Position(low),
                CellRef::Position(high)))
        } else if low == high {
            Ok(CellSelector::PositionSelector(low.into()))
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

        } else if idx_low > idx_high {
            Err(InvalidCellSelector::range_order(
                CellRef::Group { group: group_low, idx: idx_low },
                CellRef::Group { group: group_high, idx: idx_high }))

        } else if idx_low == idx_high {
            Ok(CellSelector::Group { group: group_low, idx: idx_low })
        } else {
            Ok(CellSelector::GroupRange { 
                group: group_low,
                low: idx_low,
                high: idx_high,
            })
        }
    }

    /// Returns an ordered iterator over the selected, occupied indices within
    /// the given palette.
    pub fn resolve<'p>(&self, basic: &'p BasicPalette)
        -> impl Iterator<Item=u32>
    {
        self.index_iter(basic)
            .collect::<std::collections::BTreeSet<u32>>()
            .into_iter()
    }

    /// Returns an index iterator for the selector within the given palette.
    fn index_iter<'p>(&self, basic: &'p BasicPalette)
        -> CellSelectorIndexIter<'name, 'p>
    {
        let mut pos_selector = PositionSelector::all();
        let selector = {
            use CellSelector::*;
            match self {
                All => basic
                    .occupied_index_range()
                    .map(|(low, high)| if low == high { 
                            Index(low)
                        } else {
                            IndexRange { low, high }
                        }),

                Index(idx) => Some(Index(*idx))
                    .filter(|_| basic.is_occupied_index(idx)),

                IndexRange { low, high } => basic
                    .occupied_index_range()
                    .and_then(|(l, h)| intersect((*low, *high), (l, h)))
                    .map(|(low, high)| IndexRange { low, high }),

                Name(name) => basic
                    .resolve_name_if_occupied(&name)
                    .map(Index),

                Group { group, idx } => basic
                    .resolve_group_if_occupied(group, *idx)
                    .map(Index),
                
                GroupAll(group) => basic
                    .assigned_group_range(group)
                    .map(|(low, high)| GroupRange {
                        group: group.clone(),
                        low,
                        high,
                    }),
                
                GroupRange { group, low, high } => basic
                    .assigned_group_range(group)
                    .and_then(|(l, h)| intersect((*low, *high), (l, h)))
                    .map(|(low, high)| GroupRange {
                        group: group.clone(),
                        low,
                        high,
                    }),

                PositionRange { low, high } => basic
                    .assigned_position_range()
                    .and_then(|(l, h)| intersect((*low, *high), (l, h)))
                    .map(|(low, high)| PositionRange { low, high }),

                PositionSelector(position_selector) => {
                    pos_selector = position_selector.clone();
                    basic.assigned_position_range()
                        .and_then(|(l, h)| {
                            let (low, high) = position_selector.bounds();
                            intersect((low, high), (l, h))
                        })
                        .map(|(low, high)| PositionRange { low, high })
                }
            }
        };
        CellSelectorIndexIter {
            basic,
            selector,
            pos_selector,
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
// CellSelectorIter
////////////////////////////////////////////////////////////////////////////////
/// An iterator over the valid palette indices selected by a CellSelector.
#[derive(Debug)]
struct CellSelectorIndexIter<'t, 'p> {
    basic: &'p BasicPalette,
    selector: Option<CellSelector<'t>>,
    pos_selector: PositionSelector,
} 

impl<'t, 'p> std::iter::FusedIterator for CellSelectorIndexIter<'t, 'p> {}

impl<'t, 'p> Iterator for CellSelectorIndexIter<'t, 'p> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        use CellSelector::*;
        match self.selector.take() {
            None => None,

            Some(Index(idx)) => {
                self.selector = None;
                if self.basic.is_occupied_index(&idx) {
                    Some(idx)
                } else {
                    None
                }
            },
            
            Some(IndexRange { low, high }) => match self.basic
                .next_occupied_index_after(&low)
            {
                Some(idx) if self.basic.is_occupied_index(&low) => {
                    self.selector = Some(IndexRange { low: *idx, high });
                    Some(low)
                },
                None                      => { self.selector = None; None },
                Some(idx) if *idx > high  => { self.selector = None; None },
                Some(idx) if *idx == high => {
                    self.selector = None; 
                    Some(high) 
                },
                Some(idx) => {
                    self.selector = Some(IndexRange { low: *idx, high });
                    Some(*idx)
                },
            },            
            
            Some(GroupRange { group, low, high }) => match self.basic
                .next_occupied_group_index_after(&group, low)
            {
                Some(idx)if self.basic.is_occupied_group(&group, low) => {
                    self.selector = Some(GroupRange {
                        group,
                        low: idx,
                        high,
                    });
                    Some(low)
                },
                None                     => { self.selector = None; None },
                Some(idx) if idx > high  => { self.selector = None; None },
                Some(idx) if idx == high => {
                    self.selector = None; 
                    Some(high)
                },
                Some(idx) => {
                    self.selector = Some(GroupRange { group, low: idx, high });
                    Some(idx)
                },
            },

            Some(PositionRange { low, high }) => match self.basic
                .next_occupied_position_after(&low)
            {
                Some((pos, _)) if self.basic.is_occupied_position(&low) && 
                    self.pos_selector.contains(&low) => 
                {
                    let idx = self.basic
                        .resolve_position_if_occupied(&low)
                        .unwrap();
                    self.selector = Some(PositionRange {
                        low: *pos,
                        high,
                    });
                    Some(idx)
                },
                None                          => { self.selector = None; None },
                Some((pos, _)) if pos > &high => { self.selector = None; None },
                Some((pos, idx)) if pos == &high => {
                    self.selector = None; 
                    if self.pos_selector.contains(&pos) {
                        Some(*idx)
                    } else {
                        None
                    }
                },
                Some((pos, idx)) => {
                    if self.pos_selector.contains(&pos) {
                        self.selector = Some(PositionRange { 
                            low: *pos,
                            high,
                        });
                        return Some(*idx)
                    }
                    let mut cur_pos = pos;
                    loop {
                        match self.basic.next_occupied_position_after(&cur_pos)
                        {
                            Some((pos, idx)) if self
                                .pos_selector.contains(&pos) => 
                            {
                                self.selector = Some(PositionRange { 
                                    low: *pos,
                                    high,
                                });
                                return Some(*idx)
                            },
                            Some((pos, _)) => { cur_pos = pos; },
                            None => {
                                self.selector = None; 
                                return None;
                            }
                        }
                    }
                },
            },

            // Other variants can be mapped out during construction:
            // * All should be handled by IndexRange.
            // * Name should be handled by Index.
            // * Group should be handled by Index.
            // * GroupAll should be handled by GroupRange.
            // * PositionSelector should be handled by PositionRange.
            Some(_) => unreachable!(),
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

impl PositionSelector {
    /// Returns the PositionSelector which selects all positions.
    pub fn all() -> Self {
        PositionSelector {
            page: None,
            line: None,
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
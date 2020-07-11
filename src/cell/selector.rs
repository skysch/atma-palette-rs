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
use crate::palette::BasicPalette;
use crate::cell::CellRef;
use crate::cell::PositionSelector;
use crate::cell::Position;
use crate::utility::Few;


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
/// A reference to a set of `Cell`s in a palette.
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
    /// Returns true if the selector will trivially select the entire palette
    /// without needing to be resolved.
    pub fn is_all_selector(&self) -> bool {
        match self {
            CellSelector::All => true,
            CellSelector::PositionSelector(PositionSelector {
                page: None,
                line: None,
                column: None,
            }) => true,
            _ => false,
        }
    }

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
        let mut pos_selector = PositionSelector::ALL;
        let selector = {
            use CellSelector::*;
            match self {
                All => match basic.occupied_index_range() {
                    Few::Two(low, high) => Some(IndexRange { low, high }),
                    Few::One(idx)       => Some(Index(idx)),
                    Few::Zero           => None,
                },
                Index(idx) => Some(Index(*idx))
                    .filter(|_| basic.is_occupied_index(idx)),

                IndexRange { low, high } => match basic
                    .occupied_index_subrange(*low, *high)
                {
                    Few::Two(low, high) => Some(IndexRange { low, high }),
                    Few::One(idx)       => Some(Index(idx)),
                    Few::Zero           => None,
                },

                Name(name) => basic
                    .resolve_name_if_occupied(&name)
                    .map(Index),

                Group { group, idx } => basic
                    .resolve_group_if_occupied(group, *idx)
                    .map(Index),
                
                GroupAll(group) => match basic.assigned_group_range(group) {
                    Few::Two(low, high) => Some(GroupRange {
                        group: group.clone(),
                        low,
                        high,
                    }),
                    Few::One(idx)       => basic
                        .resolve_group_if_occupied(group, idx)
                        .map(Index),
                    Few::Zero           => None,
                },
                
                GroupRange { group, low, high } => match basic
                    .assigned_group_subrange(group, *low, *high)
                {
                    Few::Two(low, high) => Some(GroupRange {
                        group: group.clone(),
                        low,
                        high,
                    }),
                    Few::One(idx)       => basic
                        .resolve_group_if_occupied(group, idx)
                        .map(Index),
                    Few::Zero           => None,
                },

                PositionRange { low, high } => match basic
                    .assigned_position_subrange(*low, *high)
                {
                    Few::Two(low, high) => Some(PositionRange {
                        low,
                        high,
                    }),
                    Few::One(pos)       => basic
                        .resolve_position_if_occupied(&pos)
                        .map(Index),
                    Few::Zero           => None,
                },

                PositionSelector(position_selector) => {
                    pos_selector = position_selector.clone();
                    let (low, high) = position_selector.bounds();
                    match basic.assigned_position_subrange(low, high) {
                        Few::Two(low, high) => Some(PositionRange {
                            low,
                            high,
                        }),
                        Few::One(pos)       => basic
                            .resolve_position_if_occupied(&pos)
                            .map(Index),
                        Few::Zero           => None,
                    }
                },
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
                Some(idx)
                    .filter(|l| self.basic.is_occupied_index(l))
            },
            
            Some(IndexRange { low, high }) => {
                let mut low = low;
                let mut res = None;

                while res.is_none() {
                    res = Some(low)
                        .filter(|l| self.basic.is_occupied_index(l));
                    low += 1;
                    self.selector = match self.basic
                        .occupied_index_subrange(low, high)
                    {
                        Few::Two(l, h) => {
                            low = l;
                            Some(IndexRange {
                                low: l,
                                high: h,
                            })
                        },
                        Few::One(idx)       => Some(Index(idx)),
                        Few::Zero           => None,
                    };
                    if self.selector.is_none() { break; }
                }
                res
            },

            Some(GroupRange { group, low, high }) => {
                let mut low = low;
                let mut res = None;

                while res.is_none() {
                    res = self.basic.resolve_group_if_occupied(&group, low);
                    low += 1;
                    self.selector = match self.basic
                        .assigned_group_subrange(&group, low, high)
                    {
                        Few::Two(l, h) => {
                            low = l;
                            Some(GroupRange {
                                group: group.clone(),
                                low: l,
                                high: h,
                            })
                        },
                        Few::One(idx)       => self.basic
                            .resolve_group_if_occupied(&group, idx)
                            .map(Index),
                        Few::Zero           => None,
                    };
                    if self.selector.is_none() { break; }
                }
                res
            },

            Some(PositionRange { low, high }) => {
                let mut low = low;
                let mut res = None;

                while res.is_none() {
                    res = self.basic
                        .resolve_position_if_occupied(&low)
                        .filter(|_| self.pos_selector.contains(&low));
                    low = low.succ();
                    self.selector = match self.basic
                        .assigned_position_subrange(low, high)
                    {
                        Few::Two(l, h) => {
                            low = l; // Skip unassigned positions.
                            Some(PositionRange {
                                low: l,
                                high: h,
                            })
                        },
                        Few::One(pos)       => self.basic
                            .resolve_position_if_occupied(&pos)
                            .filter(|_| self.pos_selector.contains(&pos))
                            .map(Index),
                        Few::Zero           => None,
                    };
                    if self.selector.is_none() { break; }
                }
                res
            },

            // Other variants should be mapped out during iterator construction:
            // * All should be handled by IndexRange.
            // * Name should be resolved and handled by Index.
            // * Group should be resolved and handled by Index.
            // * GroupAll should be handled by GroupRange.
            // * PositionSelector should be handled by PositionRange.
            Some(_) => unreachable!(),
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

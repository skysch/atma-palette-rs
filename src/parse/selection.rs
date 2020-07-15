////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Cell selection parsing.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellRef;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::Position;
use crate::cell::PositionSelector;
use crate::parse::char;
use crate::parse::char_matching;
use crate::parse::circumfix;
use crate::parse::pair;
use crate::parse::atomic_ignore_whitespace;
use crate::parse::intersperse_collect;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::repeat;
use crate::parse::uint;
use crate::parse::whitespace;



////////////////////////////////////////////////////////////////////////////////
// Parse cell selections.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellSelection.
pub fn cell_selection<'t>(text: &'t str) -> ParseResult<'t, CellSelection<'t>> {
    // TODO: Permit empty selection?
    intersperse_collect(1, None,
            cell_selector,
            circumfix(
                char(','),
                maybe(whitespace)))
        (text)
        .source_for("cell selection")
        .map_value(CellSelection::from)
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell selectors.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellSelector.
pub fn cell_selector<'t>(text: &'t str) -> ParseResult<'t, CellSelector<'t>> {
    // Parse All.
    let all = char('*')(text);
    if all.is_ok() {
        return all.map_value(|_| CellSelector::All);
    }

    // Parse Position or PositionRange.
    let pos = pair(
            position,
            atomic_ignore_whitespace(range_suffix(position)))
        (text);
    if pos.is_ok() {
        return pos.convert_value(
                "valid position range", 
                |(pos, pos_up)| if let Some(end) = pos_up {
                    CellSelector::position_range(pos.into(), end.into())
                } else {
                    Ok(CellSelector::PositionSelector(pos.into()))
                })
            .with_failure_rest(text);
    }

    // Parse PositionSelector.
    let pos_sel = position_selector(text);
    if pos_sel.is_ok() {
        return pos_sel.map_value(CellSelector::PositionSelector);
    }

    // Parse Index or IndexRange.
    let idx = pair(
            index,
            atomic_ignore_whitespace(range_suffix(index)))
        (text);
    if idx.is_ok() {
        return idx.convert_value(
                "valid index range", 
                |(idx, idx_up)| if let Some(end) = idx_up {
                    CellSelector::index_range(idx.into(), end.into())
                } else {
                    Ok(CellSelector::Index(idx.into()))
                })
            .with_failure_rest(text);
    }

    // Parse GroupAll.
    let grp_all = group_all(text);
    if grp_all.is_ok() {
        return grp_all.map_value(|g| CellSelector::GroupAll(g.into()));
    }

    // Parse Group or GroupRange.
    let grp = pair(
            group,
            atomic_ignore_whitespace(range_suffix(group)))
        (text);
    if grp.is_ok() {
        return grp.convert_value(
                "valid group range", 
                |(grp, grp_up)| if let Some(end) = grp_up {
                    CellSelector::group_range(
                        grp.0.into(),
                        grp.1,
                        end.0.into(),
                        end.1)
                } else {
                    Ok(CellSelector::Group {
                        group: grp.0.into(),
                        idx: grp.1,
                    })
                })
            .with_failure_rest(text);
    }

    // Parse Name.
    name
        (text)
        .map_value(|n| CellSelector::Name(n.into()))
}

/// Parses a PositionSelector.
pub fn position_selector<'t>(text: &'t str)
    -> ParseResult<'t, PositionSelector>
{
    let (page, suc) = prefix(u16_or_all, char(':'))
        (text)?
        .take_value();
    
    let (line, suc) = prefix(u16_or_all, char('.'))
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    prefix(u16_or_all, char('.'))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|column| PositionSelector { page, line, column })    
}

/// Parses a u16 or an 'all' selector token. Return Some if a u16 was parsed, or
/// None if 'all' was parsed.
pub fn u16_or_all<'t>(text: &'t str) -> ParseResult<'t, Option<u16>> {
    let all = char('*')
        (text);
    if all.is_ok() {
        return all.map_value(|_| None);
    }

    uint::<u16>("u16")
        (text)
        .map_value(Some)
}

/// Returns a parser which parses the separator and upper bound of a range using
/// the given parser.
pub fn range_suffix<'t, F, V>(parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    prefix(
        parser, 
        circumfix(
            char('-'),
            maybe(whitespace)))
}

/// Parses a group all selector.
pub fn group_all<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    postfix(
        name, 
        postfix(
            char(':'),
            char('*')))
        (text)
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell refs.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellRef.
pub fn cell_ref<'t>(text: &'t str) -> ParseResult<'t, CellRef<'t>> {
    // Parse Position.
    let position = position(text);
    if position.is_ok() {
        return position.map_value(CellRef::Position);
    }

    // Parse Index.
    let index = index(text);
    if index.is_ok() {
        return index.map_value(CellRef::Index);
    }

    // Parse Group.
    let group = group(text);
    if group.is_ok() {
        return group.map_value(|(group, idx)| CellRef::Group {
            group: group.into(),
            idx,
        });
    }

    // Parse Name.
    name
        (text)
        .map_value(|name| CellRef::Name(name.into()))
}

/// Parses a Position.
pub fn position<'t>(text: &'t str) -> ParseResult<'t, Position> {
    let (page, suc) = prefix(uint::<u16>("u16"), char(':'))
        (text)?
        .take_value();
    
    let (line, suc) = prefix(uint::<u16>("u16"), char('.'))
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    prefix(uint::<u16>("u16"), char('.'))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|column| Position { page, line, column })    
}

/// Parses a Index.
pub fn index<'t>(text: &'t str) -> ParseResult<'t, u32> {
    prefix(
            uint::<u32>("u32"),
            char(':'))
        (text)
        .source_for("cell ref index")
}


/// Parses a name.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    // TODO: Parse names as text strings requiring delimitters.
    let valid_char = char_matching(|c| ![
        '*',
        ',',
        '.',
        ':',
        '-',
        ')',
        '(',
    ].contains(&c) && !c.is_whitespace());

    repeat(1, None,
            valid_char)
        (text)
        .tokenize_value()
}

/// Parses a group name with its index.
pub fn group<'t>(text: &'t str) -> ParseResult<'t, (&'t str, u32)> {
    pair(name, index)
        (text)
}

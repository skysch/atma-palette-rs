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
use crate::parse::Failure;
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
    if let Ok(all_suc) = char('*')(text) {
        Ok(all_suc.map_value(|_| CellSelector::All))

    } else if let Ok(pos_suc) = position(text) {
        if let Ok(ub_suc) = range_suffix(position)(pos_suc.rest) {
            match CellSelector::position_range(pos_suc.value, ub_suc.value) {
                Ok(pos_range) => Ok(pos_suc.join_with(ub_suc, text,
                    |_, _| pos_range)),
                Err(range_err) => Err(Failure {
                    token: pos_suc.join(ub_suc, text).token,
                    expected: "valid position range".into(),
                    source: Some(Box::new(range_err)),
                    rest: text,
                }),
            }
        } else {
            Ok(pos_suc.map_value(
                |pos| CellSelector::PositionSelector(pos.into())))
        }

    } else if let Ok(pos_sel_suc) = position_selector(text) {
            Ok(pos_sel_suc.map_value(
                |pos_sel| CellSelector::PositionSelector(pos_sel)))

    } else if let Ok(index_suc) = index(text) {
        if let Ok(ub_suc) = range_suffix(index)(index_suc.rest) {
            match CellSelector::index_range(index_suc.value, ub_suc.value) {
                Ok(index_range) => Ok(index_suc.join_with(ub_suc, text,
                    |_, _| index_range)),
                Err(range_err) => Err(Failure {
                    token: index_suc.join(ub_suc, text).token,
                    expected: "valid index range".into(),
                    source: Some(Box::new(range_err)),
                    rest: text,
                }),
            }
        } else {
            Ok(index_suc.map_value(
                |index| CellSelector::Index(index)))
        }

    } else if let Ok(group_all_suc) = group_all(text) {
            Ok(group_all_suc.map_value(
                |group| CellSelector::GroupAll(group.into())))

    } else if let Ok(group_suc) = group(text) {
        if let Ok(ub_suc) = range_suffix(group)(group_suc.rest) {
            match CellSelector::group_range(
                group_suc.value.0.into(),
                group_suc.value.1,
                ub_suc.value.0.into(),
                ub_suc.value.1)
            {
                Ok(group_range) => Ok(group_suc.join_with(ub_suc, text,
                    |_, _| group_range)),
                Err(range_err) => Err(Failure {
                    token: group_suc.join(ub_suc, text).token,
                    expected: "valid group range".into(),
                    source: Some(Box::new(range_err)),
                    rest: text,
                }),
            }
        } else {
            Ok(group_suc.map_value(|(group, idx)| CellSelector::Group {
                group: group.into(), 
                idx 
            }))
        }

    } else {
        name(text)
            .map_value(|name| CellSelector::Name(name.into()))
            .source_for("cell selector value")
    }
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
    if let Ok(all_suc) = char('*')(text) {
        Ok(all_suc.map_value(|_| None))
    } else {
        uint::<u16>("u16")(text).map_value(Some)
    }
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
    if let Ok(pos_suc) = position(text) {
        Ok(pos_suc.map_value(|pos| CellRef::Position(pos)))
    
    } else if let Ok(idx_suc) = index(text) {
        Ok(idx_suc.map_value(|idx| CellRef::Index(idx)))
    
    } else if let Ok(group_suc) = group(text) {
        Ok(group_suc.map_value(|(group, idx)| CellRef::Group {
            group: group.into(),
            idx,
        }))

    } else {
        name(text)
            .map_value(|name| CellRef::Name(name.into()))
            .source_for("cell ref value")
    }
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
    prefix(uint::<u32>("u32"), char(':'))
        (text)
        .source_for("cell ref index")
}


/// Parses a name.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
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
    let (group, suc) = name
        (text)?
        .take_value();

    index
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|idx| (group, idx))
}

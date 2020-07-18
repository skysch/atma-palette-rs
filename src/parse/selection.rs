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
use crate::parse::any_literal_map;
use crate::parse::atomic_ignore_whitespace;
use crate::parse::char;
use crate::parse::circumfix;
use crate::parse::escaped_string;
use crate::parse::Failure;
use crate::parse::intersperse_collect;
use crate::parse::literal;
use crate::parse::literal_once;
use crate::parse::maybe;
use crate::parse::pair;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::uint;
use crate::parse::whitespace;

// Standard library imports.
use std::borrow::Cow;


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
        return grp_all.map_value(|group| CellSelector::GroupAll(group));
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
                    CellSelector::group_range(grp.0, grp.1, end.0, end.1)
                } else {
                    Ok(CellSelector::Group { group: grp.0, idx: grp.1 })
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
pub fn group_all<'t>(text: &'t str) -> ParseResult<'t, Cow<'t, str>> {
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
        return group.map_value(|(group, idx)| CellRef::Group { group, idx });
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



/// Parses a group name with its index.
pub fn group<'t>(text: &'t str) -> ParseResult<'t, (Cow<'t, str>, u32)> {
    pair(name, index)
        (text)
}


/// Parses a name.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, Cow<'t, str>> {
    escaped_string(
            name_open,
            name_close,
            name_escape)
        (text)
        .map_value(|v| v.into())
}

/// Parses a name opening quote. For use with escaped_string.
pub fn name_open<'t>(text: &'t str)
    -> ParseResult<'t, (Cow<'static, str>, bool)>
{
    any_literal_map(
            literal,
            "name open quote",
            vec![
                ("r\"", ("\"".into(), false)),
                ("\"",  ("\"".into(), true)),
                ("'",   ("'".into(),  true)),
            ])
        (text)
}

/// Parses a name closing quote. For use with escaped_string.
pub fn name_close<'t, 'o: 't>(text: &'t str, open: Cow<'o, str>)
    -> ParseResult<'t, &'t str>
{
    literal_once(open.as_ref())(text)
}

/// Parses a name escape character. For use with escaped_string.
pub fn name_escape<'t>(text: &'t str, is_escaped: bool)
    -> ParseResult<'t, &'static str>
{
    if !is_escaped {
        return Err(Failure {
            token: "",
            rest: text,
            expected: "".to_owned().into(),
            source: None,
        })
    }

    any_literal_map(
            literal,
            "name escape",
            vec![
                (r#"\n"#, "\n"),
                (r#"\t"#, "\t"),
                (r#"\""#, "\""),
                (r#"\\"#, "\\"),
            ])
        (text)
}

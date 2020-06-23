////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser helpers.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::parse::*;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::selection::CellSelector;
use crate::selection::PositionSelector;

// Standard library imports.
// use std::convert::TryInto;
// use std::convert::TryFrom;

pub(crate) const REF_ALL_TOKEN: char = '*';
pub(crate) const REF_POS_SEP_TOKEN: char = '.';
pub(crate) const REF_PREFIX_TOKEN: char = ':';
pub(crate) const REF_RANGE_TOKEN: char = '-';
pub(crate) const REF_SEP_TOKEN: char = ',';




////////////////////////////////////////////////////////////////////////////////
// Parse cell selections.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellSelection.
pub fn cell_selection<'t>(text: &'t str)
    -> ParseResult<'t, Vec<CellSelector<'t>>>
{
    unimplemented!()
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell selectors.
////////////////////////////////////////////////////////////////////////////////
/// Parses a CellSelector.
pub fn cell_selector<'t>(text: &'t str) -> ParseResult<'t, CellSelector<'t>> {
    unimplemented!()
}

/// Parses a PositionSelector.
pub fn position_selector<'t>(text: &'t str)
    -> ParseResult<'t, PositionSelector>
{
    unimplemented!()
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell refs.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellRef.
pub fn cell_ref<'t>(text: &'t str) -> ParseResult<'t, CellRef<'t>> {
    unimplemented!()
}

/// Parses a Position.
pub fn position<'t>(text: &'t str) -> ParseResult<'t, Position> {
    let pre = char(text, REF_PREFIX_TOKEN)
        .with_parse_context("", text)
        .into_source_for("cell ref position prefix")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let page = uint::<u16>(pre.rest, "u16")
        .with_parse_context(context, text)
        .into_source_for("cell ref position page")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let sep1 = char(page.rest, REF_POS_SEP_TOKEN)
        .with_parse_context(context, text)
        .into_source_for("cell ref position separator")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let line = uint::<u16>(sep1.rest, "u16")
        .with_parse_context(context, text)
        .into_source_for("cell ref position line")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let sep2 = char(line.rest, REF_POS_SEP_TOKEN)
        .with_parse_context(context, text)
        .into_source_for("cell ref position separator")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let column = uint::<u16>(sep2.rest, "u16")
        .with_parse_context(context, text)
        .into_source_for("cell ref position column")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    Ok(Success {
        value: Position {
            page: page.value,
            line: line.value,
            column: column.value,
        },
        token: context,
        rest: column.rest,
    })
}

/// Parses a Index.
pub fn index<'t>(text: &'t str) -> ParseResult<'t, u32> {
    let pre = char(text, REF_PREFIX_TOKEN)
        .with_parse_context("", text)
        .into_source_for("cell ref index prefix")?;

    uint::<u32>(pre.rest, "u32")
        .with_parse_context(pre.token, text)
        .into_source_for("cell ref index prefix")
}


/// Parses a name or group.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    let res = one_or_more(text, |text| 
        char_matching(text, |c| ![
                REF_ALL_TOKEN,
                REF_SEP_TOKEN,
                REF_POS_SEP_TOKEN,
                REF_PREFIX_TOKEN,
                REF_RANGE_TOKEN,
            ].contains(&c)))
        .with_parse_context("", text)
        .into_source_for("cell ref name")?;

    let context = &text[0..(text.len() - res.rest.len())];
    Ok(Success {
        value: context,
        token: context,
        rest: res.rest,
    })
}

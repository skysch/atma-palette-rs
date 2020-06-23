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
    let pre = char(REF_PREFIX_TOKEN)(text)
        .with_parse_context("", text)
        .source_for("cell selector position selector prefix")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let page = uin16_or_all(pre.rest)
        .with_parse_context(context, text)
        .source_for("cell selector position selector page")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let sep1 = char(REF_POS_SEP_TOKEN)(page.rest)
        .with_parse_context(context, text)
        .source_for("cell selector position selector separator")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let line = uin16_or_all(sep1.rest)
        .with_parse_context(context, text)
        .source_for("cell selector position selector line")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let sep2 = char(REF_POS_SEP_TOKEN)(line.rest)
        .with_parse_context(context, text)
        .source_for("cell selector position selector separator")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let column = uin16_or_all(sep2.rest)
        .with_parse_context(context, text)
        .source_for("cell selector position selector column")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    Ok(Success {
        value: PositionSelector {
            page: page.value,
            line: line.value,
            column: column.value,
        },
        token: context,
        rest: column.rest,
    })
}


pub fn uin16_or_all<'t>(text: &'t str) -> ParseResult<'t, Option<u16>> {
    if let Ok(all_suc) = char(REF_ALL_TOKEN)(text) {
        Ok(all_suc.map_value(|_| None))
    } else {
        uint::<u16>("u16")(text).map_value(Some)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parse cell refs.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellRef.
pub fn cell_ref<'t>(text: &'t str) -> ParseResult<'t, CellRef<'t>> {
    // Try Position first, as it shares a prefix with Index.
    if let Ok(pos_suc) = position(text) {
        Ok(pos_suc.map_value(|pos| CellRef::Position(pos)))
    // Try Index second.
    } else if let Ok(idx_suc) = index(text) {
        Ok(idx_suc.map_value(|idx| CellRef::Index(idx)))
    } else {
        // Try name, but parse it as a group if an Index follows.
        let name_suc = name(text)
            .with_parse_context("", text)
            .source_for("cell ref value")?;
        let name = name_suc.value;
        
        if let Ok(idx_suc) = index(name_suc.rest) {
            let idx = idx_suc.value;
            let group_suc = name_suc.join(idx_suc, text);
            Ok(group_suc.map_value(|_| CellRef::Group {
                group: name.into(),
                idx,
            }))
        } else {
            Ok(name_suc.map_value(|_| CellRef::Name(name.into())))
        }
    }
}

/// Parses a Position.
pub fn position<'t>(text: &'t str) -> ParseResult<'t, Position> {
    let pre = char(REF_PREFIX_TOKEN)(text)
        .with_parse_context("", text)
        .source_for("cell ref position prefix")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let page = uint::<u16>("u16")(pre.rest)
        .with_parse_context(context, text)
        .source_for("cell ref position page")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let sep1 = char(REF_POS_SEP_TOKEN)(page.rest)
        .with_parse_context(context, text)
        .source_for("cell ref position separator")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let line = uint::<u16>("u16")(sep1.rest)
        .with_parse_context(context, text)
        .source_for("cell ref position line")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let sep2 = char(REF_POS_SEP_TOKEN)(line.rest)
        .with_parse_context(context, text)
        .source_for("cell ref position separator")?;
    
    let context = &text[0..(text.len() - pre.rest.len())];
    let column = uint::<u16>("u16")(sep2.rest)
        .with_parse_context(context, text)
        .source_for("cell ref position column")?;
    
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
    let pre = char(REF_PREFIX_TOKEN)(text)
        .with_parse_context("", text)
        .source_for("cell ref index prefix")?;

    uint::<u32>("u32")(pre.rest)
        .with_parse_context(pre.token, text)
        .source_for("cell ref index prefix")
}


/// Parses a name or group.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    let valid_char = char_matching(|c| ![
        REF_ALL_TOKEN,
        REF_SEP_TOKEN,
        REF_POS_SEP_TOKEN,
        REF_PREFIX_TOKEN,
        REF_RANGE_TOKEN,
    ].contains(&c));

    let res = one_or_more(valid_char)(text)
        .with_parse_context("", text)
        .source_for("cell ref name")?;

    let context = &text[0..(text.len() - res.rest.len())];
    Ok(Success {
        value: context.trim(),
        token: context,
        rest: res.rest,
    })
}

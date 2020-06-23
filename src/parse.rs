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

// Internal modules.
mod result;
mod common;
mod combinator;
mod error;
mod primitive;
mod selection;

// Exports.
pub use self::result::*;
pub use self::common::*;
pub use self::combinator::*;
pub use self::error::*;
pub use self::primitive::*;
pub use self::selection::*;



// Local imports.
use crate::cell::CellRef;
use crate::cell::Position;
use crate::selection::CellSelector;
use crate::selection::PositionSelector;

// Standard library imports.
use std::convert::TryInto;
use std::convert::TryFrom;

pub(crate) const REF_ALL_TOKEN: char = '*';
pub(crate) const REF_POS_SEP_TOKEN: char = '.';
pub(crate) const REF_PREFIX_TOKEN: char = ':';
pub(crate) const REF_RANGE_TOKEN: char = '-';
pub(crate) const REF_SEP_TOKEN: char = ',';







////////////////////////////////////////////////////////////////////////////////
// Parser combinators.
////////////////////////////////////////////////////////////////////////////////

pub fn entire<'t, F, T>(text: &mut &'t str, parser: F) -> Option<T>
    where F: FnOnce(&mut &'t str) -> Option<T>
{
    let input_text = *text;

    match (parser)(text) {
        Some(res) if text.len() == 0 => Some(res),
        _ => { *text = input_text; None }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell selection.
////////////////////////////////////////////////////////////////////////////////
pub fn parse_cell_selection<'t>(text: &mut &'t str)
    -> Option<Vec<CellSelector<'t>>>
{
    // delimitted sequence of cell selectors
    unimplemented!()
}

pub fn parse_cell_selector<'t>(text: &mut &'t str)
    -> Option<CellSelector<'t>>
{
    let input_text = *text;

    // Try parsing All first to avoid parsing it is a name.
    if let Some(_) = parse_char(text, REF_ALL_TOKEN) {
        return Some(CellSelector::All);
    }

    // Try parsing a CellRef.
    if let Some(cell_ref) = parse_cell_ref(text) {
        if !cell_ref.ranged_selector() {
            // This should only return CellSelector::Name.
            return Some(cell_ref.into());
        }

        // if it succeeds and is a non-name, try a range.
        let _ = parse_whitespace(text);
        if let Some(_) = parse_char(text, REF_RANGE_TOKEN) {
            let _ = parse_whitespace(text);

            if let Some(cell_ref_high) = parse_cell_ref(text) {
                if cell_ref_high.ranged_selector() {
                    let range: Result<CellSelector<'_>, _>
                        = (cell_ref, cell_ref_high).try_into();
                    
                    if let Ok(sel) = range { return Some(sel); }
                    // TODO: Return an error if this is not a valid range.
                }
            }
            *text = input_text;
            return None;
        }   
    }
    
    // Try a PositionSelector.
    if let Some(pos_sel) = parse_position_selector(text) {
        return Some(CellSelector::PositionSelector(pos_sel));
    }

    // Try a GroupAll.
    if let Some(group) = parse_cell_ref_name(text) {
        if let Some(_) = parse_char(text, REF_PREFIX_TOKEN) {
            if let Some(_) = parse_char(text, REF_ALL_TOKEN) {
                return Some(CellSelector::GroupAll(group.trim().into()))
            }
        }
    }

    // Failure.
    *text = input_text;
    None
}


pub fn parse_position_selector<'t>(text: &mut &'t str)
    -> Option<PositionSelector> 
{
    let input_text = *text;

    if let Some(_) = parse_char(text, REF_PREFIX_TOKEN) {
        if let Some(page) = parse_uint_or_all::<u16>(text) {

            if let Some(_) = parse_char(text, REF_POS_SEP_TOKEN) {
                if let Some(line) = parse_uint_or_all::<u16>(text) {
            
                    if let Some(_) = parse_char(text, REF_POS_SEP_TOKEN) {
                        if let Some(column) = parse_uint_or_all::<u16>(text) {
                            
                            return Some(PositionSelector { page, line, column });
                        }
                    }
                }
            }
        }
    }

    *text = input_text;
    None
}

pub fn parse_uint_or_all<'t, T>(text: &mut &'t str) -> Option<Option<T>>
    where T: TryFrom<u32>
{
    // try parsing All first to avoid parsing it is a name.
    if let Some(_) = parse_char(text, REF_ALL_TOKEN) {
        Some(None)
    } else {
        parse_uint::<T>(text).map(Some)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parse CellRef.
////////////////////////////////////////////////////////////////////////////////

pub fn parse_cell_ref<'t>(text: &mut &'t str) -> Option<CellRef<'t>> {
    let input_text = *text;
    let res = match parse_cell_ref_position(text)
        .or_else(|| parse_cell_ref_index(text))
    {
        None => parse_cell_ref_name(text).and_then(
            |name| match parse_char(text, REF_PREFIX_TOKEN) {
                Some(_) => parse_uint::<u32>(text).and_then(
                    |idx| Some(CellRef::Group {
                        group: name.trim().into(),
                        idx,
                    }),
                ),
                None => Some(CellRef::Name(name.trim().into())),
            }
        ),
        some_result => some_result,
    };

    match res {
        Some(cell_ref) => Some(cell_ref),
        None => { *text = input_text; None },
    }
}

pub fn parse_cell_ref_position<'t>(text: &mut &'t str)
    -> Option<CellRef<'t>> 
{
    let input_text = *text;

    if let Some(_) = parse_char(text, REF_PREFIX_TOKEN) {
        if let Some(page) = parse_uint::<u16>(text) {

            if let Some(_) = parse_char(text, REF_POS_SEP_TOKEN) {
                if let Some(line) = parse_uint::<u16>(text) {
            
                    if let Some(_) = parse_char(text, REF_POS_SEP_TOKEN) {
                        if let Some(column) = parse_uint::<u16>(text) {

                            return Some(CellRef::Position(
                                Position { page, line, column } ));
                        }
                    }
                }
            }
        }
    }

    *text = input_text;
    None
}

pub fn parse_cell_ref_index<'t>(text: &mut &'t str)
    -> Option<CellRef<'t>> 
{
    let input_text = *text;

    if let Some(_) = parse_char(text, REF_PREFIX_TOKEN) {
        if let Some(idx) = parse_uint::<u32>(text) {
            return Some(CellRef::Index(idx));
        }
    }

    *text = input_text;
    None
}

pub fn parse_cell_ref_name<'t>(text: &mut &'t str) -> Option<&'t str> {
    let mut pre_len = 0;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == REF_PREFIX_TOKEN { break; }
        pre_len += c.len_utf8();
    }

    if pre_len > 0 {
        let res = &text[0..pre_len];
        *text = &text[pre_len..];
        Some(res)
    } else {
        None
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse srimitives.
////////////////////////////////////////////////////////////////////////////////

pub fn parse_char<'t>(text: &mut &'t str, c: char) -> Option<char> {
    if text.starts_with(c) {
        *text = &text[c.len_utf8()..];
        Some(c)
    } else {
        None
    }
}

pub fn parse_char_in<'t>(text: &mut &'t str, opts: &str) -> Option<char> {
    let mut opts = opts.chars();
    while let Some(c) = opts.next() {
        if text.starts_with(c) {
            *text = &text[c.len_utf8()..];
            return Some(c);
        }
    }

    None
}

pub fn parse_whitespace<'t>(text: &mut &'t str) -> Option<&'t str> {
    let mut pre_len = 0;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if !c.is_whitespace() { break; }
        pre_len += c.len_utf8();
    }

    if pre_len > 0 {
        let res = &text[0..pre_len];
        *text = &text[pre_len..];
        Some(res)
    } else {
        None
    }
}

pub fn parse_uint<'t, T>(text: &mut &'t str) -> Option<T>
    where T: TryFrom<u32>
{
    fn parse_uint_prefix<'t>(text: &mut &'t str) -> Option<&'t str> {
        if  text.starts_with("0b") || 
            text.starts_with("0o") ||
            text.starts_with("0x") 
        {
            let res = &text[0..2];
            *text = &text[2..];
            Some(res)
        } else {
            None
        }
    }

    let input_text = *text;

    let radix: u32 = match parse_uint_prefix(text) {
        Some("0b") => 2,
        Some("0o") => 8,
        Some("0x") => 16,
        None => 10,
        _ => unreachable!(),
    };

    let mut pre_len = 0;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '_' && !c.is_digit(radix) { break; }
        pre_len += c.len_utf8();
    }

    if pre_len == 0 { 
        *text = input_text;
        return None; 
    }

    let digits = &text[0..pre_len];
    *text = &text[pre_len..];

    let mut res: u32 = 0;
    let mut chars = digits.chars();
    while let Some(c) = chars.next() {
        if c == '_' { continue; }

        let val = match c.to_digit(radix) {
            Some(x) => x,
            None => { *text = input_text; return None; }
        };
        
        match res.checked_mul(10) {
            Some(x) => res = x,
            None => { *text = input_text; return None },
        }
        match res.checked_add(val) {
            Some(x) => res = x,
            None => { *text = input_text; return None },
        }
    }
    match res.try_into() {
        Ok(res) => Some(res),
        Err(_) => { *text = input_text; None }
    }
}

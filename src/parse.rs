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
use crate::cell::CellRef;
use crate::cell::Position;

// Standard library imports.
use std::convert::TryInto;
use std::convert::TryFrom;

pub(crate) const REF_PREFIX_TOKEN: char = ':';
pub(crate) const REF_POS_SEP_TOKEN: char = '.';
pub(crate) const REF_ALL_TOKEN: char = '*';
pub(crate) const REF_SEP_TOKEN: char = ',';



////////////////////////////////////////////////////////////////////////////////
// Parser combinators.
////////////////////////////////////////////////////////////////////////////////

pub(crate) fn entire<'t, F, T>(text: &mut &'t str, parser: F) -> Option<T>
    where F: FnOnce(&mut &'t str) -> Option<T>
{
    let input_text = *text;

    match (parser)(text) {
        Some(res) if text.len() == 0 => Some(res),
        _ => { *text = input_text; None }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parse primitives.
////////////////////////////////////////////////////////////////////////////////

pub(crate) fn parse_cell_ref<'t>(text: &mut &'t str) -> Option<CellRef<'t>> {
    let input_text = *text;
    let res = match parse_cell_ref_position(text)
        .or_else(|| parse_cell_ref_index(text))
    {
        None => match parse_name(text) {
            Some(name) => match parse_char(text, REF_PREFIX_TOKEN) {
                Some(_) => match parse_uint::<u32>(text) {
                    Some(idx) => Some(CellRef::Group {
                        group: name.trim().into(),
                        idx,
                    }),
                    None => None,
                },
                None => Some(CellRef::Name(name.trim().into()))
            },
            None => None,
        },
        some_result => some_result,
    };

    match res {
        Some(cell_ref) => Some(cell_ref),
        None => { *text = input_text; None },
    }
}

pub(crate) fn parse_cell_ref_position<'t>(text: &mut &'t str)
    -> Option<CellRef<'t>> 
{
    let input_text = *text;

    match parse_char(text, REF_PREFIX_TOKEN) {
        Some(_) => match parse_uint::<u16>(text) {
            Some(page) => match parse_char(text, REF_POS_SEP_TOKEN) {
                Some(_) => match parse_uint::<u16>(text) {
                    Some(line) => match parse_char(text, REF_POS_SEP_TOKEN) {
                        Some(_) => match parse_uint::<u16>(text) {
                            Some(column) => return Some(CellRef::Position(
                                Position { page, line, column } )),
                            None => (),
                        },
                        None => (),
                    },
                    None => (),
                },
                None => (),
            },
            None => (),
        },
        None => (),
    }

    *text = input_text;
    None
}

pub(crate) fn parse_cell_ref_index<'t>(text: &mut &'t str)
    -> Option<CellRef<'t>> 
{
    let input_text = *text;

    match parse_char(text, REF_PREFIX_TOKEN) {
        Some(_) => match parse_uint::<u32>(text) {
            Some(idx) => return Some(CellRef::Index(idx)),
            None => (),
        },
        None => (),
    }

    *text = input_text;
    None
}


pub(crate) fn parse_char<'t>(text: &mut &'t str, c: char) -> Option<char> {
    if text.starts_with(c) {
        *text = &text[c.len_utf8()..];
        Some(c)
    } else {
        None
    }
}

pub(crate) fn parse_char_in<'t>(text: &mut &'t str, opts: &str) -> Option<char> {
    let mut opts = opts.chars();
    while let Some(c) = opts.next() {
        if text.starts_with(c) {
            *text = &text[c.len_utf8()..];
            return Some(c);
        }
    }

    None
}

pub(crate) fn parse_name<'t>(text: &mut &'t str) -> Option<&'t str> {
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

pub(crate) fn parse_uint_prefix<'t>(text: &mut &'t str) -> Option<&'t str> {
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

pub(crate) fn parse_uint<'t, T>(text: &mut &'t str) -> Option<T>
    where T: TryFrom<u32>
{
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

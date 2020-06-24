////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::parse::*;

// Standard library imports.
use std::borrow::Borrow;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::convert::Into;
use std::convert::TryFrom;
use std::convert::TryInto;



////////////////////////////////////////////////////////////////////////////////
// Parser combinators.
////////////////////////////////////////////////////////////////////////////////
/// Returns a parser which will attempt a parse, wrapping the result in `Some`
/// if it succeeds, otherwise converting the failure into a success with `None`.
pub fn maybe<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, Option<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        match (parser)(text) {
            Ok(success) => Ok(success).map_value(Some),
            Err(failure) => Ok(Success {
                value: None,
                token: "",
                rest: text,
            }),
        }
    }
}

/// Returns a parser which will repeat a parse until it fails and return the
/// last successfully parsed value.
pub fn zero_or_more<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, usize>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    repeat(0, None, parser)
}

/// Returns a parser which will repeat a parse until it fails and return the
/// last successfully parsed value. If no attempts succeed, a failure is
/// returned.
pub fn one_or_more<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, usize>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    repeat(1, None, parser)
}


pub fn repeat<'t, F, V>(low: usize, high: Option<usize>, mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, usize>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        let mut result = match (parser)(text) {
            Ok(first) => first.discard_value(),
            Err(_) if low == 0 => return Ok(Success {
                value: 0,
                token: "",
                rest: text,
            }),
            Err(fail) => return Err(fail)
                .map_value(|_: V| 0)
                .with_parse_context("", text)
                .source_for(format!("repeat {}{}", low,
                    if let Some(high) = high {
                        format!(" to {}", high)
                    } else {
                        "".into()
                    })),
        };

        let mut count = 1;
        while count < low {
            let next = (parser)(result.rest)
                .discard_value()
                .with_parse_context(result.token, text)
                .source_for(format!("repeat {}{}", low,
                    if let Some(high) = high {
                        format!(" to {}", high)
                    } else {
                        "".into()
                    }))?;

            result = result.join(next, text);
            count += 1;
        }

        loop {
            let next_res = (parser)(result.rest)
                .discard_value()
                .with_parse_context(result.token, text)
                .source_for(format!("repeat {}{}", low,
                    if let Some(high) = high {
                        format!(" to {}", high)
                    } else {
                        "".into()
                    }));

            match next_res {
                Ok(next) => {
                    result = result.join(next, text);
                    count += 1;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| count >= h) {
                break;
            }
        }

        Ok(result).map_value(|_| count)
    }
}

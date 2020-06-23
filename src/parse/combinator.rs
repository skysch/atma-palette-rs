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
/// Attempts a parse, wrapping the result in `Some` if it succeeds, otherwise
/// converting the failure into a success with `None`.
pub fn maybe<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, Option<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        match (parser)(text) {
            Ok(success) => Ok(success).map_value(Some),
            Err(failure) => Ok(Success {
                value: None,
                token: &text[..0],
                rest: text,
            }),
        }
    }
}

/// Repeats a parse until it fails, then returns the last successfully parsed
/// value.
pub fn zero_or_more<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, Option<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        let mut result = Ok(Success {
            value: None,
            token: &text[..0],
            rest: text,
        });
        let mut rest = text;
        loop {
            match (parser)(rest) {
                Ok(success) => {
                    rest = success.rest;
                    result = Ok(success).map_value(Some);
                }
                Err(failure) => break,
            }
        }
        result
    }
}

/// Repeats a parse until it fails, then returns the last successfully parsed
/// value.
pub fn one_or_more<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        let mut result = (parser)(text)
            .with_parse_context("", text)
            .source_for("one or more")?;
        loop {
            match (parser)(result.rest) {
                Ok(success) => {
                    result.rest = success.rest;
                    result.value = success.value;
                }
                Err(failure) => break,
            }
        }
        Ok(result)
    }
}

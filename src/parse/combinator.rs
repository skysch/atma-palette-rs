////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::Success;



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
            Err(_) => Ok(Success {
                value: None,
                token: "",
                rest: text,
            }),
        }
    }
}

/// Returns a parser which repeats a parse a givin number of times, stopping if
/// a failure occurs or the upper limit is reached, returning the number of
/// successes. Fails if the lower limit is not reached.
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

/// Returns a parser which repeats a parse a givin number of times, stopping if
/// a failure occurs or the upper limit is reached, returning a `Vec` containing
/// each successful result in order. Fails if the lower limit is not reached.
pub fn repeat_collect<'t, F, V>(low: usize, high: Option<usize>, mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, Vec<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        let mut result = match (parser)(text) {
            Ok(first) => first.map_value(|val| vec![val]) ,
            Err(_) if low == 0 => return Ok(Success {
                value: Vec::new(),
                token: "",
                rest: text,
            }),
            Err(fail) => return Err(fail)
                .map_value(|_: V| Vec::new())
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
                .with_parse_context(result.token, text)
                .source_for(format!("repeat {}{}", low,
                    if let Some(high) = high {
                        format!(" to {}", high)
                    } else {
                        "".into()
                    }))?;

            result = result.join_with(next, text,
                |mut vals, val| { vals.push(val); vals });
            count += 1;
        }

        loop {
            let next_res = (parser)(result.rest)
                .with_parse_context(result.token, text)
                .source_for(format!("repeat {}{}", low,
                    if let Some(high) = high {
                        format!(" to {}", high)
                    } else {
                        "".into()
                    }));

            match next_res {
                Ok(next) => {
                    result = result.join_with(next, text,
                        |mut vals, val| { vals.push(val); vals });
                    count += 1;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| count >= h) {
                break;
            }
        }

        Ok(result)
    }
}


/// Returns a parser which will attempt to parse with the second argument and
/// then the first, joining the tokens while discarding the value of the second
/// parser.
pub fn prefix<'t, F, G, V, U>(mut parser: F, mut prefix_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>
{
    move |text| {
        let pre_suc = (prefix_parser)(text)
            .with_parse_context("", text)?;
        
        let parser_suc = (parser)(pre_suc.rest)
            .with_parse_context(pre_suc.token, text)
            .source_for("postfix")?;

        Ok(pre_suc.join_with(parser_suc, text, |_, r| r))
    }
}

/// Returns a parser which will attempt to parse with the first argument and
/// then the second, joining the tokens while discarding the value of the second
/// parser.
pub fn postfix<'t, F, G, V, U>(mut parser: F, mut postfix_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>
{
    move |text| {
        let parser_suc = (parser)(text)
            .with_parse_context("", text)?;
        
        let post_suc = (postfix_parser)(parser_suc.rest)
            .with_parse_context(parser_suc.token, text)
            .source_for("postfix")?;

        Ok(parser_suc.join_with(post_suc, text, |l, _| l))
    }
}


/// Returns a parser which will attempt to parse with the second argument and
/// then the first, then the second agin, joining the tokens while discarding
/// the values from the second parser.
pub fn circumfix<'t, F, G, V, U>(mut parser: F, mut circumfix_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>
{
    move |text| {
        let pre_suc = (circumfix_parser)(text)
            .with_parse_context("", text)?;
        
        let parser_suc = (parser)(pre_suc.rest)
            .with_parse_context(pre_suc.token, text)
            .source_for("postfix")?;
        let parser_suc = pre_suc.join_with(parser_suc, text, |_, r| r);

        let post_suc = (circumfix_parser)(parser_suc.rest)
            .with_parse_context(parser_suc.token, text)
            .source_for("postfix")?;

        Ok(parser_suc.join_with(post_suc, text, |l, _| l))
    }
}


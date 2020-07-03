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
/// Returns a successful parse of nothing.
#[inline]
pub fn null<'t>(text: &'t str) -> ParseResult<'t, ()> {
    Ok(Success {
        value: (),
        token: "",
        rest: text,
    })
}

/// Returns a parser which will attempt a parse, wrapping the result in `Some`
/// if it succeeds, otherwise converting the failure into a success with `None`.
#[inline]
pub fn maybe<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, Option<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>,
{
    move |text| {
        match (parser)(text) {
            Ok(success) => Ok(success.map_value(Some)),
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
pub fn repeat<'t, F, V>(low: usize, high: Option<usize>, parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, usize>
    where F: FnMut(&'t str) -> ParseResult<'t, V>,
{
    intersperse(low, high, parser, null)
}

/// Returns a parser which repeats a parse a givin number of times, stopping if
/// a failure occurs or the upper limit is reached, returning a `Vec` containing
/// each successful result in order. Fails if the lower limit is not reached.
pub fn repeat_collect<'t, F, V>(low: usize, high: Option<usize>, parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, Vec<V>>
    where F: FnMut(&'t str) -> ParseResult<'t, V>,
{
    intersperse_collect(low, high, parser, null)
}

/// Returns a parser which repeats a parse a givin number of times, stopping if
/// a failure occurs or the upper limit is reached, returning the number of
/// successes. Fails if the lower limit is not reached.
pub fn intersperse<'t, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inner_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, usize>
    where 
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>,
{
    move |text| {
        let mut sub_suc = match (parser)(text) {
            Ok(first) => first.discard_value(),
            Err(_) if low == 0 => return Ok(Success {
                value: 0,
                token: "",
                rest: text,
            }),
            Err(fail) => return Err(fail)
                .map_value(|_: V| 0)
                .source_for(format!("intersperse {}{}", low,
                    match high {
                        Some(high) if high != low => format!(" to {}", high),
                        Some(high) if high == low => "".into(),
                        _ => "+".into(),
                    }))
                .with_new_context("", text)
        };

        let mut count = 1;
        while count < low {
            let next_suc = prefix(&mut parser, &mut inner_parser)(sub_suc.rest)
                .discard_value()
                .source_for(format!("intersperse {}{}", low,
                    match high {
                        Some(high) if high != low => format!(" to {}", high),
                        Some(high) if high == low => "".into(),
                        _ => "+".into(),
                    }))
                .with_join_context(&sub_suc, text)?;

            sub_suc = sub_suc.join(next_suc, text);
            count += 1;
        }

        while high.map_or(true, |h| count < h) {
            let next_res = prefix(&mut parser, &mut inner_parser)(sub_suc.rest)
                .discard_value()
                .source_for(format!("intersperse {}{}", low,
                    match high {
                        Some(high) if high != low => format!(" to {}", high),
                        Some(high) if high == low => "".into(),
                        _ => "+".into(),
                    }))
                .with_join_context(&sub_suc, text);

            match next_res {
                Ok(next_suc) => {
                    sub_suc = sub_suc.join(next_suc, text);
                    count += 1;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| count >= h) {
                break;
            }
        }

        Ok(sub_suc.map_value(|_| count))
    }
}

/// Returns a parser which repeats a parse a givin number of times, stopping if
/// a failure occurs or the upper limit is reached, returning a `Vec` containing
/// each successful result in order. Fails if the lower limit is not reached.
pub fn intersperse_collect<'t, F, G, V, U>(
    low: usize,
    high: Option<usize>,
    mut parser: F,
    mut inner_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, Vec<V>>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>,
{
    move |text| {
        let mut sub_suc = match (parser)(text) {
            Ok(first) => first.map_value(|val| vec![val]) ,
            Err(_) if low == 0 => return Ok(Success {
                value: Vec::new(),
                token: "",
                rest: text,
            }),
            Err(fail) => return Err(fail)
                .map_value(|_: V| Vec::new())
                .source_for(format!("intersperse {}{}", low,
                    match high {
                        Some(high) if high != low => format!(" to {}", high),
                        Some(high) if high == low => "".into(),
                        _ => "+".into(),
                    }))
                .with_new_context("", text),
        };

        let mut count = 1;
        while count < low {
            let next_suc = prefix(&mut parser, &mut inner_parser)(sub_suc.rest)
                .with_join_context(&sub_suc, text)
                .source_for(format!("intersperse {}{}", low,
                    match high {
                        Some(high) if high != low => format!(" to {}", high),
                        Some(high) if high == low => "".into(),
                        _ => "+".into(),
                    }))?;

            sub_suc = sub_suc.join_with(next_suc, text,
                |mut vals, val| { vals.push(val); vals });
            count += 1;
        }

        while high.map_or(true, |h| count < h) {
            let next_res = prefix(&mut parser, &mut inner_parser)(sub_suc.rest)
                .source_for(format!("intersperse {}{}", low,
                    match high {
                        Some(high) if high != low => format!(" to {}", high),
                        Some(high) if high == low => "".into(),
                        _ => "+".into(),
                    }))
                .with_join_context(&sub_suc, text);

            match next_res {
                Ok(next_suc) => {
                    sub_suc = sub_suc.join_with(next_suc, text,
                        |mut vals, val| { vals.push(val); vals });
                    count += 1;
                }
                Err(_) => break,
            }

            if high.map_or(false, |h| count >= h) {
                break;
            }
        }

        Ok(sub_suc)
    }
}

/// Returns a parser which will attempt to parse with the second argument and
/// then the first, joining the tokens while discarding the value of the second
/// parser.
pub fn prefix<'t, F, G, V, U>(mut parser: F, mut prefix_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>,
{
    move |text| {
        let pre_suc = (prefix_parser)(text)
            .with_new_context("", text)?;
        
        let sub_suc = (parser)(pre_suc.rest)
            .with_join_context(&pre_suc, text)?;

        Ok(pre_suc.join_with(sub_suc, text, |_, r| r))
    }
}

/// Returns a parser which will attempt to parse with the first argument and
/// then the second, joining the tokens while discarding the value of the second
/// parser.
pub fn postfix<'t, F, G, V, U>(mut parser: F, mut postfix_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>,
{
    move |text| {
        let sub_suc = (parser)(text)
            .with_new_context("", text)?;
        
        let post_suc = (postfix_parser)(sub_suc.rest)
            .with_join_context(&sub_suc, text)?;

        Ok(sub_suc.join_with(post_suc, text, |l, _| l))
    }
}


/// Returns a parser which will attempt to parse with the second argument and
/// then the first, then the second agin, joining the tokens while discarding
/// the values from the second parser.
pub fn circumfix<'t, F, G, V, U>(mut parser: F, mut circumfix_parser: G)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>,
{
    move |text| {
        let pre_suc = (circumfix_parser)(text)
            .with_new_context("", text)?;
        
        let sub_suc = (parser)(pre_suc.rest)
            .with_join_context(&pre_suc, text)?;
        let sub_suc = pre_suc.join_with(sub_suc, text, |_, r| r);

        let post_suc = (circumfix_parser)(sub_suc.rest)
            .with_join_context(&sub_suc, text)?;

        Ok(sub_suc.join_with(post_suc, text, |l, _| l))
    }
}


/// Returns a parser which will attempt to parse with the second argument, first
/// argument, and then the third, joining the tokens while discarding the value
/// of the second and third parser.
pub fn bracket<'t, F, G, H, V, U, T>(
    mut parser: F,
    mut prefix_parser: G,
    mut postfix_parser: H)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where
        F: FnMut(&'t str) -> ParseResult<'t, V>,
        G: FnMut(&'t str) -> ParseResult<'t, U>,
        H: FnMut(&'t str) -> ParseResult<'t, T>,
{
    move |text| {
        let pre_suc = (prefix_parser)(text)
            .with_new_context("", text)?;
        
        let sub_suc = (parser)(pre_suc.rest)
            .with_join_context(&pre_suc, text)?;
        let sub_suc = pre_suc.join_with(sub_suc, text, |_, r| r);

        let post_suc = (postfix_parser)(sub_suc.rest)
            .with_join_context(&sub_suc, text)?;

        Ok(sub_suc.join_with(post_suc, text, |l, _| l))
    }
}

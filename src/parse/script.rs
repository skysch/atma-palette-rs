////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Script parsers.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::command::CommandOption;
use crate::command::Script;
use crate::error::ScriptError;
use crate::parse::any_literal_map;
use crate::parse::bracket;
use crate::parse::char;
use crate::parse::char_matching;
use crate::parse::circumfix;
use crate::parse::escaped_string;
use crate::parse::Failure;
use crate::parse::intersperse;
use crate::parse::literal;
use crate::parse::literal_once;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::QuoteType;
use crate::parse::repeat;
use crate::parse::repeat_collect;
use crate::parse::repeat_until;
use crate::parse::Success;
use crate::parse::tokenize;
use crate::parse::whitespace;

// External library imports.
use structopt::StructOpt as _;
use log::*;

// Standard library imports.
use std::borrow::Cow;


/// Parses a script.
pub fn script<'t>(text: &'t str) -> ParseResult<'t, Script> {
    trace!("Parsing script from {:?}",
        if text.len() < 20 { text } else { &text[..20] });
    circumfix(
        statements,
        maybe(whitespace_or_comments))
        (text)
        .source_for("script")
        .map_value(|statements| Script { statements })
}

/// Parses a series of statements.
pub fn statements<'t>(text: &'t str) -> ParseResult<'t, Vec<CommandOption>> {
    trace!("  Parsing statements from {:?}",
        if text.len() < 20 { text } else { &text[..20] });
    repeat_collect(0, None,
            statement)
        (text)
        .source_for("statements")
        .map_value(|v| {
            let stmts = v.into_iter().filter_map(|c| c).collect();
            trace!("    Found statements {:?}", stmts);
            stmts
        })
}

/// Parses a statment.
pub fn statement<'t>(text: &'t str) -> ParseResult<'t, Option<CommandOption>> {
    trace!("    Parsing statement from {:?}",
        if text.len() < 20 { text } else { &text[..20] });
    let empty_stmt = circumfix(
            char(';'),
            maybe(whitespace_or_comments))
        (text);
    if empty_stmt.is_ok() {
        trace!("      Found empty statement.");
        return empty_stmt.map_value(|_| None);
    }

    command_option(text)
        .convert_value(|opt| {
            if opt.disallowed_in_scripts() {
                return Err(ScriptError {
                    msg: "unsupported operation".into() 
                })
            }
            trace!("      Found non-empty statement {:?}.", opt);
            Ok(Some(opt))
        })
        .source_for("statement")
}

/// Parses a CommandOption.
pub fn command_option<'t>(text: &'t str) -> ParseResult<'t, CommandOption> {
    trace!("      Parsing command_option from {:?}",
        if text.len() < 20 { text } else { &text[..20] });
    let mut chunks = vec!["script".into()];
    let mut suc = Success { value: (), token: "", rest: text };

    loop {
        if char(';')(suc.rest).is_ok() {
            trace!("      End of command options.");
            return Ok(suc).convert_value(|_| {
                trace!("        Found command option {:?}.", chunks);
                CommandOption::from_iter_safe(chunks)
            });
        }

        match prefix(
                chunk,
                maybe(whitespace_or_comments))
            (suc.rest)
            .with_join_previous(suc, text)
        {
            Ok(chunk_suc) if chunk_suc.token.trim().is_empty() => {
                trace!("      Invalid chunk (empty.)");
                return Ok(suc).convert_value(|_| {
                    trace!("        Found command option {:?}.", chunks);
                    CommandOption::from_iter_safe(chunks)
                });
            },
            Err(_) => {
                trace!("  Invalid chunk (failed.)");
                return Ok(suc).convert_value(|_| {
                    trace!("        Found command option {:?}.", chunks);
                    CommandOption::from_iter_safe(chunks)
                });
            }
            Ok(chunk_suc) => {
                let (val, next_suc) = chunk_suc.take_value();
                chunks.push(val.to_string());
                suc = next_suc;
            },
        }
    }
}

/// Parses a potentially quoted chunk of text.
pub fn chunk<'t>(text: &'t str) -> ParseResult<'t, Cow<'t, str>> {
    trace!("        Parsing chunk from {:?}",
        if text.len() < 20 { text } else { &text[..20] });
    let escaped = escaped_string(
            script_string_open,
            script_string_close,
            script_string_escape)
        (text);
    if escaped.is_ok() {
        trace!("          Found escaped chunk: {:?}",
            escaped.as_ref().unwrap().value);
        return escaped;
    }

    repeat_until(1, None,
            char_matching(|c| c != ';' && c != '#' && !c.is_whitespace()),
            char_matching(|c| c == ';' || c == '#' || c.is_whitespace()))
        (text)
        .tokenize_value()
        .map_value(|val| {
            trace!("          Found chunk: {:?}", val);
            Cow::from(val)
        })
        .source_for("chunk")
}

////////////////////////////////////////////////////////////////////////////////
// Comment parsing
////////////////////////////////////////////////////////////////////////////////
/// Parses any number of comments or whitespace tokens.
pub fn whitespace_or_comments<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    repeat(1, None,
            whitespace_or_comment)
        (text)
        .tokenize_value()
}

/// Parses a comment or whitespace.
pub fn whitespace_or_comment<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    let comment_res = line_comment(text);
    if comment_res.is_ok() { 
        trace!("Found comment: {:?}", comment_res.as_ref().unwrap().value);
        return comment_res; 
    }

    whitespace(text)
}

/// Parses a line comment.
pub fn line_comment<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    prefix(
            repeat_until(0, None,
                char_matching(|c| c != '\n'),
                char_matching(|c| c == '\n')),
            char('#'))
        (text)
        .tokenize_value()
}

////////////////////////////////////////////////////////////////////////////////
// String parsing
////////////////////////////////////////////////////////////////////////////////

/// Parses a script string opening quote. For use with escaped_string.
pub fn script_string_open<'t>(text: &'t str)
    -> ParseResult<'t, (Cow<'static, str>, QuoteType)>
{
    let raw_hashed = bracket(
            tokenize(repeat(1, None, char('#'))),
            char('r'),
            char('"'))
        (text);
    if raw_hashed.is_ok() {
        return raw_hashed
            .map_value(|close| {
                let mut s = "\"".to_string();
                s.push_str(close);
                (s.into(), QuoteType::Raw)
            });
    }

    any_literal_map(
            literal,
            "string open quote",
            vec![
                ("r\"", ("\"".into(), QuoteType::Raw)),
                ("\"",  ("\"".into(), QuoteType::Double)),
                ("'",   ("'".into(),  QuoteType::Single)),
            ])
        (text)
}

/// Parses a script string closing quote. For use with escaped_string.
pub fn script_string_close<'t, 'o: 't>(text: &'t str, open: Cow<'o, str>)
    -> ParseResult<'t, &'t str>
{
    literal_once(open.as_ref())(text)
}

/// Parses a script string escape character. For use with escaped_string.
pub fn script_string_escape<'t>(text: &'t str, quote_type: QuoteType)
    -> ParseResult<'t, &'static str> 
{
    match quote_type {
        QuoteType::Double => any_literal_map(
                literal,
                "string escape",
                vec![
                    ("\\n",  "\n"),
                    ("\\t",  "\t"),
                    ("\"",   "\""),
                    ("\\\\", "\\"),
                ])
            (text),
        QuoteType::Single => any_literal_map(
                literal,
                "string escape",
                vec![
                    ("\\n",  "\n"),
                    ("\\t",  "\t"),
                    ("\\'",  "'"),
                    ("\\\\", "\\"),
                ])
            (text),
        QuoteType::Raw => Err(Failure {
            token: "",
            rest: text,
            expected: "".to_owned().into(),
            source: None,
        }),
    }
}

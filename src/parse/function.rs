////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma intermediate function call.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::FnArg;
use crate::parse::FnCall;
use crate::parse::uint;
use crate::parse::float;

// External library imports.
use tephra::combinator::both;
use tephra::combinator::intersperse_collect;
use tephra::combinator::one;
use tephra::combinator::section;
use tephra::combinator::spanned;
use tephra::combinator::text;
use tephra::combinator::bracket;
use tephra::lexer::Lexer;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;
use tephra::position::ColumnMetrics;


////////////////////////////////////////////////////////////////////////////////
// FnCall
////////////////////////////////////////////////////////////////////////////////

pub fn fn_call<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, FnCall<'text>>
    where Cm: ColumnMetrics,
{
    both(
        text(one(AtmaToken::Ident)),
        bracket(
            one(AtmaToken::OpenParen),
            intersperse_collect(0, None,
                section(spanned(fn_arg)),
                one(AtmaToken::Comma)),
            one(AtmaToken::CloseParen)))
        (lexer)
        .map_value(|(name, args)| FnCall { name, args })
}

pub fn fn_arg<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, FnArg>
    where Cm: ColumnMetrics,
{
    match float::<_, f32>
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(FnArg::F32),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }
    
    uint::<_, u32>
        (lexer)
        .map_value(FnArg::U32)
}

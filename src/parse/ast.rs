////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for the Atma AST.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::cell::CellRef;
use crate::command::Stmt;
use crate::palette::InsertExpr;
use crate::parse::AstExprMatch as _;
use crate::parse::AtmaScanner;
use crate::parse::AtmaToken;
use crate::parse::cell_ref;
use crate::parse::color;
use crate::parse::string;
use crate::parse::uint;

// External library imports.
use ::color::Color;
use tephra::combinator::atomic;
use tephra::combinator::both;
use tephra::combinator::bracket;
use tephra::combinator::fail;
use tephra::combinator::intersperse_collect;
use tephra::combinator::intersperse_collect_until;
use tephra::combinator::left;
use tephra::combinator::maybe;
use tephra::combinator::one;
use tephra::combinator::repeat;
use tephra::combinator::repeat_collect;
use tephra::combinator::right;
use tephra::combinator::section;
use tephra::combinator::spanned;
use tephra::combinator::text;
use tephra::lexer::Lexer;
use tephra::position::ColumnMetrics;
use tephra::result::Failure;
use tephra::result::ParseError;
use tephra::result::ParseResult;
use tephra::result::ParseResultExt as _;
use tephra::result::Spanned;
use tephra::result::Success;
use tephra::span::Span;
use tracing::event;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::borrow::Cow;



////////////////////////////////////////////////////////////////////////////////
// AstExpr
////////////////////////////////////////////////////////////////////////////////

/// The top-level AST expression. Has the lowest precedence.
#[derive(Debug, Clone, PartialEq)]
pub enum AstExpr<'text> {
    /// A unary expression.  Defer to higher precedence operators.
    Unary(Spanned<'text, UnaryExpr<'text>>),
}

impl<'text> AstExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use AstExpr::*;
        match self {
            Unary(_) => "expression".into(),
        }
    }

    pub fn span(&self) -> Span<'text> {
        match self {
            AstExpr::Unary(Spanned { span, .. }) => *span,
        }
    }
}

/// A unary AST expression. Has lower precedence than CallExpr.
#[derive(Debug, Clone, PartialEq)]
#[allow(variant_size_differences)]
pub enum UnaryExpr<'text> {
    /// A numerical negation expression.
    Minus {
        op: Span<'text>,
        operand: Box<Spanned<'text, UnaryExpr<'text>>>,
    },
    /// A function call expression. Defer to higher precedence operators.
    Call(CallExpr<'text>),
}

impl<'text> UnaryExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use UnaryExpr::*;
        match self {
            Minus { operand, .. } => {
                format!("negated {}", operand.value.description()).into()
            },
            Call(p) => p.description()
        }
    }
}

/// A function call AST expression. Has lower precedence than PrimaryExpr.
#[derive(Debug, Clone, PartialEq)]
pub enum CallExpr<'text> {
    /// A function call expression.
    Call {
        operand: Box<Spanned<'text, CallExpr<'text>>>,
        args: Vec<AstExpr<'text>>,
    },
    /// A primary expression. Defer to higher precedence operators.
    Primary(PrimaryExpr<'text>),
}

impl<'text> CallExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use CallExpr::*;
        match self {
            Call { .. } => "function call".into(),
            Primary(p)  => p.description()
        }
    }
}

/// A primitive or grouped AST expression. Has the highest precedence.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimaryExpr<'text> {
    /// An identifier.
    Ident(&'text str),
    /// An integral value.
    Uint(&'text str),
    /// A floating point value.
    Float(&'text str),
    /// A Color value.
    Color(Color),
    /// A CellRef value.
    CellRef(CellRef<'text>),
    /// A bracketted group of values.
    Array(Vec<AstExpr<'text>>),
    /// A parenthesized group of values.
    Tuple(Vec<AstExpr<'text>>),
}

impl<'text> PrimaryExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use PrimaryExpr::*;
        match self {
            Ident(_)     => "identifier".into(),
            Uint(_)      => "integer value".into(),
            Float(_)     => "float value".into(),
            Color(_)     => "color value".into(),
            CellRef(_)   => "cell reference".into(),
            Array(elems) => format!("{} element array", elems.len()).into(),
            Tuple(elems) => format!("{} element tuple", elems.len()).into(),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Stmt Parsers
////////////////////////////////////////////////////////////////////////////////

pub fn stmts<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Vec<Stmt>>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "stmts");
    let _enter = span.enter();

    // Skip beginning empty statements.
    lexer = repeat(0, None, one(AtmaToken::Semicolon))
        (lexer)?
        .lexer;
    event!(Level::TRACE, "finished parsing starting empty stmts");

    repeat_collect(0, None, stmt)
        (lexer)
}


pub fn stmt<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Stmt>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "stmt");
    let _enter = span.enter();

    // header statements
    match header_stmt(lexer.sublexer()) {
        Ok(mut stmt) => {
            event!(Level::TRACE,
                "header_stmt succeeds; parsing trailing empty stmts");
            // Parse any following empty statements.
            stmt.lexer = repeat(0, None, one(AtmaToken::Semicolon))
                (stmt.lexer)?
                .lexer;

            return Ok(stmt);
        },
        Err(_) => (),
    }
    event!(Level::TRACE, "header_stmt failed");

    // expr statement
    match expr_stmt(lexer.sublexer()) {
        Ok(mut stmt) => {
            event!(Level::TRACE, "expr_statement succeeds: parsing trailing \
                empty stmts.");
            // Parse any following empty statements.
            stmt.lexer = repeat(1, None, one(AtmaToken::Semicolon))
                (stmt.lexer)?
                .lexer;

            return Ok(stmt);
        },
        Err(_) => (),
    }
    event!(Level::TRACE, "expr_stmt failed");

    
    match lexer.peek() {
        Some(_) => Err(Failure {
            parse_error: ParseError::new("unrecognized statement")
                .with_span(
                    "expected 'palette', 'page', 'line' or expression",
                    lexer.parse_span(),
                    lexer.column_metrics()),
            lexer,
            source: None,
        }),

        None => {
            event!(Level::TRACE, "end of text");
            Err(Failure {
                parse_error: ParseError::new("empty statement")
                    .with_span(
                    "expected 'palette', 'page', 'line' or expression",
                    lexer.end_span(),
                    lexer.column_metrics()),
                lexer,
                source: None,
            })
        },
    }
}


pub fn expr_stmt<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Stmt>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "expr_stmt");
    let _enter = span.enter();

    use AtmaToken::*;

    match ast_expr
        (lexer.sublexer())
    {
        Ok(Success { lexer, value }) => match InsertExpr::match_expr(
            value,
            lexer.column_metrics())
        {
            Ok(expr) => Ok(Success { 
                lexer,
                value: Stmt::Expr {
                    expr,
                }
            }),

            Err(parse_error) => Err(Failure {
                parse_error,
                lexer,
                source: None,
            }),
        },
        Err(e) => Err(e),
    }
}

pub fn header_stmt<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Stmt>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "header_stmt");
    let _enter = span.enter();

    use AtmaToken::*;

    // introducer
    let (lexer, mut stmt) = match text(one(Ident))
        (lexer)
    {
        Ok(Success { lexer, value }) => match value {
            "palette" => (lexer, Stmt::PaletteHeader { name: None }),
            "page"    => (lexer, Stmt::PageHeader { name: None, number: None }),
            "line"    => (lexer, Stmt::LineHeader { name: None, number: None }),
            _         => return Err(Failure {
                parse_error: ParseError::new("invalid header statement")
                    .with_span(
                        "expected 'palette', 'page', or 'line'",
                        lexer.parse_span(),
                        lexer.column_metrics()),
                lexer,
                source: None,
            }),
        },
        Err(e) => return Err(e),
    };
    event!(Level::TRACE, "introducer: {:?}", stmt);

    let (parsed_number, succ) = maybe(uint::<Cm, u16>)
        (lexer)
        .expect("header number parse cannot fail")
        .take_value();
    event!(Level::TRACE, "parsed_number: {:?}", parsed_number);

    let (parsed_name, succ) = maybe(string)
        (succ.lexer)
        .expect("header string parse cannot fail")
        .take_value();
    event!(Level::TRACE, "parsed_name: {:?}", parsed_name);

    let lexer = one(Colon)
        (succ.lexer)?
        .lexer;

    match &mut stmt {
        Stmt::PaletteHeader { name } => {
            *name = parsed_name.map(|t| t.to_string().into());
            if parsed_number.is_some() {
                return Err(Failure {
                    parse_error: ParseError::new("invalid palette header")
                        .with_span(
                            "palette numbering is not supported",
                            lexer.parse_span(),
                            lexer.column_metrics()),
                    lexer,
                    source: None,
                });
            }
        },
        Stmt::LineHeader { name, number } |
        Stmt::PageHeader { name, number } => {
            *name = parsed_name.map(|t| t.to_string().into());
            *number = parsed_number;
        },
        _ => unreachable!(),
    }

    event!(Level::TRACE, "result: {:?}", stmt);
    Ok(Success {
        value: stmt,
        lexer,
    })
}


////////////////////////////////////////////////////////////////////////////////
// Expr Parsers
////////////////////////////////////////////////////////////////////////////////

pub fn ast_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, AstExpr<'text>>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "ast_expr");
    let _enter = span.enter();

    spanned(unary_expr)
        (lexer)
        .map_value(AstExpr::Unary)
}

pub fn unary_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, UnaryExpr<'text>>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "unary_expr");
    let _enter = span.enter();

    use AtmaToken::*;
    match lexer.peek() {
        Some(Minus) => both(
                spanned(one(Minus)),
                spanned(unary_expr))
            (lexer)
            .map_value(|(op, u)| UnaryExpr::Minus {
                op: op.span,
                operand: Box::new(u)
            }),

        Some(_) => call_expr
            (lexer)
            .map_value(UnaryExpr::Call),

        None => Err(Failure {
            parse_error: ParseError::unexpected_end_of_text(
                lexer.end_span(),
                lexer.column_metrics()),
            lexer,
            source: None,
        }),
    }
}

pub fn call_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, CallExpr<'text>>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "call_expr");
    let _enter = span.enter();

    use AtmaToken::*;

    // event!(Level::TRACE, "lexer before {}", lexer);
    let (Spanned { value, mut span }, mut succ) = spanned(primary_expr)
        (lexer)?
        .take_value();

    event!(Level::TRACE, "prefix: '{:?}'", value);
    event!(Level::TRACE, "suffix: {}", succ.lexer);
    let mut res = CallExpr::Primary(value);

    match atomic(
        spanned(
            bracket(
                one(OpenParen),
                intersperse_collect(0, None,
                    ast_expr,
                    one(Comma)),
                one(CloseParen))))
        (succ.lexer.clone())
        .filter_lexer_error()
    {
        Ok(Success { value, lexer }) => {
            match value {
                Some(Spanned { value: args, span: args_span }) => {
                    res = CallExpr::Call {
                        operand: Box::new(Spanned {
                            value: res,
                            span,
                        }),
                        args,
                    };
                    span = span.enclose(args_span);
                    event!(Level::TRACE, "success lexer: {}", lexer);
                    succ.lexer = lexer;
                },
                None => (),
            }
        },
        Err(None)     => (),
        Err(Some(e))  => return Err(e),
    }

    event!(Level::TRACE, "final lexer: {}", succ.lexer);
    event!(Level::TRACE, "result: {:?}", res);
    Ok(Success {
        value: res,
        lexer: succ.lexer,
    })
}

pub fn primary_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, PrimaryExpr<'text>>
    where Cm: ColumnMetrics,
{
    let span = span!(Level::DEBUG, "primary_expr");
    let _enter = span.enter();

    use AtmaToken::*;
    match lexer.peek() {
        Some(Ident) => text(one(Ident))
            (lexer)
            .map_value(PrimaryExpr::Ident),

        Some(Float) => text(one(Float))
            (lexer)
            .map_value(PrimaryExpr::Float),

        Some(Uint) => text(one(Uint))
            (lexer)
            .map_value(PrimaryExpr::Uint),

        Some(OpenParen) => bracket(
                one(OpenParen),
                intersperse_collect(0, None,
                    section(ast_expr),
                    one(Comma)),
                one(CloseParen))
            (lexer)
            .map_value(PrimaryExpr::Tuple),

        Some(OpenBracket) => bracket(
                one(OpenBracket),
                intersperse_collect(0, None,
                    section(ast_expr),
                    one(Comma)),
                one(CloseBracket))
            (lexer)
            .map_value(PrimaryExpr::Array),
        
        Some(Hash) => color
            (lexer)
            .map_value(PrimaryExpr::Color),

        Some(Colon)             |
        Some(Mult)              |
        Some(StringOpenSingle)  |
        Some(StringOpenDouble)  |
        Some(RawStringOpen)     => cell_ref
            (lexer)
            .map_value(PrimaryExpr::CellRef),

        _ => fail
            (lexer)
            .map_value(|_| unreachable!())
    }
}

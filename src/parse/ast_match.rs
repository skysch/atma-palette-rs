////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma AST expression matchers.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::color::Color;
use crate::color::Rgb;
use crate::color::Xyz;
use crate::color::Hsl;
use crate::color::Hsv;
use crate::color::Cmyk;
use crate::parse::AstExpr;
use crate::parse::PrimaryExpr;
use crate::parse::UnaryExpr;
use crate::parse::CallExpr;
use crate::parse::Ident;
use crate::cell::CellRef;

// External library imports.
use tephra::position::ColumnMetrics;
use tephra::result::ParseError;
use tephra::result::Spanned;
use tephra::span::Span;

// Standard library imports.
use std::str::FromStr as _;

////////////////////////////////////////////////////////////////////////////////
// AstExprMatch
////////////////////////////////////////////////////////////////////////////////
/// A trait implemented by types which can be pattern matched with an AST node.
pub trait AstExprMatch: Sized {
    /// Attempt to match an AstExpr.
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics;


    /// Attempt to match a UnaryExpr.
    fn match_unary_expr<'text, Cm>(
        unary_expr: UnaryExpr<'text>,
        span: Span<'text>,
        metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        Self::match_expr(
            AstExpr::Unary(Spanned { span, value: unary_expr }),
            metrics)
    }

    /// Attempt to match a CallExpr.
    fn match_call_expr<'text, Cm>(
        call_expr: CallExpr<'text>,
        span: Span<'text>,
        metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        Self::match_unary_expr(
            UnaryExpr::Call(call_expr),
            span,
            metrics)
    }

    /// Attempt to match an PrimaryExpr.
    fn match_primary_expr<'text, Cm>(
        primary_expr: PrimaryExpr<'text>,
        span: Span<'text>,
        metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        Self::match_call_expr(
            CallExpr::Primary(primary_expr),
            span,
            metrics)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Ident matcher
////////////////////////////////////////////////////////////////////////////////
impl AstExprMatch for Ident {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        log::trace!("MATCH: Ident from AstExpr: {:?}", ast_expr);
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new("expected identifier")
            .with_span("not a valid identifier",
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Ident(ident))) => {
                Ok(Ident(ident.to_string()))
            },

            _ => Err(default_error),
        }
    }    
}


////////////////////////////////////////////////////////////////////////////////
// Float matchers
////////////////////////////////////////////////////////////////////////////////
macro_rules! float_matcher {
    ($t:ty, $rep:expr) => {
        impl AstExprMatch for $t {
            fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                log::trace!(
                    std::concat!("MATCH: ", $rep, " from AstExpr: {:?}"),
                    ast_expr);
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("expected ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { operand, .. } => {
                        <$t>::match_expr(AstExpr::Unary(*operand), metrics)
                            .map(|f| -f)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Float(float))) => {
                        <$t>::from_str(float)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
                            .with_span(concat!($rep, " is not callable"),
                                ast_span,
                                metrics)),

                    _ => Err(default_error),
                }
            }    
        }
    };
}

float_matcher!(f32, "f32");
float_matcher!(f64, "f64");


////////////////////////////////////////////////////////////////////////////////
// Integer matchers
////////////////////////////////////////////////////////////////////////////////
macro_rules! negatable_integer_matcher {
    ($t:ty, $rep:expr) => {
        impl AstExprMatch for $t {
            fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                log::trace!(
                    std::concat!("MATCH: ", $rep, " from AstExpr: {:?}"),
                    ast_expr);
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("expected ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { operand, .. } => {
                        <$t>::match_expr(AstExpr::Unary(*operand), metrics)
                            .map(|f| -f)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Uint(uint))) => {
                        <$t>::from_str(uint)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
                            .with_span(concat!($rep, " is not callable"),
                                ast_span,
                                metrics)),

                    _ => Err(default_error),
                }
            }    
        }
    };
}

macro_rules! nonnegatable_integer_matcher {
    ($t:ty, $rep:expr) => {
        impl AstExprMatch for $t {
            fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                log::trace!(
                    std::concat!("MATCH: ", $rep, " from AstExpr: {:?}"),
                    ast_expr);
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("expected ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { .. } => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
                            .with_span(concat!($rep, " is not negatable"),
                                ast_span,
                                metrics)),

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Uint(uint))) => {
                        <$t>::from_str(uint)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
                            .with_span(concat!($rep, " is not callable"),
                                ast_span,
                                metrics)),

                    _ => Err(default_error),
                }
            }    
        }
    };
}

negatable_integer_matcher!(i8, "i8");
negatable_integer_matcher!(i16, "i16");
negatable_integer_matcher!(i32, "i32");
negatable_integer_matcher!(i64, "i64");
negatable_integer_matcher!(isize, "isize");

nonnegatable_integer_matcher!(u8, "u8");
nonnegatable_integer_matcher!(u16, "u16");
nonnegatable_integer_matcher!(u32, "u32");
nonnegatable_integer_matcher!(u64, "u64");
nonnegatable_integer_matcher!(usize, "usize");

////////////////////////////////////////////////////////////////////////////////
// Tuple matchers
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for () {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        log::trace!("MATCH: () from AstExpr: {:?}", ast_expr);
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Tuple(tuple)))
                if tuple.is_empty() => 
            {
                Ok(())
            },

            _ => Err(ParseError::new("expected unit value")
                .with_span("not a valid unit value",
                    ast_span,
                    metrics)),
        }
    }    
}

macro_rules! tuple_impls {
    ($(
        $Tuple:ident { $($T:ident)+ }
    )+) => {
        $(
            impl<$($T:AstExprMatch),+> AstExprMatch for ($($T),+,) {
                fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                    -> Result<Self, ParseError<'text, Cm>>
                    where Cm: ColumnMetrics
                {
                    log::trace!("MATCH: (tuple) from AstExpr: {:?}", ast_expr);
                    let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                    let ast_span = span;

                    match value {
                        UnaryExpr::Call(
                            CallExpr::Primary(
                                PrimaryExpr::Tuple(mut tuple))) =>
                        {
                            let res = ($({
                                if tuple.is_empty() {
                                    return Err(ParseError::new("expected tuple value")
                                        .with_span("not a valid tuple value",
                                            ast_span,
                                            metrics));
                                };

                                <$T>::match_expr(tuple.remove(0), metrics)?
                                },
                            )+);

                            if tuple.is_empty() {
                                Ok(res)
                            } else {
                                Err(ParseError::new("expected tuple value")
                                    .with_span("not a valid tuple value",
                                        ast_span,
                                        metrics))
                            }
                        },

                        _ => Err(ParseError::new("expected tuple value")
                            .with_span("not a valid tuple value",
                                ast_span,
                                metrics)),
                    }
                }    
            }
        )+
    }
}

tuple_impls! {
    Tuple1  { A }
    Tuple2  { A B }
    Tuple3  { A B C }
    Tuple4  { A B C D }
    Tuple5  { A B C D E }
    Tuple6  { A B C D E F }
    Tuple7  { A B C D E F G }
    Tuple8  { A B C D E F G H }
    Tuple9  { A B C D E F G H I }
    Tuple10 { A B C D E F G H I J }
    Tuple11 { A B C D E F G H I J K }
    Tuple12 { A B C D E F G H I J K L }
}



////////////////////////////////////////////////////////////////////////////////
// Array matcher
////////////////////////////////////////////////////////////////////////////////
// TODO: Attempting to initialize an array [T; N] butts up against the rules
// for safely initializing an array dynamically, which forbids a generic
// transmute of [MaybeUninit<T>; N] to [T; N]. I don't know of a good
// workaround, so we'll just use Vec instead and verify the length externally.
impl<T> AstExprMatch for Vec<T> where T: AstExprMatch {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        log::trace!("MATCH: (Array) from AstExpr: {:?}", ast_expr);
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        match value {
            UnaryExpr::Call(
                CallExpr::Primary(
                    PrimaryExpr::Array(array))) =>
            {
                let mut res = Vec::with_capacity(array.len());
                    
                for elem in array.into_iter() {
                    res.push(T::match_expr(elem, metrics)?);
                }

                Ok(res)
            },

            _ => Err(ParseError::new("expected array value")
                .with_span("not a valid array value",
                    ast_span,
                    metrics)),
        }
    }    
}


////////////////////////////////////////////////////////////////////////////////
// Color matcher
////////////////////////////////////////////////////////////////////////////////
impl AstExprMatch for Color {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        log::trace!("MATCH: Color from AstExpr: {:?}", ast_expr);
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new(
                concat!("expected color"))
            .with_span(concat!("not a valid color"),
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Color(color))) => {
                Ok(color)
            },

            UnaryExpr::Call(CallExpr::Call { operand, args }) => {
                let operand = Ident::match_call_expr(
                    operand.value,
                    operand.span,
                    metrics)?.0;
                match operand.as_ref() {
                    "rgb"  => {
                        let (r, g, b) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Rgb::from([r, g, b])))
                    },
                    "xzy"  => {
                        let (x, y, z) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Xyz::from([x, y, z])))
                    },
                    "hsl"  => {
                        let (h, s, l) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Hsl::from([h, s, l])))
                    },
                    "hsv"  => {
                        let (h, s, v) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Hsv::from([h, s, v])))
                    },
                    "cmyk" => {
                        let (c, m, y, k) = <(f32, f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Cmyk::from([c, m, y, k])))
                    },
                    _      => Err(default_error)
                }
            }

            _ => Err(default_error),
        }
    }    
}

////////////////////////////////////////////////////////////////////////////////
// CellRef matcher
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for CellRef<'static> {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        log::trace!("MATCH: CellRef from AstExpr: {:?}", ast_expr);
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new("expected cell reference")
            .with_span("not a valid cell reference",
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::CellRef(cell_ref))) => {
                Ok(cell_ref.into_static())
            },

            _ => Err(default_error),
        }
    }    
}



////////////////////////////////////////////////////////////////////////////////
// FunctionCall matcher
////////////////////////////////////////////////////////////////////////////////
/// An AST matcher for a function call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionCall<T, A> where T: AstExprMatch, A: AstExprMatch {
    /// The function being called.
    pub operand: T,
    /// The arguments to the function.
    pub args: A,
}

impl<T, A> AstExprMatch for FunctionCall<T, A> 
    where T: AstExprMatch, A: AstExprMatch
{
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {

        log::trace!("MATCH: FunctionCall from AstExpr: {:?}", ast_expr);
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new("expected function call expression")
            .with_span("not a valid function call expression",
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Call { operand, args }) => {
                let Spanned { span, value } = *operand;
                Ok(FunctionCall {
                    operand: T::match_call_expr(value, span, metrics)?,
                    args: A::match_primary_expr(
                        PrimaryExpr::Tuple(args),
                        ast_span,
                        metrics)?,
                })
            },

            _ => Err(default_error),
        }
    }    
}

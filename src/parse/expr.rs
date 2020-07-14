////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Expr parse functions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::palette::BlendExpr;
use crate::palette::BlendFunction;
use crate::palette::BlendMethod;
use crate::palette::ColorSpace;
use crate::palette::InsertExpr;
use crate::palette::Interpolate;
use crate::palette::InterpolateFunction;
use crate::palette::InterpolateRange;
use crate::error::PaletteError;
use crate::parse::any_literal_map_once;
use crate::parse::bracket;
use crate::parse::cell_ref;
use crate::parse::float;
use crate::parse::require_if;
use crate::parse::char;
use crate::parse::circumfix;
use crate::parse::intersperse_collect;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::Success;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::whitespace;



////////////////////////////////////////////////////////////////////////////////
// insert_expr
////////////////////////////////////////////////////////////////////////////////
/// Parses an InsertExpr.
pub fn insert_expr<'t>(text: &'t str) -> ParseResult<'t, InsertExpr> {
    unimplemented!()
}

////////////////////////////////////////////////////////////////////////////////
// blend_expr
////////////////////////////////////////////////////////////////////////////////
/// Parses an BlendFunction.
pub fn blend_expr<'t>(text: &'t str) -> ParseResult<'t, BlendExpr> {
    let (color_space, suc) = maybe(postfix(color_space, char('_')))
        (text)?
        .take_value();
    let color_space = color_space.unwrap_or(ColorSpace::Rgb);

    let (blend_method, suc) = blend_method
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    let suc = postfix(char('('), maybe(whitespace))
        (suc.rest)
        .with_join_previous(suc, text)?;

    let (refs, suc) = intersperse_collect(2, Some(2), 
            cell_ref,
            circumfix(
                char(','),
                maybe(whitespace)))
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    let (interpolate, suc) = maybe(
            prefix(
                interpolate,
                circumfix(
                    char(','),
                    maybe(whitespace))))
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();
    let interpolate = interpolate.unwrap_or_default();

    prefix(char(')'), maybe(whitespace))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|_| BlendExpr { 
            blend_fn: BlendFunction {
                color_space,
                blend_method,
                source: refs[0].clone().into_static(),
                target: refs[1].clone().into_static(),
            },
            interpolate,
        })
}

////////////////////////////////////////////////////////////////////////////////
// blend_function
////////////////////////////////////////////////////////////////////////////////
/// Parses an BlendFunction.
pub fn blend_function<'t>(text: &'t str) -> ParseResult<'t, BlendFunction> {
    let (color_space, suc) = maybe(postfix(color_space, char('_')))
        (text)?
        .take_value();
    let color_space = color_space.unwrap_or(ColorSpace::Rgb);

    let (blend_method, suc) = blend_method
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    let suc = postfix(char('('), maybe(whitespace))
        (suc.rest)
        .with_join_previous(suc, text)?;

    let (refs, suc) = intersperse_collect(2, Some(2), 
            cell_ref,
            circumfix(
                char(','),
                maybe(whitespace)))
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    prefix(char(')'), maybe(whitespace))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|_| BlendFunction { 
            color_space,
            blend_method,
            source: refs[0].clone().into_static(),
            target: refs[1].clone().into_static(),
        })
}


////////////////////////////////////////////////////////////////////////////////
// blend_method
////////////////////////////////////////////////////////////////////////////////
/// Parses a BlendMethod.
pub fn blend_method<'t>(text: &'t str) -> ParseResult<'t, BlendMethod> {
    any_literal_map_once(
            literal_ignore_ascii_case,
            "blend method",
            vec![
                ("blend",        BlendMethod::Blend),
                ("multiply",     BlendMethod::Multiply),
                ("divide",       BlendMethod::Divide),
                ("subtract",     BlendMethod::Subtract),
                ("difference",   BlendMethod::Difference),
                ("screen",       BlendMethod::Screen),
                ("overlay",      BlendMethod::Overlay),
                ("hard_light",   BlendMethod::HardLight),
                ("soft_light",   BlendMethod::SoftLight),
                ("color_dodge",  BlendMethod::ColorDodge),
                ("color_burn",   BlendMethod::ColorBurn),
                ("vivid_light",  BlendMethod::VividLight),
                ("linear_dodge", BlendMethod::LinearDodge),
                ("linear_burn",  BlendMethod::LinearBurn),
                ("linear_light", BlendMethod::LinearLight),
            ])
        (text)
}

////////////////////////////////////////////////////////////////////////////////
// color_space
////////////////////////////////////////////////////////////////////////////////
/// Parses a BlendMethod.
pub fn color_space<'t>(text: &'t str) -> ParseResult<'t, ColorSpace> {
    any_literal_map_once(
            literal_ignore_ascii_case,
            "color space",
            vec![
                ("rgb", ColorSpace::Rgb),
            ])
        (text)
}


////////////////////////////////////////////////////////////////////////////////
// interpolate
////////////////////////////////////////////////////////////////////////////////
/// Parses an Interpolate.
pub fn interpolate<'t>(text: &'t str) -> ParseResult<'t, Interpolate> {
    let linear = interpolate_linear(text);
    if linear.is_ok() {
        linear.convert_value("valid interpolate", Interpolate::validate)
    } else {
        interpolate_cubic(text)
            .convert_value("valid interpolate", Interpolate::validate)
    }
}

/// Parses an Interpolate if it is linear.
pub fn interpolate_linear<'t>(text: &'t str) -> ParseResult<'t, Interpolate> {
    let simple = float::<f32>("f32")(text);
    if simple.is_ok() {
        return simple.map_value(|amount| Interpolate {
            color_space: ColorSpace::default(),
            interpolate_fn: InterpolateFunction::Linear,
            amount,
        });
    }

    prefix(
            bracket(
                interpolate_linear_args,
                postfix(
                    char('('),
                    maybe(whitespace)),
                prefix(
                    char(')'),
                    maybe(whitespace))),
            literal_ignore_ascii_case("linear"))
        (text)
}

/// Parses an Interpolate from arguments for linear interpolation.
pub fn interpolate_linear_args<'t>(text: &'t str)
    -> ParseResult<'t, Interpolate>
{
    let (color_space, suc) = maybe(color_space)
        (text)?
        .take_value();
    let cs_sep = color_space.is_some();
    let color_space = color_space.unwrap_or(ColorSpace::Rgb);

    prefix(
            float::<f32>("f32"),
            require_if("separator after color space", 
                move || cs_sep,
                circumfix(
                    char(','),
                    maybe(whitespace))))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|amount| Interpolate {
            color_space,
            interpolate_fn: InterpolateFunction::Linear,
            amount,
        })
}

/// Parses an Interpolate if it is cubic.
pub fn interpolate_cubic<'t>(text: &'t str) -> ParseResult<'t, Interpolate> {
    prefix(
            bracket(
                interpolate_cubic_args,
                postfix(
                    char('('),
                    maybe(whitespace)),
                prefix(
                    char(')'),
                    maybe(whitespace))),
            literal_ignore_ascii_case("cubic"))
        (text)
}

/// Parses an Interpolate from arguments for cubic interpolation.
pub fn interpolate_cubic_args<'t>(text: &'t str)
    -> ParseResult<'t, Interpolate>
{
    let (color_space, suc) = maybe(color_space)
        (text)?
        .take_value();
    let cs_sep = color_space.is_some();
    let color_space = color_space.unwrap_or(ColorSpace::Rgb);

    let (amount, suc) = prefix(
            float::<f32>("f32"),
            require_if("separator after color space", 
                move || cs_sep,
                circumfix(
                    char(','),
                    maybe(whitespace))))
        (suc.rest)
        .with_join_previous(suc, text)?
        .take_value();

    maybe(
        prefix(
            intersperse_collect(2, Some(2),
                float::<f32>("f32"),
                circumfix(
                    char(','),
                    maybe(whitespace))),
            circumfix(
                char(','),
                maybe(whitespace))))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|val| match val {
            None => Interpolate {
                color_space,
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                amount,
            },
            Some(vals) => Interpolate {
                color_space,
                interpolate_fn: InterpolateFunction::Cubic(vals[0], vals[1]),
                amount,
            },
        })
}


////////////////////////////////////////////////////////////////////////////////
// interpolate_range
////////////////////////////////////////////////////////////////////////////////
/// Parses an InterpolateRange.
pub fn interpolate_range<'t>(text: &'t str)
    -> ParseResult<'t, InterpolateRange>
{
    let linear = interpolate_range_linear(text);
    if linear.is_ok() {
        linear.convert_value(
            "valid interpolate range",
            InterpolateRange::validate)
    } else {
        interpolate_range_cubic(text)
            .convert_value(
                "valid interpolate range",
                InterpolateRange::validate)
    }
}

/// Parses an InterpolateRange if it is linear.
pub fn interpolate_range_linear<'t>(text: &'t str)
    -> ParseResult<'t, InterpolateRange>
{
    let suc = literal_ignore_ascii_case("linear")(text)?;

    maybe(
        bracket(
            interpolate_range_linear_args,
            postfix(
                char('('),
                maybe(whitespace)),
            prefix(
                char(')'),
                maybe(whitespace))))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|val| val.unwrap_or_else(|| InterpolateRange {
            color_space: ColorSpace::default(),
            interpolate_fn: InterpolateFunction::Linear,
            start: 0.0,
            end: 1.0,
        }))
}


/// Parses an InterpolateRange from arguments for linear interpolation.
pub fn interpolate_range_linear_args<'t>(text: &'t str)
    -> ParseResult<'t, InterpolateRange>
{
    let (color_space, suc) = maybe(color_space)
        (text)?
        .take_value();
    let cs_sep = color_space.is_some();
    let color_space = color_space.unwrap_or(ColorSpace::Rgb);

    maybe(
        prefix(
            intersperse_collect(2, Some(2),
                float::<f32>("f32"),
                circumfix(
                    char(','),
                    maybe(whitespace))),
            require_if("separator after color space", 
                move || cs_sep,
                circumfix(
                    char(','),
                    maybe(whitespace)))))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|val| match val {
            None => InterpolateRange {
                color_space,
                interpolate_fn: InterpolateFunction::Linear,
                start: 0.0,
                end: 1.0,
            },
            Some(vals) => InterpolateRange {
                color_space,
                interpolate_fn: InterpolateFunction::Linear,
                start: vals[0],
                end: vals[1],
            },
        })
}

/// Parses an InterpolateRange if it is cubic.
pub fn interpolate_range_cubic<'t>(text: &'t str)
    -> ParseResult<'t, InterpolateRange>
{
    let suc = literal_ignore_ascii_case("cubic")(text)?;

    maybe(
        bracket(
            interpolate_range_cubic_args,
            postfix(
                char('('),
                maybe(whitespace)),
            prefix(
                char(')'),
                maybe(whitespace))))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|val| val.unwrap_or_else(|| InterpolateRange {
            color_space: ColorSpace::default(),
            interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
            start: 0.0,
            end: 1.0,
        }))
}



/// Parses an InterpolateRange from arguments for cubic interpolation.
pub fn interpolate_range_cubic_args<'t>(text: &'t str)
    -> ParseResult<'t, InterpolateRange>
{
    let (color_space, suc) = maybe(color_space)
        (text)?
        .take_value();
    let cs_sep = color_space.is_some();
    let color_space = color_space.unwrap_or(ColorSpace::Rgb);

    let (range, suc) = maybe(
            prefix(
                intersperse_collect(2, Some(2),
                    float::<f32>("f32"),
                    circumfix(
                        char(','),
                        maybe(whitespace))),
                require_if("separator after color space", 
                    move || cs_sep,
                    circumfix(
                        char(','),
                        maybe(whitespace)))))
        (suc.rest)
        .with_join_previous(suc, text)?
        .map_value(|val| match val {
            None => None,
            Some(vals) => Some((vals[0], vals[1])),
        })
        .take_value();

    let r_sep = range.is_some();
    let (start, end) = range.clone().unwrap_or((0.0, 1.0));
    
    maybe(
        prefix(
            intersperse_collect(2, Some(2),
                float::<f32>("f32"),
                circumfix(
                    char(','),
                    maybe(whitespace))),
            require_if("separator after color space", 
                move || cs_sep && r_sep,
                circumfix(
                    char(','),
                    maybe(whitespace)))))
        (suc.rest)
        .with_join_previous(suc, text)
        .map_value(|val| match val {
            None => InterpolateRange {
                color_space,
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                start: start,
                end: end,
            },
            Some(vals) => InterpolateRange {
                color_space,
                interpolate_fn: InterpolateFunction::Cubic(vals[0], vals[1]),
                start,
                end,
            }
        })
}

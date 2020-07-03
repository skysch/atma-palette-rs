////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse primitives.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::color::Cmyk;
use crate::color::Color;
use crate::color::Hsl;
use crate::color::Hsv;
use crate::color::Rgb;
use crate::color::Xyz;
use crate::parse::bracket;
use crate::parse::char_in;
use crate::parse::circumfix;
use crate::parse::Failure;
use crate::parse::float;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::repeat;
use crate::parse::intersperse_collect;
use crate::parse::Success;
use crate::parse::whitespace;

// Standard library imports.
use std::borrow::Cow;



/// Floating point sign token.
pub const RGB_HEX_PREFIX: char = '#';

/// Floating point sign token.
pub const FUNCTIONAL_SEPARATOR: char = ',';
pub const FUNCTIONAL_OPEN_BRACKET: char = '(';
pub const FUNCTIONAL_CLOSE_BRACKET: char = ')';

////////////////////////////////////////////////////////////////////////////////
// Color
////////////////////////////////////////////////////////////////////////////////
/// Parses a Color.
pub fn color<'t>(text: &'t str) -> ParseResult<'t, Color> {
    unimplemented!()
}

////////////////////////////////////////////////////////////////////////////////
// rgb_hex
////////////////////////////////////////////////////////////////////////////////
/// Parses an RGB hex code.
pub fn rgb_hex<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    unimplemented!()
}


/// Parses a 6-digit RGB hex code.
pub fn rgb_hex_6<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    unimplemented!()
}

/// Parses a 3-digit RGB hex code.
pub fn rgb_hex_3<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    unimplemented!()
}

////////////////////////////////////////////////////////////////////////////////
// functional notation
////////////////////////////////////////////////////////////////////////////////
/// Parses an RGB value from it functional notation.
pub fn rgb_functional<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    let suc = prefix(
            functional(3),
            literal_ignore_ascii_case("rgb"))
        (text)?;
    let rgb = Rgb::from([
        suc.value[0],
        suc.value[1],
        suc.value[2],
    ]);

    Ok(suc.map_value(|_| rgb))
}


/// Parses an HSV value from it functional notation.
pub fn hsv_functional<'t>(text: &'t str) -> ParseResult<'t, Hsv> {
    let h_suc = char_in("Hh")(text)?;

    let s_suc = char_in("Ss")(h_suc.rest)
        .with_join_context(&h_suc, text)?;
    let s_suc = h_suc.join(s_suc, text);

    let v_suc = char_in("Vv")(s_suc.rest)
        .with_join_context(&s_suc, text)?;
    let v_suc = s_suc.join(v_suc, text);

    let f_suc = functional(3)(v_suc.rest)
        .with_join_context(&v_suc, text)?;
    let hsv = Hsv::from([
        f_suc.value[0],
        f_suc.value[1],
        f_suc.value[2],
    ]);

    Ok(v_suc.join(f_suc, text)
        .map_value(|_| hsv))
}


/// Parses an HSL value from it functional notation.
pub fn hsl_functional<'t>(text: &'t str) -> ParseResult<'t, Hsl> {
    unimplemented!()
}


/// Parses an CMYK value from it functional notation.
pub fn cmyk_functional<'t>(text: &'t str) -> ParseResult<'t, Cmyk> {
    unimplemented!()
}

/// Parses an XYZ value from it functional notation.
pub fn xyz_functional<'t>(text: &'t str) -> ParseResult<'t, Xyz> {
    unimplemented!()
}


/// Returns a parser which parses a functional suffix with n float parameters.
fn functional<'t>(n: usize)
    -> impl FnMut(&'t str) -> ParseResult<'t, Vec<f32>>
{
    use crate::parse::char;
    bracket(
        intersperse_collect(n, Some(n),
            float::<f32>("f32"),
            circumfix(
                char(FUNCTIONAL_SEPARATOR),
                maybe(whitespace))),
        postfix(
            char(FUNCTIONAL_OPEN_BRACKET),
            maybe(whitespace)),
        prefix(
            char(FUNCTIONAL_CLOSE_BRACKET),
            maybe(whitespace)))
}

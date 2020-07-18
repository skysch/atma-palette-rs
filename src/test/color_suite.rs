////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! CellSelection parser test suite.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::parse::*;
use crate::color::Rgb;
use crate::color::Hsv;
use crate::color::Hsl;
use crate::color::Cmyk;
use crate::color::Xyz;
use crate::color::Color;

////////////////////////////////////////////////////////////////////////////////
// Color parsing.
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::rgb_functional`.
#[test]
fn rgb_functional_match() {
    assert_eq!(
        rgb_functional("RGB(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Rgb { r: 255, g: 127, b: 0 },
            token: "RGB(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));
}

/// Tests `parse::hsv_functional`.
#[test]
fn hsv_functional_match() {
    assert_eq!(
        hsv_functional("HSV(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Hsv::new(1.0, 0.5, 0.0),
            token: "HSV(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));
}

/// Tests `parse::hsl_functional`.
#[test]
fn hsl_functional_match() {
    assert_eq!(
        hsl_functional("HSL(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Hsl::new(1.0, 0.5, 0.0),
            token: "HSL(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));
}

/// Tests `parse::cmyk_functional`.
#[test]
fn cmyk_functional_match() {
    assert_eq!(
        cmyk_functional("CMYK(1.0, 0.5, 0.0, 0.25)abcd"),
        Ok(Success {
            value: Cmyk::new(255, 127, 0, 63),
            token: "CMYK(1.0, 0.5, 0.0, 0.25)",
            rest: "abcd",
        }));
}

/// Tests `parse::xyz_functional`.
#[test]
fn xyz_functional_match() {
    assert_eq!(
        xyz_functional("XYZ(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Xyz::new(1.0, 0.5, 0.0),
            token: "XYZ(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));
}

/// Tests `parse::rgb_hex_3`.
#[test]
fn rgb_hex_3_match() {
    assert_eq!(
        rgb_hex_3("#1BFabcd"),
        Ok(Success {
            value: Rgb { r: 0x11, g: 0xBB, b: 0xFF },
            token: "#1BF",
            rest: "abcd",
        }));
}

/// Tests `parse::rgb_hex_6`.
#[test]
fn rgb_hex_6_match() {
    assert_eq!(
        rgb_hex_6("#11BBFFabcd"),
        Ok(Success {
            value: Rgb { r: 0x11, g: 0xBB, b: 0xFF },
            token: "#11BBFF",
            rest: "abcd",
        }));
}

/// Tests `parse::rgb_hex`.
#[test]
fn rgb_hex_match() {
    assert_eq!(
        rgb_hex("#11BBFFabcd"),
        Ok(Success {
            value: Rgb { r: 0x11, g: 0xBB, b: 0xFF },
            token: "#11BBFF",
            rest: "abcd",
        }));
    assert_eq!(
        rgb_hex("#1BF abcd"),
        Ok(Success {
            value: Rgb { r: 0x11, g: 0xBB, b: 0xFF },
            token: "#1BF",
            rest: " abcd",
        }));
}


/// Tests `parse::color`.
#[test]
fn color_match() {
    assert_eq!(
        color("RGB(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Color::from(Rgb { r: 255, g: 127, b: 0 }),
            token: "RGB(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));

    assert_eq!(
        color("HSV(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Color::from(Hsv::new(1.0, 0.5, 0.0)),
            token: "HSV(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));

    assert_eq!(
        color("HSL(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Color::from(Hsl::new(1.0, 0.5, 0.0)),
            token: "HSL(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));

    assert_eq!(
        color("CMYK(1.0, 0.5, 0.0, 0.25)abcd"),
        Ok(Success {
            value: Color::from(Cmyk::new(255, 127, 0, 63)),
            token: "CMYK(1.0, 0.5, 0.0, 0.25)",
            rest: "abcd",
        }));

    assert_eq!(
        color("XYZ(1.0, 0.5, 0.0)abcd"),
        Ok(Success {
            value: Color::from(Xyz::new(1.0, 0.5, 0.0)),
            token: "XYZ(1.0, 0.5, 0.0)",
            rest: "abcd",
        }));

    assert_eq!(
        color("#1BF abcd"),
        Ok(Success {
            value: Color::from(Rgb { r: 0x11, g: 0xBB, b: 0xFF }),
            token: "#1BF",
            rest: " abcd",
        }));

    assert_eq!(
        color("#11BBFFabcd"),
        Ok(Success {
            value: Color::from(Rgb { r: 0x11, g: 0xBB, b: 0xFF }),
            token: "#11BBFF",
            rest: "abcd",
        }));
}

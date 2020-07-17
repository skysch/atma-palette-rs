////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Expr and InsertExpr test suite.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellRef;
use crate::cell::Position;
use crate::palette::BlendExpr;
use crate::palette::BinaryBlendFunction;
use crate::palette::BinaryBlendMethod;
use crate::color::Rgb;
use crate::palette::ColorSpace;
use crate::palette::InsertExpr;
use crate::palette::Interpolate;
use crate::palette::InterpolateFunction;
use crate::palette::InterpolateRange;
use crate::parse::*;


////////////////////////////////////////////////////////////////////////////////
// InsertExpr
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::insert_expr` with ramp variants.
#[test]
fn insert_expr_ramp_match() {
    assert_eq!(
        insert_expr("ramp(3, overlay(:0,:1))abcd"),
        Ok(Success {
            value: InsertExpr::Ramp {
                count: 3,
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Overlay,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: InterpolateRange::default(),
            },
            token: "ramp(3, overlay(:0,:1))",
            rest: "abcd",
        }));

    assert_eq!(
        insert_expr("ramp(3, overlay(:0,:1), linear)abcd"),
        Ok(Success {
            value: InsertExpr::Ramp {
                count: 3,
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Overlay,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: InterpolateRange {
                    color_space: ColorSpace::default(),
                    interpolate_fn: InterpolateFunction::Linear,
                    start: 0.0,
                    end: 1.0,
                },
            },
            token: "ramp(3, overlay(:0,:1), linear)",
            rest: "abcd",
        }));

    assert_eq!(
        insert_expr("ramp(3, overlay(:0,:1), linear(rgb, 0.0, 0.8))abcd"),
        Ok(Success {
            value: InsertExpr::Ramp {
                count: 3,
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Overlay,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: InterpolateRange {
                    color_space: ColorSpace::Rgb,
                    interpolate_fn: InterpolateFunction::Linear,
                    start: 0.0,
                    end: 0.8,
                },
            },
            token: "ramp(3, overlay(:0,:1), linear(rgb, 0.0, 0.8))",
            rest: "abcd",
        }));

    assert_eq!(
        insert_expr("ramp(3, overlay(:0,:1),cubic(rgb,0.0,0.8,3.0,3.0))abcd"),
        Ok(Success {
            value: InsertExpr::Ramp {
                count: 3,
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Overlay,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: InterpolateRange {
                    color_space: ColorSpace::Rgb,
                    interpolate_fn: InterpolateFunction::Cubic(3.0, 3.0),
                    start: 0.0,
                    end: 0.8,
                },
            },
            token: "ramp(3, overlay(:0,:1),cubic(rgb,0.0,0.8,3.0,3.0))",
            rest: "abcd",
        }));

    assert_eq!(
        insert_expr("ramp(3, overlay(:0,:1),cubic(rgb,0.0,0.8,-3.0,-3.0))abcd"),
        Ok(Success {
            value: InsertExpr::Ramp {
                count: 3,
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Overlay,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: InterpolateRange {
                    color_space: ColorSpace::Rgb,
                    interpolate_fn: InterpolateFunction::Cubic(-3.0, -3.0),
                    start: 0.0,
                    end: 0.8,
                },
            },
            token: "ramp(3, overlay(:0,:1),cubic(rgb,0.0,0.8,-3.0,-3.0))",
            rest: "abcd",
        }));
}


/// Tests `parse::insert_expr` with blend variants.
#[test]
fn insert_expr_blend_match() {
    assert_eq!(
        insert_expr("blend(:0,:1,0.5)abcd"),
        Ok(Success {
            value: InsertExpr::Blend(BlendExpr {
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Blend,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: Interpolate {
                    color_space: ColorSpace::Rgb,
                    interpolate_fn: InterpolateFunction::Linear,
                    amount: 0.5,
                },
            }),
            token: "blend(:0,:1,0.5)",
            rest: "abcd",
        }));
}

/// Tests `parse::insert_expr` with color variants.
#[test]
fn insert_expr_color_match() {
    assert_eq!(
        insert_expr("rgb(0.2,0.3,0.4)abcd"),
        Ok(Success {
            value: InsertExpr::Color(Rgb::from([0.2, 0.3, 0.4]).into()),
            token: "rgb(0.2,0.3,0.4)",
            rest: "abcd",
        }));

    assert_eq!(
        insert_expr("#123456abcd"),
        Ok(Success {
            value: InsertExpr::Color(Rgb::from(0x123456).into()),
            token: "#123456",
            rest: "abcd",
        }));
}

/// Tests `parse::blend_expr`.
#[test]
fn blend_expr_match() {
    assert_eq!(
        blend_expr("blend(:0,:1,0.5)abcd"),
        Ok(Success {
            value: BlendExpr {
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::default(),
                    blend_method: BinaryBlendMethod::Blend,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: Interpolate {
                    color_space: ColorSpace::Rgb,
                    interpolate_fn: InterpolateFunction::Linear,
                    amount: 0.5,
                },
            },
            token: "blend(:0,:1,0.5)",
            rest: "abcd",
        }));

    assert_eq!(
        blend_expr("rgb_subtract(:0,:1,linear( 0.5 ))abcd"),
        Ok(Success {
            value: BlendExpr {
                blend_fn: BinaryBlendFunction {
                    color_space: ColorSpace::Rgb,
                    blend_method: BinaryBlendMethod::Subtract,
                    source: CellRef::Index(0),
                    target: CellRef::Index(1),
                },
                interpolate: Interpolate {
                    color_space: ColorSpace::Rgb,
                    interpolate_fn: InterpolateFunction::Linear,
                    amount: 0.5,
                },
            },
            token: "rgb_subtract(:0,:1,linear( 0.5 ))",
            rest: "abcd",
        }));
}


/// Tests `parse::blend_function`.
#[test]
fn blend_function_match() {
    assert_eq!(
        blend_function("blend(:0,:1)abcd"),
        Ok(Success {
            value: BinaryBlendFunction {
                color_space: ColorSpace::default(),
                blend_method: BinaryBlendMethod::Blend,
                source: CellRef::Index(0),
                target: CellRef::Index(1),
            },
            token: "blend(:0,:1)",
            rest: "abcd",
        }));

    assert_eq!(
        blend_function("rgb_vivid_light(:0.1.1,whatever)abcd"),
        Ok(Success {
            value: BinaryBlendFunction {
                color_space: ColorSpace::Rgb,
                blend_method: BinaryBlendMethod::VividLight,
                source: CellRef::Position(Position { 
                    page: 0,
                    line: 1,
                    column: 1,
                }),
                target: CellRef::Name("whatever".into()),
            },
            token: "rgb_vivid_light(:0.1.1,whatever)",
            rest: "abcd",
        }));

    assert_eq!(
        blend_function("rgb_vivid_light( :0.1.1 , whatever )abcd"),
        Ok(Success {
            value: BinaryBlendFunction {
                color_space: ColorSpace::Rgb,
                blend_method: BinaryBlendMethod::VividLight,
                source: CellRef::Position(Position { 
                    page: 0,
                    line: 1,
                    column: 1,
                }),
                target: CellRef::Name("whatever".into()),
            },
            token: "rgb_vivid_light( :0.1.1 , whatever )",
            rest: "abcd",
        }));

    assert_eq!(
        blend_function("color_burn(abc,whatever)abcd"),
        Ok(Success {
            value: BinaryBlendFunction {
                color_space: ColorSpace::default(),
                blend_method: BinaryBlendMethod::ColorBurn,
                source: CellRef::Name("abc".into()),
                target: CellRef::Name("whatever".into()),
            },
            token: "color_burn(abc,whatever)",
            rest: "abcd",
        }));
}


////////////////////////////////////////////////////////////////////////////////
// ColorSpace
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::color_space`.
#[test]
fn color_space_match() {
    assert_eq!(
        color_space("rgbabcd"),
        Ok(Success {
            value: ColorSpace::Rgb,
            token: "rgb",
            rest: "abcd",
        }));
}

////////////////////////////////////////////////////////////////////////////////
// Interpolate & InterpolateRange
////////////////////////////////////////////////////////////////////////////////

/// Tests `parse::interpolate_range`.
#[test]
fn interpolate_range_match() {
    assert_eq!(
        interpolate_range("lIneArabcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Linear,
                start: 0.0,
                end: 1.0,
            },
            token: "lIneAr",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate_range("cubicabcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                start: 0.0,
                end: 1.0,
            },
            token: "cubic",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate_range("linear( rgb )abcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Linear,
                start: 0.0,
                end: 1.0,
            },
            token: "linear( rgb )",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate_range("linear( rgb , 0.2, 0.5 )abcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Linear,
                start: 0.2,
                end: 0.5,
            },
            token: "linear( rgb , 0.2, 0.5 )",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate_range("cubic( rgb )abcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                start: 0.0,
                end: 1.0,
            },
            token: "cubic( rgb )",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate_range("cubic( rgb, 0.2, 0.4 )abcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                start: 0.2,
                end: 0.4,
            },
            token: "cubic( rgb, 0.2, 0.4 )",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate_range("cubic( rgb, 0.2, 0.4 , 0.1, 0.9)abcd"),
        Ok(Success {
            value: InterpolateRange {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Cubic(0.1, 0.9),
                start: 0.2,
                end: 0.4,
            },
            token: "cubic( rgb, 0.2, 0.4 , 0.1, 0.9)",
            rest: "abcd",
        }));
}


/// Tests `parse::interpolate_range`.
#[test]
fn interpolate_range_nonmatch() {
    assert_eq!(
        interpolate_range("lineaabcd"),
        Err(Failure {
            token: "",
            rest: "lineaabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

/// Tests `parse::interpolate`.
#[test]
fn interpolate_match() {
    assert_eq!(
        interpolate("0.1abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Linear,
                amount: 0.1,
            },
            token: "0.1",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate("linear(0.1)abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Linear,
                amount: 0.1,
            },
            token: "linear(0.1)",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate("linear(rgb, 0.1)abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Linear,
                amount: 0.1,
            },
            token: "linear(rgb, 0.1)",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate("cubic(0.1)abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                amount: 0.1,
            },
            token: "cubic(0.1)",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate("cubic(rgb,0.1)abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Cubic(0.0, 0.0),
                amount: 0.1,
            },
            token: "cubic(rgb,0.1)",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate("cubic(rgb,0.1, 0.2, 0.3)abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::Rgb,
                interpolate_fn: InterpolateFunction::Cubic(0.2, 0.3),
                amount: 0.1,
            },
            token: "cubic(rgb,0.1, 0.2, 0.3)",
            rest: "abcd",
        }));

    assert_eq!(
        interpolate("cubic(0.1, 0.2, 0.3)abcd"),
        Ok(Success {
            value: Interpolate {
                color_space: ColorSpace::default(),
                interpolate_fn: InterpolateFunction::Cubic(0.2, 0.3),
                amount: 0.1,
            },
            token: "cubic(0.1, 0.2, 0.3)",
            rest: "abcd",
        }));
}


/// Tests `parse::interpolate`.
#[test]
fn interpolate_nonmatch() {
    assert_eq!(
        interpolate("lineaabcd"),
        Err(Failure {
            token: "",
            rest: "lineaabcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        interpolate("linear(100.0)abcd"),
        Err(Failure {
            token: "linear(100.0)",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        interpolate("linear(rgb, 100.0)abcd"),
        Err(Failure {
            token: "linear(rgb, 100.0)",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        interpolate("cubic(100.0)abcd"),
        Err(Failure {
            token: "cubic(100.0)",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        interpolate("cubic(rgb, 100.0)abcd"),
        Err(Failure {
            token: "cubic(rgb, 100.0)",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));

    assert_eq!(
        interpolate("cubic(rgb, 3.1, 0.3, 0.2)abcd"),
        Err(Failure {
            token: "cubic(rgb, 3.1, 0.3, 0.2)",
            rest: "abcd",
            // These fields are unchecked:
            expected: "".into(), source: None,
        }));
}

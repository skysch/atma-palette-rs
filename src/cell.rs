////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette cell definitions.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
mod position;
mod reference;
mod selection;
mod selector;

// Local imports.
use crate::color::Color;
use crate::expr::Expr;
use crate::basic::BasicPalette;
use crate::error::Error;

// External library imports.
use serde::Serialize;
use serde::Deserialize;

// Standard library imports.
use std::cell::Cell as StdCell;
use std::collections::HashSet;

// Exports.
pub use position::*;
pub use reference::*;
pub use selection::*;
pub use selector::*;


////////////////////////////////////////////////////////////////////////////////
// Cell
////////////////////////////////////////////////////////////////////////////////
/// A cell holding a color expression.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cell {
    /// The cell's expression.
    expr: Expr,
    #[serde(skip)]
    cached: StdCell<Option<Color>>,
}

impl Cell {
    /// Constructs a new `Cell`.
    pub fn new() -> Self {
        Cell {
            expr: Default::default(),
            cached: StdCell::new(None),
        }
    }

    /// Constructs a new `Cell` containing the given `Expr`.
    pub fn new_with_expr(expr: Expr) -> Self {
        Cell {
            expr,
            cached: StdCell::new(None),
        }
    }

    /// Returns a reference to the cell's color expression.
    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    /// Returns a mut reference to the cell's color expression.
    pub fn expr_mut(&mut self) -> &mut Expr {
        &mut self.expr
    }

    /// Returns the Expr's color.
    pub fn color(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>)
        -> Result<Option<Color>, Error>
    {
        let cached = self.cached.clone().take();
        if cached.is_none() {
            self.color_evaluate(basic, index_list)
        } else {
            Ok(cached)
        }
    }

    /// Returns the Expr's color, forcing evaluation.
    pub fn color_evaluate(
        &self,
        basic: &BasicPalette,
        index_list: &mut HashSet<u32>)
        -> Result<Option<Color>, Error>
    {
        let eval = self.expr.color(basic, index_list)?;
        self.cached.set(eval.clone());
        Ok(eval)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new()
    }
}

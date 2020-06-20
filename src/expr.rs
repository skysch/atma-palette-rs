////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette color expression definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.

// External library imports.
use color::Color;
use serde::Deserialize;
use serde::Serialize;

/// Atma color expression for defining the behavior of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum Expr {
    /// An color expression with no color.
    Empty,
    /// A color.
    Color(Color),
    
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Empty
    }
}
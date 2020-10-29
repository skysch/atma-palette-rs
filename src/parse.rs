////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parsing module.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
mod ast;
mod ast_helper;
mod ast_match;
mod color;
mod common;
mod expr;
mod scanner;
mod selection;

// Exports.
pub use ast::*;
pub use ast_helper::*;
pub use ast_match::*;
pub use common::*;
pub use expr::*;
pub use scanner::*;
pub use selection::*;
pub use self::color::*;


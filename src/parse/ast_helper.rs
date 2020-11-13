////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma color expressions and selection types.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::color::Color;
use crate::cell::CellRef;
use crate::cell::Position;

// Standard library imports.
use std::borrow::Cow;
use std::str::FromStr;

// External library imports.
use tephra::span::Span;
use tephra::result::Spanned;


////////////////////////////////////////////////////////////////////////////////
// Identifier
////////////////////////////////////////////////////////////////////////////////
/// And AST matcher for parsing an identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);


////////////////////////////////////////////////////////////////////////////////
// Selection types
////////////////////////////////////////////////////////////////////////////////
/// An AST matcher for parsing a Position or Index selector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionOrIndex {
    Index(u32),
    Position(Position),
}

impl<'name> From<PositionOrIndex> for CellRef<'name> {
    fn from(pos_or_idx: PositionOrIndex) -> Self {
        match pos_or_idx {
            PositionOrIndex::Position(pos) => CellRef::Position(pos),
            PositionOrIndex::Index(idx)    => CellRef::Index(idx),
        }
    }
}


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
// Selection types
////////////////////////////////////////////////////////////////////////////////


impl<'name> From<PositionOrIndex> for CellRef<'name> {
    fn from(poi: PositionOrIndex) -> Self {
        match poi {
            PositionOrIndex::Position(pos) => CellRef::Position(pos),
            PositionOrIndex::Index(idx)    => CellRef::Index(idx),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum PositionOrIndex {
    Index(u32),
    Position(Position),
}


////////////////////////////////////////////////////////////////////////////////
// Function calls
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub struct FnCall<'text> {
    pub name: &'text str,
    pub args: Vec<Spanned<'text, FnArg>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FnArg {
    U32(u32),
    F32(f32),
}

////////////////////////////////////////////////////////////////////////////////
// Identifier
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

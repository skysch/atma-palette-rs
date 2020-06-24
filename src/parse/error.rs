////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse errors.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::parse::*;

// Standard library imports.
use std::borrow::Borrow;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::convert::Into;
use std::convert::TryFrom;
use std::convert::TryInto;

////////////////////////////////////////////////////////////////////////////////
// ParseIntegerOverflow
////////////////////////////////////////////////////////////////////////////////
/// An overflow error occurred while parsing an integer.
#[derive(Debug, Clone)]
pub struct ParseIntegerOverflow {
    /// The integer type.
    pub int_type: Cow<'static, str>,
    /// The integer text.
    pub int_text: Cow<'static, str>,
}


impl std::fmt::Display for ParseIntegerOverflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "integer value '{}' does not fit in type {}",
            self.int_text, self.int_type)
    }
}

impl std::error::Error for ParseIntegerOverflow {}




////////////////////////////////////////////////////////////////////////////////
// GroupRangeMismatch
////////////////////////////////////////////////////////////////////////////////
/// A group range was parsed with mismatched group names.
#[derive(Debug, Clone)]
pub struct GroupRangeMismatch {
    /// The low group name.
    pub group_low: Cow<'static, str>,
    /// The high group name.
    pub group_high: Cow<'static, str>,
}


impl std::fmt::Display for GroupRangeMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "group range with lower bound in '{}' \
            does not match the upper bound in '{}'",
            self.group_low, self.group_high)
    }
}

impl std::error::Error for GroupRangeMismatch {}


////////////////////////////////////////////////////////////////////////////////
// RangeIndexOrder
////////////////////////////////////////////////////////////////////////////////
/// A range was parsed with the wrong element order.
#[derive(Debug, Clone)]
pub struct RangeIndexOrder {
    /// The parsed range.
    pub range: Cow<'static, str>,
}


impl std::fmt::Display for RangeIndexOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "range with lower bound exceeding upper bound '{}'",
            self.range)
    }
}

impl std::error::Error for RangeIndexOrder {}


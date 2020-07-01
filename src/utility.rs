////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common utility functions.
////////////////////////////////////////////////////////////////////////////////

pub use few::Few;

/// Performs a set intersection of the ranges bound (inclusively) by the given
/// tuples.
pub(crate) fn split_intersect<T: Ord>(l: (T, T), r: (T, T)) -> Few<T> {
    // Check if range order is wrong.
    if l.0 > l.1 || r.0 > r.1 ||
        // Check if ranges are non-overlapping.
        l.1 < r.0 || l.0 > r.1
    {
        return Few::Zero;
    }

    let low  = if l.0 > r.0 { l.0 } else { r.0 };
    let high = if l.1 < r.1 { l.1 } else { r.1 };
    if low == high {
        Few::One(low)
    } else {
        Few::Two(low, high)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Common utility functions.
////////////////////////////////////////////////////////////////////////////////


/// Performs a set intersection of the ranges bound (inclusively) by the given tuples.
pub fn inclusive_range_intersect<T: Ord>(l: (T, T), r: (T, T)) 
    -> Option<(T, T)>
{
    // Check if ranges are non-overlapping.
    if l.1 < r.0 || l.0 > r.1 { return None; }

    let low  = if l.0 > r.0 { l.0 } else { r.0 };
    let high = if l.1 < r.1 { l.1 } else { r.1 };
    Some((low, high))
}

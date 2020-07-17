////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface module.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
mod ancillary;
mod dispatch;
mod option;

/// Public modules.
pub mod new;
pub mod export_png;

// Exports.
pub use ancillary::*;
pub use dispatch::*;
pub use option::*;

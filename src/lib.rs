////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma library modules.
////////////////////////////////////////////////////////////////////////////////
#![warn(anonymous_parameters)]
#![warn(bad_style)]
#![warn(bare_trait_objects)]
#![warn(const_err)]
#![warn(dead_code)]
#![warn(elided_lifetimes_in_paths)]
#![warn(improper_ctypes)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_doc_code_examples)]
#![warn(missing_docs)]
#![warn(no_mangle_generic_items)]
#![warn(non_shorthand_field_patterns)]
#![warn(nonstandard_style)]
#![warn(overflowing_literals)]
#![warn(path_statements)]
#![warn(patterns_in_fns_without_body)]
#![warn(private_in_public)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unconditional_recursion)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(unused_allocation)]
#![warn(unused_comparisons)]
#![warn(unused_parens)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
#![warn(while_true)]

// Internal modules.
#[cfg(test)]
mod test;

// Public modules.
pub mod bimap;
pub mod cell;
pub mod command;
pub mod error;
pub mod palette;
pub mod parse;
pub mod setup;
pub mod utility;

// Exports.
/// Color encodings.
pub mod color {
    pub use color::*;
}
pub use palette::Expr;
pub use palette::Palette;
pub use palette::DEFAULT_PALETTE_PATH;
pub use setup::*;

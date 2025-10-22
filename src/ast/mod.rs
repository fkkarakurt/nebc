//! # Abstract Syntax Tree (AST) for Nebulang
//!
//! This module defines the core structures for the Abstract Syntax Tree (AST)
//! of the Nebulang language. The AST is an intermediate representation of
//! the source code used for subsequent phases like type checking and interpretation.
//!
//! It includes:
//! - [`nodes`]: Definitions for various statement, expression, and program nodes.
//! - [`types`]: Definitions for the basic data types and type-related utilities.

pub mod nodes;
pub mod types;
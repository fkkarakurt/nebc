//! # Parser Module
//!
//! This module serves as the primary interface for the syntactic analysis (parsing)
//! phase of the Nebulang compiler. It orchestrates the process of converting the
//! linear stream of tokens produced by the lexer into a hierarchical Abstract
//! Syntax Tree (AST).

// Sub-modules containing the core parsing logic.
pub mod common;
pub mod expression_parser;
pub mod statement_parser;

use crate::ast::nodes::Program;
use crate::compiler::error::CompileError;
use crate::compiler::lexer::Token;

/// The main entry point for the parsing phase.
///
/// This function initializes the concrete [`Parser`] and starts the recursive
/// descent process by calling `parse_program`.
///
/// # Arguments
///
/// * `tokens` - The vector of tokens received from the lexer, including
///   positional information.
///
/// # Returns
///
/// A `Result` containing the root [`Program`] AST node or a [`CompileError`].
pub fn parse(tokens: Vec<(Token, usize, usize, String)>) -> Result<Program, CompileError> {
    use common::Parser;

    // Create the parser instance with the token stream.
    let mut parser = Parser::new(tokens);

    // Begin parsing at the top level (Program).
    parser.parse_program()
}

//! # Print Statement Code Generator
//!
//! This module provides the [`PrintGenerator`], responsible for translating the
//! `Print` statement from the Abstract Syntax Tree (AST) into executable
//! assembly code.
//!
//! It handles both literal string segments and expression evaluations, ensuring
//! proper output to the standard stream, including support for a custom
//! newline delimiter (`>|`).

use super::common::CodeGenCommon;
use super::expression_generator::ExpressionGenerator;
use crate::ast::nodes::{Expression, PrintPart};
use crate::compiler::error::CompileError;

/// A static utility struct dedicated to generating assembly code for `Print` statements.
pub struct PrintGenerator;

impl PrintGenerator {
    /// Generates assembly code for a single `Print` statement, iterating over its parts.
    ///
    /// This function handles dynamic expression evaluation and the output of static
    /// strings, as well as checking for and emitting the newline character.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `parts` - A slice of [`PrintPart`] nodes (strings or expressions).
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated assembly code as a `String` or a [`CompileError`].
    pub fn generate_print(
        common: &mut CodeGenCommon,
        parts: &[PrintPart],
    ) -> Result<String, CompileError> {
        let mut asm = String::new();

        for part in parts {
            match part {
                PrintPart::String(s) => {
                    // Nebulang uses ">|" as a custom newline marker within strings.
                    let has_newline = s.contains(">|");
                    let clean_string = s.replace(">|", "");

                    // 1. Generate code for the string content (if any remains).
                    if !clean_string.is_empty() {
                        let label = common.add_string_to_pool(&clean_string);
                        // Load address (RSI) and length (RDX)
                        asm.push_str(&format!("    mov rsi, {}\n", label));
                        asm.push_str(&format!("    mov rdx, {}\n", clean_string.len()));
                        asm.push_str("    call _nebula_print\n");
                    }

                    // 2. Generate code for the explicit newline if the marker was found.
                    if has_newline {
                        // The 'newline' label is defined in the .data section of CodeGenCommon.
                        asm.push_str("    mov rsi, newline\n");
                        asm.push_str("    mov rdx, 1\n"); // Length of the newline character (LF, 0xA)
                        asm.push_str("    call _nebula_print\n");
                    }
                }
                PrintPart::Expression(expr) => {
                    // Handle Expression parts, which require evaluation before printing.
                    match expr.as_ref() {
                        // Special handling for boolean literals to print "TRUE" or "FALSE" strings.
                        Expression::Boolean(b) => {
                            let s_val = if *b { "TRUE" } else { "FALSE" };
                            let len = s_val.len();
                            let label = common.add_string_to_pool(s_val);

                            // Load address (RSI) and length (RDX)
                            asm.push_str(&format!("    mov rsi, {}\n", label));
                            asm.push_str(&format!("    mov rdx, {}\n", len));
                            asm.push_str("    call _nebula_print\n");
                        }
                        // Defer to the ExpressionGenerator's dedicated print function for other types.
                        _ => {
                            asm.push_str(&ExpressionGenerator::generate_expression_print(
                                common, expr,
                            )?);
                        }
                    }
                }
            }
        }

        Ok(asm)
    }
}

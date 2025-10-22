//! # Nebulang Compiler Crate
//!
//! This crate contains the core implementation of the Nebulang compiler,
//! providing modules for lexical analysis, parsing, semantic analysis,
//! and code generation. It exposes the main `Compiler` structure and helper
//! functions for building and running Nebulang source code.

// --- Core Modules ---
/// Defines the Abstract Syntax Tree (AST) node structures and related types.
pub mod ast;
/// Contains the code generation phase logic.
pub mod codegen;
/// The core compiler logic, encompassing all phases and the main `Compiler` struct.
pub mod compiler;
/// Utilities for debugging and introspection of compiler stages.
pub mod debug;
// pub mod vm; // Virtual machine for execution (currently commented out)

// --- Public Re-exports (The Compiler API) ---
/// Re-exports the main compiler structure for managing build configurations.
pub use compiler::Compiler;
/// Re-exports the semantic analysis function.
pub use compiler::analyze;
/// Re-exports the parsing function.
pub use compiler::parse;
/// Re-exports the lexical analysis (tokenization) function.
pub use compiler::tokenize;

// Re-exports the specific code generator for users who need low-level access
// to the generated quantum assembly.
pub use crate::codegen::quantum_asm::QuantumAssemblyGenerator;

// --- Convenience Functions ---

/// Provides a simple, end-to-end compilation pipeline for a given source string.
///
/// This function performs: Lexing -> Parsing -> Semantic Analysis.
/// It does **not** include code generation, assembly, or linking.
///
/// # Arguments
///
/// * `source` - The raw Nebulang source code string.
///
/// # Returns
///
/// `Ok(())` if the source is syntactically and semantically valid, or a
/// [`compiler::error::CompileError`] otherwise.
pub fn compile(source: &str) -> Result<(), compiler::error::CompileError> {
    let tokens = tokenize(source)?;
    let ast = parse(tokens)?;
    analyze(&ast)?;
    Ok(())
}

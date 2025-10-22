//! # Code Generation Module
//!
//! This module orchestrates the process of translating the Abstract Syntax Tree (AST)
//! into executable target code (currently x86-64 assembly).
//!
//! It serves as the primary entry point for all sub-components involved in code emission.
//!
//! Key components include:
//! - **Common Context**: Manages shared state like variable addresses and string pools.
//! - **Generators**: Specialized logic for handling different AST node types (statements, expressions, etc.).
//! - **Quantum ASM**: Handles the final assembly structure and advanced, optional features (like runtime integrity).

pub mod common;
pub mod expression_generator;
pub mod print_generator;
pub mod quantum_asm;
pub mod statement_generator;

// Note: The public re-export is commented out in the original, but the structure
// is maintained for modularity. Uncommenting this line would simplify imports
// from external modules, promoting a cleaner API.
// pub use quantum_asm::QuantumAssemblyGenerator;

//! # Compiler Error Definitions
//!
//! This module centralizes all custom error types that can occur during the
//! entire compilation and execution process of Nebulang programs.
//!
//! It leverages the `thiserror` crate to simplify error reporting and provide
//! clear, descriptive messages to the user.

use std::process::ExitStatus;
use thiserror::Error;

/// The primary error enumeration for the Nebulang compiler.
#[derive(Error, Debug)]
pub enum CompileError {
    /// Wrapper for standard I/O errors (e.g., file reading/writing).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error raised when the compiler cannot find any source files to process.
    #[error("No .neb source files found in current directory")]
    NoSourceFiles,

    /// General type-related errors caught during semantic analysis.
    #[error("Type error: {message}")]
    TypeError { message: String },

    /// Error raised if the compiled executable file is missing after the compilation stage.
    #[error("Binary not found after compilation")]
    BinaryNotFound,

    /// Error raised during program execution if an I/O issue occurs.
    #[error("Execution error: {0}")]
    ExecutionError(#[source] std::io::Error),

    /// Error raised if the executed program terminates with a non-zero exit status.
    #[error("Execution failed with status: {0}")]
    ExecutionFailed(ExitStatus),

    /// Error raised when running test suites, indicating one or more tests failed.
    #[error("One or more tests failed")]
    TestFailed,

    /// Syntax errors caught during the lexical analysis or parsing stages.
    /// Includes positional information for user feedback.
    #[error("Syntax error at position {position}: {message}")]
    SyntaxError { position: usize, message: String },

    /// Semantic error indicating a variable was used before it was declared.
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },

    /// Semantic error indicating operations between incompatible types.
    #[error("Type mismatch: {details}")]
    TypeMismatch { details: String },
}

impl CompileError {
    /// Constructs a `SyntaxError` specific to the **Lexer** phase.
    pub fn lexer(message: impl Into<String>) -> Self {
        Self::SyntaxError {
            position: 0, // Positional data is often imprecise at the lexer level
            message: message.into(),
        }
    }

    /// Constructs a `SyntaxError` specific to the **Parser** phase.
    pub fn parser(message: impl Into<String>) -> Self {
        Self::SyntaxError {
            position: 0, // Positional data is often gathered and set here in full compilers
            message: message.into(),
        }
    }

    /// Constructs a general `TypeError` for semantic analysis failures.
    pub fn analysis(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    /// Alias for creating a general `TypeError` (e.g., "Loop start must be integer").
    pub fn r#type(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    /// Another alias for creating a general `TypeError`.
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    /// Constructs a `SyntaxError` with explicit positional information.
    pub fn syntax(position: usize, message: impl Into<String>) -> Self {
        Self::SyntaxError {
            position,
            message: message.into(),
        }
    }

    /// Constructs an `UndefinedVariable` error, typically used by the analyzer.
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::UndefinedVariable { name: name.into() }
    }

    /// Constructs a `TypeMismatch` error, providing specific details about the mismatched types/operation.
    pub fn type_mismatch(details: impl Into<String>) -> Self {
        Self::TypeMismatch {
            details: details.into(),
        }
    }
}

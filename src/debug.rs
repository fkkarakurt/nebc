//! # Debugging and Logging Utilities
//!
//! This module provides utility functions and structures for conditional logging,
//! debugging, and performance measurement throughout the compiler pipeline.
//! Logging is typically gated by environment variables or compilation settings.

use std::time::Instant;

/// Checks if the general debug mode is active.
///
/// Debugging is enabled if the `NEBC_DEBUG` environment variable is set
/// or if the code is compiled with `debug_assertions` enabled (default for `cargo build`).
pub fn is_debug_enabled() -> bool {
    std::env::var("NEBC_DEBUG").is_ok() || cfg!(debug_assertions)
}

/// Checks if performance/verbose tracking mode is active.
///
/// Performance tracking is enabled if the `NEBC_VERBOSE` environment variable is set.
pub fn is_perf_enabled() -> bool {
    std::env::var("NEBC_VERBOSE").is_ok()
}

// --- Conditional Logging Functions ---

/// Logs a message related to the Parser phase, only if debug mode is active.
///
/// # Arguments
/// * `msg` - The message to display.
pub fn log_parser(msg: &str) {
    if is_debug_enabled() {
        println!("🔍 PARSER: {}", msg);
    }
}

/// Logs a message related to the Lexer phase, only if debug mode is active.
pub fn log_lexer(msg: &str) {
    if is_debug_enabled() {
        println!("🔍 LEXER: {}", msg);
    }
}

/// Logs a message related to the Code Generation phase, only if debug mode is active.
pub fn log_codegen(msg: &str) {
    if is_debug_enabled() {
        println!("🔍 CODEGEN: {}", msg);
    }
}

/// Logs a general message related to the Compiler orchestration, only if debug mode is active.
pub fn log_compiler(msg: &str) {
    if is_debug_enabled() {
        println!("🔍 COMPILER: {}", msg);
    }
}

// --- Performance Tracking Structure ---

/// A simple structure for timing the duration of specific operations.
///
/// Usage: `let timer = PerfTimer::new("Operation X"); ... timer.finish();`
pub struct PerfTimer {
    start: Instant,
    label: String,
}

impl PerfTimer {
    /// Creates a new timer instance, capturing the current moment.
    ///
    /// # Arguments
    /// * `label` - A descriptive name for the timed operation.
    pub fn new(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    /// Stops the timer, calculates the elapsed duration, and prints the result
    /// if performance tracking (`is_perf_enabled`) is active.
    pub fn finish(self) {
        if is_perf_enabled() {
            let duration = self.start.elapsed();
            println!("⏱️  {}: {:?}", self.label, duration);
        }
    }
}

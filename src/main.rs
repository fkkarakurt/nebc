//! # Nebulang Quantum Compiler (NEBC) Executable
//!
//! This is the main entry point for the Nebulang compiler's command-line
//! interface (CLI). It uses the `clap` crate to parse user arguments and
//! orchestrates the `compiler::Compiler` to perform build, run, and test actions.

use clap::{Arg, Command};
use std::path::PathBuf;

// Import internal compiler components.
mod ast;
mod compiler;
// mod vm; // Virtual machine module
mod codegen;

use compiler::Compiler;
use std::process;

fn main() {
    // Define the command-line interface structure using clap.
    let matches = Command::new("nebc")
        .version("0.1.0")
        .author("Nebula Project Systems")
        .about("🌌 Nebulang Quantum Compiler")
        // --- 'build' Subcommand ---
        .subcommand(
            Command::new("build")
                .about("Build quantum binary")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .value_name("FILE")
                        .help("Nebulang source file to build"),
                )
                .arg(
                    Arg::new("target")
                        .long("target")
                        .value_name("OS")
                        .help("Target platform (windows, linux, mac)"),
                )
                .arg(
                    Arg::new("show-asm")
                        .long("show-asm")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show generated assembly code"),
                )
                .arg(
                    Arg::new("no-protection")
                        .long("no-protection")
                        .action(clap::ArgAction::SetTrue)
                        .help("Disable quantum protection (for debugging)"),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show step-by-step compilation process"),
                ),
        )
        // --- 'run' Subcommand ---
        .subcommand(
            Command::new("run")
                .about("Compile and run quantum program")
                .arg(
                    Arg::new("file")
                        .required(true)
                        .value_name("FILE")
                        .help("Nebulang source file to run"),
                ),
        )
        // --- 'test' Subcommand ---
        .subcommand(
            Command::new("test")
                .about("Test quantum program files")
                .arg(
                    Arg::new("file")
                        .value_name("FILE")
                        .help("Specific file to test"),
                ),
        )
        .get_matches();

    // Initialize the main compiler instance with default settings.
    let mut compiler = Compiler::new();

    // Dispatch the command based on the user's input.
    match matches.subcommand() {
        Some(("build", sub_matches)) => {
            // Set source file path.
            let file = sub_matches.get_one::<String>("file").unwrap();
            compiler.source_path = PathBuf::from(file);

            // Get target, defaulting to "current".
            let target = sub_matches
                .get_one::<String>("target")
                .map(|s| s.as_str())
                .unwrap_or("current");

            // Set compiler flags.
            compiler.show_asm = sub_matches.get_flag("show-asm");
            compiler.no_protection = sub_matches.get_flag("no-protection");
            compiler.verbose = sub_matches.get_flag("verbose");

            // Execute the build command.
            if let Err(e) = compiler.build(target) {
                eprintln!("❌ Build failed: {}", e);
                process::exit(1);
            }
        }
        Some(("run", sub_matches)) => {
            // Set source file path.
            let file = sub_matches.get_one::<String>("file").unwrap();
            compiler.source_path = PathBuf::from(file);

            // Execute the run command (which includes build and execute).
            if let Err(e) = compiler.run_single_file() {
                eprintln!("❌ Run failed: {}", e);
                process::exit(1);
            }
        }
        Some(("test", sub_matches)) => {
            // Get optional specific file to test.
            let file = sub_matches.get_one::<String>("file").map(PathBuf::from);

            // Execute the test command.
            if let Err(e) = compiler.test(file) {
                eprintln!("❌ Test failed: {}", e);
                process::exit(1);
            }
        }
        // Default case: show help message.
        _ => {
            println!("🌌 Nebulang Quantum Compiler (NEBC)");
            println!("Use 'nebc --help' for usage information");
        }
    }
}

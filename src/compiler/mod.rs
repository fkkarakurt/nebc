//! # Compiler Module
//!
//! This module acts as the core entry point and orchestrator for the Nebulang
//! compiler. It aggregates all sub-modules (lexer, parser, analyzer, codegen)
//! and defines the main [`Compiler`] struct responsible for managing the build,
//! run, and test lifecycles of a Nebulang source file.

// Publicly exposes the compiler phases.
pub mod analyzer;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;

// Re-exports essential functions for external use.
pub use analyzer::analyze;
pub use lexer::tokenize;
pub use parser::parse;

// Internal dependencies for the compilation process.
use crate::codegen::quantum_asm::QuantumAssemblyGenerator;
use error::CompileError;
use std::path::PathBuf;
use std::process::Command;

/// The central structure that manages the compilation, assembly, and execution
/// of a Nebulang program.
pub struct Compiler {
    /// The path to the source file or directory to compile.
    pub source_path: std::path::PathBuf,
    /// The directory where build artifacts (ASM, objects, binary) are placed.
    pub build_path: std::path::PathBuf,
    /// The target architecture/OS (e.g., "current", "windows", "mac").
    pub target: String,
    /// Flag to print the generated assembly code to stdout instead of compiling.
    pub show_asm: bool,
    /// Flag to disable quantum assembly protections (if implemented).
    pub no_protection: bool,
    /// Flag for detailed output messages during the build process.
    pub verbose: bool,
}

/*
// Example of the simple compilation pipeline (currently commented out but defines the core steps):
pub fn compile(source: &str) -> Result<(), error::CompileError> {
    let tokens = tokenize(source)?;
    let ast = parse(tokens)?;
    analyze(&ast)?;
    // codegen::generate(&ast)?;
    Ok(())
}
*/

impl Compiler {
    /// Creates a new Compiler instance with default settings.
    pub fn new() -> Self {
        Self {
            source_path: std::path::PathBuf::from("."),
            build_path: std::path::PathBuf::from("./build"),
            target: "current".to_string(),
            show_asm: false,
            no_protection: false,
            verbose: false,
        }
    }

    /// Initiates the build process.
    ///
    /// It first checks if the source path points to a single `.neb` file or
    /// delegates to directory compilation logic (if implemented).
    ///
    /// # Arguments
    /// * `target` - The target platform for the resulting binary.
    pub fn build(&mut self, target: &str) -> Result<(), CompileError> {
        self.target = target.to_string();

        if self.source_path.is_file() && self.is_neb_file(&self.source_path) {
            return self.build_single_file(target);
        }

        Err(CompileError::NoSourceFiles)
    }

    /// Executes the full compilation pipeline for a single Nebulang source file.
    ///
    /// The pipeline includes: Lexing -> Parsing -> Semantic Analysis -> Code Generation -> Assembly -> Linking.
    fn build_single_file(&mut self, target: &str) -> Result<(), CompileError> {
        self.target = target.to_string();

        if !self.source_path.exists() || !self.is_neb_file(&self.source_path) {
            return Err(CompileError::NoSourceFiles);
        }

        self.log_verbose(&format!("Processing: {:?}", self.source_path));

        // 1. Read Source
        let content = std::fs::read_to_string(&self.source_path)?;

        // 2. Lexing (Tokenize)
        let tokens = tokenize(&content)?;

        // 3. Parsing (Build AST)
        let ast = parse(tokens)?;

        // 4. Semantic Analysis (Type/Symbol Check)
        analyze(&ast)?;

        // 5. Code Generation (Generate ASM)
        let mut quantum_gen = QuantumAssemblyGenerator::new();
        let asm_code = quantum_gen.generate(&ast)?;

        // Output ASM if requested
        if self.show_asm {
            println!("{}", asm_code);
            return Ok(());
        }

        // 6. Write Assembly to File
        let asm_file_path = self.build_path.join("quantum_output.asm");
        std::fs::write(&asm_file_path, &asm_code)?;

        self.log_verbose(&format!(
            "Generated quantum assembly: {} lines",
            asm_code.lines().count()
        ));

        // 7. Assemble and Link to Binary
        self.compile_assembly_to_binary(&asm_file_path)?;

        println!(
            "✅ {:?} - Quantum compilation successful!",
            self.source_path
        );

        Ok(())
    }

    /// Compiles and then executes a single Nebulang file.
    pub fn run_single_file(&mut self) -> Result<(), CompileError> {
        self.build_single_file("current")?;
        self.execute_binary()
    }

    /// Discovers and executes tests on Nebulang files.
    ///
    /// # Arguments
    /// * `specific_file` - An optional path to run only a single test file.
    pub fn test(&self, specific_file: Option<PathBuf>) -> Result<(), CompileError> {
        let files_to_test = if let Some(file) = specific_file {
            vec![file]
        } else {
            self.find_neb_files_in_directory()?
        };

        println!("Testing {} files", files_to_test.len());

        let mut all_passed = true;
        for file in files_to_test {
            print!("Testing {:?}... ", file);
            match self.test_file(&file) {
                Ok(_) => println!("✓ PASSED"),
                Err(e) => {
                    println!("✗ FAILED");
                    eprintln!("  Error: {}", e);
                    all_passed = false;
                }
            }
        }

        if all_passed {
            println!("All tests passed! 🎉");
            Ok(())
        } else {
            Err(CompileError::TestFailed)
        }
    }

    /// Executes the final steps: invoking an assembler (nasm) and a linker (ld/gcc).
    fn compile_assembly_to_binary(&self, asm_file_path: &PathBuf) -> Result<(), CompileError> {
        let output_name = self.get_output_name();
        let output_path = self.build_path.join(&output_name);

        self.log_verbose("Assembling quantum code...");

        std::fs::create_dir_all(&self.build_path)?;

        let obj_file_path = self.build_path.join("quantum_object.o");

        // 8. Assembly (Using nasm)
        let assemble_status = Command::new("nasm")
            .arg("-f")
            .arg(self.get_target_assembly_format())
            .arg(asm_file_path)
            .arg("-o")
            .arg(&obj_file_path)
            .status()
            .map_err(CompileError::ExecutionError)?;

        if !assemble_status.success() {
            return Err(CompileError::ExecutionFailed(assemble_status));
        }

        self.log_verbose("Linking quantum binary...");

        // 9. Linking (Using ld or gcc)
        let link_result = self.link_binary(&obj_file_path, &output_path);

        match link_result {
            Ok(_) => {
                println!("📦 Quantum binary generated: {:?}", output_path);
                self.make_executable(&output_path)?;
                Ok(())
            }
            // If the primary linker fails, try the alternative (e.g., trying `gcc` if `ld` failed).
            Err(e) => self
                .try_alternative_linker(&obj_file_path, &output_path)
                .map_err(|_| e),
        }
    }

    /// Calls the primary linker tool specified by the target.
    fn link_binary(
        &self,
        obj_file_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<(), CompileError> {
        let linker = self.get_target_linker();
        let mut command = Command::new(linker);

        command.arg(obj_file_path).arg("-o").arg(output_path);

        if linker == "gcc" {
            // Needed for linking raw assembly objects without C runtime startup files.
            command.arg("-nostartfiles");
        }

        let status = command.status().map_err(CompileError::ExecutionError)?;

        if status.success() {
            Ok(())
        } else {
            Err(CompileError::ExecutionFailed(status))
        }
    }

    /// Attempts to link using the alternate linker (gcc if ld was primary, or ld if gcc was primary).
    fn try_alternative_linker(
        &self,
        obj_file_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<(), CompileError> {
        let alternative_linker = if self.get_target_linker() == "ld" {
            "gcc"
        } else {
            "ld"
        };

        let status = Command::new(alternative_linker)
            .arg(obj_file_path)
            .arg("-o")
            .arg(output_path)
            .arg("-nostartfiles")
            .status()
            .map_err(CompileError::ExecutionError)?;

        if status.success() {
            println!("✅ Binary linked successfully with {}", alternative_linker);
            Ok(())
        } else {
            Err(CompileError::ExecutionFailed(status))
        }
    }

    /// Executes the final compiled binary.
    fn execute_binary(&self) -> Result<(), CompileError> {
        let binary_path = self.build_path.join(self.get_output_name());

        if !binary_path.exists() {
            return Err(CompileError::BinaryNotFound);
        }

        let status = std::process::Command::new(&binary_path)
            .status()
            .map_err(CompileError::ExecutionError)?;

        if status.success() {
            Ok(())
        } else {
            Err(CompileError::ExecutionFailed(status))
        }
    }

    /// Finds all files with the `.neb` extension in the source directory.
    fn find_neb_files_in_directory(&self) -> Result<Vec<PathBuf>, CompileError> {
        let mut files = Vec::new();

        if self.source_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&self.source_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && self.is_neb_file(&path) {
                        files.push(path);
                    }
                }
            }
        }

        if files.is_empty() {
            Err(CompileError::NoSourceFiles)
        } else {
            Ok(files)
        }
    }

    /// Stub function to run a specific file in test mode (currently only performs parse).
    fn test_file(&self, file_path: &PathBuf) -> Result<(), CompileError> {
        let content = std::fs::read_to_string(file_path)?;
        let tokens = tokenize(&content)?;
        let _ast = parse(tokens)?;
        // NOTE: A complete test would also execute the binary and verify its output/exit code.
        Ok(())
    }

    /// Checks if a given path has the `.neb` extension.
    fn is_neb_file(&self, path: &PathBuf) -> bool {
        path.extension().map_or(false, |ext| ext == "neb")
    }

    /// Determines the final executable name based on the target platform.
    fn get_output_name(&self) -> String {
        match self.target.as_str() {
            "windows" => "quantum_output.exe".to_string(),
            _ => "quantum_output".to_string(),
        }
    }

    /// Determines the assembly format required by NASM based on the target.
    fn get_target_assembly_format(&self) -> &str {
        match self.target.as_str() {
            "windows" => "win64",
            "mac" => "macho64",
            _ => "elf64", // Default for Linux/Unix
        }
    }

    /// Determines the appropriate linker tool based on the target.
    fn get_target_linker(&self) -> &str {
        match self.target.as_str() {
            "windows" => "gcc", // Often used on Windows for simpler linking
            _ => "ld",          // Default linker on Unix-like systems
        }
    }

    /// Sets the executable permission on the generated binary (Unix-specific).
    fn make_executable(&self, path: &PathBuf) -> Result<(), CompileError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                // Set executable permissions (rwxr-xr-x)
                perms.set_mode(0o755);
                std::fs::set_permissions(path, perms)?;
            }
        }
        Ok(())
    }

    /// Prints a message only if verbose mode is enabled.
    fn log_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

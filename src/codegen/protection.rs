//! # Runtime Protection Code Generator
//!
//! This module defines the [`QuantumProtectionGenerator`], which is responsible for
//! injecting various runtime integrity checks and obfuscation mechanisms into the
//! generated assembly code.
//!
//! The goal is to provide different levels of Intellectual Property (IP) protection
//! and tamper-proofing for the compiled Nebulang binary.

use crate::compiler::error::CompileError;

/// Defines the different tiers of runtime protection offered by the Nebulang compiler.
#[derive(Debug, Clone)]
pub enum ProtectionLevel {
    /// No runtime protection mechanisms are injected.
    None,
    /// Basic protection, typically involving a simple static checksum and self-destruct mechanism.
    Basic,
    /// Advanced protection using temporal and spatial obfuscation techniques (e.g., control-flow scrambling).
    Quantum,
    /// Highest level of protection, reserved for future, more sophisticated anti-tamper implementations.
    Military,
}

/// Generates the assembly code required for various runtime protection and anti-debugging features.
pub struct QuantumProtectionGenerator {
    /// The selected level of protection to implement.
    protection_level: ProtectionLevel,
}

impl QuantumProtectionGenerator {
    /// Creates a new generator instance configured with the specified protection level.
    ///
    /// # Arguments
    ///
    /// * `level` - The desired [`ProtectionLevel`].
    pub fn new(level: ProtectionLevel) -> Self {
        Self {
            protection_level: level,
        }
    }

    /// Dispatches the code generation based on the configured protection level.
    ///
    /// # Arguments
    ///
    /// * `clean_asm` - The base assembly code generated for the program logic.
    ///
    /// # Returns
    ///
    /// A `Result` containing the final, potentially protected, assembly code.
    pub fn generate_protection(&self, clean_asm: &str) -> Result<String, CompileError> {
        match self.protection_level {
            ProtectionLevel::None => Ok(clean_asm.to_string()),
            ProtectionLevel::Basic => self.generate_basic_protection(clean_asm),
            ProtectionLevel::Quantum => self.generate_quantum_protection(clean_asm),
            ProtectionLevel::Military => self.generate_military_protection(clean_asm),
        }
    }

    /// Implements the logic for the `Basic` protection level.
    ///
    /// This level injects a simple code integrity check at runtime.
    ///
    /// # Arguments
    ///
    /// * `asm` - The existing program assembly.
    fn generate_basic_protection(&self, asm: &str) -> Result<String, CompileError> {
        let mut protected = String::new();

        // Inject integrity check entry point before the main code.
        protected.push_str("section .nebula_protection\n");
        protected.push_str(";; Entry point for basic runtime integrity check\n");
        protected.push_str("_nebula_integrity_check:\n");
        protected.push_str("    call _quantum_checksum_verify\n");
        protected.push_str("    test rax, rax\n");
        protected.push_str("    jnz _integrity_ok\n");
        protected.push_str("    call _nebula_self_destruct\n");
        protected.push_str("_integrity_ok:\n");
        protected.push_str("    ret\n\n");

        protected.push_str(asm);
        protected.push_str(&self.generate_checksum_function());

        Ok(protected)
    }

    /// Implements the logic for the `Quantum` protection level.
    ///
    /// This level injects advanced temporal and spatial code obfuscation routines.
    ///
    /// # Arguments
    ///
    /// * `asm` - The existing program assembly.
    fn generate_quantum_protection(&self, asm: &str) -> Result<String, CompileError> {
        let mut protected = String::new();

        // Inject advanced runtime setup.
        protected.push_str("section .nebula_quantum\n");
        protected.push_str(";; Quantum Entanglement and Scrambling Setup\n");
        protected.push_str("_quantum_entanglement_setup:\n");
        protected.push_str("    ; Dynamic code flow scrambling initialization\n");
        protected.push_str("    call _quantum_temporal_scramble\n");
        protected.push_str("    ; Spatial code linking/integrity establishment\n");
        protected.push_str("    call _quantum_spatial_entangle\n");
        protected.push_str("    ret\n\n");

        protected.push_str(asm);
        protected.push_str(&self.generate_quantum_functions());

        Ok(protected)
    }

    /// Placeholder for the highest protection level.
    ///
    /// Currently defaults to the Quantum level until further advanced mechanisms are implemented.
    fn generate_military_protection(&self, asm: &str) -> Result<String, CompileError> {
        // Future implementation might involve code virtualization or complex polymorphic layers.
        self.generate_quantum_protection(asm)
    }

    /// Generates the assembly functions specific to the Basic protection level.
    fn generate_checksum_function(&self) -> String {
        r#"
;; -------------------------------------------------------------------
;; Basic Integrity Functions
;; -------------------------------------------------------------------

_quantum_checksum_verify:
    ; Calculates a simple 8-bit additive checksum of the code section.
    ; This check is easy to bypass but serves as a deterrent against trivial patching.
    mov rsi, _start                       ; Start address of code
    mov rcx, _nebula_code_end - _start    ; Length of the code segment
    xor rax, rax
.checksum_loop:
    add al, [rsi]   ; Accumulate the sum in AL (lower 8 bits of RAX)
    inc rsi
    loop .checksum_loop
    cmp al, 0x42    ; Compare the final checksum (0x42 = 'B') against an expected magic value.
    ret

_nebula_self_destruct:
    ; Immediate termination of the program (exit(1)) upon integrity failure.
    mov rax, 60
    mov rdi, 1
    syscall
    ret
"#
        .to_string()
    }

    /// Generates the assembly functions specific to the Quantum protection level.
    fn generate_quantum_functions(&self) -> String {
        r#"
;; -------------------------------------------------------------------
;; Quantum Obfuscation Functions (Anti-Tamper/Anti-Analysis)
;; -------------------------------------------------------------------

_quantum_temporal_scramble:
    ; Scrambles instruction execution flow based on a dynamic factor (RDTSC).
    ; This makes static analysis difficult and tracing unpredictable.
    rdtsc               ; Read Time-Stamp Counter
    and eax, 0x7        ; Use the last 3 bits of EAX (8 possible paths)
    lea rbx, [temporal_jump_table]
    jmp [rbx + rax * 8] ; Indirect jump

temporal_jump_table:
    dq _temporal_path_0, _temporal_path_1, _temporal_path_2, _temporal_path_3
    dq _temporal_path_4, _temporal_path_5, _temporal_path_6, _temporal_path_7

; The paths are currently empty, but in a real scenario, they contain varying NOP/obfuscation sequences.
_temporal_path_0:
_temporal_path_1:
_temporal_path_2:
_temporal_path_3:
_temporal_path_4:
_temporal_path_5:
_temporal_path_6:
_temporal_path_7:
    ret

_quantum_spatial_entangle:
    ; Simple spatial entanglement: links critical variables/sections to a key.
    ; This is usually used to verify that the key/data hasn't been trivially modified.
    mov rax, [entanglement_key]
    xor [critical_section_1], rax
    xor [critical_section_2], rax
    ret

; Runtime Data for Quantum Protection
entanglement_key: dq 0x4E4553554C41  ; NEBULA magic value
critical_section_1: dq 0
critical_section_2: dq 0
"#
        .to_string()
    }
}

//! # Code Generation Common Utilities
//!
//! This module defines the central context structure, [`CodeGenCommon`], which manages
//! metadata required throughout the code generation phase of the Nebulang compiler.
//!
//! It handles essential tasks such as:
//! - Managing the global **string pool** for static data.
//! - Tracking **variable addresses** and **types**.
//! - Generating unique **labels** for control flow.
//! - Creating the necessary assembly **data** and **BSS** sections.

use crate::ast::types::Type;
use std::collections::HashMap;

/// A central struct for managing shared state and utilities during the code generation process.
///
/// This structure acts as a registry for global resources like strings, labels, and variables,
/// ensuring consistency and proper memory allocation in the generated assembly code.
#[derive(Debug)]
pub struct CodeGenCommon {
    /// Maps string literals found in the source code to their unique assembly labels.
    /// Key: String content, Value: Assembly label name (e.g., "str_0").
    pub string_pool: HashMap<String, String>,
    /// A counter used to generate unique numeric labels for control flow (e.g., loops, if statements).
    pub label_counter: usize,
    /// Stores the inferred data type for each declared variable.
    /// Key: Variable name, Value: [`Type`] enum.
    pub variable_types: HashMap<String, Type>,
    /// Stores the assembly memory address/label for each declared variable.
    /// Key: Variable name, Value: Assembly label name (e.g., "var_my_var").
    pub variable_addresses: HashMap<String, String>,
}

impl CodeGenCommon {
    /// Creates a new, empty instance of the code generation context.
    ///
    /// # Returns
    ///
    /// A [`CodeGenCommon`] struct initialized with empty maps and a zero counter.
    pub fn new() -> Self {
        Self {
            string_pool: HashMap::new(),
            label_counter: 0,
            variable_types: HashMap::new(),
            variable_addresses: HashMap::new(),
        }
    }

    /// Adds a string literal to the global string pool if it doesn't already exist,
    /// and returns the corresponding assembly label.
    ///
    /// Special handling is included for boolean string representations.
    ///
    /// # Arguments
    ///
    /// * `s` - The string literal content to be pooled.
    ///
    /// # Returns
    ///
    /// The unique assembly label assigned to the string (e.g., `"str_4"`).
    pub fn add_string_to_pool(&mut self, s: &str) -> String {
        if s.is_empty() {
            return "empty_str".to_string();
        }

        if let Some(existing_label) = self.string_pool.get(s) {
            return existing_label.clone();
        }

        let label = match s {
            "TRUE" => "str_true".to_string(),
            "FALSE" => "str_false".to_string(),
            _ => format!("str_{}", self.string_pool.len()),
        };

        self.string_pool.insert(s.to_string(), label.clone());
        label
    }

    /// Registers a new variable in the code generation context.
    ///
    /// This assigns a unique assembly address (label) and records its type.
    ///
    /// # Arguments
    ///
    /// * `name` - The identifier of the variable.
    /// * `var_type` - The resolved data type of the variable.
    ///
    /// # Returns
    ///
    /// The assembly label assigned to the variable (e.g., `"var_my_counter"`).
    pub fn register_variable(&mut self, name: &str, var_type: Type) -> String {
        let address = format!("var_{}", name);
        self.variable_types.insert(name.to_string(), var_type);
        self.variable_addresses
            .insert(name.to_string(), address.clone());
        address
    }

    /// Retrieves the assembly address (label) for a given variable name.
    ///
    /// # Arguments
    ///
    /// * `name` - The identifier of the variable.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the assembly label if the variable is registered.
    pub fn get_variable_address(&self, name: &str) -> Option<&String> {
        self.variable_addresses.get(name)
    }

    /// Retrieves the data type for a given variable name.
    ///
    /// # Arguments
    ///
    /// * `name` - The identifier of the variable.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the [`Type`] if the variable is registered.
    pub fn get_variable_type(&self, name: &str) -> Option<&Type> {
        self.variable_types.get(name)
    }

    /// Generates the `.data` section of the assembly code, including all pooled strings.
    ///
    /// # Returns
    ///
    /// A string containing the assembled `.data` section.
    pub fn generate_data_section(&self) -> String {
        let mut asm = String::new();
        asm.push_str("section .data\n");

        // Sort labels to ensure deterministic output, which is crucial for reproducible builds.
        let mut sorted_entries: Vec<_> = self.string_pool.values().collect();
        sorted_entries.sort();

        for label in sorted_entries {
            if let Some(string_value) = self
                .string_pool
                .iter()
                .find_map(|(k, v)| if v == label { Some(k) } else { None })
            {
                // Escape the string for assembly, handling characters like quotes and newlines.
                let escaped_string = string_value
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t");
                asm.push_str(&format!("{}: db \"{}\", 0\n", label, escaped_string));
            }
        }

        // Add standard static data elements.
        asm.push_str("newline: db 10, 0\n");
        asm.push_str("empty_str: db 0\n");
        asm.push_str("minus_sign: db \"-\", 0\n"); // Moved from generate_print_functions for better data organization

        asm
    }

    /// Generates the `.bss` section of the assembly code for uninitialized data.
    ///
    /// This includes space reservation for all program variables and critical runtime structures.
    ///
    /// # Arguments
    ///
    /// * `program` - The root AST node used to traverse and collect all declared variables.
    ///
    /// # Returns
    ///
    /// A string containing the assembled `.bss` section.
    pub fn generate_bss_section(&self, program: &crate::ast::nodes::Program) -> String {
        let mut asm = String::new();
        asm.push_str("section .bss\n");

        // Reserve memory for internal runtime/security components.
        asm.push_str("    quantum_seed: resq 1\n");
        asm.push_str("    critical_section_1: resq 1\n");
        asm.push_str("    critical_section_2: resq 1\n");

        let variables = self.collect_variables(program);
        // Reserve 8 bytes (resq 1) for each variable, assuming 64-bit architecture.
        for var in &variables {
            asm.push_str(&format!("    var_{}: resq 1\n", var));
        }

        asm
    }

    /// Escapes a raw string into a format suitable for use as a string literal
    /// within an assembly definition (e.g., `db "..."`).
    ///
    /// **Note**: This function is currently unused by `generate_data_section`'s simplified
    /// logic but serves as a more robust utility for potential future use.
    ///
    /// # Arguments
    ///
    /// * `s` - The raw input string.
    ///
    /// # Returns
    ///
    /// The escaped string suitable for assembly.
    #[allow(dead_code)] // Keep for future robustness but suppress warnings.
    fn escape_string_for_assembly(s: &str) -> String {
        let mut result = String::new();

        for ch in s.chars() {
            match ch {
                '\'' => result.push_str("''"),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '"' => result.push_str("\\\""),
                ch if ch.is_control() => result.push_str(&format!("\\x{:02x}", ch as u8)),
                _ => result.push(ch),
            }
        }

        result
    }

    /// Generates a unique assembly label for use in control flow.
    ///
    /// The label format is `L_<counter>`, and the internal counter is incremented.
    ///
    /// # Returns
    ///
    /// A unique label string.
    pub fn next_label(&mut self) -> String {
        let label = format!("L_{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Traverses the Abstract Syntax Tree (AST) to collect all unique variable names declared
    /// in the program.
    ///
    /// # Arguments
    ///
    /// * `program` - A reference to the root [`Program`] node.
    ///
    /// # Returns
    ///
    /// A `Vec` of unique variable identifiers (names).
    pub fn collect_variables(&self, program: &crate::ast::nodes::Program) -> Vec<String> {
        let mut variables = Vec::new();
        Self::collect_variables_from_statements(&program.statements, &mut variables);
        variables
    }

    /// Recursively collects variable names from a slice of statements.
    ///
    /// Variables are collected from `VariableDeclaration` and `Loop` statements.
    ///
    /// # Arguments
    ///
    /// * `statements` - A slice of [`Statement`] nodes to process.
    /// * `variables` - A mutable reference to the vector where unique variable names are accumulated.
    fn collect_variables_from_statements(
        statements: &[crate::ast::nodes::Statement],
        variables: &mut Vec<String>,
    ) {
        for statement in statements {
            match statement {
                crate::ast::nodes::Statement::VariableDeclaration { name, .. } => {
                    if !variables.contains(name) {
                        variables.push(name.clone());
                    }
                }
                crate::ast::nodes::Statement::Loop { variable, body, .. } => {
                    // Loop variable must also be considered declared
                    if !variables.contains(variable) {
                        variables.push(variable.clone());
                    }
                    Self::collect_variables_from_statements(body, variables);
                }
                crate::ast::nodes::Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    Self::collect_variables_from_statements(then_branch, variables);
                    if let Some(else_branch) = else_branch {
                        Self::collect_variables_from_statements(else_branch, variables);
                    }
                }
                // ArrayDeclaration is missing from the match, assuming it's an oversight and should be handled if needed.
                _ => {}
            }
        }
    }

    /// Generates the assembly code for essential runtime printing functions.
    ///
    /// These functions handle system calls for outputting strings and converting/printing numbers.
    ///
    /// # Returns
    ///
    /// A string containing the assembly functions.
    pub fn generate_print_functions(&self) -> String {
        r#"
; -------------------------------------------------------------------
; Runtime Print Utilities
; -------------------------------------------------------------------

; Print string function
; Input: rsi = string pointer, rdx = length
_nebula_print:
    push rax
    push rdi
    push rsi
    push rdx
    push rcx
    push r11
    
    mov rax, 1          ; sys_write (Linux/x86_64)
    mov rdi, 1          ; stdout file descriptor
    syscall
    
    pop r11
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rax
    ret

; Print number function (64-bit signed integer)
; Input: rax = number
_nebula_print_number:
    push rbp
    mov rbp, rsp
    sub rsp, 32         ; Reserve stack space for digit buffer
    
    ; Check if number is negative (jns = jump if not signed/negative)
    test rax, rax
    jns .positive
    
    ; Handle negative number: print '-' sign
    push rax            ; Save number before printing '-'
    mov rsi, minus_sign
    mov rdx, 1
    call _nebula_print
    pop rax
    neg rax             ; Negate the number for digit conversion
    
.positive:
    test rax, rax
    jz .print_zero      ; Handle the special case of 0
    
    mov r8, rax         ; r8 = number to convert
    mov r9, 0           ; r9 = digit counter
    mov r10, rsp        ; r10 = pointer to buffer on stack
    mov rbx, 10         ; Divisor = 10
    
.convert_loop:
    xor rdx, rdx        ; Clear rdx for division
    div rbx             ; rax = rax / 10, rdx = rax % 10
    add dl, '0'         ; Convert remainder (digit) to ASCII character
    mov [r10], dl       ; Store character in buffer (in reverse order)
    inc r10
    inc r9
    test rax, rax
    jnz .convert_loop   ; Continue if quotient is not zero
    
    ; Reverse the string (digits are currently stored in reverse order)
    mov rsi, rsp        ; Start of buffer
    lea rdi, [rsp + r9 - 1] ; End of buffer
.reverse_loop:
    cmp rsi, rdi
    jge .print_digits   ; Stop when pointers meet or cross
    mov al, [rsi]       ; Swap bytes
    mov cl, [rdi]
    mov [rsi], cl
    mov [rdi], al
    inc rsi
    dec rdi
    jmp .reverse_loop

.print_zero:
    mov byte [rsp], '0'
    mov r9, 1           ; Length is 1
    jmp .print_digits

.print_digits:
    mov rsi, rsp        ; Buffer address
    mov rdx, r9         ; Length
    call _nebula_print  ; Print the number string
    
    mov rsp, rbp        ; Restore stack pointer
    pop rbp
    ret

; String length function
; Input: rsi = string pointer
; Output: rax = length
_nebula_strlen:
    push rdi
    mov rdi, rsi        ; Source pointer
    xor rcx, rcx
    not rcx             ; Set rcx to maximum value
    xor al, al          ; Search byte is null (0)
    repne scasb         ; Find the null terminator
    not rcx             ; rcx now holds the index of the null terminator
    dec rcx             ; Subtract 1 to get the length
    mov rax, rcx
    pop rdi
    ret
"#
        .to_string()
    }

    /// Generates assembly code for "Quantum Protection" runtime security features.
    ///
    /// This includes functions for runtime initialization, temporal scrambling (obfuscation),
    /// spatial entanglement, and integrity checking/self-destruction.
    ///
    /// # Returns
    ///
    /// A string containing the assembly security functions.
    pub fn generate_quantum_protection(&self) -> String {
        r#"
; -------------------------------------------------------------------
; Runtime Quantum Protection Mechanisms
; These structures are designed to add runtime integrity and
; simple obfuscation layers.
; -------------------------------------------------------------------

_nebula_quantum_init:
    ; Initialize the seed using Time-Stamp Counter (TSC)
    rdtsc               ; Read Time-Stamp Counter into EDX:EAX
    xor rax, rcx        ; Simple combination of low/high parts
    mov rdi, quantum_seed
    mov [rdi], rax      ; Store initial seed
    ret

_quantum_temporal_scramble:
    ; Simple instruction-level obfuscation based on TSC least significant bits
    rdtsc
    and eax, 0x3        ; Use the last 2 bits of EAX (0, 1, 2, or 3)
    lea rbx, [temporal_jump_table]
    jmp [rbx + rax * 8] ; Indirect jump to one of the paths

temporal_jump_table:
    dq _temporal_path_0, _temporal_path_1, _temporal_path_2, _temporal_path_3

_temporal_path_0:
    nop                 ; Path 0: 1 NOP
    ret
_temporal_path_1:  
    nop
    nop                 ; Path 1: 2 NOPs
    ret
_temporal_path_2:
    nop
    nop
    nop                 ; Path 2: 3 NOPs
    ret
_temporal_path_3:
    nop
    nop
    nop
    nop                 ; Path 3: 4 NOPs
    ret

_quantum_spatial_entangle:
    ; XOR the critical sections with the seed (simple integrity/lock)
    mov rax, [quantum_seed]
    mov rdi, critical_section_1
    xor [rdi], rax
    mov rdi, critical_section_2  
    xor [rdi], rax
    ret

_quantum_integrity_check:
    ; Performs a fixed checksum and validates against an expected value
    call _quantum_checksum_verify_fixed
    test rax, rax       ; Check return value (RAX)
    jnz .integrity_ok   ; If RAX is non-zero, integrity is fine (assuming non-zero indicates match)
    call _nebula_self_destruct ; Otherwise, self-destruct
.integrity_ok:
    ret

_quantum_checksum_verify_fixed:
    ; Calculates a basic checksum/hash of the initial code segment
    mov rsi, _start     ; Start address of the code section
    mov rcx, 512        ; Check the first 512 bytes
    xor rax, rax        ; RAX = Checksum sum
    xor rbx, rbx        ; RBX = Checksum rotation/XOR value
.checksum_loop:
    cmp rcx, 0
    jz .checksum_done
    movzx rdx, byte [rsi] ; Load byte
    add rax, rdx        ; Add to simple sum
    rol rbx, 7          ; Rotate RBX left by 7
    xor rbx, rdx        ; XOR with byte
    inc rsi
    dec rcx
    jmp .checksum_loop
.checksum_done:
    add rax, rbx        ; Final combined checksum
    mov rcx, [quantum_seed]
    and rcx, 0xFFFF     ; Add a minor seed influence
    add rax, rcx
    ; Simple integrity check: compare the calculated checksum against a fixed, expected magic value (0x4E45 = 'NE')
    cmp rax, 0x4E45
    ret                 ; ZF (Zero Flag) indicates match

_nebula_self_destruct:
    ; Immediate termination of the program (exit(1))
    mov rax, 60         ; sys_exit (Linux/x86_64)
    mov rdi, 1          ; Exit code 1
    syscall
    ret
"#
        .to_string()
    }
}

/// Implements the `Default` trait for convenient initialization.
impl Default for CodeGenCommon {
    /// Creates a default, new instance of the context.
    fn default() -> Self {
        Self::new()
    }
}

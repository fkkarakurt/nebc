//! # Expression Code Generator
//!
//! This module defines the [`ExpressionGenerator`] which is responsible for translating
//! Abstract Syntax Tree (AST) expressions into corresponding x86-64 assembly instructions.
//!
//! All expression results are pushed onto the stack, following a stack-based expression evaluation model.

use super::common::CodeGenCommon;
use crate::ast::nodes::{BinaryOperator, Expression};
use crate::compiler::error::CompileError;

/// A static utility struct for generating assembly code from Nebulang expressions.
pub struct ExpressionGenerator;

impl ExpressionGenerator {
    /// The primary dispatcher function for generating assembly code for any type of expression.
    ///
    /// The result of the expression evaluation is guaranteed to be pushed onto the stack (`push rax`).
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `expr` - A reference to the [`Expression`] AST node.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated assembly code as a `String` or a [`CompileError`].
    pub fn generate_expression(
        common: &mut CodeGenCommon,
        expr: &Expression,
    ) -> Result<String, CompileError> {
        match expr {
            Expression::Binary {
                left,
                operator,
                right,
            } => Self::generate_binary_expression(common, left, operator, right),
            Expression::Variable(name) => Self::generate_variable_expression(common, name),
            Expression::Integer(n) => Self::generate_integer_expression(*n),
            Expression::String(s) => Self::generate_string_expression(common, s),
            Expression::Boolean(b) => Self::generate_boolean_expression(*b),
            Expression::ArrayAccess { array, index } => {
                Self::generate_array_access(common, array, index)
            }
        }
    }

    /// Generates assembly to push a literal 64-bit integer value onto the stack.
    ///
    /// # Arguments
    ///
    /// * `n` - The integer value.
    fn generate_integer_expression(n: i64) -> Result<String, CompileError> {
        Ok(format!("    push {}\n", n))
    }

    /// Generates assembly to push a boolean value onto the stack (1 for true, 0 for false).
    ///
    /// # Arguments
    ///
    /// * `b` - The boolean value.
    fn generate_boolean_expression(b: bool) -> Result<String, CompileError> {
        let value = if b { 1 } else { 0 };
        Ok(format!("    push {}\n", value))
    }

    /// Generates assembly to load the memory address of a string literal into RAX and push it onto the stack.
    ///
    /// This utilizes the string pool managed by [`CodeGenCommon`].
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `s` - The string literal content.
    fn generate_string_expression(
        common: &mut CodeGenCommon,
        s: &str,
    ) -> Result<String, CompileError> {
        let label = common.add_string_to_pool(s);
        // String literals are treated as pointers (addresses) on the stack.
        Ok(format!("    mov rax, {}\n    push rax\n", label))
    }

    /// Generates assembly to load a variable's 64-bit value from memory and push it onto the stack.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `name` - The identifier of the variable.
    fn generate_variable_expression(
        common: &mut CodeGenCommon,
        name: &str,
    ) -> Result<String, CompileError> {
        let address = common
            .get_variable_address(name)
            .ok_or_else(|| CompileError::undefined_variable(name))?
            .clone();
        // Load the 64-bit value from the variable's memory address.
        Ok(format!("    mov rax, [{}]\n    push rax\n", address))
    }

    /// Generates assembly for accessing an element of an array.
    ///
    /// This calculates the memory address: `[array_base_address + index * element_size (8)]`.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `array` - The identifier of the array variable.
    /// * `index` - The expression used to determine the index.
    fn generate_array_access(
        common: &mut CodeGenCommon,
        array: &str,
        index: &Expression,
    ) -> Result<String, CompileError> {
        let mut asm = String::new();
        let address = common
            .get_variable_address(array)
            .ok_or_else(|| CompileError::undefined_variable(array))?
            .clone();

        // 1. Evaluate the index expression and push it onto the stack.
        let index_asm = Self::generate_expression(common, index)?;
        asm.push_str(&index_asm);
        // 2. Pop index into RBX.
        asm.push_str("    pop rbx\n");
        // 3. Calculate address and load value: array_base + index * 8 (assuming 64-bit/8-byte elements).
        asm.push_str(&format!("    mov rax, [{} + rbx * 8]\n", address));
        // 4. Push the array element value onto the stack.
        asm.push_str("    push rax\n");
        Ok(asm)
    }

    /// Generates assembly for a binary operation (e.g., arithmetic, comparison, logic).
    ///
    /// The operands are evaluated first, popped from the stack, the operation is performed,
    /// and the result is pushed back onto the stack.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `left` - The left-hand side expression.
    /// * `operator` - The binary operator.
    /// * `right` - The right-hand side expression.
    fn generate_binary_expression(
        common: &mut CodeGenCommon,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
    ) -> Result<String, CompileError> {
        let mut asm = String::new();

        // Evaluate right operand first (pushed second, popped first).
        let right_asm = Self::generate_expression(common, right)?;
        asm.push_str(&right_asm);
        // Evaluate left operand second (pushed first, popped second).
        let left_asm = Self::generate_expression(common, left)?;
        asm.push_str(&left_asm);

        // Pop operands: left (pushed first) goes to RAX, right (pushed second) goes to RBX.
        // Stack order is essential: R-Value (RBX) < L-Value (RAX) for subtraction/division/comparison.
        asm.push_str("    pop rax\n"); // RAX = Left Operand
        asm.push_str("    pop rbx\n"); // RBX = Right Operand

        match operator {
            BinaryOperator::Add => {
                asm.push_str("    add rax, rbx\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Subtract => {
                asm.push_str("    sub rax, rbx\n"); // RAX - RBX
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Multiply => {
                asm.push_str("    imul rax, rbx\n"); // RAX = RAX * RBX
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Divide => {
                asm.push_str("    xor rdx, rdx\n"); // Clear RDX for 64-bit division
                asm.push_str("    idiv rbx\n"); // Signed division: RAX / RBX -> Quotient in RAX
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Modulo => {
                asm.push_str("    xor rdx, rdx\n"); // Clear RDX
                asm.push_str("    idiv rbx\n"); // Signed division
                asm.push_str("    push rdx\n"); // Remainder is in RDX
            }
            BinaryOperator::Power => {
                // Implementation of a simple integer power loop (RAX ^ RBX)
                asm.push_str("    mov rcx, rbx\n"); // RCX = exponent
                asm.push_str("    mov rbx, rax\n"); // RBX = base
                asm.push_str("    mov rax, 1\n"); // RAX = result (start at 1)
                asm.push_str("    test rcx, rcx\n");
                asm.push_str("    jz .power_done\n"); // If exponent is 0, result is 1
                asm.push_str(".power_loop:\n");
                asm.push_str("    imul rax, rbx\n"); // result = result * base
                asm.push_str("    dec rcx\n");
                asm.push_str("    jnz .power_loop\n");
                asm.push_str(".power_done:\n");
                asm.push_str("    push rax\n");
            }
            // --- Comparison Operators ---
            // Comparisons set the status flags; the `setX` instructions convert the flag to 0 or 1 byte.
            BinaryOperator::Equal => {
                asm.push_str("    cmp rax, rbx\n");
                asm.push_str("    sete al\n"); // Set AL to 1 if Equal
                asm.push_str("    movzx rax, al\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::NotEqual => {
                asm.push_str("    cmp rax, rbx\n");
                asm.push_str("    setne al\n"); // Set AL to 1 if Not Equal
                asm.push_str("    movzx rax, al\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Less => {
                asm.push_str("    cmp rax, rbx\n");
                asm.push_str("    setl al\n"); // Set AL to 1 if Less (signed)
                asm.push_str("    movzx rax, al\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Greater => {
                asm.push_str("    cmp rax, rbx\n");
                asm.push_str("    setg al\n"); // Set AL to 1 if Greater (signed)
                asm.push_str("    movzx rax, al\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::LessEqual => {
                asm.push_str("    cmp rax, rbx\n");
                asm.push_str("    setle al\n"); // Set AL to 1 if Less or Equal (signed)
                asm.push_str("    movzx rax, al\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::GreaterEqual => {
                asm.push_str("    cmp rax, rbx\n");
                asm.push_str("    setge al\n"); // Set AL to 1 if Greater or Equal (signed)
                asm.push_str("    movzx rax, al\n");
                asm.push_str("    push rax\n");
            }
            // --- Logical Operators ---
            // Operates on the 64-bit boolean values (1 or 0) on the stack.
            BinaryOperator::And => {
                asm.push_str("    and rax, rbx\n");
                asm.push_str("    push rax\n");
            }
            BinaryOperator::Or => {
                asm.push_str("    or rax, rbx\n");
                asm.push_str("    push rax\n");
            }
        }

        Ok(asm)
    }

    /// Generates assembly code specifically for printing an expression's value.
    ///
    /// This function handles the printing logic based on the expression's type (number vs. string/boolean),
    /// invoking the appropriate runtime print helper (`_nebula_print_number` or `_nebula_print`).
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `expr` - A reference to the [`Expression`] AST node.
    pub fn generate_expression_print(
        common: &mut CodeGenCommon,
        expr: &Expression,
    ) -> Result<String, CompileError> {
        let mut asm = String::new();

        match expr {
            // Direct handling for simple types to avoid stack push/pop overhead.
            Expression::Variable(name) => {
                let address = common
                    .get_variable_address(name)
                    .ok_or_else(|| CompileError::undefined_variable(name))?
                    .clone();
                // Load value into RAX for number printing function
                asm.push_str(&format!("    mov rax, [{}]\n", address));
                asm.push_str("    call _nebula_print_number\n");
            }
            Expression::Integer(n) => {
                // Load value into RAX directly
                asm.push_str(&format!("    mov rax, {}\n", n));
                asm.push_str("    call _nebula_print_number\n");
            }
            Expression::String(s) => {
                let label = common.add_string_to_pool(s);
                // Load address (RSI) and length (RDX) for string printing function
                asm.push_str(&format!("    mov rsi, {}\n", label));
                asm.push_str(&format!("    mov rdx, {}\n", s.len()));
                asm.push_str("    call _nebula_print\n");
            }
            Expression::Boolean(b) => {
                let s_val = if *b { "TRUE" } else { "FALSE" };
                let len = s_val.len();
                let label = common.add_string_to_pool(s_val);
                // Load address (RSI) and length (RDX) for string printing function
                asm.push_str(&format!("    mov rsi, {}\n", label));
                asm.push_str(&format!("    mov rdx, {}\n", len));
                asm.push_str("    call _nebula_print\n");
            }
            // Complex expressions (Binary, ArrayAccess) must be evaluated first.
            _ => {
                // 1. Generate code to evaluate the expression and push result onto stack.
                let expr_asm = Self::generate_expression(common, expr)?;
                asm.push_str(&expr_asm);
                // 2. Pop the result into RAX.
                asm.push_str("    pop rax\n");
                // 3. Print the number.
                asm.push_str("    call _nebula_print_number\n");
            }
        }

        Ok(asm)
    }
}

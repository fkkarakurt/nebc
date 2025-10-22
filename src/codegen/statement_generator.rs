//! # Statement Code Generator
//!
//! This module defines the [`StatementGenerator`], the core component responsible for
//! translating Nebulang's high-level statements (declarations, assignments, control flow)
//! into sequential x86-64 assembly instructions.
//!
//! It manages control flow labels and variable storage based on the shared code generation context.

use super::common::CodeGenCommon;
use super::expression_generator::ExpressionGenerator;
use super::print_generator::PrintGenerator;
use crate::ast::nodes::*;
use crate::ast::types::Type;
use crate::compiler::error::CompileError;

/// A static utility struct for generating assembly code from Nebulang statements.
pub struct StatementGenerator;

impl StatementGenerator {
    /// The primary dispatcher function for translating an AST statement node into assembly.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `statement` - A reference to the [`Statement`] AST node.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated assembly code as a `String`.
    pub fn generate_statement(
        common: &mut CodeGenCommon,
        statement: &Statement,
    ) -> Result<String, CompileError> {
        match statement {
            Statement::VariableDeclaration { name, value } => {
                Self::generate_variable_declaration(common, name, value)
            }
            Statement::ArrayDeclaration { name, elements } => {
                Self::generate_array_declaration(common, name, elements)
            }
            Statement::Print { parts } => Self::generate_print_statement(common, parts),
            Statement::Loop {
                variable,
                start,
                end,
                body,
            } => Self::generate_loop(common, variable, start, end, body),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => Self::generate_conditional(common, condition, then_branch, else_branch),
            Statement::Assignment {
                name,
                value,
                operator,
            } => Self::generate_assignment(common, name, value, operator),
        }
    }

    /// Generates assembly for a variable declaration.
    ///
    /// This function registers the variable in the BSS section and initializes its value.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `name` - The variable identifier.
    /// * `value` - The initial value expression.
    fn generate_variable_declaration(
        common: &mut CodeGenCommon,
        name: &str,
        value: &Expression,
    ) -> Result<String, CompileError> {
        let mut asm = String::new();
        // Register variable and get its assembly address. Default type is assumed to be Integer/Pointer.
        let address = common.register_variable(name, Type::Integer);

        match value {
            Expression::Integer(n) => {
                // Direct assignment of a 64-bit integer literal.
                asm.push_str(&format!("    mov qword [{}], {}\n", address, n));
            }
            Expression::String(s) => {
                // Get string label, load address into RAX, then store RAX into variable address.
                let label = common.add_string_to_pool(s);
                asm.push_str(&format!("    mov rax, {}\n", label));
                asm.push_str(&format!("    mov [{}], rax\n", address));
            }
            Expression::Variable(src_name) => {
                // Copy value from another variable.
                let src_address = common
                    .get_variable_address(src_name)
                    .ok_or_else(|| CompileError::undefined_variable(src_name))?;
                asm.push_str(&format!("    mov rax, [{}]\n", src_address));
                asm.push_str(&format!("    mov [{}], rax\n", address));
            }
            _ => {
                // Evaluate a complex expression and store the result (from stack).
                let expr_asm = ExpressionGenerator::generate_expression(common, value)?;
                asm.push_str(&expr_asm);
                asm.push_str("    pop rax\n"); // Result is in RAX
                asm.push_str(&format!("    mov [{}], rax\n", address));
            }
        }
        Ok(asm)
    }

    /// Generates assembly for an array declaration.
    ///
    /// Note: This simplified implementation only reserves the first element's space in BSS.
    /// A full implementation would require memory allocation and element storage.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `name` - The array identifier.
    /// * `elements` - The initial element expressions.
    fn generate_array_declaration(
        common: &mut CodeGenCommon,
        name: &str,
        elements: &[Expression],
    ) -> Result<String, CompileError> {
        let mut asm = String::new();
        // Register array identifier. The `var_<name>` label will point to the first element's space.
        let address = common.register_variable(name, Type::Integer);

        // Simple initialization of the first element (for demonstration/basic use).
        if let Some(first_element) = elements.first() {
            match first_element {
                Expression::Integer(n) => {
                    asm.push_str(&format!("    mov qword [{}], {}\n", address, n));
                }
                Expression::String(s) => {
                    let label = common.add_string_to_pool(s);
                    asm.push_str(&format!("    mov rax, {}\n", label));
                    asm.push_str(&format!("    mov [{}], rax\n", address));
                }
                _ => {
                    // Complex expression initialization (e.g., array = [1+1, 2+2]) is not fully implemented here
                }
            }
        }
        Ok(asm)
    }

    /// Generates assembly for an assignment statement (simple or compound).
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `name` - The variable identifier being assigned to.
    /// * `value` - The expression for the right-hand side.
    /// * `operator` - The assignment operator (e.g., `*=` or `+=`).
    fn generate_assignment(
        common: &mut CodeGenCommon,
        name: &str,
        value: &Expression,
        operator: &AssignmentOperator,
    ) -> Result<String, CompileError> {
        let mut asm = String::new();
        let address = common
            .get_variable_address(name)
            .ok_or_else(|| CompileError::undefined_variable(name))?
            .clone();

        // 1. Load the current value of the variable (LHS) into RAX.
        asm.push_str(&format!("    mov rax, [{}]\n", address));

        // 2. Evaluate the RHS expression and push its result onto the stack.
        let expr_asm = ExpressionGenerator::generate_expression(common, value)?;
        asm.push_str(&expr_asm);

        // 3. Pop the RHS value into RBX.
        asm.push_str("    pop rbx\n");

        // 4. Perform the compound operation (RAX = RAX op RBX).
        match operator {
            AssignmentOperator::Multiply => {
                asm.push_str("    imul rax, rbx\n");
            }
            AssignmentOperator::Plus => {
                asm.push_str("    add rax, rbx\n");
            }
        }
        // 5. Store the final result back into the variable's memory location.
        asm.push_str(&format!("    mov [{}], rax\n", address));

        Ok(asm)
    }

    /// Delegates the generation of the `Print` statement to the dedicated `PrintGenerator`.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `parts` - The parts of the print statement (strings or expressions).
    fn generate_print_statement(
        common: &mut CodeGenCommon,
        parts: &[PrintPart],
    ) -> Result<String, CompileError> {
        PrintGenerator::generate_print(common, parts)
    }

    /// Generates assembly code for a count-controlled loop construct.
    ///
    /// The loop structure is: `for (variable = start; variable <= end; variable++)`.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `variable` - The loop variable identifier.
    /// * `start` - The starting expression for the loop range.
    /// * `end` - The ending expression for the loop range.
    /// * `body` - The statements inside the loop.
    fn generate_loop(
        common: &mut CodeGenCommon,
        variable: &str,
        start: &Expression,
        end: &Expression,
        body: &[Statement],
    ) -> Result<String, CompileError> {
        let mut asm = String::new();
        let loop_label = common.next_label();
        let end_label = common.next_label();
        // Register the loop variable.
        let address = common.register_variable(variable, Type::Integer);

        // --- 1. Loop Initialization (variable = start) ---
        match start {
            Expression::Integer(n) => {
                asm.push_str(&format!("    mov qword [{}], {}\n", address, n));
            }
            Expression::Variable(name) => {
                let src_address = common
                    .get_variable_address(name)
                    .ok_or_else(|| CompileError::undefined_variable(name))?;
                asm.push_str(&format!("    mov rax, [{}]\n", src_address));
                asm.push_str(&format!("    mov [{}], rax\n", address));
            }
            _ => {
                // Evaluate complex start expression.
                let expr_asm = ExpressionGenerator::generate_expression(common, start)?;
                asm.push_str(&expr_asm);
                asm.push_str("    pop rax\n");
                asm.push_str(&format!("    mov [{}], rax\n", address));
            }
        }

        // --- 2. Loop Condition Check ---
        asm.push_str(&format!("{}:\n", loop_label));
        asm.push_str(&format!("    mov rax, [{}]\n", address)); // Load loop variable (i)

        // Compare loop variable (RAX) with the end expression.
        match end {
            Expression::Integer(n) => {
                asm.push_str(&format!("    cmp rax, {}\n", n));
            }
            Expression::Variable(name) => {
                let src_address = common
                    .get_variable_address(name)
                    .ok_or_else(|| CompileError::undefined_variable(name))?;
                asm.push_str(&format!("    mov rbx, [{}]\n", src_address));
                asm.push_str("    cmp rax, rbx\n");
            }
            _ => {
                // Evaluate complex end expression.
                let expr_asm = ExpressionGenerator::generate_expression(common, end)?;
                asm.push_str(&expr_asm);
                asm.push_str("    pop rbx\n");
                asm.push_str("    cmp rax, rbx\n");
            }
        }

        // Jump Greater (jg) to end if loop variable is past the end limit.
        asm.push_str(&format!("    jg {}\n", end_label));

        // --- 3. Loop Body ---
        for stmt in body {
            let stmt_asm = Self::generate_statement(common, stmt)?;
            asm.push_str(&stmt_asm);
        }

        // --- 4. Loop Step (variable++) and Re-entry ---
        asm.push_str(&format!("    inc qword [{}]\n", address)); // Increment loop variable
        asm.push_str(&format!("    jmp {}\n", loop_label)); // Jump back to condition check
        asm.push_str(&format!("{}:\n", end_label)); // Loop termination label

        Ok(asm)
    }

    /// Generates assembly code for a conditional (`If` / `If-Else`) statement.
    ///
    /// # Arguments
    ///
    /// * `common` - The mutable code generation context.
    /// * `condition` - The boolean expression to evaluate.
    /// * `then_branch` - The statements executed if the condition is true.
    /// * `else_branch` - The optional statements executed if the condition is false.
    fn generate_conditional(
        common: &mut CodeGenCommon,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: &Option<Vec<Statement>>,
    ) -> Result<String, CompileError> {
        let mut asm = String::new();
        let else_label = common.next_label();
        let end_label = common.next_label();

        // 1. Evaluate the condition expression. Result (0 or 1) is pushed to stack.
        let cond_asm = ExpressionGenerator::generate_expression(common, condition)?;
        asm.push_str(&cond_asm);
        asm.push_str("    pop rax\n");
        // 2. Test RAX (condition result). RAX=0 (false) sets ZF.
        asm.push_str("    test rax, rax\n");

        // 3. Jump to the `else` block or `end` if the condition is false (ZF is set).
        if else_branch.is_some() {
            asm.push_str(&format!("    jz {}\n", else_label)); // Jump to ELSE
        } else {
            asm.push_str(&format!("    jz {}\n", end_label)); // Jump to END (skip THEN)
        }

        // 4. Generate 'Then' Branch
        for stmt in then_branch {
            let stmt_asm = Self::generate_statement(common, stmt)?;
            asm.push_str(&stmt_asm);
        }
        // Jump over the 'Else' block to the 'End' label.
        asm.push_str(&format!("    jmp {}\n", end_label));

        // 5. Generate 'Else' Branch (Optional)
        if let Some(else_branch) = else_branch {
            asm.push_str(&format!("{}:\n", else_label));
            for stmt in else_branch {
                let stmt_asm = Self::generate_statement(common, stmt)?;
                asm.push_str(&stmt_asm);
            }
        }

        // 6. End of Conditional
        asm.push_str(&format!("{}:\n", end_label));
        Ok(asm)
    }
}

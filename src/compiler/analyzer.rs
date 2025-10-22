//! # Semantic Analyzer
//!
//! This module defines the [`Analyzer`], which performs semantic analysis on the
//! Abstract Syntax Tree (AST). Its primary roles are **type checking** and
//! **symbol management** (tracking variable definitions) to ensure the program
//! is logically sound before code generation.

use crate::ast::nodes::*;
use crate::ast::types::Type;
use crate::compiler::error::CompileError;
use std::collections::HashMap;

/// The central structure for performing semantic analysis.
pub struct Analyzer {
    /// Symbol table: Maps variable names (`String`) to their declared [`Type`].
    symbols: HashMap<String, Type>,
    /// Accumulates all semantic errors found during the visit phase.
    errors: Vec<CompileError>,
}

impl Analyzer {
    /// Creates a new, empty analyzer instance.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// The main entry point for starting the analysis of a program.
    ///
    /// It consumes the AST and returns an error if any semantic problems are found.
    ///
    /// # Arguments
    ///
    /// * `ast` - The root [`Program`] AST node.
    pub fn analyze(ast: &Program) -> Result<(), CompileError> {
        let mut analyzer = Self::new();
        analyzer.visit_program(ast);

        if analyzer.errors.is_empty() {
            Ok(())
        } else {
            // Only return the first error found for simplicity.
            Err(analyzer.errors.remove(0))
        }
    }

    /// Recursively visits all statements in the program.
    fn visit_program(&mut self, program: &Program) {
        for statement in &program.statements {
            self.visit_statement(statement);
        }
    }

    /// Visits a single statement, performing type checks and updating the symbol table.
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VariableDeclaration { name, value } => {
                // 1. Determine the type of the value expression.
                let value_type = self.visit_expression(value);
                // 2. Register the variable with its inferred type in the symbol table.
                self.symbols.insert(name.clone(), value_type);
            }
            Statement::ArrayDeclaration { name, elements } => {
                // Check all element expressions (though a proper check would ensure all are the same type).
                for element in elements {
                    self.visit_expression(element);
                }
                // Array type is simplified to Integer/Pointer for the current code generation.
                self.symbols.insert(name.clone(), Type::Integer);
            }
            Statement::Print { parts } => {
                // Ensure all expression parts within the print statement are analyzed.
                for part in parts {
                    match part {
                        PrintPart::String(_) => {}
                        PrintPart::Expression(expr) => {
                            self.visit_expression(expr);
                        }
                    }
                }
            }
            Statement::Loop {
                variable,
                start,
                end,
                body,
            } => {
                // Check that loop bounds are compatible with Integer type.
                let start_type = self.visit_expression(start);
                let end_type = self.visit_expression(end);

                if !start_type.is_compatible_with(&Type::Integer) {
                    self.errors
                        .push(CompileError::r#type("Loop start must be integer"));
                }
                if !end_type.is_compatible_with(&Type::Integer) {
                    self.errors
                        .push(CompileError::r#type("Loop end must be integer"));
                }

                // Register loop variable (scoped to the loop body).
                self.symbols.insert(variable.clone(), Type::Integer);
                for stmt in body {
                    self.visit_statement(stmt);
                }
                // Remove variable after loop body traversal (basic scope management).
                self.symbols.remove(variable);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Check that the condition expression evaluates to a Boolean.
                let cond_type = self.visit_expression(condition);
                if !cond_type.is_compatible_with(&Type::Boolean) {
                    self.errors
                        .push(CompileError::r#type("If condition must be boolean"));
                }

                // Visit statement blocks recursively.
                for stmt in then_branch {
                    self.visit_statement(stmt);
                }

                if let Some(else_branch) = else_branch {
                    for stmt in else_branch {
                        self.visit_statement(stmt);
                    }
                }
            }
            Statement::Assignment {
                name,
                value,
                operator: _,
            } => {
                // 1. Analyze the assigned value's type.
                self.visit_expression(value);
                // 2. Check if the assigned variable exists.
                if !self.symbols.contains_key(name) {
                    self.errors.push(CompileError::undefined_variable(name));
                }
                // A full analyzer would also check if the variable's existing type is compatible with the new value's type.
            }
        }
    }

    /// Recursively visits an expression, validates compatibility, and returns its resultant type.
    ///
    /// # Arguments
    ///
    /// * `expression` - The expression AST node.
    ///
    /// # Returns
    ///
    /// The [`Type`] that the expression evaluates to.
    fn visit_expression(&mut self, expression: &Expression) -> Type {
        match expression {
            Expression::Integer(_) => Type::Integer,
            Expression::String(_) => Type::String,
            Expression::Boolean(_) => Type::Boolean,
            Expression::Variable(name) => {
                // Look up the variable type in the symbol table.
                self.symbols.get(name).cloned().unwrap_or_else(|| {
                    // Report an error if the variable is undefined.
                    self.errors.push(CompileError::undefined_variable(name));
                    Type::Unknown
                })
            }
            Expression::ArrayAccess { array, index } => {
                // Ensure the index expression is checked.
                self.visit_expression(index);
                // Check if the array variable exists.
                if !self.symbols.contains_key(array) {
                    self.errors.push(CompileError::undefined_variable(array));
                }
                // Arrays are assumed to hold Integers for now.
                Type::Integer
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                // Recursively determine the types of both operands.
                let left_type = self.visit_expression(left);
                let right_type = self.visit_expression(right);

                // Check for type compatibility between operands.
                if !left_type.is_compatible_with(&right_type) {
                    self.errors.push(CompileError::type_mismatch(&format!(
                        "{:?} {:?} {:?}",
                        left_type, operator, right_type
                    )));
                }

                // Determine the resulting type of the binary operation.
                match operator {
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Less
                    | BinaryOperator::Greater
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual
                    | BinaryOperator::And
                    | BinaryOperator::Or => Type::Boolean, // Comparison/Logical operations yield a boolean
                    _ => left_type, // Arithmetic operations yield the operand type
                }
            }
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to call the analyzer.
pub fn analyze(ast: &Program) -> Result<(), CompileError> {
    Analyzer::analyze(ast)
}

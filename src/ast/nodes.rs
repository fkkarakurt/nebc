//! # AST Nodes
//!
//! This module defines the fundamental data structures (nodes) that make up
//! the Abstract Syntax Tree (AST) of the Nebulang language.

use std::vec::Vec; // Vec is used implicitly, but good practice to show context.

/// Represents the root of a Nebulang program's Abstract Syntax Tree (AST).
#[derive(Debug, Clone)]
pub struct Program {
    /// A vector of statements that constitute the entire program.
    pub statements: Vec<Statement>,
}

// -----------------------------------------------------------------------------

/// Defines the supported assignment operators in Nebulang.
#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    /// Compound multiplication assignment (e.g., `x *= y`).
    Multiply,
    /// Compound addition assignment (e.g., `x += y`).
    Plus,
}

// -----------------------------------------------------------------------------

/// Represents all possible statement types in the Nebulang language.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A declaration for a mutable variable.
    VariableDeclaration {
        /// The name (identifier) of the variable.
        name: String,
        /// The initial value assigned to the variable.
        value: Box<Expression>,
    },
    /// A declaration for an array variable.
    ArrayDeclaration {
        /// The name (identifier) of the array.
        name: String,
        /// A vector of initial elements in the array.
        elements: Vec<Expression>,
    },
    /// A statement for outputting data (e.g., printing to console).
    Print {
        /// The parts to be printed, which can be strings or expressions.
        parts: Vec<PrintPart>,
    },
    /// A loop construct (e.g., a count-controlled loop).
    Loop {
        /// The loop variable identifier.
        variable: String,
        /// The starting expression for the loop range.
        start: Box<Expression>,
        /// The ending expression for the loop range (exclusive or inclusive, defined by semantics).
        end: Box<Expression>,
        /// The list of statements within the loop body.
        body: Vec<Statement>,
    },
    /// A conditional execution construct.
    If {
        /// The condition expression that determines execution.
        condition: Box<Expression>,
        /// The list of statements executed if the condition is true.
        then_branch: Vec<Statement>,
        /// Optional list of statements executed if the condition is false.
        else_branch: Option<Vec<Statement>>,
    },
    /// An assignment operation to update the value of an existing variable.
    Assignment {
        /// The name of the variable being assigned to.
        name: String,
        /// The new value expression.
        value: Box<Expression>,
        /// The specific assignment operator used (e.g., simple or compound).
        operator: AssignmentOperator,
    },
}

// -----------------------------------------------------------------------------

/// Represents all possible expression types in the Nebulang language.
#[derive(Debug, Clone)]
pub enum Expression {
    /// A literal integer value.
    Integer(i64),
    /// A literal string value.
    String(String),
    /// A literal boolean value (`true` or `false`).
    Boolean(bool),
    /// A reference to a variable by its identifier.
    Variable(String),
    /// Accessing an element within an array using an index.
    ArrayAccess {
        /// The name of the array being accessed.
        array: String,
        /// The index expression used to select the element.
        index: Box<Expression>,
    },
    /// A binary operation involving two operands and an operator.
    Binary {
        /// The expression on the left-hand side of the operator.
        left: Box<Expression>,
        /// The binary operator (e.g., Add, Multiply, Equal).
        operator: BinaryOperator,
        /// The expression on the right-hand side of the operator.
        right: Box<Expression>,
    },
}

// -----------------------------------------------------------------------------

/// Represents a single part within a `Print` statement.
#[derive(Debug, Clone)]
pub enum PrintPart {
    /// A literal string segment to be printed.
    String(String),
    /// An expression whose resulting value is to be printed.
    Expression(Box<Expression>),
}

// -----------------------------------------------------------------------------

/// Defines all supported binary operators in Nebulang.
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    /// Addition operator (`+`).
    Add,
    /// Subtraction operator (`-`).
    Subtract,
    /// Multiplication operator (`*`).
    Multiply,
    /// Division operator (`/`).
    Divide,
    /// Modulo (remainder) operator (`%`).
    Modulo,
    /// Exponentiation operator (`^` or similar).
    Power,
    /// Equality comparison operator (`==`).
    Equal,
    /// Inequality comparison operator (`!=`).
    NotEqual,
    /// Less than comparison operator (`<`).
    Less,
    /// Greater than comparison operator (`>`).
    Greater,
    /// Less than or equal to comparison operator (`<=`).
    LessEqual,
    /// Greater than or equal to comparison operator (`>=`).
    GreaterEqual,
    /// Logical AND operator (`AND`).
    And,
    /// Logical OR operator (`OR`).
    Or,
}

//! # Expression Parser
//!
//! This module defines the [`ExpressionParser`], which implements the logic for parsing
//! expressions in Nebulang. It primarily uses the **Pratt Parsing** (or Operator-Precedence)
//! technique for handling binary operations and respecting operator precedence and associativity.

use super::common::Parser;
use crate::ast::nodes::{BinaryOperator, Expression};
use crate::compiler::error::CompileError;
use crate::compiler::lexer::Token;

/// A static utility struct dedicated to parsing expressions and building expression AST nodes.
pub struct ExpressionParser;

impl ExpressionParser {
    /// The entry point for parsing any expression.
    ///
    /// It delegates to `parse_binary_expression` with a starting precedence of 0.
    ///
    /// # Arguments
    ///
    /// * `parser` - The mutable parser instance.
    ///
    /// # Returns
    ///
    /// The root [`Expression`] node.
    pub fn parse_expression(parser: &mut Parser) -> Result<Expression, CompileError> {
        Self::parse_binary_expression(parser, 0)
    }

    /// Parses binary expressions using the operator-precedence climbing algorithm (Pratt Parsing).
    ///
    /// This function iteratively checks the precedence of the current operator against the
    /// minimum required precedence, handling left-associativity naturally.
    ///
    /// # Arguments
    ///
    /// * `parser` - The mutable parser instance.
    /// * `precedence` - The minimum precedence level the operator must meet to be consumed.
    ///
    /// # Returns
    ///
    /// The resulting [`Expression`] tree (e.g., `A + B * C`).
    fn parse_binary_expression(
        parser: &mut Parser,
        precedence: u8,
    ) -> Result<Expression, CompileError> {
        // Start with the left-most primary expression (e.g., a literal, variable, or grouped expression).
        let mut left = Self::parse_primary(parser)?;

        // Loop as long as we find an operator with sufficient precedence.
        while let Some(operator) = Self::parse_operator(parser) {
            let op_precedence = Self::get_precedence(&operator);

            // Stop if the current operator's precedence is lower than the required minimum.
            if op_precedence < precedence {
                break;
            }

            // Consume the operator token.
            parser.advance();

            // Recursively parse the right-hand side with a higher precedence level (op_precedence + 1)
            // to ensure correct binding for left-associative operators (e.g., A + B + C).
            let right = Self::parse_binary_expression(parser, op_precedence + 1)?;

            // Combine the current expression and the newly parsed right expression.
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Maps the current token to a [`BinaryOperator`] if a match is found.
    ///
    /// # Arguments
    ///
    /// * `parser` - The immutable parser instance for peeking.
    ///
    /// # Returns
    ///
    /// An `Option` containing the corresponding [`BinaryOperator`].
    fn parse_operator(parser: &mut Parser) -> Option<BinaryOperator> {
        match parser.peek().0 {
            Token::Plus => Some(BinaryOperator::Add),
            Token::Minus => Some(BinaryOperator::Subtract),
            Token::Multiply => Some(BinaryOperator::Multiply),
            Token::Caret => Some(BinaryOperator::Power),
            Token::Divide => Some(BinaryOperator::Divide),
            Token::Modulo => Some(BinaryOperator::Modulo),
            Token::Equal => Some(BinaryOperator::Equal),
            Token::NotEqual => Some(BinaryOperator::NotEqual),
            Token::Less => Some(BinaryOperator::Less),
            Token::Greater => Some(BinaryOperator::Greater),
            Token::LessEqual => Some(BinaryOperator::LessEqual),
            Token::GreaterEqual => Some(BinaryOperator::GreaterEqual),
            Token::And => Some(BinaryOperator::And),
            Token::Or => Some(BinaryOperator::Or),
            _ => None,
        }
    }

    /// Returns the precedence level for a given binary operator. Higher numbers mean tighter binding.
    ///
    /// The levels are designed to follow standard mathematical and logical precedence rules.
    ///
    /// # Arguments
    ///
    /// * `operator` - The binary operator.
    ///
    /// # Returns
    ///
    /// A `u8` representing the operator's precedence level.
    fn get_precedence(operator: &BinaryOperator) -> u8 {
        match operator {
            BinaryOperator::Power => 7,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 6,
            BinaryOperator::Add | BinaryOperator::Subtract => 5,
            BinaryOperator::Less
            | BinaryOperator::Greater
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual => 4,
            BinaryOperator::Equal | BinaryOperator::NotEqual => 3,
            BinaryOperator::And => 2,
            BinaryOperator::Or => 1,
        }
    }

    /// Parses the most basic, non-binary components of an expression (literals, variables, groups).
    ///
    /// This also handles implicit unary operations like negation (`-`).
    ///
    /// # Arguments
    ///
    /// * `parser` - The mutable parser instance.
    ///
    /// # Returns
    ///
    /// A simple [`Expression`] node.
    fn parse_primary(parser: &mut Parser) -> Result<Expression, CompileError> {
        match &parser.peek().0 {
            // Unary Minus: Treat as multiplication by -1.
            Token::Minus => {
                parser.advance();
                let expr = Self::parse_primary(parser)?;
                // Rewrites `-X` as `(-1 * X)`
                Ok(Expression::Binary {
                    left: Box::new(Expression::Integer(-1)),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(expr),
                })
            }
            // Unary Caret (^): Placeholder logic (often used for unary negation/bitwise complement,
            // but here it seems to be incorrectly used or a placeholder for a specific language feature).
            // NOTE: The current implementation has a placeholder Left-hand side (Integer 0) for a BinaryOperator::Power.
            Token::Caret => {
                parser.advance();
                let expr = Self::parse_primary(parser)?;
                Ok(Expression::Binary {
                    left: Box::new(Expression::Integer(0)), // Placeholder or error-prone logic
                    operator: BinaryOperator::Power,
                    right: Box::new(expr),
                })
            }
            // Literal Integers
            Token::Integer(n) => {
                let value = *n;
                parser.advance();
                Ok(Expression::Integer(value))
            }
            // Literal Strings
            Token::StringLiteral(s) => {
                let value = s.clone();
                parser.advance();
                Ok(Expression::String(value))
            }
            // Literal Booleans
            Token::Boolean(b) => {
                let value = *b;
                parser.advance();
                Ok(Expression::Boolean(value))
            }
            // Identifiers (Variables or Array Access)
            Token::Identifier(name) => {
                let name = name.clone();
                parser.advance();

                // Check for array access syntax (e.g., array_name{index})
                if parser.check(Token::BraceOpen) {
                    parser.advance();
                    // Array access index is treated as an expression
                    let index_expr = Self::parse_expression(parser)?;
                    parser.expect(Token::BraceClose)?;

                    // NOTE: The current implementation rewrites array access `array{index}` as a binary expression `array + index`.
                    // This is incorrect for typical array access which should return `Expression::ArrayAccess`.
                    // It's assumed to be a temporary language design choice or a bug.
                    Ok(Expression::Binary {
                        left: Box::new(Expression::Variable(name)),
                        operator: BinaryOperator::Add,
                        right: Box::new(index_expr),
                    })
                } else {
                    // Simple variable access
                    Ok(Expression::Variable(name))
                }
            }
            // Grouping with curly braces (BraceOpen/BraceClose)
            Token::BraceOpen => {
                parser.advance();
                let expr = Self::parse_expression(parser)?;
                parser.expect(Token::BraceClose)?;
                Ok(expr)
            }
            // Grouping with parentheses (ParenOpen/ParenClose)
            Token::ParenOpen => {
                parser.advance();
                let expr = Self::parse_expression(parser)?;
                parser.expect(Token::ParenClose)?;
                Ok(expr)
            }
            // Error case: Found a token that does not start an expression.
            _ => {
                let token = parser.peek().0.clone();
                Err(CompileError::parser(format!(
                    "Expected expression, found {:?}",
                    token
                )))
            }
        }
    }
}

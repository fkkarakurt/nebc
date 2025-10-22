//! # Statement Parser
//!
//! This module defines the [`StatementParser`], which is responsible for parsing
//! high-level language constructs such as variable declarations, assignments,
//! control flow statements (`if`, `loop`), and the output statement (`print`).
//!
//! It handles token consumption and delegates expression parsing to the [`ExpressionParser`].

use super::common::Parser;
use super::expression_parser::ExpressionParser;
use crate::ast::nodes::*;
use crate::compiler::error::CompileError;
use crate::compiler::lexer::Token;

/// A static utility struct dedicated to parsing statements and building statement AST nodes.
pub struct StatementParser;

impl StatementParser {
    /// Attempts to parse a sequence of statements until the end of the file or a new control flow block.
    ///
    /// It handles consuming intermediate newline tokens to allow for multi-line code.
    ///
    /// # Arguments
    ///
    /// * `parser` - The mutable parser instance.
    ///
    /// # Returns
    ///
    /// An `Option` containing a vector of parsed [`Statement`] nodes, or `None` if no statements were found.
    pub fn parse_statements(parser: &mut Parser) -> Result<Option<Vec<Statement>>, CompileError> {
        let mut statements = Vec::new();

        while !parser.is_at_end() {
            // Consume leading newlines/whitespace equivalents
            while parser.check(Token::Newline) {
                parser.advance();
            }

            if parser.is_at_end() {
                break;
            }

            // Attempt to parse a single statement.
            if let Some(statement) = Self::parse_statement(parser)? {
                statements.push(statement);
            } else {
                // If a token was not recognized as a statement, consume it and continue.
                parser.advance();
            }

            // Consume trailing newlines/whitespace equivalents
            while parser.check(Token::Newline) {
                parser.advance();
            }
        }

        if statements.is_empty() {
            Ok(None)
        } else {
            Ok(Some(statements))
        }
    }

    /// Tries to parse the next token as the start of a specific statement type.
    ///
    /// # Arguments
    ///
    /// * `parser` - The mutable parser instance.
    ///
    /// # Returns
    ///
    /// An `Option` containing the parsed [`Statement`] node, or `None` if the token does not start a known statement.
    pub fn parse_statement(parser: &mut Parser) -> Result<Option<Statement>, CompileError> {
        match parser.peek().0 {
            Token::Identifier(_) => Self::parse_variable_or_assignment(parser),
            Token::Print => Self::parse_print_statement(parser),
            Token::Loop => Self::parse_loop_statement(parser),
            Token::If => Self::parse_if_statement(parser),
            _ => Ok(None),
        }
    }

    /// Parses a statement starting with an `Identifier`, which could be a variable declaration
    /// (e.g., `x 10`) or a compound assignment (e.g., `x += 5`).
    fn parse_variable_or_assignment(
        parser: &mut Parser,
    ) -> Result<Option<Statement>, CompileError> {
        let name = parser.get_identifier();
        parser.advance(); // Consume the identifier

        // Check for array declaration syntax (e.g., `array_name [ ... ]`)
        if parser.check(Token::BracketOpen) {
            return Self::parse_array_declaration(parser, name);
        }

        // Check for compound assignment operators
        if parser.check(Token::MultiplyAssign) {
            parser.advance();
            let value = ExpressionParser::parse_expression(parser)?;
            Ok(Some(Statement::Assignment {
                name,
                value: Box::new(value),
                operator: AssignmentOperator::Multiply,
            }))
        } else if parser.check(Token::PlusAssign) {
            parser.advance();
            let value = ExpressionParser::parse_expression(parser)?;
            Ok(Some(Statement::Assignment {
                name,
                value: Box::new(value),
                operator: AssignmentOperator::Plus,
            }))
        } else {
            // Assume simple variable declaration/re-assignment if no operator is found.
            // The value must be an expression immediately following the identifier.
            let value = ExpressionParser::parse_expression(parser)?;
            Ok(Some(Statement::VariableDeclaration {
                name,
                value: Box::new(value),
            }))
        }
    }

    /// Parses an array declaration statement (e.g., `list [ 1, "a", 3 ]`).
    fn parse_array_declaration(
        parser: &mut Parser,
        name: String,
    ) -> Result<Option<Statement>, CompileError> {
        parser.advance(); // Consume BracketOpen '['

        let mut elements = Vec::new();

        while !parser.check(Token::BracketClose) && !parser.is_at_end() {
            // Simplified logic to parse elements, primarily looking for literals (string, integer).
            match &parser.peek().0 {
                Token::Identifier(ident) if ident == "as" => {
                    // Handle 'as' keyword (for potential type aliasing, currently skipped)
                    parser.advance();
                    let _alias = parser.get_identifier();
                    parser.advance();
                }
                Token::Identifier(ident) => {
                    // Treat bare identifiers inside array as string literals (simplification)
                    let value = ident.clone();
                    parser.advance();
                    elements.push(Expression::String(value));
                }
                Token::StringLiteral(s) => {
                    let value = s.clone();
                    parser.advance();
                    elements.push(Expression::String(value));
                }
                Token::Integer(n) => {
                    let value = *n;
                    parser.advance();
                    elements.push(Expression::Integer(value));
                }
                _ => {
                    // Skip unrecognized tokens inside the array
                    parser.advance();
                }
            }

            if parser.check(Token::Comma) {
                parser.advance();
            }
        }

        parser.expect(Token::BracketClose)?;

        Ok(Some(Statement::ArrayDeclaration { name, elements }))
    }

    /// Parses the `print` statement, which can contain string literals, booleans, and interpolated expressions.
    fn parse_print_statement(parser: &mut Parser) -> Result<Option<Statement>, CompileError> {
        parser.advance(); // Consume 'print' token
        let mut parts = Vec::new();

        while !parser.is_at_end() && !matches!(parser.peek().0, Token::Newline) {
            match &parser.peek().0 {
                Token::StringLiteral(s) => {
                    // Handle string literals and check for interpolation (e.g., "Hello {name}!")
                    let interpolation_parts = Self::parse_string_interpolation(s);
                    parts.extend(interpolation_parts);
                    parser.advance();
                }
                Token::Boolean(b) => {
                    // Direct boolean literal output
                    let value = *b;
                    parser.advance();
                    parts.push(PrintPart::Expression(Box::new(Expression::Boolean(value))));
                }
                Token::BraceOpen => {
                    // Explicit expression to print (e.g., `print {a + b}`)
                    parser.advance(); // Consume '{'
                    let expr = ExpressionParser::parse_expression(parser)?;
                    parts.push(PrintPart::Expression(Box::new(expr)));
                    parser.expect(Token::BraceClose)?; // Expect '}'
                }
                _ => break, // End of the print statement parts
            }
        }

        Ok(Some(Statement::Print { parts }))
    }

    /// Splits a string literal based on interpolation markers (`{...}`) and recursively
    /// attempts to parse the content inside the markers as expressions.
    fn parse_string_interpolation(s: &str) -> Vec<PrintPart> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // End of the static string part
                if !current_text.is_empty() {
                    parts.push(PrintPart::String(current_text));
                    current_text = String::new();
                }

                let mut expr_content = String::new();
                let mut brace_count = 1;

                // Consume characters until the matching '}' is found
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '{' {
                        brace_count += 1;
                        expr_content.push(chars.next().unwrap());
                    } else if next_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            chars.next(); // Consume the final '}'
                            break;
                        } else {
                            expr_content.push(chars.next().unwrap());
                        }
                    } else {
                        expr_content.push(chars.next().unwrap());
                    }
                }

                // Attempt to parse the extracted content as an expression
                if !expr_content.trim().is_empty() {
                    match Self::parse_interpolation_expression(&expr_content) {
                        Ok(expr) => {
                            parts.push(PrintPart::Expression(Box::new(expr)));
                        }
                        Err(_) => {
                            // On failure, treat the full original {content} as a literal string
                            parts.push(PrintPart::String(format!("{{{}}}", expr_content)));
                        }
                    }
                }
            } else {
                current_text.push(ch);
            }
        }

        // Add any remaining static text
        if !current_text.is_empty() {
            parts.push(PrintPart::String(current_text));
        }
        parts
    }

    /// Utility function to tokenize and parse a string slice as a standalone expression.
    ///
    /// This is necessary because interpolation content must be re-lexed and re-parsed.
    fn parse_interpolation_expression(expr_str: &str) -> Result<Expression, CompileError> {
        use crate::compiler::lexer::tokenize;
        use crate::compiler::parser::common::Parser;
        use crate::compiler::parser::expression_parser::ExpressionParser;

        let tokens = match tokenize(expr_str) {
            Ok(tokens) => tokens,
            Err(_) => return Err(CompileError::parser("Failed to tokenize expression in interpolation")),
        };

        // Filter out structural tokens (like Newline, Indent, Dedent) which aren't valid inside an expression
        let filtered_tokens: Vec<_> = tokens
            .into_iter()
            .filter(|(token, _, _, _)| {
                !matches!(token, Token::Newline | Token::Indent | Token::Dedent)
            })
            .collect();

        if filtered_tokens.is_empty() {
            return Err(CompileError::parser("Empty expression in interpolation"));
        }

        let mut parser = Parser::new(filtered_tokens);
        ExpressionParser::parse_expression(&mut parser)
    }

    /// Parses the `loop` statement (e.g., `loop i, 1..10: ...`).
    ///
    /// This assumes an inclusive range loop structure.
    fn parse_loop_statement(parser: &mut Parser) -> Result<Option<Statement>, CompileError> {
        parser.advance(); // Consume 'loop' token
        let variable = parser.get_identifier();
        parser.advance(); // Consume loop variable identifier
        parser.expect(Token::Comma)?; // Expect ','

        let start = ExpressionParser::parse_expression(parser)?;
        parser.expect(Token::Range)?; // Expect '..'
        let end = ExpressionParser::parse_expression(parser)?;

        // Consume any newlines before the block
        while parser.check(Token::Newline) {
            parser.advance();
        }

        // Parse indented loop body
        let mut body = Vec::new();
        if parser.check(Token::Indent) {
            parser.advance(); // Consume 'Indent'
            while !parser.check(Token::Dedent) && !parser.is_at_end() {
                if let Some(statement) = Self::parse_statement(parser)? {
                    body.push(statement);
                } else {
                    parser.advance();
                }
            }
            if parser.check(Token::Dedent) {
                parser.advance(); // Consume 'Dedent'
            }
        }

        Ok(Some(Statement::Loop {
            variable,
            start: Box::new(start),
            end: Box::new(end),
            body,
        }))
    }

    /// Parses the `if` and `if-else` conditional statements, handling block structure via indentation.
    fn parse_if_statement(parser: &mut Parser) -> Result<Option<Statement>, CompileError> {
        parser.advance(); // Consume 'if' token
        let condition = ExpressionParser::parse_expression(parser)?;

        // Consume any newlines before the 'then' block
        while parser.check(Token::Newline) {
            parser.advance();
        }

        // Parse 'Then' Branch (Indented Block)
        let mut then_branch = Vec::new();
        if parser.check(Token::Indent) {
            parser.advance();
            while !parser.check(Token::Dedent) && !parser.is_at_end() {
                if let Some(statement) = Self::parse_statement(parser)? {
                    then_branch.push(statement);
                } else {
                    parser.advance();
                }
            }
            if parser.check(Token::Dedent) {
                parser.advance();
            }
        }

        // Parse 'Else' Branch (Optional)
        let else_branch = if parser.check(Token::Else) {
            parser.advance(); // Consume 'else' token

            // Consume newlines before the 'else' block
            while parser.check(Token::Newline) {
                parser.advance();
            }

            let mut else_statements = Vec::new();
            if parser.check(Token::Indent) {
                parser.advance();
                while !parser.check(Token::Dedent) && !parser.is_at_end() {
                    if let Some(statement) = Self::parse_statement(parser)? {
                        else_statements.push(statement);
                    } else {
                        parser.advance();
                    }
                }
                if parser.check(Token::Dedent) {
                    parser.advance();
                }
            }
            Some(else_statements)
        } else {
            None
        };

        Ok(Some(Statement::If {
            condition: Box::new(condition),
            then_branch,
            else_branch,
        }))
    }
}
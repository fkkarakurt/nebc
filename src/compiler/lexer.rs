use crate::compiler::error::CompileError;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Literals
    Identifier(String),
    StringLiteral(String),
    Integer(i64),
    Boolean(bool),

    // Keywords
    Print,
    Loop,
    If,
    Else,
    True,
    False,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Caret,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,

    // Assignment
    MultiplyAssign,
    PlusAssign,

    // Punctuation
    BraceOpen,
    BraceClose,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    Range,
    Newline,

    // Indentation
    Indent,
    Dedent,
}

pub fn tokenize(source: &str) -> Result<Vec<(Token, usize, usize, String)>, CompileError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut position = 0;
    let mut indentation_stack: Vec<usize> = vec![0];

    while let Some(&ch) = chars.peek() {
        let start = position;

        match ch {
            ' ' => {
                let mut _space_count = 0;
                while let Some(' ') = chars.peek() {
                    chars.next();
                    position += 1;
                    _space_count += 1;
                }
            }
            '\t' => {
                // TAB karakteri
                chars.next();
                position += 1;
            }
            '\n' => {
                chars.next();
                position += 1;
                tokens.push((Token::Newline, start, position, "\n".to_string()));

                // Yeni satır - girinti hesapla
                let mut indent_level = 0;
                let line_start = position;

                while let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        ' ' => {
                            chars.next();
                            position += 1;
                            // Her 4 space = 1 girinti seviyesi
                            if (position - line_start) % 4 == 0 {
                                indent_level += 1;
                            }
                        }
                        '\t' => {
                            chars.next();
                            position += 1;
                            indent_level += 1;
                        }
                        _ => break,
                    }
                }

                let current_indent = *indentation_stack.last().unwrap();

                if indent_level > current_indent {
                    tokens.push((Token::Indent, line_start, position, "indent".to_string()));
                    indentation_stack.push(indent_level);
                } else if indent_level < current_indent {
                    while let Some(&stack_indent) = indentation_stack.last() {
                        if stack_indent > indent_level {
                            tokens.push((
                                Token::Dedent,
                                line_start,
                                position,
                                "dedent".to_string(),
                            ));
                            indentation_stack.pop();
                        } else {
                            break;
                        }
                    }
                }
            }
            '\r' => {
                chars.next();
                position += 1;
            }
            '"' => {
                chars.next();
                position += 1;

                let mut string_content = String::new();
                while let Some(ch) = chars.next() {
                    position += ch.len_utf8();
                    if ch == '"' {
                        break;
                    }
                    if ch == '\n' {
                        return Err(CompileError::lexer("Unterminated string"));
                    }
                    string_content.push(ch);
                }

                tokens.push((
                    Token::StringLiteral(string_content),
                    start,
                    position,
                    source[start..position].to_string(),
                ));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                        position += ch.len_utf8();
                    } else {
                        break;
                    }
                }

                let token = match ident.as_str() {
                    "OR" => Token::Or,
                    "AND" => Token::And,
                    "TRUE" => Token::Boolean(true),
                    "FALSE" => Token::Boolean(false),
                    _ => Token::Identifier(ident),
                };
                tokens.push((token, start, position, source[start..position].to_string()));
            }
            '0'..='9' => {
                let mut num_str = String::new();
                let is_negative = if position > 0 {
                    let prev_token = if tokens.is_empty() {
                        None
                    } else {
                        tokens.last()
                    };
                    matches!(prev_token, Some((Token::Minus, _, _, _)))
                } else {
                    false
                };

                if is_negative {
                    tokens.pop();
                    num_str.push('-');
                }

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(ch);
                        chars.next();
                        position += ch.len_utf8();
                    } else {
                        break;
                    }
                }

                match num_str.parse() {
                    Ok(n) => tokens.push((Token::Integer(n), start, position, num_str)),
                    Err(_) => {
                        return Err(CompileError::lexer(&format!(
                            "Invalid integer: {}",
                            num_str
                        )));
                    }
                }
            }
            '[' => {
                chars.next();
                position += 1;
                tokens.push((Token::BracketOpen, start, position, "[".to_string()));
            }
            ']' => {
                chars.next();
                position += 1;
                tokens.push((Token::BracketClose, start, position, "]".to_string()));
            }
            ':' => {
                chars.next();
                position += 1;
                tokens.push((Token::Colon, start, position, ":".to_string()));
            }
            '!' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'?') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::Else, start, position, "!?".to_string()));
                } else {
                    tokens.push((Token::Print, start, position, "!".to_string()));
                }
            }
            '?' => {
                chars.next();
                position += 1;
                tokens.push((Token::If, start, position, "?".to_string()));
            }
            '@' => {
                chars.next();
                position += 1;
                tokens.push((Token::Loop, start, position, "@".to_string()));
            }
            '>' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'|') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::Newline, start, position, ">|".to_string()));
                } else if chars.peek() == Some(&'=') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::GreaterEqual, start, position, ">=".to_string()));
                } else {
                    tokens.push((Token::Greater, start, position, ">".to_string()));
                }
            }
            '<' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'=') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::LessEqual, start, position, "<=".to_string()));
                } else {
                    tokens.push((Token::Less, start, position, "<".to_string()));
                }
            }
            '=' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'=') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::Equal, start, position, "==".to_string()));
                } else {
                    tokens.push((
                        Token::Identifier("=".to_string()),
                        start,
                        position,
                        "=".to_string(),
                    ));
                }
            }
            '+' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'=') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::PlusAssign, start, position, "+=".to_string()));
                } else {
                    tokens.push((Token::Plus, start, position, "+".to_string()));
                }
            }
            '-' => {
                chars.next();
                position += 1;
                tokens.push((Token::Minus, start, position, "-".to_string()));
            }
            '*' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'=') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::MultiplyAssign, start, position, "*=".to_string()));
                } else {
                    tokens.push((Token::Multiply, start, position, "*".to_string()));
                }
            }
            '/' => {
                chars.next();
                position += 1;
                tokens.push((Token::Divide, start, position, "/".to_string()));
            }
            '^' => {
                chars.next();
                position += 1;
                tokens.push((Token::Caret, start, position, "^".to_string()));
            }
            '.' => {
                chars.next();
                position += 1;
                if chars.peek() == Some(&'.') {
                    chars.next();
                    position += 1;
                    tokens.push((Token::Range, start, position, "..".to_string()));
                }
            }
            '{' => {
                chars.next();
                position += 1;
                tokens.push((Token::BraceOpen, start, position, "{".to_string()));
            }
            '}' => {
                chars.next();
                position += 1;
                tokens.push((Token::BraceClose, start, position, "}".to_string()));
            }
            '(' => {
                chars.next();
                position += 1;
                tokens.push((Token::ParenOpen, start, position, "(".to_string()));
            }
            ')' => {
                chars.next();
                position += 1;
                tokens.push((Token::ParenClose, start, position, ")".to_string()));
            }
            ',' => {
                chars.next();
                position += 1;
                tokens.push((Token::Comma, start, position, ",".to_string()));
            }
            '%' => {
                chars.next();
                position += 1;
                tokens.push((Token::Modulo, start, position, "%".to_string()));
            }
            _ => {
                chars.next();
                position += 1;
            }
        }
    }

    // Dosya sonunda kalan girintileri kapat
    while indentation_stack.len() > 1 {
        tokens.push((Token::Dedent, position, position, "dedent".to_string()));
        indentation_stack.pop();
    }

    Ok(tokens)
}

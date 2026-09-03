use crate::formula::error::FormulaError;

const MAX_FORMULA_LENGTH: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
    Bang,
    LParen,
    RParen,
    Comma,
    Colon,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, FormulaError> {
    if input.len() > MAX_FORMULA_LENGTH {
        return Err(FormulaError::Parse("formula too long".to_string()));
    }

    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '%' => {
                tokens.push(Token::Percent);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::NotEq);
                } else {
                    tokens.push(Token::Bang);
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::LtEq);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::GtEq);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::EqEq);
                } else {
                    return Err(FormulaError::Parse("unexpected '='".to_string()));
                }
            }
            '&' => {
                chars.next();
                if chars.peek() == Some(&'&') {
                    chars.next();
                    tokens.push(Token::AndAnd);
                } else {
                    return Err(FormulaError::Parse("expected '&&'".to_string()));
                }
            }
            '|' => {
                chars.next();
                if chars.peek() == Some(&'|') {
                    chars.next();
                    tokens.push(Token::OrOr);
                } else {
                    return Err(FormulaError::Parse("expected '||'".to_string()));
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('"') => s.push('"'),
                            Some('\\') => s.push('\\'),
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('r') => s.push('\r'),
                            Some(other) => {
                                return Err(FormulaError::Parse(format!(
                                    "invalid escape sequence: \\{}",
                                    other
                                )));
                            }
                            None => {
                                return Err(FormulaError::Parse(
                                    "unterminated string literal".to_string(),
                                ));
                            }
                        },
                        Some(c) => s.push(c),
                        None => {
                            return Err(FormulaError::Parse(
                                "unterminated string literal".to_string(),
                            ));
                        }
                    }
                }
                tokens.push(Token::String(s));
            }
            c if c.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(c);
                        chars.next();
                    } else if c == 'e' || c == 'E' {
                        num_str.push(c);
                        chars.next();
                        if let Some(&sign) = chars.peek()
                            && (sign == '+' || sign == '-')
                        {
                            num_str.push(sign);
                            chars.next();
                        }
                        let mut has_digits = false;
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() {
                                num_str.push(c);
                                chars.next();
                                has_digits = true;
                            } else {
                                break;
                            }
                        }
                        if !has_digits {
                            return Err(FormulaError::Parse("invalid number exponent".to_string()));
                        }
                    } else {
                        break;
                    }
                }
                if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
                    let val: f64 = num_str
                        .parse()
                        .map_err(|_| FormulaError::Parse("invalid float".to_string()))?;
                    tokens.push(Token::Float(val));
                } else {
                    let val: i64 = num_str
                        .parse()
                        .map_err(|_| FormulaError::Parse("invalid integer".to_string()))?;
                    tokens.push(Token::Integer(val));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' || c == '$' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if ident.eq_ignore_ascii_case("true") {
                    tokens.push(Token::Boolean(true));
                } else if ident.eq_ignore_ascii_case("false") {
                    tokens.push(Token::Boolean(false));
                } else if ident.eq_ignore_ascii_case("null") {
                    tokens.push(Token::Null);
                } else {
                    tokens.push(Token::Identifier(ident));
                }
            }
            _ => {
                return Err(FormulaError::Parse(format!("unexpected character: {}", c)));
            }
        }
    }

    Ok(tokens)
}

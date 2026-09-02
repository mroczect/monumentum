use crate::coordinates::{CellRange, CellRef, parse_cell_ref};
use crate::formula::ast::{BinaryOp, Expr, UnaryOp};
use crate::formula::error::FormulaError;
use crate::formula::lexer::Token;
use monumentum_db::core::value::Value;

const MAX_PARSE_DEPTH: usize = 128;

pub fn parse(tokens: &[Token]) -> Result<Expr, FormulaError> {
    let mut parser = Parser {
        tokens,
        pos: 0,
        depth: 0,
    };
    let expr = parser.parse_expr()?;
    if parser.peek().is_some() {
        return Err(FormulaError::Parse(
            "unexpected token after end of expression".to_string(),
        ));
    }
    Ok(expr)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, token: Token) -> Result<(), FormulaError> {
        if self.peek() == Some(&token) {
            self.pos += 1;
            Ok(())
        } else {
            Err(FormulaError::Parse(format!(
                "expected {:?}, found {:?}",
                token,
                self.peek()
            )))
        }
    }

    fn enter_depth(&mut self) -> Result<(), FormulaError> {
        self.depth += 1;
        if self.depth > MAX_PARSE_DEPTH {
            return Err(FormulaError::Parse(
                "expression too deeply nested".to_string(),
            ));
        }
        Ok(())
    }

    fn exit_depth(&mut self) {
        self.depth -= 1;
    }

    fn parse_expr(&mut self) -> Result<Expr, FormulaError> {
        self.enter_depth()?;
        let result = self.parse_or();
        self.exit_depth();
        result
    }

    fn parse_or(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_and()?;
        while let Some(token) = self.peek() {
            match token {
                Token::OrOr => {
                    self.next();
                    let rhs = self.parse_and()?;
                    expr = Expr::BinaryOp(BinaryOp::Or, Box::new(expr), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_equality()?;
        while let Some(token) = self.peek() {
            match token {
                Token::AndAnd => {
                    self.next();
                    let rhs = self.parse_equality()?;
                    expr = Expr::BinaryOp(BinaryOp::And, Box::new(expr), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_comparison()?;
        while let Some(token) = self.peek() {
            match token {
                Token::EqEq => {
                    self.next();
                    let rhs = self.parse_comparison()?;
                    expr = Expr::BinaryOp(BinaryOp::Eq, Box::new(expr), Box::new(rhs));
                }
                Token::NotEq => {
                    self.next();
                    let rhs = self.parse_comparison()?;
                    expr = Expr::BinaryOp(BinaryOp::NotEq, Box::new(expr), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_additive()?;
        while let Some(token) = self.peek() {
            match token {
                Token::Lt => {
                    self.next();
                    let rhs = self.parse_additive()?;
                    expr = Expr::BinaryOp(BinaryOp::Lt, Box::new(expr), Box::new(rhs));
                }
                Token::LtEq => {
                    self.next();
                    let rhs = self.parse_additive()?;
                    expr = Expr::BinaryOp(BinaryOp::Lte, Box::new(expr), Box::new(rhs));
                }
                Token::Gt => {
                    self.next();
                    let rhs = self.parse_additive()?;
                    expr = Expr::BinaryOp(BinaryOp::Gt, Box::new(expr), Box::new(rhs));
                }
                Token::GtEq => {
                    self.next();
                    let rhs = self.parse_additive()?;
                    expr = Expr::BinaryOp(BinaryOp::Gte, Box::new(expr), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_multiplicative()?;
        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.next();
                    let rhs = self.parse_multiplicative()?;
                    expr = Expr::BinaryOp(BinaryOp::Add, Box::new(expr), Box::new(rhs));
                }
                Token::Minus => {
                    self.next();
                    let rhs = self.parse_multiplicative()?;
                    expr = Expr::BinaryOp(BinaryOp::Sub, Box::new(expr), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_power()?;
        while let Some(token) = self.peek() {
            match token {
                Token::Star => {
                    self.next();
                    let rhs = self.parse_power()?;
                    expr = Expr::BinaryOp(BinaryOp::Mul, Box::new(expr), Box::new(rhs));
                }
                Token::Slash => {
                    self.next();
                    let rhs = self.parse_power()?;
                    expr = Expr::BinaryOp(BinaryOp::Div, Box::new(expr), Box::new(rhs));
                }
                Token::Percent => {
                    self.next();
                    let rhs = self.parse_power()?;
                    expr = Expr::BinaryOp(BinaryOp::Mod, Box::new(expr), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_unary()?;
        if let Some(Token::Caret) = self.peek() {
            self.next();
            let rhs = self.parse_power()?;
            expr = Expr::BinaryOp(BinaryOp::Pow, Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, FormulaError> {
        self.enter_depth()?;
        let result = match self.peek() {
            Some(Token::Minus) => {
                self.next();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(operand)))
            }
            Some(Token::Bang) => {
                self.next();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(operand)))
            }
            _ => self.parse_primary(),
        };
        self.exit_depth();
        result
    }

    fn parse_primary(&mut self) -> Result<Expr, FormulaError> {
        let token = self
            .next()
            .ok_or_else(|| FormulaError::Parse("unexpected end of expression".to_string()))?;

        match token {
            Token::Integer(i) => Ok(Expr::Literal(Value::Integer((*i).into()))),
            Token::Float(f) => {
                let value = monumentum_db::types::Float::try_new(*f)
                    .map_err(|e| FormulaError::Parse(e.to_string()))?;
                Ok(Expr::Literal(Value::Float(value)))
            }
            Token::String(s) => Ok(Expr::Literal(Value::Text(s.clone().into()))),
            Token::Boolean(b) => Ok(Expr::Literal(Value::Boolean(*b))),
            Token::Null => Ok(Expr::Literal(Value::Null)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::Identifier(ident) => {
                if let Some(Token::LParen) = self.peek() {
                    self.next();
                    let mut args = Vec::new();
                    if let Some(Token::RParen) = self.peek() {
                        self.next();
                    } else {
                        loop {
                            let arg = self.parse_expr()?;
                            args.push(arg);
                            match self.peek() {
                                Some(Token::Comma) => {
                                    self.next();
                                }
                                Some(Token::RParen) => {
                                    self.next();
                                    break;
                                }
                                _ => {
                                    return Err(FormulaError::Parse(
                                        "expected ',' or ')' in function arguments".to_string(),
                                    ));
                                }
                            }
                        }
                    }
                    return Ok(Expr::FunctionCall(ident.clone(), args));
                }

                let cell_ref = self.parse_cell_ref_from_ident(ident)?;

                if let Some(Token::Colon) = self.peek() {
                    self.next();
                    let end_ident = match self.next() {
                        Some(Token::Identifier(id)) => id,
                        _ => {
                            return Err(FormulaError::Parse(
                                "expected cell reference after ':'".to_string(),
                            ));
                        }
                    };
                    let mut end_ref = self.parse_cell_ref_from_ident(end_ident)?;

                    if let Some(start_sheet) = &cell_ref.sheet {
                        match &end_ref.sheet {
                            None => end_ref.sheet = Some(start_sheet.clone()),
                            Some(end_sheet) if end_sheet != start_sheet => {
                                return Err(FormulaError::Parse(
                                    "sheet mismatch in range".to_string(),
                                ));
                            }
                            _ => {}
                        }
                    } else if end_ref.sheet.is_some() {
                        return Err(FormulaError::Parse("sheet mismatch in range".to_string()));
                    }

                    let range = CellRange::try_new(cell_ref, end_ref)
                        .map_err(|e| FormulaError::Parse(e.to_string()))?;
                    Ok(Expr::Range(range))
                } else {
                    let cell_str = self.cell_ref_to_string(&cell_ref);
                    Ok(Expr::CellRef(cell_str))
                }
            }
            _ => Err(FormulaError::Parse(format!(
                "unexpected token: {:?}",
                token
            ))),
        }
    }

    fn parse_cell_ref_from_ident(&mut self, ident: &str) -> Result<CellRef, FormulaError> {
        if let Some(Token::Bang) = self.peek() {
            self.next();
            let sheet_ident = match self.next() {
                Some(Token::Identifier(id)) => id,
                _ => {
                    return Err(FormulaError::Parse(
                        "expected identifier after '!'".to_string(),
                    ));
                }
            };
            let full_ref = format!("{}!{}", ident, sheet_ident);
            parse_cell_ref(&full_ref).map_err(|e| FormulaError::Parse(e.to_string()))
        } else {
            parse_cell_ref(ident).map_err(|e| FormulaError::Parse(e.to_string()))
        }
    }

    fn cell_ref_to_string(&self, cell: &CellRef) -> String {
        cell.to_string()
    }
}

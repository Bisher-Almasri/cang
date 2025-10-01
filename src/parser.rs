use crate::{ResourceValidator, Token, TokenTypes, ValidationError, CoinType};

#[derive(Debug)]
pub enum Expr {
    Number(i64),
    Binary(Box<Expr>, TokenTypes, Box<Expr>),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn eat(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    pub fn parse_expr(&mut self) -> Expr {
        let mut node = self.parse_term();
        while let Some(tok) = self.peek() {
            match tok.token_type {
                TokenTypes::Plus | TokenTypes::Minus => {
                    let op = self.eat().unwrap().token_type;
                    let rhs = self.parse_term();
                    node = Expr::Binary(Box::new(node), op, Box::new(rhs));
                }
                _ => break,
            }
        }
        node
    }

    pub fn parse_term(&mut self) -> Expr {
        let mut node = self.parse_factor();
        while let Some(tok) = self.peek() {
            match tok.token_type {
                TokenTypes::Star | TokenTypes::Slash => {
                    let op = self.eat().unwrap().token_type;
                    let rhs = self.parse_factor();
                    node = Expr::Binary(Box::new(node), op, Box::new(rhs));
                }
                _ => break,
            }
        }
        node
    }

    pub fn parse_factor(&mut self) -> Expr {
        match self.eat() {
            Some(tok) if tok.token_type == TokenTypes::Number => {
                let n = tok.value.unwrap().parse::<i64>().unwrap();
                Expr::Number(n)
            }
            Some(tok) if tok.token_type == TokenTypes::LParen => {
                let expr = self.parse_expr();
                if self.eat().map(|t| t.token_type) != Some(TokenTypes::RParen) {
                    panic!("Expected closing paren");
                }
                expr
            }
            other => panic!("Unexpected token {:?}", other),
        }
    }
}

pub fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Binary(lhs, op, rhs) => {
            let lval = eval(lhs);
            let rval = eval(rhs);
            match op {
                TokenTypes::Plus => lval + rval,
                TokenTypes::Minus => lval - rval,
                TokenTypes::Star => lval * rval,
                TokenTypes::Slash => lval / rval,
                _ => panic!("Invalid operator: {:?}", op),
            }
        }
    }
}

/// Evaluates an expression with coin validation and spending
pub fn eval_with_validation(
    expr: &Expr,
    validator: &mut ResourceValidator,
) -> Result<i64, ValidationError> {
    // First validate that we have enough coins
    let costs = validator.validate_expression(expr)?;

    // Spend the required coins
    for cost in costs {
        match cost.coin_type {
            CoinType::Variable => {
                for _ in 0..cost.amt {
                    validator
                        .coin_manager_mut()
                        .spend_var_coin()
                        .map_err(ValidationError::CoinError)?;
                }
            }
            CoinType::Function => {
                for _ in 0..cost.amt {
                    validator
                        .coin_manager_mut()
                        .spend_func_coin()
                        .map_err(ValidationError::CoinError)?;
                }
            }
        }
    }

    // Now evaluate the expression
    Ok(eval(expr))
}

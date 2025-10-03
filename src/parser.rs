use std::collections::HashMap;

use crate::{CoinType, ResourceValidator, Token, TokenTypes, ValidationError};

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Binary(Box<Expr>, TokenTypes, Box<Expr>),
    Let(String, Box<Expr>), // ident, val
    FnDef(String, Vec<String>, Box<Expr>),
    FnCall(String, Vec<Expr>),
    Var(String),
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
            Some(tok) if tok.token_type == TokenTypes::Identifier => {
                let name = tok.value.unwrap();
                if let Some(next) = self.peek() {
                    if next.token_type == TokenTypes::LParen {
                        self.eat();
                        let mut args = Vec::new();
                        while let Some(tok) = self.peek() {
                            if tok.token_type == TokenTypes::RParen {
                                self.eat();
                                break;
                            } else {
                                args.push(self.parse_expr());
                                
                                if let Some(next_tok) = self.peek() {
                                    match next_tok.token_type {
                                        TokenTypes::Comma => {
                                            self.eat(); 
                                            continue;
                                        }
                                        TokenTypes::RParen => {
                                            continue;
                                        }
                                        _ => panic!("Expected ',' or ')' after function argument"),
                                    }
                                }
                            }
                        }
                        Expr::FnCall(name, args)
                    } else {
                        Expr::Var(name)
                    }
                } else {
                    Expr::Var(name)
                }
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

    pub fn parse_fn_def(&mut self) -> Expr {
        // fn
        self.eat();

        // function name
        let name = match self.eat() {
            Some(Token {
                token_type: TokenTypes::Identifier,
                value: Some(id),
                ..
            }) => id,
            _ => panic!("Expected identifier after 'fn'"),
        };

        // expect (
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::LParen,
                ..
            }) => {}
            _ => panic!("Expected '(' after function name"),
        };

        // params
        let mut params = Vec::new();
        while let Some(tok) = self.peek() {
            match tok.token_type {
                TokenTypes::Identifier => {
                    let id = self.eat().unwrap().value.unwrap();
                    params.push(id);
                    
                    if let Some(next_tok) = self.peek() {
                        match next_tok.token_type {
                            TokenTypes::Comma => {
                                self.eat(); 
                                continue;
                            }
                            TokenTypes::RParen => {
                                continue;
                            }
                            _ => panic!("Expected ',' or ')' after parameter"),
                        }
                    }
                }
                TokenTypes::RParen => {
                    self.eat();
                    break;
                }
                _ => panic!("Unexpected token in parameter list: {:?}", tok),
            }
        }

        // expect {
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::LCurly,
                ..
            }) => {}
            _ => panic!("Expected '{{' before function body"),
        };

        let body = self.parse_expr();

        // expect }
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::RCurly,
                ..
            }) => {}
            _ => panic!("Expected '}}' at end of function body"),
        }

        Expr::FnDef(name, params, Box::new(body))
    }

    fn parse_let(&mut self) -> Expr {
        self.eat();
        let ident = match self.eat() {
            Some(Token {
                token_type: TokenTypes::Identifier,
                value: Some(name),
                ..
            }) => name,
            _ => panic!("Expected identifier after 'let'"),
        };
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::Eq,
                ..
            }) => {}
            _ => panic!("Expected '=' after identifier in let"),
        }
        let expr = self.parse_expr();
        Expr::Let(ident, Box::new(expr))
    }

    pub fn parse_stmt(&mut self) -> Expr {
        match self.peek() {
            Some(Token {
                token_type: TokenTypes::Let,
                ..
            }) => self.parse_let(),
            Some(Token {
                token_type: TokenTypes::Fn,
                ..
            }) => self.parse_fn_def(),
            _ => self.parse_expr(),
        }
    }
}

pub fn eval(expr: &Expr, env: &mut HashMap<String, Expr>) -> i64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Binary(lhs, op, rhs) => {
            let lval = eval(lhs, env);
            let rval = eval(rhs, env);
            match op {
                TokenTypes::Plus => lval + rval,
                TokenTypes::Minus => lval - rval,
                TokenTypes::Star => lval * rval,
                TokenTypes::Slash => lval / rval,
                _ => panic!("Invalid operator"),
            }
        }
        Expr::Let(name, val) => {
            let v = eval(val, env);
            env.insert(name.clone(), Expr::Number(v));
            v
        }
        Expr::FnDef(name, params, body) => {
            env.insert(
                name.clone(),
                Expr::FnDef(name.clone(), params.clone(), body.clone()),
            );
            0
        }
        Expr::FnCall(name, args) => {
            let func = env.get(name).cloned(); // clone out, avoid borrow checker issues
            if let Some(Expr::FnDef(_, params, body)) = func {
                if params.len() != args.len() {
                    panic!(
                        "Function {} expects {} args, got {}",
                        name,
                        params.len(),
                        args.len()
                    );
                }
                let mut local_env = env.clone();
                for (param, arg_expr) in params.iter().zip(args) {
                    let val = eval(arg_expr, env);
                    local_env.insert(param.clone(), Expr::Number(val));
                }
                eval(&body, &mut local_env)
            } else {
                panic!("Undefined function: {}", name);
            }
        }
        Expr::Var(name) => {
            if let Some(val) = env.get(name) {
                match val {
                    Expr::Number(n) => *n,
                    _ => panic!("Variable {} is not a number", name),
                }
            } else {
                panic!("Undefined variable {}", name);
            }
        }
    }
}
pub fn eval_with_validation(
    expr: &Expr,
    validator: &mut ResourceValidator,
    env: &mut HashMap<String, Expr>,
) -> Result<i64, ValidationError> {
    let costs = validator.validate_expression(expr)?;

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

    Ok(eval(expr, env))
}

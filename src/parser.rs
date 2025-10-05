use std::collections::HashMap;

use crate::{CoinType, ResourceValidator, Token, TokenTypes, ValidationError};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    ExpectedToken(String),
    UnexpectedEof,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
            ParseError::ExpectedToken(msg) => write!(f, "Expected: {}", msg),
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Binary(Box<Expr>, TokenTypes, Box<Expr>),
    Let(String, Box<Expr>), // ident, val
    FnDef(String, Vec<String>, Box<Expr>),
    FnCall(String, Vec<Expr>),
    Var(String),
    Block(Vec<Expr>), // for multiple statements
    Print(Box<Expr>), // print expression
    String(String), // string literal
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

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok.token_type {
                TokenTypes::Plus | TokenTypes::Minus => {
                    let op = self.eat().unwrap().token_type;
                    let rhs = self.parse_term()?;
                    node = Expr::Binary(Box::new(node), op, Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    pub fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.parse_factor()?;
        while let Some(tok) = self.peek() {
            match tok.token_type {
                TokenTypes::Star | TokenTypes::Slash => {
                    let op = self.eat().unwrap().token_type;
                    let rhs = self.parse_factor()?;
                    node = Expr::Binary(Box::new(node), op, Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(node)
    }

    pub fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        match self.eat() {
            Some(tok) if tok.token_type == TokenTypes::Number => {
                let n = tok.value.unwrap().parse::<i64>().unwrap();
                Ok(Expr::Number(n))
            }
            Some(tok) if tok.token_type == TokenTypes::String => {
                let s = tok.value.unwrap();
                Ok(Expr::String(s))
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
                                args.push(self.parse_expr()?);
                                
                                if let Some(next_tok) = self.peek() {
                                    match next_tok.token_type {
                                        TokenTypes::Comma => {
                                            self.eat(); 
                                            continue;
                                        }
                                        TokenTypes::RParen => {
                                            continue;
                                        }
                                        _ => return Err(ParseError::ExpectedToken("',' or ')' after function argument".to_string())),
                                    }
                                }
                            }
                        }
                        Ok(Expr::FnCall(name, args))
                    } else {
                        Ok(Expr::Var(name))
                    }
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(tok) if tok.token_type == TokenTypes::LParen => {
                let expr = self.parse_expr()?;
                if self.eat().map(|t| t.token_type) != Some(TokenTypes::RParen) {
                    return Err(ParseError::ExpectedToken("closing parenthesis".to_string()));
                }
                Ok(expr)
            }
            Some(tok) => Err(ParseError::UnexpectedToken(format!("{:?}", tok))),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    pub fn parse_fn_def(&mut self) -> Result<Expr, ParseError> {
        // fn
        self.eat();

        // function name
        let name = match self.eat() {
            Some(Token {
                token_type: TokenTypes::Identifier,
                value: Some(id),
                ..
            }) => id,
            _ => return Err(ParseError::ExpectedToken("identifier after 'fn'".to_string())),
        };

        // expect (
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::LParen,
                ..
            }) => {}
            _ => return Err(ParseError::ExpectedToken("'(' after function name".to_string())),
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
                            _ => return Err(ParseError::ExpectedToken("',' or ')' after parameter".to_string())),
                        }
                    }
                }
                TokenTypes::RParen => {
                    self.eat();
                    break;
                }
                _ => return Err(ParseError::UnexpectedToken(format!("in parameter list: {:?}", tok))),
            }
        }

        // expect {
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::LCurly,
                ..
            }) => {}
            _ => return Err(ParseError::ExpectedToken("'{' before function body".to_string())),
        };

        let body = self.parse_expr()?;

        // expect }
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::RCurly,
                ..
            }) => {}
            _ => return Err(ParseError::ExpectedToken("'}' at end of function body".to_string())),
        }

        Ok(Expr::FnDef(name, params, Box::new(body)))
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.eat();
        let ident = match self.eat() {
            Some(Token {
                token_type: TokenTypes::Identifier,
                value: Some(name),
                ..
            }) => name,
            _ => return Err(ParseError::ExpectedToken("identifier after 'let'".to_string())),
        };
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::Eq,
                ..
            }) => {}
            _ => return Err(ParseError::ExpectedToken("'=' after identifier in let".to_string())),
        }
        let expr = self.parse_expr()?;
        Ok(Expr::Let(ident, Box::new(expr)))
    }

    fn parse_print(&mut self) -> Result<Expr, ParseError> {
        self.eat(); // consume 'print'
        
        // expect (
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::LParen,
                ..
            }) => {}
            _ => return Err(ParseError::ExpectedToken("'(' after 'print'".to_string())),
        };
        
        let expr = self.parse_expr()?;
        
        // expect )
        match self.eat() {
            Some(Token {
                token_type: TokenTypes::RParen,
                ..
            }) => {}
            _ => return Err(ParseError::ExpectedToken("')' after print expression".to_string())),
        };
        
        Ok(Expr::Print(Box::new(expr)))
    }

    pub fn parse_stmt(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token {
                token_type: TokenTypes::Let,
                ..
            }) => self.parse_let(),
            Some(Token {
                token_type: TokenTypes::Fn,
                ..
            }) => self.parse_fn_def(),
            Some(Token {
                token_type: TokenTypes::Print,
                ..
            }) => self.parse_print(),
            _ => self.parse_expr(),
        }
    }

    pub fn parse_program(&mut self) -> Result<Expr, ParseError> {
        let mut statements = Vec::new();
        
        while self.pos < self.tokens.len() {
            statements.push(self.parse_stmt()?);
            
            // Check for semicolon separator
            if let Some(tok) = self.peek() {
                if tok.token_type == TokenTypes::Semicolon {
                    self.eat(); // consume semicolon
                } else if self.pos < self.tokens.len() {
                    // If there are more tokens but no semicolon, that's an error
                    break;
                }
            }
        }
        
        if statements.len() == 1 {
            Ok(statements.into_iter().next().unwrap())
        } else {
            Ok(Expr::Block(statements))
        }
    }
}

pub fn eval(expr: &Expr, env: &mut HashMap<String, Expr>) -> Result<i64, ValidationError> {
    let mut output = Vec::new();
    eval_with_output(expr, env, &mut output)
}

pub fn eval_with_output(expr: &Expr, env: &mut HashMap<String, Expr>, output: &mut Vec<String>) -> Result<i64, ValidationError> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::String(_) => Ok(0), // String literals evaluate to 0 for numeric context
        Expr::Binary(lhs, op, rhs) => {
            let lval = eval_with_output(lhs, env, output)?;
            let rval = eval_with_output(rhs, env, output)?;
            match op {
                TokenTypes::Plus => Ok(lval + rval),
                TokenTypes::Minus => Ok(lval - rval),
                TokenTypes::Star => Ok(lval * rval),
                TokenTypes::Slash => {
                    if rval == 0 {
                        Err(ValidationError::RuntimeError("Division by zero".to_string()))
                    } else {
                        Ok(lval / rval)
                    }
                }
                _ => Err(ValidationError::RuntimeError("Invalid operator".to_string())),
            }
        }
        Expr::Let(name, val) => {
            let v = eval_with_output(val, env, output)?;
            env.insert(name.clone(), Expr::Number(v));
            Ok(v)
        }
        Expr::FnDef(name, params, body) => {
            env.insert(
                name.clone(),
                Expr::FnDef(name.clone(), params.clone(), body.clone()),
            );
            Ok(0)
        }
        Expr::FnCall(name, args) => {
            let func = env.get(name).cloned(); // clone out, avoid borrow checker issues
            if let Some(Expr::FnDef(_, params, body)) = func {
                if params.len() != args.len() {
                    return Err(ValidationError::RuntimeError(format!(
                        "Function '{}' expects {} arguments, got {}",
                        name,
                        params.len(),
                        args.len()
                    )));
                }
                let mut local_env = env.clone();
                for (param, arg_expr) in params.iter().zip(args) {
                    let val = eval_with_output(arg_expr, env, output)?;
                    local_env.insert(param.clone(), Expr::Number(val));
                }
                eval_with_output(&body, &mut local_env, output)
            } else {
                Err(ValidationError::RuntimeError(format!("Undefined function '{}'", name)))
            }
        }
        Expr::Var(name) => {
            if let Some(val) = env.get(name) {
                match val {
                    Expr::Number(n) => Ok(*n),
                    Expr::FnDef(_, _, _) => {
                        Err(ValidationError::RuntimeError(format!(
                            "Cannot use function '{}' as a variable. Did you mean to call it with parentheses?", 
                            name
                        )))
                    }
                    _ => Err(ValidationError::RuntimeError(format!("Variable '{}' is not a number", name))),
                }
            } else {
                Err(ValidationError::RuntimeError(format!("Undefined variable '{}'", name)))
            }
        }
        Expr::Block(statements) => {
            let mut result = 0;
            for stmt in statements {
                result = eval_with_output(stmt, env, output)?;
            }
            Ok(result)
        }
        Expr::Print(expr) => {
            let output_str = match expr.as_ref() {
                Expr::String(s) => s.clone(),
                Expr::Number(n) => n.to_string(),
                Expr::Var(name) => {
                    if let Some(Expr::Number(n)) = env.get(name) {
                        n.to_string()
                    } else {
                        return Err(ValidationError::RuntimeError(format!("Undefined variable: {}", name)));
                    }
                }
                other => {
                    let val = eval_with_output(other, env, output)?;
                    val.to_string()
                }
            };
            
            println!("{}", output_str);
            output.push(output_str);
            Ok(0) // print statements return 0
        }
    }
}
pub fn eval_with_validation(
    expr: &Expr,
    validator: &mut ResourceValidator,
    env: &mut HashMap<String, Expr>,
) -> Result<(i64, Vec<String>), ValidationError> {
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

    let mut output = Vec::new();
    let result = eval_with_output(expr, env, &mut output)?;
    Ok((result, output))
}

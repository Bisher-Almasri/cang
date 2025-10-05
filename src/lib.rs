pub mod coin_manager;
pub mod parser;
pub mod quest_system;
pub mod repl;
pub mod resource_validator;

pub use coin_manager::{CoinError, CoinManager, CoinReward, CoinType};
pub use parser::Expr;
pub use quest_system::{ExecutionContext, FunctionDef, Quest, QuestManager, QuestObjective, QuestProgress};
pub use repl::Repl;
pub use resource_validator::{CoinCost, ResourceValidator, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenTypes {
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Let,
    Identifier,
    Eq,
    Fn,
    LCurly,
    RCurly,
    Semicolon,
    Comma,
    Print,
    String,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenTypes,
    pub value: Option<String>,
    pub pos: (usize, usize),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();

    let mut line = 1;
    let mut col = 0;

    // add let

    while let Some(&ch) = chars.peek() {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }

                let token_type = match ident.as_str() {
                    "let" => TokenTypes::Let,
                    "fn" => TokenTypes::Fn,
                    "print" => TokenTypes::Print,
                    _ => TokenTypes::Identifier,
                };

                tokens.push(Token {
                    token_type,
                    value: Some(ident),
                    pos: (line, col),
                });
            }

            '0'..='9' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        num.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    token_type: TokenTypes::Number,
                    value: Some(num),
                    pos: (line, col),
                });
            }
            '+' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Plus,
                    value: None,
                    pos: (line, col),
                });
            }
            '-' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Minus,
                    value: None,
                    pos: (line, col),
                });
            }
            '*' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Star,
                    value: None,
                    pos: (line, col),
                });
            }
            '/' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Slash,
                    value: None,
                    pos: (line, col),
                });
            }
            '(' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::LParen,
                    value: None,
                    pos: (line, col),
                });
            }
            ')' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::RParen,
                    value: None,
                    pos: (line, col),
                });
            }
            '{' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::LCurly,
                    value: None,
                    pos: (line, col),
                });
            }
            '}' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::RCurly,
                    value: None,
                    pos: (line, col),
                });
            }
            '=' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Eq,
                    value: None,
                    pos: (line, col),
                });
            }
            ';' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Semicolon,
                    value: None,
                    pos: (line, col),
                });
            }
            ',' => {
                chars.next();
                col += 1;
                tokens.push(Token {
                    token_type: TokenTypes::Comma,
                    value: None,
                    pos: (line, col),
                });
            }

            ' ' | '\t' => {
                chars.next();
                col += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                col = 0;
            }
            '"' => {
                chars.next(); // consume opening quote
                col += 1;
                let mut string_val = String::new();
                
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next(); // consume closing quote
                        col += 1;
                        break;
                    } else if c == '\\' {
                        chars.next(); // consume backslash
                        col += 1;
                        if let Some(&escaped) = chars.peek() {
                            match escaped {
                                'n' => string_val.push('\n'),
                                't' => string_val.push('\t'),
                                'r' => string_val.push('\r'),
                                '\\' => string_val.push('\\'),
                                '"' => string_val.push('"'),
                                _ => {
                                    string_val.push('\\');
                                    string_val.push(escaped);
                                }
                            }
                            chars.next();
                            col += 1;
                        }
                    } else {
                        string_val.push(c);
                        chars.next();
                        col += 1;
                    }
                }
                
                tokens.push(Token {
                    token_type: TokenTypes::String,
                    value: Some(string_val),
                    pos: (line, col),
                });
            }
            _ => {
                chars.next();
                col += 1;
            }
        }
    }

    tokens
}

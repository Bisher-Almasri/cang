pub mod coin_manager;
pub mod parser;
pub mod resource_validator;

pub use coin_manager::{CoinError, CoinManager, CoinReward, CoinType};
pub use parser::Expr;
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
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenTypes,
    pub value: Option<String>,
    pub pos: (usize, usize),
}


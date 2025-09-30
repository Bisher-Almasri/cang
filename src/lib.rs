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

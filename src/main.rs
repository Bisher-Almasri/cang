#[derive(Debug)]
enum TokenTypes {
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

#[derive(Debug)]
struct Token {
    token_type: TokenTypes,
    value: Option<String>,
    pos: (usize, usize),
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();

    let mut line = 1;
    let mut col = 0;

    while let Some(&ch) = chars.peek() {
        match ch {
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
            ' ' | '\t' => {
                chars.next();
                col += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                col = 0;
            }
            _ => {
                chars.next();
                col += 1;
            }
        }
    }

    tokens
}

fn main() {
    let input = "12 + 34 * (56 - 78)";
    let tokens = tokenize(input);
    for t in tokens {
        println!("{:?}", t);
    }
}

use cang::{
    parser::{eval, eval_with_validation, Parser},
    CoinManager, ResourceValidator, Token, TokenTypes,
};

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
    let input = "1 + 2 * (3 - 4) + 10";

    let tokens = tokenize(input);
    println!("Tokens: {:#?}", tokens);
    println!("");

    let mut parser = Parser::new(tokens);
    let ast = parser.parse_expr();
    println!("AST: {:#?}", ast);

    println!("");

    let result = eval(&ast);
    println!("Result (no validation): {}", result);

    let coin_manager = CoinManager::new();
    let mut validator = ResourceValidator::new(coin_manager);

    println!(
        "Initial coin balances: {:?}",
        validator.coin_manager().get_all_balences()
    );

    match eval_with_validation(&ast, &mut validator) {
        Ok(result) => {
            println!("Result (with validation): {}", result);
            println!(
                "Remaining coin balances: {:?}",
                validator.coin_manager().get_all_balences()
            );
        }
        Err(e) => {
            println!("Validation error: {}", e);
        }
    }

    println!("\n--- Testing insufficient funds scenario ---");
    let low_coin_manager = CoinManager::with_balances(1, 0);
    let mut low_validator = ResourceValidator::new(low_coin_manager);

    println!(
        "Low coin balances: {:?}",
        low_validator.coin_manager().get_all_balences()
    );

    match eval_with_validation(&ast, &mut low_validator) {
        Ok(result) => {
            println!("Unexpected success: {}", result);
        }
        Err(e) => {
            println!("Expected validation error: {}", e);
        }
    }

    println!("");
}

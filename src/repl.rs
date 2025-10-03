use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{
    parser::{eval_with_validation, Parser},
    tokenize, CoinManager, Expr, ResourceValidator,
};

pub struct Repl {
    pub validator: ResourceValidator,
    pub env: HashMap<String, Expr>,
}

impl Repl {
    pub fn new() -> Self {
        let coin_manager = CoinManager::new();
        let validator = ResourceValidator::new(coin_manager);

        Self {
            validator,
            env: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("\nCAng> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    if input.is_empty() {
                        continue;
                    }

                    match input {
                        "quit" | "exit" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" => self.show_help(),
                        "status" => self.display_status(),
                        "balance" | "coins" => self.show_coinbal(),
                        _ => self.execute(input),
                    }
                }
                Err(e) => {
                    println!("Error reading input: {}", e);
                    break;
                }
            }
        }
    }

    fn execute(&mut self, input: &str) {
        let tokens = tokenize(input);

        if tokens.is_empty() {
            println!("No valid tokens");
            return;
        }

        let mut parser = Parser::new(tokens);
        let ast =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse_stmt())) {
                Ok(ast) => ast,
                Err(_) => {
                    println!("Parse Error: Invalid Syntax");
                    return;
                }
            };

        println!("Before execution");
        self.show_coinbal();

        match eval_with_validation(&ast, &mut self.validator, &mut self.env) {
            Ok(res) => {
                println!("Result: {}", res);
                println!("\nAfter execution:");
                self.show_coinbal();
            }
            Err(e) => {
                println!("Error: {}", e);
                self.show_coinbal();
            }
        }
    }

    fn show_help(&self) {
        println!("\nAvailable commands:");
        println!("  help     - Show this help message");
        println!("  status   - Show current status and coin balances");
        println!("  balance  - Show coin balances");
        println!("  coins    - Show coin balances");
        println!("  quit     - Exit the REPL");
        println!("\nYou can also enter expressions to evaluate:");
        println!("  Examples: 1 + 2 * 3");
        println!("           let x = 10 + 5");
    }

    pub fn display_status(&self) {
        println!("\n=== CAng Status ===");
        self.show_coinbal();
        println!("Status: Ready for input");
    }

    pub fn show_coinbal(&self) {
        let bal = self.validator.coin_manager().get_all_balances();
        println!("Coin Balances:");
        for (coint_type, amt) in bal {
            let coin_name = match coint_type {
                crate::CoinType::Variable => "Variable",
                crate::CoinType::Function => "Function",
            };
            println!("  {} coins: {}", coin_name, amt);
        }
    }
}

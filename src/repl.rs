use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{
    parser::{eval_with_validation, Parser},
    tokenize, CoinManager, Expr, ResourceValidator, QuestManager, ExecutionContext,
};

pub struct Repl {
    pub validator: ResourceValidator,
    pub env: HashMap<String, Expr>,
    pub quest_manager: QuestManager,
    pub execution_context: ExecutionContext,
}

impl Repl {
    pub fn new() -> Self {
        let coin_manager = CoinManager::new();
        let validator = ResourceValidator::new(coin_manager);
        let mut quest_manager = QuestManager::new();
        quest_manager.initialize_starter_quests();

        Self {
            validator,
            env: HashMap::new(),
            quest_manager,
            execution_context: ExecutionContext::new(),
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
                        "quests" => self.show_quests(),
                        "progress" => self.show_detailed_quest_progress(),
                        "available" => self.show_available_quests(),
                        "completed" => self.show_completed_quests(),
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
        let ast = match parser.parse_program() {
            Ok(ast) => ast,
            Err(e) => {
                println!("Parse Error: {}", e);
                return;
            }
        };

        
        self.track_expression_execution(&ast);

        match eval_with_validation(&ast, &mut self.validator, &mut self.env) {
            Ok((res, output)) => {
                
                if !matches!(ast, Expr::Print(_)) {
                    println!("Result: {}", res);
                }
                
                
                self.update_execution_context(&ast, res);
                
                
                for output_line in output {
                    self.execution_context.add_output(output_line);
                }
                
                
                let rewards = self.quest_manager.check_completion(&self.execution_context);
                if !rewards.is_empty() {
                    self.display_quest_completion_notification(&rewards);
                }
                
                
                self.show_quest_progress_summary();
            }
            Err(e) => {
                println!("Error: {}", e);
                
                let error_string = format!("{}", e);
                if error_string.contains("Insufficient") {
                    self.suggest_quests_for_coins(&error_string);
                }
            }
        }
    }

    fn track_expression_execution(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(_) => {
                self.execution_context.record_expression("Number".to_string());
            }
            Expr::String(_) => {
                self.execution_context.record_expression("String".to_string());
            }
            Expr::Binary(_, _op, _) => {
                self.execution_context.record_expression("Binary".to_string());
                self.execution_context.record_expression("arithmetic".to_string());
            }
            Expr::Let(name, _) => {
                self.execution_context.record_expression(format!("Let({})", name));
            }
            Expr::FnDef(name, params, _) => {
                self.execution_context.record_expression(format!("FnDef({}, {} params)", name, params.len()));
            }
            Expr::FnCall(name, args) => {
                self.execution_context.record_expression(format!("FnCall({}, {} args)", name, args.len()));
            }
            Expr::Var(name) => {
                self.execution_context.record_expression(format!("Var({})", name));
            }
            Expr::Print(inner_expr) => {
                self.execution_context.record_expression("Print".to_string());
                self.track_expression_execution(inner_expr);
            }
            Expr::Block(statements) => {
                self.execution_context.record_expression("Block".to_string());
                for stmt in statements {
                    self.track_expression_execution(stmt);
                }
            }
        }
    }

    fn update_execution_context(&mut self, expr: &Expr, result: i64) {
        match expr {
            Expr::Let(name, _) => {
                self.execution_context.add_variable(name.clone(), result);
            }
            Expr::FnDef(name, params, body) => {
                
                let body_str = format!("{:?}", body);
                self.execution_context.add_function(name.clone(), params.clone(), body_str);
            }
            Expr::Block(statements) => {
                for stmt in statements {
                    
                    match stmt {
                        Expr::Let(name, _) => {
                            
                            if let Some(Expr::Number(val)) = self.env.get(name) {
                                self.execution_context.add_variable(name.clone(), *val);
                            }
                        }
                        Expr::FnDef(name, params, body) => {
                            let body_str = format!("{:?}", body);
                            self.execution_context.add_function(name.clone(), params.clone(), body_str);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn show_help(&self) {
        println!("\nAvailable commands:");
        println!("  help       - Show this help message");
        println!("  status     - Show current status and coin balances");
        println!("  balance    - Show coin balances");
        println!("  coins      - Show coin balances");
        println!("  quests     - Show all quests (active and completed)");
        println!("  available  - Show only available quests");
        println!("  completed  - Show only completed quests");
        println!("  progress   - Show detailed progress on all active quests");
        println!("  quit       - Exit the REPL");
        println!("\nYou can also enter expressions to evaluate:");
        println!("  Examples: 1 + 2 * 3");
        println!("           let x = 10 + 5");
        println!("           fn add(a, b) {{ a + b }}");
    }

    pub fn display_status(&self) {
        println!("\n🎮 CAng Interpreter Status");
        
        
        self.show_coinbal();
        
        
        let available_quests = self.quest_manager.get_available_quests();
        let locked_quests = self.quest_manager.get_locked_quests();
        let completed_quests = self.quest_manager.get_completed_quests();
        
        println!("\n📊 Quest Progress:");
        println!("  Available: {} | Locked: {} | Completed: {}", 
                 available_quests.len(), locked_quests.len(), completed_quests.len());
        
        
        if !available_quests.is_empty() {
            println!("\n🎯 Current Quest Progress:");
            for quest in available_quests.iter().take(3) {
                if let Some(progress) = self.quest_manager.get_quest_progress(&quest.id, &self.execution_context) {
                    let percentage = progress.completion_percentage();
                    let progress_bar = self.create_progress_bar(percentage);
                    println!("  {} {} ({:.0}%)", progress_bar, quest.title, percentage);
                }
            }
        }
        
        
        println!("\n📈 Session Statistics:");
        println!("  Variables created: {}", self.execution_context.variables.len());
        println!("  Functions defined: {}", self.execution_context.functions.len());
        println!("  Expressions executed: {}", self.execution_context.executed_expressions.len());
        
        
        if let Some(next_quest) = available_quests.first() {
            if let Some(progress) = self.quest_manager.get_quest_progress(&next_quest.id, &self.execution_context) {
                if progress.completion_percentage() == 0.0 {
                    println!("\n💡 Suggested next action:");
                    println!("  Try working on: {} - {}", next_quest.title, next_quest.description);
                }
            }
        }
        
        println!("\n✅ Status: Ready for input");
    }

    fn show_quests(&self) {
        println!("\n🎯 Quest Overview");
        
        
        let available_quests = self.quest_manager.get_available_quests();
        let locked_quests = self.quest_manager.get_locked_quests();
        let completed_quests = self.quest_manager.get_completed_quests();
        
        println!("📊 Quest Statistics:");
        println!("  Available: {} | Locked: {} | Completed: {}", 
                 available_quests.len(), locked_quests.len(), completed_quests.len());
        
        
        if !available_quests.is_empty() {
            println!("\n🎯 Available Quests:");
            for quest in available_quests.iter().take(5) { 
                if let Some(progress) = self.quest_manager.get_quest_progress(&quest.id, &self.execution_context) {
                    let percentage = progress.completion_percentage();
                    let progress_bar = self.create_progress_bar(percentage);
                    println!("  {} {} [{}] ({:.0}%)", progress_bar, quest.title, quest.difficulty.description(), percentage);
                } else {
                    println!("  [░░░░░░░░░░] {} [{}] (0%)", quest.title, quest.difficulty.description());
                }
            }
            
            if available_quests.len() > 5 {
                println!("  ... and {} more (use 'available' to see all)", available_quests.len() - 5);
            }
        }
        
        
        if !completed_quests.is_empty() {
            println!("\n🏆 Recently Completed:");
            for quest in completed_quests.iter().rev().take(3) { 
                println!("  ✅ {} [{}]", quest.title, quest.difficulty.description());
            }
            
            if completed_quests.len() > 3 {
                println!("  ... and {} more (use 'completed' to see all)", completed_quests.len() - 3);
            }
        }
        
        
        if !locked_quests.is_empty() {
            println!("\n🔒 {} quests are locked. Complete prerequisites to unlock them!", locked_quests.len());
        }
        
        println!("\nUse 'available', 'completed', or 'progress' for detailed views.");
    }

    pub fn show_coinbal(&self) {
        let bal = self.validator.coin_manager().get_all_balances();
        println!("💰 Coin Balances:");
        for (coint_type, amt) in bal {
            let coin_name = match coint_type {
                crate::CoinType::Variable => "Variable",
                crate::CoinType::Function => "Function",
            };
            println!("  {} coins: {}", coin_name, amt);
        }
    }

    fn display_quest_completion_notification(&mut self, rewards: &[crate::CoinReward]) {
        println!("\n🎉 QUEST COMPLETED! 🎉");
        println!("Congratulations! You've completed one or more quests!");
        
        let mut total_variable_coins = 0;
        let mut total_function_coins = 0;
        
        for reward in rewards {
            match reward.coin_type {
                crate::CoinType::Variable => total_variable_coins += reward.amount,
                crate::CoinType::Function => total_function_coins += reward.amount,
            }
            self.validator.coin_manager_mut().add_coins(reward.amount, reward.coin_type);
        }
        
        println!("Rewards earned:");
        if total_variable_coins > 0 {
            println!("  💎 {} Variable coins", total_variable_coins);
        }
        if total_function_coins > 0 {
            println!("  🔧 {} Function coins", total_function_coins);
        }
        
        
        let available_quests = self.quest_manager.get_available_quests();
        let newly_unlocked: Vec<_> = available_quests.iter()
            .filter(|q| !q.prerequisites.is_empty())
            .collect();
            
        if !newly_unlocked.is_empty() {
            println!("\n🔓 New quests unlocked:");
            for quest in newly_unlocked {
                println!("  📋 {} - {}", quest.title, quest.description);
            }
        }
    }

    fn show_quest_progress_summary(&self) {
        let available_quests = self.quest_manager.get_available_quests();
        if available_quests.is_empty() {
            return;
        }

        println!("\n📊 Quest Progress Summary:");
        for quest in available_quests.iter().take(3) { 
            if let Some(progress) = self.quest_manager.get_quest_progress(&quest.id, &self.execution_context) {
                let percentage = progress.completion_percentage();
                let progress_bar = self.create_progress_bar(percentage);
                println!("  {} {} ({:.0}%)", progress_bar, quest.title, percentage);
            }
        }
        
        if available_quests.len() > 3 {
            println!("  ... and {} more quests (use 'quests' to see all)", available_quests.len() - 3);
        }
    }

    fn create_progress_bar(&self, percentage: f32) -> String {
        let filled_blocks = (percentage / 10.0) as usize;
        let empty_blocks = 10 - filled_blocks;
        
        let filled = "█".repeat(filled_blocks);
        let empty = "░".repeat(empty_blocks);
        
        format!("[{}{}]", filled, empty)
    }

    fn suggest_quests_for_coins(&self, error_message: &str) {
        println!("\n💡 Hint: You need more coins!");
        
        let available_quests = self.quest_manager.get_available_quests();
        
        if error_message.contains("Variable") {
            println!("To earn Variable coins, try these available quests:");
            for quest in available_quests.iter().take(2) {
                if quest.rewards.iter().any(|r| matches!(r.coin_type, crate::CoinType::Variable)) {
                    println!("  📋 {} - {}", quest.title, quest.description);
                }
            }
        } else if error_message.contains("Function") {
            println!("To earn Function coins, try these available quests:");
            for quest in available_quests.iter().take(2) {
                if quest.rewards.iter().any(|r| matches!(r.coin_type, crate::CoinType::Function)) {
                    println!("  📋 {} - {}", quest.title, quest.description);
                }
            }
        }
        
        if available_quests.is_empty() {
            println!("Complete some basic quests first to unlock more opportunities!");
        }
    }

    fn show_available_quests(&self) {
        println!("\n🎯 Available Quests:");
        let available_quests = self.quest_manager.get_available_quests();
        
        if available_quests.is_empty() {
            println!("No quests currently available. Complete existing quests to unlock more!");
            return;
        }

        for quest in available_quests {
            println!("\n📋 {} [{}]", quest.title, quest.difficulty.description());
            println!("   {}", quest.description);
            
            
            if let Some(progress) = self.quest_manager.get_quest_progress(&quest.id, &self.execution_context) {
                let percentage = progress.completion_percentage();
                let progress_bar = self.create_progress_bar(percentage);
                println!("   Progress: {} {:.0}%", progress_bar, percentage);
            }
            
            println!("   Objectives:");
            for (i, objective) in quest.objectives.iter().enumerate() {
                let status = if let Some(progress) = self.quest_manager.get_quest_progress(&quest.id, &self.execution_context) {
                    if *progress.completed_objectives.get(i).unwrap_or(&false) {
                        "✅"
                    } else {
                        "⭕"
                    }
                } else {
                    "⭕"
                };
                println!("     {} {}", status, objective.description());
            }
            
            println!("   Rewards:");
            for reward in &quest.rewards {
                println!("     💰 {} {:?} coins", reward.amount, reward.coin_type);
            }
        }
    }

    fn show_completed_quests(&self) {
        println!("\n🏆 Completed Quests:");
        let completed_quests = self.quest_manager.get_completed_quests();
        
        if completed_quests.is_empty() {
            println!("No quests completed yet. Start with some basic arithmetic to begin your journey!");
            return;
        }

        for quest in completed_quests {
            println!("✅ {} [{}] - {}", quest.title, quest.difficulty.description(), quest.description);
            let total_rewards: u32 = quest.rewards.iter().map(|r| r.amount).sum();
            println!("   Earned {} total coins", total_rewards);
        }
        
        println!("\nTotal completed: {}", completed_quests.len());
    }

    fn show_detailed_quest_progress(&self) {
        println!("\n📈 Detailed Quest Progress:");
        let available_quests = self.quest_manager.get_available_quests();
        
        if available_quests.is_empty() {
            println!("No active quests to track progress for.");
            return;
        }

        for quest in available_quests {
            println!("\n📋 {} [{}]", quest.title, quest.difficulty.description());
            
            if let Some(progress) = self.quest_manager.get_quest_progress(&quest.id, &self.execution_context) {
                let percentage = progress.completion_percentage();
                let progress_bar = self.create_progress_bar(percentage);
                println!("   Overall Progress: {} {:.0}%", progress_bar, percentage);
                
                println!("   Objective Status:");
                for (i, objective) in quest.objectives.iter().enumerate() {
                    let status = if *progress.completed_objectives.get(i).unwrap_or(&false) {
                        "✅ COMPLETED"
                    } else {
                        "⭕ PENDING"
                    };
                    println!("     {} {}", status, objective.description());
                }
                
                if percentage == 100.0 {
                    println!("   🎉 Ready to claim rewards! Execute any code to complete this quest.");
                } else if percentage > 0.0 {
                    println!("   💪 Keep going! You're making progress.");
                } else {
                    println!("   🚀 Ready to start! Try the suggested actions above.");
                }
            }
        }
    }
}

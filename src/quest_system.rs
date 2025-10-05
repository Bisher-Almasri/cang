use crate::{CoinReward, CoinType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum QuestDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl QuestDifficulty {
    pub fn description(&self) -> &str {
        match self {
            QuestDifficulty::Beginner => "Beginner",
            QuestDifficulty::Intermediate => "Intermediate", 
            QuestDifficulty::Advanced => "Advanced",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<CoinReward>,
    pub completed: bool,
    pub difficulty: QuestDifficulty,
    pub prerequisites: Vec<String>, 
    pub unlocked: bool,
}

impl Quest {
    pub fn new(
        id: String,
        title: String,
        description: String,
        objectives: Vec<QuestObjective>,
        rewards: Vec<CoinReward>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            objectives,
            rewards,
            completed: false,
            difficulty: QuestDifficulty::Beginner,
            prerequisites: Vec::new(),
            unlocked: true, 
        }
    }

    pub fn new_with_difficulty(
        id: String,
        title: String,
        description: String,
        objectives: Vec<QuestObjective>,
        rewards: Vec<CoinReward>,
        difficulty: QuestDifficulty,
        prerequisites: Vec<String>,
    ) -> Self {
        let unlocked = prerequisites.is_empty(); 
        Self {
            id,
            title,
            description,
            objectives,
            rewards,
            completed: false,
            difficulty,
            prerequisites,
            unlocked,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }

    pub fn mark_completed(&mut self) {
        self.completed = true;
    }

    pub fn unlock(&mut self) {
        self.unlocked = true;
    }

    pub fn get_difficulty_description(&self) -> String {
        format!("[{}] {}", self.difficulty.description(), self.title)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestObjective {
    ExecuteProgram { pattern: String },
    DefineFunction { min_params: usize },
    UseVariables { count: usize },
    ProduceOutput { expected: String },
    CreateVariable { name: Option<String> },
    CallFunction { name: Option<String> },
    PerformArithmetic,
}

impl QuestObjective {
    pub fn description(&self) -> String {
        match self {
            QuestObjective::ExecuteProgram { pattern } => {
                format!("Execute a program matching pattern: {}", pattern)
            }
            QuestObjective::DefineFunction { min_params } => {
                format!("Define a function with at least {} parameters", min_params)
            }
            QuestObjective::UseVariables { count } => {
                format!("Use {} variables in your program", count)
            }
            QuestObjective::ProduceOutput { expected } => {
                format!("Produce output: {}", expected)
            }
            QuestObjective::CreateVariable { name } => {
                if let Some(var_name) = name {
                    format!("Create a variable named '{}'", var_name)
                } else {
                    "Create any variable".to_string()
                }
            }
            QuestObjective::CallFunction { name } => {
                if let Some(func_name) = name {
                    format!("Call function '{}'", func_name)
                } else {
                    "Call any function".to_string()
                }
            }
            QuestObjective::PerformArithmetic => "Perform arithmetic operations".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, i64>,
    pub functions: HashMap<String, FunctionDef>,
    pub output: Vec<String>,
    pub executed_expressions: Vec<String>, 
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            output: Vec::new(),
            executed_expressions: Vec::new(),
        }
    }

    pub fn add_variable(&mut self, name: String, value: i64) {
        self.variables.insert(name, value);
    }

    pub fn add_function(&mut self, name: String, params: Vec<String>, body: String) {
        self.functions.insert(
            name.clone(),
            FunctionDef {
                name,
                params,
                body,
            },
        );
    }

    pub fn add_output(&mut self, output: String) {
        self.output.push(output);
    }

    pub fn record_expression(&mut self, expr_type: String) {
        self.executed_expressions.push(expr_type);
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
}

#[derive(Debug)]
pub struct QuestManager {
    active_quests: Vec<Quest>,
    completed_quests: Vec<Quest>,
}

impl QuestManager {
    pub fn new() -> Self {
        Self {
            active_quests: Vec::new(),
            completed_quests: Vec::new(),
        }
    }

    pub fn add_quest(&mut self, quest: Quest) {
        if !self.has_quest(&quest.id) {
            self.active_quests.push(quest);
        }
    }

    pub fn get_available_quests(&self) -> Vec<&Quest> {
        self.active_quests.iter().filter(|q| q.is_unlocked() && !q.is_completed()).collect()
    }

    pub fn get_locked_quests(&self) -> Vec<&Quest> {
        self.active_quests.iter().filter(|q| !q.is_unlocked()).collect()
    }

    pub fn unlock_dependent_quests(&mut self, completed_quest_id: &str) {
        let mut quests_to_unlock = Vec::new();
        
        for (index, quest) in self.active_quests.iter().enumerate() {
            if !quest.is_unlocked() && quest.prerequisites.contains(&completed_quest_id.to_string()) {
                let all_prerequisites_met = quest.prerequisites.iter().all(|prereq_id| {
                    self.completed_quests.iter().any(|completed| completed.id == *prereq_id)
                });
                
                if all_prerequisites_met {
                    quests_to_unlock.push(index);
                }
            }
        }
        
        for &index in &quests_to_unlock {
            self.active_quests[index].unlock();
        }
    }

    pub fn has_quest(&self, quest_id: &str) -> bool {
        self.active_quests.iter().any(|q| q.id == quest_id)
            || self.completed_quests.iter().any(|q| q.id == quest_id)
    }

    pub fn get_active_quests(&self) -> &[Quest] {
        &self.active_quests
    }

    pub fn get_completed_quests(&self) -> &[Quest] {
        &self.completed_quests
    }

    pub fn check_completion(&mut self, execution_context: &ExecutionContext) -> Vec<CoinReward> {
        let mut rewards = Vec::new();
        let mut completed_quest_indices = Vec::new();

        for (index, quest) in self.active_quests.iter().enumerate() {
            if quest.completed {
                continue;
            }

            let all_objectives_met = quest.objectives.iter().all(|objective| {
                QuestManager::check_objective_static(objective, execution_context)
            });

            if all_objectives_met {
                rewards.extend(quest.rewards.clone());
                completed_quest_indices.push(index);
            }
        }

        for &index in completed_quest_indices.iter().rev() {
            self.active_quests[index].mark_completed();
            let completed_quest = self.active_quests.remove(index);
            let quest_id = completed_quest.id.clone();
            self.completed_quests.push(completed_quest);
            
            self.unlock_dependent_quests(&quest_id);
        }

        rewards
    }

    fn check_objective(&self, objective: &QuestObjective, context: &ExecutionContext) -> bool {
        QuestManager::check_objective_static(objective, context)
    }

    fn check_objective_static(objective: &QuestObjective, context: &ExecutionContext) -> bool {
        match objective {
            QuestObjective::ExecuteProgram { pattern } => {
                context.executed_expressions.iter().any(|expr| expr.contains(pattern))
            }
            QuestObjective::DefineFunction { min_params } => {
                context.functions.values().any(|func| func.params.len() >= *min_params)
            }
            QuestObjective::UseVariables { count } => {
                context.variables.len() >= *count
            }
            QuestObjective::ProduceOutput { expected } => {
                context.output.iter().any(|output| output == expected)
            }
            QuestObjective::CreateVariable { name } => {
                if let Some(var_name) = name {
                    context.variables.contains_key(var_name)
                } else {
                    !context.variables.is_empty()
                }
            }
            QuestObjective::CallFunction { name } => {
                if let Some(func_name) = name {
                    context.executed_expressions.iter().any(|expr| {
                        expr.contains("FnCall") && expr.contains(func_name)
                    })
                } else {
                    context.executed_expressions.iter().any(|expr| expr.contains("FnCall"))
                }
            }
            QuestObjective::PerformArithmetic => {
                context.executed_expressions.iter().any(|expr| {
                    expr.contains("Binary") || expr.contains("arithmetic")
                })
            }
        }
    }

    pub fn get_quest_by_id(&self, quest_id: &str) -> Option<&Quest> {
        self.active_quests.iter().find(|q| q.id == quest_id)
            .or_else(|| self.completed_quests.iter().find(|q| q.id == quest_id))
    }

    pub fn get_quest_progress(&self, quest_id: &str, context: &ExecutionContext) -> Option<QuestProgress> {
        if let Some(quest) = self.get_quest_by_id(quest_id) {
            let completed_objectives = quest.objectives.iter()
                .map(|obj| self.check_objective(obj, context))
                .collect();
            
            Some(QuestProgress {
                quest_id: quest_id.to_string(),
                total_objectives: quest.objectives.len(),
                completed_objectives,
                is_complete: quest.completed,
            })
        } else {
            None
        }
    }

    pub fn initialize_starter_quests(&mut self) {
        let hello_world_quest = Quest::new_with_difficulty(
            "hello_world".to_string(),
            "Hello World".to_string(),
            "Welcome to CAng! Start by performing a simple arithmetic calculation like '2 + 3' to get familiar with the interpreter.".to_string(),
            vec![QuestObjective::PerformArithmetic],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 2,
            }],
            QuestDifficulty::Beginner,
            vec![], 
        );

        let first_variable_quest = Quest::new_with_difficulty(
            "first_variable".to_string(),
            "First Variable".to_string(),
            "Learn to store values by creating your first variable. Try 'let x = 5' to create a variable named 'x' with value 5.".to_string(),
            vec![QuestObjective::CreateVariable { name: None }],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 3,
            }],
            QuestDifficulty::Beginner,
            vec!["hello_world".to_string()], 
        );

        let variable_arithmetic_quest = Quest::new_with_difficulty(
            "variable_arithmetic".to_string(),
            "Variable Arithmetic".to_string(),
            "Combine variables with arithmetic! Create a variable and then use it in a calculation.".to_string(),
            vec![
                QuestObjective::CreateVariable { name: None },
                QuestObjective::PerformArithmetic,
            ],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 2,
            }],
            QuestDifficulty::Beginner,
            vec!["first_variable".to_string()],
        );

        
        let first_function_quest = Quest::new_with_difficulty(
            "first_function".to_string(),
            "First Function".to_string(),
            "Define your first function to reuse code. Try 'fn add(a, b) { a + b }' to create a function that adds two numbers.".to_string(),
            vec![QuestObjective::DefineFunction { min_params: 0 }],
            vec![CoinReward {
                coin_type: CoinType::Function,
                amount: 2,
            }],
            QuestDifficulty::Intermediate,
            vec!["variable_arithmetic".to_string()],
        );

        let function_with_params_quest = Quest::new_with_difficulty(
            "function_with_params".to_string(),
            "Parameterized Function".to_string(),
            "Create a function that takes at least one parameter. Parameters make functions flexible and reusable.".to_string(),
            vec![QuestObjective::DefineFunction { min_params: 1 }],
            vec![CoinReward {
                coin_type: CoinType::Function,
                amount: 1,
            }],
            QuestDifficulty::Intermediate,
            vec!["first_function".to_string()],
        );

        let multiple_variables_quest = Quest::new_with_difficulty(
            "multiple_variables".to_string(),
            "Variable Master".to_string(),
            "Show your mastery by creating at least 3 different variables in your program.".to_string(),
            vec![QuestObjective::UseVariables { count: 3 }],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 4,
            }],
            QuestDifficulty::Intermediate,
            vec!["variable_arithmetic".to_string()],
        );

        
        let function_caller_quest = Quest::new_with_difficulty(
            "function_caller".to_string(),
            "Function Caller".to_string(),
            "Define a function and then call it! This demonstrates the full function lifecycle.".to_string(),
            vec![
                QuestObjective::DefineFunction { min_params: 1 },
                QuestObjective::CallFunction { name: None },
            ],
            vec![
                CoinReward {
                    coin_type: CoinType::Function,
                    amount: 2,
                },
                CoinReward {
                    coin_type: CoinType::Variable,
                    amount: 2,
                },
            ],
            QuestDifficulty::Advanced,
            vec!["function_with_params".to_string()],
        );

        let complex_program_quest = Quest::new_with_difficulty(
            "complex_program".to_string(),
            "Complex Program".to_string(),
            "Create a sophisticated program that uses multiple variables, defines a function, and performs calculations.".to_string(),
            vec![
                QuestObjective::UseVariables { count: 2 },
                QuestObjective::DefineFunction { min_params: 1 },
                QuestObjective::PerformArithmetic,
            ],
            vec![
                CoinReward {
                    coin_type: CoinType::Variable,
                    amount: 5,
                },
                CoinReward {
                    coin_type: CoinType::Function,
                    amount: 3,
                },
            ],
            QuestDifficulty::Advanced,
            vec!["multiple_variables".to_string(), "function_caller".to_string()],
        );

        let print_hello_quest = Quest::new_with_difficulty(
            "print_hello".to_string(),
            "Print Hello".to_string(),
            "Use the print statement to output 'Hello World' to the console. Try: print(\"Hello World\")".to_string(),
            vec![QuestObjective::ProduceOutput { expected: "Hello World".to_string() }],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 1,
            }],
            QuestDifficulty::Beginner,
            vec!["hello_world".to_string()],
        );

        self.add_quest(hello_world_quest);
        self.add_quest(print_hello_quest);
        self.add_quest(first_variable_quest);
        self.add_quest(variable_arithmetic_quest);
        self.add_quest(first_function_quest);
        self.add_quest(function_with_params_quest);
        self.add_quest(multiple_variables_quest);
        self.add_quest(function_caller_quest);
        self.add_quest(complex_program_quest);
    }
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct QuestProgress {
    pub quest_id: String,
    pub total_objectives: usize,
    pub completed_objectives: Vec<bool>,
    pub is_complete: bool,
}

impl QuestProgress {
    pub fn completion_percentage(&self) -> f32 {
        if self.total_objectives == 0 {
            return 100.0;
        }
        
        let completed_count = self.completed_objectives.iter().filter(|&&completed| completed).count();
        (completed_count as f32 / self.total_objectives as f32) * 100.0
    }
}
#
[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let quest = Quest::new(
            "test_quest".to_string(),
            "Test Quest".to_string(),
            "A test quest".to_string(),
            vec![QuestObjective::PerformArithmetic],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 1,
            }],
        );

        assert_eq!(quest.id, "test_quest");
        assert_eq!(quest.title, "Test Quest");
        assert!(!quest.completed);
        assert_eq!(quest.objectives.len(), 1);
        assert_eq!(quest.rewards.len(), 1);
        assert_eq!(quest.difficulty, QuestDifficulty::Beginner);
        assert!(quest.unlocked);
        assert!(quest.prerequisites.is_empty());
    }

    #[test]
    fn test_quest_creation_with_difficulty() {
        let quest = Quest::new_with_difficulty(
            "advanced_quest".to_string(),
            "Advanced Quest".to_string(),
            "An advanced quest".to_string(),
            vec![QuestObjective::PerformArithmetic],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 5,
            }],
            QuestDifficulty::Advanced,
            vec!["prerequisite_quest".to_string()],
        );

        assert_eq!(quest.difficulty, QuestDifficulty::Advanced);
        assert!(!quest.unlocked); 
        assert_eq!(quest.prerequisites.len(), 1);
        assert_eq!(quest.get_difficulty_description(), "[Advanced] Advanced Quest");
    }

    #[test]
    fn test_quest_manager_basic_operations() {
        let mut quest_manager = QuestManager::new();
        
        let quest = Quest::new(
            "test_quest".to_string(),
            "Test Quest".to_string(),
            "A test quest".to_string(),
            vec![QuestObjective::PerformArithmetic],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 1,
            }],
        );

        quest_manager.add_quest(quest);
        
        assert_eq!(quest_manager.get_active_quests().len(), 1);
        assert_eq!(quest_manager.get_completed_quests().len(), 0);
        assert!(quest_manager.has_quest("test_quest"));
        assert!(!quest_manager.has_quest("nonexistent_quest"));
    }

    #[test]
    fn test_execution_context() {
        let mut context = ExecutionContext::new();
        
        context.add_variable("x".to_string(), 42);
        context.add_function("test_func".to_string(), vec!["param1".to_string()], "body".to_string());
        context.add_output("Hello World".to_string());
        context.record_expression("Binary".to_string());

        assert_eq!(context.variables.len(), 1);
        assert_eq!(context.functions.len(), 1);
        assert_eq!(context.output.len(), 1);
        assert_eq!(context.executed_expressions.len(), 1);
        assert_eq!(context.variables.get("x"), Some(&42));
    }

    #[test]
    fn test_quest_completion_detection() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "arithmetic_quest".to_string(),
            "Arithmetic Quest".to_string(),
            "Perform arithmetic operations".to_string(),
            vec![QuestObjective::PerformArithmetic],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 2,
            }],
        );

        quest_manager.add_quest(quest);

        
        let rewards = quest_manager.check_completion(&context);
        assert!(rewards.is_empty());
        assert_eq!(quest_manager.get_active_quests().len(), 1);

        
        context.record_expression("Binary".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].coin_type, CoinType::Variable);
        assert_eq!(rewards[0].amount, 2);
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_starter_quests_initialization() {
        let mut quest_manager = QuestManager::new();
        quest_manager.initialize_starter_quests();

        assert_eq!(quest_manager.get_active_quests().len(), 9);
        
        let quest_ids: Vec<&String> = quest_manager.get_active_quests().iter().map(|q| &q.id).collect();
        assert!(quest_ids.contains(&&"hello_world".to_string()));
        assert!(quest_ids.contains(&&"print_hello".to_string()));
        assert!(quest_ids.contains(&&"first_variable".to_string()));
        assert!(quest_ids.contains(&&"first_function".to_string()));
        assert!(quest_ids.contains(&&"variable_arithmetic".to_string()));
        assert!(quest_ids.contains(&&"function_with_params".to_string()));
        assert!(quest_ids.contains(&&"multiple_variables".to_string()));
        assert!(quest_ids.contains(&&"function_caller".to_string()));
        assert!(quest_ids.contains(&&"complex_program".to_string()));

        
        let available_quests = quest_manager.get_available_quests();
        assert_eq!(available_quests.len(), 1);
        assert_eq!(available_quests[0].id, "hello_world");

        
        let hello_world = quest_manager.get_quest_by_id("hello_world").unwrap();
        assert_eq!(hello_world.difficulty, QuestDifficulty::Beginner);
        
        let first_function = quest_manager.get_quest_by_id("first_function").unwrap();
        assert_eq!(first_function.difficulty, QuestDifficulty::Intermediate);
        
        let complex_program = quest_manager.get_quest_by_id("complex_program").unwrap();
        assert_eq!(complex_program.difficulty, QuestDifficulty::Advanced);
    }

    #[test]
    fn test_quest_progress_tracking() {
        let mut quest_manager = QuestManager::new();
        let context = ExecutionContext::new();

        let quest = Quest::new(
            "multi_objective_quest".to_string(),
            "Multi Objective Quest".to_string(),
            "A quest with multiple objectives".to_string(),
            vec![
                QuestObjective::PerformArithmetic,
                QuestObjective::CreateVariable { name: None },
            ],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 3,
            }],
        );

        quest_manager.add_quest(quest);

        let progress = quest_manager.get_quest_progress("multi_objective_quest", &context).unwrap();
        assert_eq!(progress.total_objectives, 2);
        assert_eq!(progress.completion_percentage(), 0.0);
        assert!(!progress.is_complete);
    }

    #[test]
    fn test_integrated_quest_completion_flow() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "variable_quest".to_string(),
            "Variable Quest".to_string(),
            "Create a variable".to_string(),
            vec![QuestObjective::CreateVariable { name: None }],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 2,
            }],
        );

        quest_manager.add_quest(quest);

        
        assert_eq!(quest_manager.get_active_quests().len(), 1);
        assert_eq!(quest_manager.get_completed_quests().len(), 0);

        
        context.add_variable("test_var".to_string(), 42);
        context.record_expression("Let(test_var)".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].coin_type, CoinType::Variable);
        assert_eq!(rewards[0].amount, 2);

        
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_function_definition_quest_completion() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "function_quest".to_string(),
            "Function Quest".to_string(),
            "Define a function".to_string(),
            vec![QuestObjective::DefineFunction { min_params: 1 }],
            vec![CoinReward {
                coin_type: CoinType::Function,
                amount: 1,
            }],
        );

        quest_manager.add_quest(quest);

        
        context.add_function("test_func".to_string(), vec!["a".to_string(), "b".to_string()], "a + b".to_string());
        context.record_expression("FnDef(test_func, 2 params)".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].coin_type, CoinType::Function);
        assert_eq!(rewards[0].amount, 1);

        
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_hello_world_quest_completion() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "hello_world".to_string(),
            "Hello World".to_string(),
            "Perform arithmetic".to_string(),
            vec![QuestObjective::PerformArithmetic],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 2,
            }],
        );

        quest_manager.add_quest(quest);

        
        context.record_expression("Binary(2 + 3)".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].coin_type, CoinType::Variable);
        assert_eq!(rewards[0].amount, 2);

        
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_variable_creation_quest_completion() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "variable_quest".to_string(),
            "Variable Quest".to_string(),
            "Create a variable".to_string(),
            vec![QuestObjective::CreateVariable { name: None }],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 3,
            }],
        );

        quest_manager.add_quest(quest);

        
        context.add_variable("x".to_string(), 42);
        context.record_expression("Let(x = 42)".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].coin_type, CoinType::Variable);
        assert_eq!(rewards[0].amount, 3);

        
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_multiple_variables_quest_completion() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "multiple_vars_quest".to_string(),
            "Multiple Variables Quest".to_string(),
            "Create multiple variables".to_string(),
            vec![QuestObjective::UseVariables { count: 3 }],
            vec![CoinReward {
                coin_type: CoinType::Variable,
                amount: 4,
            }],
        );

        quest_manager.add_quest(quest);

        
        context.add_variable("x".to_string(), 1);
        context.add_variable("y".to_string(), 2);

        let rewards = quest_manager.check_completion(&context);
        assert!(rewards.is_empty());
        assert_eq!(quest_manager.get_active_quests().len(), 1);

        
        context.add_variable("z".to_string(), 3);

        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].amount, 4);
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_complex_multi_objective_quest_completion() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "complex_quest".to_string(),
            "Complex Quest".to_string(),
            "Complete multiple objectives".to_string(),
            vec![
                QuestObjective::UseVariables { count: 2 },
                QuestObjective::DefineFunction { min_params: 1 },
                QuestObjective::PerformArithmetic,
            ],
            vec![
                CoinReward {
                    coin_type: CoinType::Variable,
                    amount: 5,
                },
                CoinReward {
                    coin_type: CoinType::Function,
                    amount: 3,
                },
            ],
        );

        quest_manager.add_quest(quest);

        
        
        context.add_variable("x".to_string(), 10);
        context.add_variable("y".to_string(), 20);
        
        
        let rewards = quest_manager.check_completion(&context);
        assert!(rewards.is_empty());

        
        context.add_function("add".to_string(), vec!["a".to_string(), "b".to_string()], "a + b".to_string());
        
        
        let rewards = quest_manager.check_completion(&context);
        assert!(rewards.is_empty());

        
        context.record_expression("Binary(x + y)".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 2);
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }

    #[test]
    fn test_quest_progression_and_unlocking() {
        let mut quest_manager = QuestManager::new();
        quest_manager.initialize_starter_quests();

        
        let available = quest_manager.get_available_quests();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "hello_world");

        
        let mut context = ExecutionContext::new();
        context.record_expression("Binary(2 + 3)".to_string());
        
        let rewards = quest_manager.check_completion(&context);
        assert!(!rewards.is_empty());

        
        let available = quest_manager.get_available_quests();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "first_variable");

        
        let mut context2 = ExecutionContext::new();
        context2.add_variable("x".to_string(), 5);
        let rewards = quest_manager.check_completion(&context2);
        assert!(!rewards.is_empty());

        
        let available = quest_manager.get_available_quests();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, "variable_arithmetic");

        
        let mut context3 = ExecutionContext::new();
        context3.add_variable("y".to_string(), 10);
        context3.record_expression("Binary(y + 5)".to_string());
        let rewards = quest_manager.check_completion(&context3);
        assert!(!rewards.is_empty());

        
        let available = quest_manager.get_available_quests();
        assert_eq!(available.len(), 2);
        let quest_ids: Vec<&str> = available.iter().map(|q| q.id.as_str()).collect();
        assert!(quest_ids.contains(&"first_function"));
        assert!(quest_ids.contains(&"multiple_variables"));
    }

    #[test]
    fn test_quest_difficulty_descriptions() {
        assert_eq!(QuestDifficulty::Beginner.description(), "Beginner");
        assert_eq!(QuestDifficulty::Intermediate.description(), "Intermediate");
        assert_eq!(QuestDifficulty::Advanced.description(), "Advanced");

        let quest = Quest::new_with_difficulty(
            "test".to_string(),
            "Test Quest".to_string(),
            "Description".to_string(),
            vec![],
            vec![],
            QuestDifficulty::Advanced,
            vec![],
        );

        assert_eq!(quest.get_difficulty_description(), "[Advanced] Test Quest");
    }

    #[test]
    fn test_function_call_quest_completion() {
        let mut quest_manager = QuestManager::new();
        let mut context = ExecutionContext::new();

        
        let quest = Quest::new(
            "function_call_quest".to_string(),
            "Function Call Quest".to_string(),
            "Call a function".to_string(),
            vec![QuestObjective::CallFunction { name: None }],
            vec![CoinReward {
                coin_type: CoinType::Function,
                amount: 1,
            }],
        );

        quest_manager.add_quest(quest);

        
        context.record_expression("FnCall(test_func)".to_string());

        
        let rewards = quest_manager.check_completion(&context);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].coin_type, CoinType::Function);
        assert_eq!(rewards[0].amount, 1);

        
        assert_eq!(quest_manager.get_active_quests().len(), 0);
        assert_eq!(quest_manager.get_completed_quests().len(), 1);
    }
}
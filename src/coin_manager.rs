use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoinType {
    Variable,
    Function,
}

// for now i tihnk it cost to make stuff not to use, due to change rpob

#[derive(Debug, PartialEq)]
pub enum CoinError {
    InsufficientFunds {
        required: u32,
        availabe: u32,
        coin_type: CoinType,
    },
    InvalidCoinType,
}

impl std::fmt::Display for CoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinError::InsufficientFunds {
                required,
                availabe,
                coin_type,
            } => {
                write!(
                    f,
                    "Insufficient {:?} coins (need {}, have {})",
                    coin_type, required, availabe
                )
            }
            CoinError::InvalidCoinType => write!(f, "Invalid coin type"),
        }
    }
}

impl std::error::Error for CoinError {}

#[derive(Debug, Clone, PartialEq)]
pub struct CoinReward {
    pub coin_type: CoinType,
    pub amount: u32,
}

#[derive(Debug, Clone)]
pub struct CoinManager {
    balences: HashMap<CoinType, u32>,
}

impl CoinManager {
    // def 10 var 3 func
    pub fn new() -> Self {
        let mut balences = HashMap::new();
        balences.insert(CoinType::Variable, 10);
        balences.insert(CoinType::Function, 3);

        Self { balences }
    }

    // create wirh amt
    pub fn with_balances(variable_coins: u32, function_coins: u32) -> Self {
        let mut balences = HashMap::new();
        balences.insert(CoinType::Variable, variable_coins);
        balences.insert(CoinType::Function, function_coins);

        Self { balences }
    }

    pub fn spend_var_coin(&mut self) -> Result<(), CoinError> {
        self.spend_coins(CoinType::Variable, 1)
    }

    pub fn spend_func_coin(&mut self) -> Result<(), CoinError> {
        self.spend_coins(CoinType::Function, 1)
    }

    fn spend_coins(&mut self, coin_type: CoinType, amt: u32) -> Result<(), CoinError> {
        let current_balence = self.get_balence(coin_type);

        if current_balence < amt {
            return Err(CoinError::InsufficientFunds {
                required: amt,
                availabe: current_balence,
                coin_type,
            });
        }

        self.balences.insert(coin_type, current_balence - amt);
        Ok(())
    }

    pub fn get_balence(&self, coin_type: CoinType) -> u32 {
        *self.balences.get(&coin_type).unwrap_or(&0)
    }

    pub fn add_coins(&mut self, amt: u32, coin_type: CoinType) {
        let current_balence = self.get_balence(coin_type);
        self.balences.insert(coin_type, current_balence + amt);
    }

    pub fn get_all_balences(&self) -> &HashMap<CoinType, u32> {
        &self.balences
    }

    pub fn apply_rewards(&mut self, rewards: &[CoinReward]) {
        // for multiple at once
        for reward in rewards {
            self.add_coins(reward.amount, reward.coin_type);
        }
    }
}

impl Default for CoinManager {
    fn default() -> Self {
        Self::new()
    }
}

// purpouse of tihsi s to analyze hte ast an check if user can run
use crate::{CoinError, CoinManager, CoinType, Expr};

#[derive(Debug, Clone, PartialEq)]
pub struct CoinCost {
    pub coin_type: CoinType,
    pub amt: u32,
}

#[derive(Debug)]
pub enum ValidationError {
    CoinError(CoinError),
    ParseError(String),
    RuntimeError(String),
}

impl From<CoinError> for ValidationError {
    fn from(value: CoinError) -> Self {
        ValidationError::CoinError(value)
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::CoinError(e) => write!(f, "Coin err: {}", e),
            ValidationError::ParseError(e) => write!(f, "Parse err: {}", e),
            ValidationError::RuntimeError(e) => write!(f, "Runtime err: {}", e),
        }
    }
}

impl std::error::Error for ValidationError {}

pub struct ResourceValidator {
    coin_manager: CoinManager,
}

impl ResourceValidator {
    pub fn new(coin_manager: CoinManager) -> Self {
        Self { coin_manager }
    }

    pub fn validate_expression(&self, expr: &Expr) -> Result<Vec<CoinCost>, ValidationError> {
        let costs = self.calculate_costs(expr);

        for cost in &costs {
            let available = self.coin_manager.get_balence(cost.coin_type);
            if available < cost.amt {
                return Err(ValidationError::CoinError(CoinError::InsufficientFunds {
                    required: cost.amt,
                    availabe: available,
                    coin_type: cost.coin_type,
                }));
            }
        }

        Ok(costs)
    }

    pub fn calculate_costs(&self, expr: &Expr) -> Vec<CoinCost> {
        match expr {
            Expr::Number(_) => {
                vec![] //nums are free for now
            }
            Expr::Binary(lhs, _op, rhs) => {
                // for now lets say math costs var coins
                let mut costs = vec![CoinCost {
                    coin_type: CoinType::Variable,
                    amt: 1,
                }];

                costs.extend(self.calculate_costs(lhs));
                costs.extend(self.calculate_costs(rhs));

                self.merge_costs(costs)
            }
        }
    }

    pub fn merge_costs(&self, costs: Vec<CoinCost>) -> Vec<CoinCost> {
        use std::collections::HashMap;

        let mut merged: HashMap<CoinType, u32> = HashMap::new();
        for cost in costs {
            *merged.entry(cost.coin_type).or_insert(0) += cost.amt;
        }

        merged
            .into_iter()
            .map(|(coin_type, amt)| CoinCost { coin_type, amt })
            .collect()
    }

    pub fn coin_manager(&self) -> &CoinManager {
        &self.coin_manager
    }

    pub fn coin_manager_mut(&mut self) -> &mut CoinManager {
        &mut self.coin_manager
    }
}

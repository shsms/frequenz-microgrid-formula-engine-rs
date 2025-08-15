// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::traits::NumberLike;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::str::FromStr;

use crate::{error::FormulaError, expression::Expr, parser};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ComponentWithMetric<M> {
    pub component_id: u64,
    pub metric: M,
}

impl<M> std::fmt::Display for ComponentWithMetric<M>
where
    M: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.component_id, self.metric)
    }
}

impl<M: FromStr + std::fmt::Display> FromStr for ComponentWithMetric<M>
where
    <M as FromStr>::Err: std::fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        println!("Parsing component with metric: {}", s);
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err("Invalid format, expected 'component_id.metric'".to_string());
        }
        let component_id = parts[0]
            .parse::<u64>()
            .map_err(|_| "Invalid component ID".to_string())?;
        let metric = parts[1].to_string();
        Ok(Self {
            component_id,
            metric: metric
                .parse()
                .map_err(|e| format!("Invalid metric: {}", e))?,
        })
    }
}

/// FormulaEngine holds the parsed expression and can calculate the result
/// based on the provided component values.
#[derive(Debug)]
pub struct FormulaEngine<T, C = u64> {
    expr: Expr<T, C>,
    components: HashSet<C>,
}

impl<T: FromStr + NumberLike<T> + PartialOrd, C> FormulaEngine<T, C>
where
    <T as FromStr>::Err: Debug,
    C: std::hash::Hash + Eq + FromStr + Clone,
    <C as FromStr>::Err: Debug,
{
    /// Create a new FormulaEngine from a formula string.
    pub fn try_new(s: &str) -> Result<Self, FormulaError> {
        let expr = parser::parse(s)?;

        let components = expr.components();

        Ok(Self { expr, components })
    }

    /// Get the components of the formula.
    pub fn components(&self) -> &HashSet<C> {
        &self.components
    }

    /// Calculate the result of the formula based on the provided component values.
    pub fn calculate(&self, values: &HashMap<C, Option<T>>) -> Result<Option<T>, FormulaError> {
        self.expr.calculate(values)
    }
}

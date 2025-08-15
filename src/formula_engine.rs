// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::traits::NumberLike;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::str::FromStr;

use crate::{error::FormulaError, expression::Expr, parser};

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

// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::{
    parser,
    traits::{MetricStreamFetcher, NumberLike},
};
use std::collections::HashSet;
use std::fmt::Debug;
use std::str::FromStr;

use crate::{error::FormulaError, expression::Expr};

/// FormulaEngine holds the parsed expression and can calculate the result
/// based on the provided component values.
#[derive(Debug)]
pub struct FormulaEngine<T, S: Iterator<Item = Option<T>>> {
    expr: Expr<T, S>,
    components: HashSet<usize>,
}

impl<'a, T: FromStr + NumberLike<T> + PartialOrd, S> FormulaEngine<T, S>
where
    <T as FromStr>::Err: Debug,
    S: Iterator<Item = Option<T>>,
{
    /// Create a new FormulaEngine from a formula string.
    pub fn try_new<M>(formula: &'a str, metric_stream_fetcher: &mut M) -> Result<Self, FormulaError>
    where
        M: MetricStreamFetcher<T, S>,
    {
        let expr = parser::parse(formula, metric_stream_fetcher)?;
        let components = expr.components();

        Ok(Self { expr, components })
    }

    /// Get the components of the formula.
    pub fn components(&self) -> &HashSet<usize> {
        &self.components
    }
}

impl<T, S> Iterator for FormulaEngine<T, S>
where
    T: NumberLike<T> + PartialOrd,
    S: Iterator<Item = Option<T>>,
{
    type Item = Option<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.expr.next()
    }
}

// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use rand::Rng;
use std::{
    collections::HashMap,
    ops::{Add, Sub},
    vec,
};

use crate::{formula_engine::FormulaEngine, traits::MetricStreamFetcher, FormulaError};

fn max<T>(a: OptionW<T>, b: OptionW<T>) -> OptionW<T>
where
    T: PartialOrd,
{
    OptionW(match (a.inner(), b.inner()) {
        (Some(a), Some(b)) => {
            if a > b {
                Some(a)
            } else {
                Some(b)
            }
        }
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    })
}

fn min<T>(a: OptionW<T>, b: OptionW<T>) -> OptionW<T>
where
    T: PartialOrd,
{
    OptionW(match (a.inner(), b.inner()) {
        (Some(a), Some(b)) => {
            if a < b {
                Some(a)
            } else {
                Some(b)
            }
        }
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    })
}

fn coalesce<T>(list: Vec<OptionW<T>>) -> OptionW<T> {
    list.into_iter()
        .find(|x| x.is_some())
        .unwrap_or(OptionW(None))
}

#[derive(Debug, Clone)]
struct OptionW<T>(Option<T>);

impl<T> OptionW<T> {
    fn inner(self) -> Option<T> {
        self.0
    }

    fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl Add for OptionW<f32> {
    type Output = OptionW<f32>;

    fn add(self, other: Self) -> Self::Output {
        OptionW(match (self.inner(), other.inner()) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        })
    }
}

impl Sub for OptionW<f32> {
    type Output = OptionW<f32>;

    fn sub(self, other: Self) -> Self::Output {
        OptionW(match (self.inner(), other.inner()) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        })
    }
}

struct TestStream {
    start: Option<f32>,
    increment: Option<f32>,
}

impl TestStream {
    fn new(start: Option<f32>, increment: Option<f32>) -> Self {
        Self { start, increment }
    }
}

impl Iterator for TestStream {
    type Item = Option<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.start;
        self.start = if let (Some(start), Some(increment)) = (self.start, self.increment) {
            Some(start + increment)
        } else {
            None
        };

        Some(next)
    }
}

type FetcherFnPtr = fn(usize) -> Option<TestStream>;

#[derive(Default)]
struct TestFetcher<F: FnMut(usize) -> Option<TestStream>> {
    fetcher: Option<F>,
}

impl<F: FnMut(usize) -> Option<TestStream>> TestFetcher<F> {
    fn new(fetcher: Option<F>) -> Self {
        Self { fetcher }
    }
}

impl<F: FnMut(usize) -> Option<TestStream>> MetricStreamFetcher<f32, TestStream>
    for TestFetcher<F>
{
    fn from_component_id(&mut self, id: usize) -> Option<TestStream> {
        if let Some(vv) = self.fetcher.as_mut().map(|fetcher| fetcher(id)) {
            vv
        } else {
            None
        }
    }
}

#[test]
fn test_parse_addition() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1 + 1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. + 1.);
}

#[test]
fn test_parse_multiplication() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "0.9 * 1.1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 0.9 * 1.1);
}

#[test]
fn test_parse_subtraction() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1 - 1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. - 1.);
}

#[test]
fn test_parse_division() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1 / 1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. / 1.);
}

#[test]
fn test_parse_addition_whitespace() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1+1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. + 1.);
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1+ 1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. + 1.);
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1 +1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. + 1.);
}

#[test]
fn test_combination() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "1 + 1 * 2",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1. + 1. * 2.);
}

#[test]
fn test_combination_mul_add() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "2 * 1 + 2",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 2. * 1. + 2.);
}

#[test]
fn test_negative_value() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "-1",
        &mut TestFetcher::new(None::<FetcherFnPtr>),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), -1.);
}

#[test]
fn test_placeholder() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "#0",
        &mut TestFetcher::new(Some(|_| Some(TestStream::new(Some(1.), None)))),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1.);
}

#[test]
fn test_negative_placeholder() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "-#0",
        &mut TestFetcher::new(Some(|_| Some(TestStream::new(Some(1.), None)))),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), -1.);
}

#[test]
fn test_invalid_placeholder() {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| match id {
        0 => Some(TestStream::new(Some(1.), Some(1.))),
        _ => None,
    }));
    assert!(
        FormulaEngine::<f32, TestStream>::try_new("#1", metric_stream_fetcher)
            .is_err_and(|e| e == FormulaError("Unknown component id: 1".to_string()))
    );
}

#[test]
fn test_placeholder_addition() {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| match id {
        0 => Some(TestStream::new(Some(1.), Some(1.))),
        1 => Some(TestStream::new(Some(2.), Some(1.))),
        _ => None,
    }));
    let mut fe =
        FormulaEngine::<f32, TestStream>::try_new("#0 + #1", metric_stream_fetcher).unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 3.);
}

#[test]
fn test_calculating_with_nones() {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| {
        if id == 0 {
            Some(TestStream::new(Some(1.), Some(1.)))
        } else {
            Some(TestStream::new(None, Some(1.)))
        }
    }));
    let mut fe =
        FormulaEngine::<f32, TestStream>::try_new("#0 + #1", metric_stream_fetcher).unwrap();
    assert!(fe.calculate().unwrap().is_none());
}

#[test]
fn test_function_coalesce() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "COALESCE(#0, #1,#2)",
        &mut TestFetcher::new(Some(|id| match id {
            0 => Some(TestStream::new(None, None)),
            1 => Some(TestStream::new(Some(1.), None)),
            2 => Some(TestStream::new(Some(2.), None)),
            _ => None,
        })),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1.);
}

#[test]
fn test_function_min() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "MIN(#0, #1,#2)",
        &mut TestFetcher::new(Some(|id| match id {
            0 => Some(TestStream::new(Some(3.), None)),
            1 => Some(TestStream::new(Some(1.), None)),
            2 => Some(TestStream::new(Some(2.), None)),
            _ => None,
        })),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1.);
}

#[test]
fn test_function_min_none() {
    let mut fe = FormulaEngine::<f32, TestStream>::try_new(
        "MIN(#0, #1,#2)",
        &mut TestFetcher::new(Some(|id| match id {
            0 => Some(TestStream::new(None, None)),
            1 => Some(TestStream::new(Some(1.), None)),
            2 => Some(TestStream::new(Some(2.), None)),
            _ => None,
        })),
    )
    .unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 1.);
}

#[test]
fn test_function_max() {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| match id {
        0 => Some(TestStream::new(Some(3.), None)),
        1 => Some(TestStream::new(Some(1.), None)),
        2 => Some(TestStream::new(Some(2.), None)),
        _ => None,
    }));
    let mut fe =
        FormulaEngine::<f32, TestStream>::try_new("MAX(#0, #1,#2)", metric_stream_fetcher).unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 3.);
}

#[test]
fn test_function_max_none() {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| match id {
        0 | 1 => Some(TestStream::new(None, None)),
        2 => Some(TestStream::new(Some(2.), None)),
        _ => None,
    }));
    let mut fe =
        FormulaEngine::<f32, TestStream>::try_new("MAX(#0, #1,#2)", metric_stream_fetcher).unwrap();
    assert_eq!(fe.calculate().unwrap().unwrap(), 2.);
}

#[test]
fn test_components_getter_op() {
    let fe = FormulaEngine::<f32, TestStream>::try_new(
        "#0 + #1",
        &mut TestFetcher::new(Some(|_| Some(TestStream::new(Some(1.), None)))),
    )
    .unwrap();
    assert_eq!(fe.components(), &vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_neg() {
    let fe = FormulaEngine::<f32, TestStream>::try_new(
        "#0 + (-#1)",
        &mut TestFetcher::new(Some(|_| Some(TestStream::new(Some(1.), None)))),
    )
    .unwrap();
    assert_eq!(fe.components(), &vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_function() {
    let fe = FormulaEngine::<f32, TestStream>::try_new(
        "-MAX(#0, #1)",
        &mut TestFetcher::new(Some(|_| Some(TestStream::new(Some(1.), None)))),
    )
    .unwrap();
    assert_eq!(fe.components(), &vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_function_function() {
    let fe = FormulaEngine::<f32, TestStream>::try_new(
        "MAX(#0, COALESCE(#1, #2))",
        &mut TestFetcher::new(Some(|_| Some(TestStream::new(Some(1.), None)))),
    )
    .unwrap();
    assert_eq!(fe.components(), &vec![0, 1, 2].into_iter().collect());
}

fn test_large_microgrid_formula(components: HashMap<usize, Option<f32>>) {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| {
        components
            .get(&id)
            .map(|value| TestStream::new(value.clone(), None))
    }));
    let formula_result = FormulaEngine::try_new(
        concat!(
            "MIN(0.0, COALESCE(#4 + #3, #2, COALESCE(#4, 0.0) + COALESCE(#3, 0.0))) + ",
            "MIN(0.0, COALESCE(#6, #5, 0.0)) + ",
            "MIN(0.0, COALESCE(#7, 0.0))"
        ),
        metric_stream_fetcher,
    )
    .unwrap()
    .calculate()
    .unwrap();

    let expected_result = min(
        OptionW(Some(0.0)),
        coalesce(vec![
            OptionW(components.get(&4).unwrap().clone())
                + OptionW(components.get(&3).unwrap().clone()),
            OptionW(components.get(&2).unwrap().clone()),
            coalesce(vec![
                OptionW(components.get(&4).unwrap().clone()),
                OptionW(Some(0.0)),
            ]) + coalesce(vec![
                OptionW(components.get(&3).unwrap().clone()),
                OptionW(Some(0.0)),
            ]),
        ]),
    ) + min(
        OptionW(Some(0.0)),
        coalesce(vec![
            OptionW(components.get(&6).unwrap().clone()),
            OptionW(components.get(&5).unwrap().clone()),
            OptionW(Some(0.0)),
        ]),
    ) + min(
        OptionW(Some(0.0)),
        coalesce(vec![
            OptionW(components.get(&7).unwrap().clone()),
            OptionW(Some(0.0)),
        ]),
    );

    assert_eq!(formula_result, expected_result.inner());
}

#[test]
fn test_large_microgrid_formula_fuzz() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut components = HashMap::new();
        for i in 2..8 {
            let value = if rng.gen_bool(0.5) {
                Some(0.5 - rng.gen::<f32>())
            } else {
                None
            };
            components.insert(i, value);
        }
        test_large_microgrid_formula(components);
    }
}

fn test_large_microgrid_formula_2(components: HashMap<usize, Option<f32>>) {
    let metric_stream_fetcher = &mut TestFetcher::new(Some(|id| {
        components
            .get(&id)
            .map(|value| TestStream::new(value.clone(), None))
    }));
    let formula_result = FormulaEngine::try_new(
        concat!(
            "MAX(0.0, #1 - COALESCE(#2, #3, 0.0) - ",
            "COALESCE(#5, COALESCE(#7, 0.0) + COALESCE(#6, 0.0))) + ",
            "COALESCE(MAX(0.0, #2 - #3), 0.0) + COALESCE(MAX(0.0, #5 - #6 - #7), 0.0)",
        ),
        metric_stream_fetcher,
    )
    .unwrap()
    .calculate()
    .unwrap();

    let expected_result = max(
        OptionW(Some(0.0)),
        OptionW(components.get(&1).unwrap().clone())
            - coalesce(vec![
                OptionW(components.get(&2).unwrap().clone()),
                OptionW(components.get(&3).unwrap().clone()),
                OptionW(Some(0.0)),
            ])
            - coalesce(vec![
                OptionW(components.get(&5).unwrap().clone()),
                coalesce(vec![
                    OptionW(components.get(&7).unwrap().clone()),
                    OptionW(Some(0.0)),
                ]) + coalesce(vec![
                    OptionW(components.get(&6).unwrap().clone()),
                    OptionW(Some(0.0)),
                ]),
            ]),
    ) + coalesce(vec![
        max(
            OptionW(Some(0.0)),
            OptionW(components.get(&2).unwrap().clone())
                - OptionW(components.get(&3).unwrap().clone()),
        ),
        OptionW(Some(0.0)),
    ]) + coalesce(vec![
        max(
            OptionW(Some(0.0)),
            OptionW(components.get(&5).unwrap().clone())
                - OptionW(components.get(&6).unwrap().clone())
                - OptionW(components.get(&7).unwrap().clone()),
        ),
        OptionW(Some(0.0)),
    ]);

    assert_eq!(formula_result, expected_result.inner());
}

#[test]
fn test_large_microgrid_formula_2_fuzz() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut components = HashMap::new();
        for i in 1..8 {
            let value = if rng.gen_bool(0.5) {
                Some(0.5 - rng.gen::<f32>())
            } else {
                None
            };
            components.insert(i, value);
        }
        test_large_microgrid_formula_2(components);
    }
}

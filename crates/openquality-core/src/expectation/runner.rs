use chrono::Utc;
use polars::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::error::{OpenQualityError, Result};
use crate::types::*;

pub struct ExpectationRunner;

impl ExpectationRunner {
    pub fn run(suite: &ExpectationSuite, df: &DataFrame) -> Result<SuiteResult> {
        let mut results = Vec::with_capacity(suite.expectations.len());
        for exp in &suite.expectations {
            let result = Self::run_single(exp, df)?;
            results.push(result);
        }
        let total = results.len();
        let passed = results.iter().filter(|r| r.success).count();
        let failed = total - passed;
        let success_percent = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            100.0
        };
        Ok(SuiteResult {
            suite_name: suite.name.clone(),
            results,
            success: failed == 0,
            run_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            summary: SuiteSummary {
                total,
                passed,
                failed,
                success_percent,
            },
        })
    }

    fn run_single(exp: &Expectation, df: &DataFrame) -> Result<ExpectationResult> {
        let col_name = match &exp.column {
            Some(c) => c.clone(),
            None => String::new(),
        };
        let build = |success: bool, observed: serde_json::Value, expected: serde_json::Value| {
            ExpectationResult {
                expectation_id: exp.id,
                expectation_type: exp.expectation_type.clone(),
                column: exp.column.clone(),
                success,
                observed_value: observed,
                expected_value: expected,
                details: HashMap::new(),
                exception_info: None,
            }
        };

        match &exp.expectation_type {
            ExpectationType::NotNull => {
                let col = Self::get_col(df, &col_name)?;
                let nulls = col.null_count();
                let total = col.len();
                let threshold = ((exp.tolerance * 100.0) as usize).min(total);
                let success = nulls <= threshold;
                Ok(build(
                    success,
                    serde_json::json!({"null_count": nulls, "total": total, "null_pct": (nulls as f64 / total as f64) * 100.0}),
                    serde_json::json!({"tolerance": exp.tolerance, "max_nulls": threshold}),
                ))
            }
            ExpectationType::Unique => {
                let col = Self::get_col(df, &col_name)?;
                let distinct = col.n_unique()?;
                let total = col.len();
                let success = distinct == total;
                Ok(build(
                    success,
                    serde_json::json!({"distinct": distinct, "total": total, "duplicates": total - distinct}),
                    serde_json::json!({"distinct": total}),
                ))
            }
            ExpectationType::Between(min, max) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                let out_of_range = values.iter().filter(|v| *v < min || *v > max).count();
                let success = out_of_range == 0;
                Ok(build(
                    success,
                    serde_json::json!({"out_of_range": out_of_range, "total": values.len()}),
                    serde_json::json!({"min": min, "max": max}),
                ))
            }
            ExpectationType::RowCountBetween(min, max) => {
                let n = df.height() as u64;
                let success = n >= *min && n <= *max;
                Ok(build(
                    success,
                    serde_json::json!({"row_count": n}),
                    serde_json::json!({"min": min, "max": max}),
                ))
            }
            ExpectationType::ColumnMeanBetween(min, max) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                if values.is_empty() {
                    return Ok(build(
                        false,
                        serde_json::json!({"mean": null, "count": 0}),
                        serde_json::json!({"min": min, "max": max}),
                    ));
                }
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let success = mean >= *min && mean <= *max;
                Ok(build(
                    success,
                    serde_json::json!({"mean": mean, "count": values.len()}),
                    serde_json::json!({"min": min, "max": max}),
                ))
            }
            ExpectationType::ColumnStddevBetween(min, max) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                if values.len() < 2 {
                    return Ok(build(
                        false,
                        serde_json::json!({"stddev": null, "count": values.len()}),
                        serde_json::json!({"min": min, "max": max}),
                    ));
                }
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / (values.len() - 1) as f64;
                let stddev = variance.sqrt();
                let success = stddev >= *min && stddev <= *max;
                Ok(build(
                    success,
                    serde_json::json!({"stddev": stddev, "mean": mean, "count": values.len()}),
                    serde_json::json!({"min": min, "max": max}),
                ))
            }
            ExpectationType::ColumnMinBetween(min, max) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                if values.is_empty() {
                    return Ok(build(
                        false,
                        serde_json::json!({"min": null, "count": 0}),
                        serde_json::json!({"min": min, "max": max}),
                    ));
                }
                let col_min = values.iter().cloned().fold(f64::MAX, f64::min);
                let success = col_min >= *min && col_min <= *max;
                Ok(build(
                    success,
                    serde_json::json!({"min": col_min, "count": values.len()}),
                    serde_json::json!({"expected_min": min, "expected_max": max}),
                ))
            }
            ExpectationType::ColumnMaxBetween(min, max) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                if values.is_empty() {
                    return Ok(build(
                        false,
                        serde_json::json!({"max": null, "count": 0}),
                        serde_json::json!({"min": min, "max": max}),
                    ));
                }
                let col_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let success = col_max >= *min && col_max <= *max;
                Ok(build(
                    success,
                    serde_json::json!({"max": col_max, "count": values.len()}),
                    serde_json::json!({"expected_min": min, "expected_max": max}),
                ))
            }
            ExpectationType::MatchRegex(pattern) => {
                let col = Self::get_col(df, &col_name)?;
                let str_col = col.str()?;
                let re = regex_lite::Regex::new(pattern)
                    .map_err(|e| OpenQualityError::InvalidExpectation(format!("Bad regex: {e}")))?;
                let total = str_col.len();
                let mismatches = str_col
                    .into_iter()
                    .flatten()
                    .filter(|v| !re.is_match(v))
                    .count();
                let success = mismatches == 0;
                Ok(build(
                    success,
                    serde_json::json!({"mismatches": mismatches, "total": total}),
                    serde_json::json!({"pattern": pattern}),
                ))
            }
            ExpectationType::NotMatchRegex(pattern) => {
                let col = Self::get_col(df, &col_name)?;
                let str_col = col.str()?;
                let re = regex_lite::Regex::new(pattern)
                    .map_err(|e| OpenQualityError::InvalidExpectation(format!("Bad regex: {e}")))?;
                let total = str_col.len();
                let matches = str_col
                    .into_iter()
                    .flatten()
                    .filter(|v| re.is_match(v))
                    .count();
                let success = matches == 0;
                Ok(build(
                    success,
                    serde_json::json!({"matches": matches, "total": total}),
                    serde_json::json!({"pattern": pattern}),
                ))
            }
            ExpectationType::DistinctValuesEqualSet(expected_set) => {
                let col = Self::get_col(df, &col_name)?;
                let str_col = col.str()?;
                let actual: HashSet<String> = str_col
                    .into_iter()
                    .flatten()
                    .map(|s| s.to_string())
                    .collect();
                let expected: HashSet<String> = expected_set.iter().cloned().collect();
                let success = actual == expected;
                let missing: Vec<String> = expected.difference(&actual).cloned().collect();
                let extra: Vec<String> = actual.difference(&expected).cloned().collect();
                Ok(build(
                    success,
                    serde_json::json!({"actual_distinct": actual.iter().cloned().collect::<Vec<_>>(), "missing": missing, "extra": extra}),
                    serde_json::json!({"expected_set": expected_set}),
                ))
            }
            ExpectationType::DistinctValuesContainedInSet(super_set) => {
                let col = Self::get_col(df, &col_name)?;
                let str_col = col.str()?;
                let actual: HashSet<String> = str_col
                    .into_iter()
                    .flatten()
                    .map(|s| s.to_string())
                    .collect();
                let allowed: HashSet<String> = super_set.iter().cloned().collect();
                let outside: Vec<String> = actual.difference(&allowed).cloned().collect();
                let success = outside.is_empty();
                Ok(build(
                    success,
                    serde_json::json!({"actual_distinct": actual.iter().cloned().collect::<Vec<_>>(), "outside_set": outside}),
                    serde_json::json!({"allowed_set": super_set}),
                ))
            }
            ExpectationType::ColumnValuesToBeInSet(allowed_set) => {
                let col = Self::get_col(df, &col_name)?;
                let str_col = col.str()?;
                let allowed: HashSet<String> = allowed_set.iter().cloned().collect();
                let mut outside = Vec::new();
                for v in str_col.into_iter().flatten() {
                    if !allowed.contains(v) {
                        outside.push(v.to_string());
                    }
                }
                let success = outside.is_empty();
                Ok(build(
                    success,
                    serde_json::json!({"outside_set_count": outside.len(), "outside_values": outside}),
                    serde_json::json!({"allowed_set": allowed_set}),
                ))
            }
            ExpectationType::ColumnKLDivergenceLessThan(max_divergence) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                if values.is_empty() {
                    return Ok(build(
                        false,
                        serde_json::json!({"kl_divergence": null, "count": 0}),
                        serde_json::json!({"max_divergence": max_divergence}),
                    ));
                }
                let n = values.len() as f64;
                let mean = values.iter().sum::<f64>() / n;
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
                let stddev = variance.sqrt().max(1e-10);

                let nbins = (n.sqrt().round() as usize).clamp(10, 100);
                let bin_width = (4.0 * stddev) / nbins as f64;
                let center = mean;
                let bins: Vec<f64> = (0..nbins)
                    .map(|i| center - 2.0 * stddev + (i as f64 + 0.5) * bin_width)
                    .collect();

                let observed: Vec<f64> = bins
                    .iter()
                    .map(|b| {
                        let lo = b - bin_width / 2.0;
                        let hi = b + bin_width / 2.0;
                        values.iter().filter(|v| **v >= lo && **v < hi).count() as f64
                    })
                    .collect();
                let o_sum: f64 = observed.iter().sum();
                let observed_norm: Vec<f64> =
                    observed.into_iter().map(|v| v / o_sum.max(1.0)).collect();

                let expected_norm: Vec<f64> = bins
                    .iter()
                    .map(|b| {
                        let z = (b - mean) / stddev;
                        (-0.5 * z * z).exp() / (stddev * (2.0 * std::f64::consts::PI).sqrt())
                            * bin_width
                    })
                    .collect();
                let e_sum: f64 = expected_norm.iter().sum();
                let expected_norm: Vec<f64> = expected_norm
                    .into_iter()
                    .map(|v| v / e_sum.max(1e-10))
                    .collect();

                let kl: f64 = observed_norm
                    .iter()
                    .zip(expected_norm.iter())
                    .map(|(o, e)| {
                        if *o > 0.0 && *e > 0.0 {
                            o * (o / e).ln()
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let success = kl <= *max_divergence;
                Ok(build(
                    success,
                    serde_json::json!({"kl_divergence": kl, "bins": nbins}),
                    serde_json::json!({"max_divergence": max_divergence}),
                ))
            }
            ExpectationType::ColumnQuantileBetween(quantile, low, high) => {
                let values = Self::get_numeric_vals(df, &col_name)?;
                if values.is_empty() {
                    return Ok(build(
                        false,
                        serde_json::json!({"quantile_value": null, "count": 0}),
                        serde_json::json!({"quantile": quantile, "low": low, "high": high}),
                    ));
                }
                let mut sorted = values.clone();
                sorted
                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = ((sorted.len() as f64 - 1.0) * quantile).round() as usize;
                let q_val = sorted[idx.min(sorted.len() - 1)];
                let success = q_val >= *low && q_val <= *high;
                Ok(build(
                    success,
                    serde_json::json!({"quantile_value": q_val, "quantile": quantile, "count": values.len()}),
                    serde_json::json!({"quantile": quantile, "low": low, "high": high}),
                ))
            }
            ExpectationType::TableColumnsMatchOrderedList(expected) => {
                let actual: Vec<String> = df
                    .get_column_names()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let success = &actual == expected;
                Ok(build(
                    success,
                    serde_json::json!({"actual_columns": actual}),
                    serde_json::json!({"expected_columns": expected}),
                ))
            }
            ExpectationType::Custom(_name, _config) => Ok(build(
                true,
                serde_json::json!({"note": "custom expectation — no-op in default runner"}),
                serde_json::json!({"type": "custom"}),
            )),
        }
    }

    fn get_col<'a>(df: &'a DataFrame, name: &str) -> Result<&'a Column> {
        df.column(name)
            .map_err(|_| OpenQualityError::InvalidExpectation(format!("Column '{name}' not found")))
    }

    fn get_numeric_vals(df: &DataFrame, name: &str) -> Result<Vec<f64>> {
        let col = Self::get_col(df, name)?;
        if let Ok(ca) = col.f64() {
            return Ok(ca.into_iter().flatten().collect());
        }
        if let Ok(ca) = col.i32() {
            return Ok(ca.into_iter().flatten().map(|v| v as f64).collect());
        }
        if let Ok(ca) = col.i64() {
            return Ok(ca.into_iter().flatten().map(|v| v as f64).collect());
        }
        if let Ok(ca) = col.u32() {
            return Ok(ca.into_iter().flatten().map(|v| v as f64).collect());
        }
        Err(OpenQualityError::InvalidExpectation(format!(
            "Column '{name}' is not numeric"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_df() -> DataFrame {
        df!(
            "name" => &["alice", "bob", "charlie", "diana", "eve"],
            "age" => &[25.0, 30.0, 35.0, 40.0, 45.0],
            "salary" => &[50000.0, 60000.0, 70000.0, 80000.0, 90000.0],
            "dept" => &["eng", "eng", "sales", "sales", "eng"],
        )
        .unwrap()
    }

    #[test]
    fn test_not_null() {
        let df = test_df();
        let suite = ExpectationSuite::new("test")
            .with(Expectation::new(ExpectationType::NotNull, Some("name")));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_unique() {
        let df = test_df();
        let suite = ExpectationSuite::new("test")
            .with(Expectation::new(ExpectationType::Unique, Some("name")));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_between() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::Between(20.0, 50.0),
            Some("age"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_column_min_between() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::ColumnMinBetween(20.0, 30.0),
            Some("age"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_column_max_between() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::ColumnMaxBetween(40.0, 50.0),
            Some("age"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_not_match_regex() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::NotMatchRegex(r"^\d+$".to_string()),
            Some("name"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_distinct_values_equal_set() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::DistinctValuesEqualSet(vec![
                "alice".into(),
                "bob".into(),
                "charlie".into(),
                "diana".into(),
                "eve".into(),
            ]),
            Some("name"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_distinct_values_contained_in_set() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::DistinctValuesContainedInSet(vec![
                "alice".into(),
                "bob".into(),
                "charlie".into(),
                "diana".into(),
                "eve".into(),
                "frank".into(),
            ]),
            Some("name"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_values_in_set() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::ColumnValuesToBeInSet(vec!["eng".into(), "sales".into()]),
            Some("dept"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_quantile_between() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::ColumnQuantileBetween(0.5, 30.0, 40.0),
            Some("age"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_kl_divergence() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::ColumnKLDivergenceLessThan(1.0),
            Some("age"),
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_row_count() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::RowCountBetween(1, 10),
            None,
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_table_columns_match() {
        let df = test_df();
        let suite = ExpectationSuite::new("test").with(Expectation::new(
            ExpectationType::TableColumnsMatchOrderedList(vec![
                "name".into(),
                "age".into(),
                "salary".into(),
                "dept".into(),
            ]),
            None,
        ));
        let result = ExpectationRunner::run(&suite, &df).unwrap();
        assert!(result.success);
    }
}

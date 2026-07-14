use crate::expectation::suite::ExpectationSuite;
use crate::types::Expectation;

pub struct BuiltinExpectations;

impl BuiltinExpectations {
    pub fn add_not_null(suite: &mut ExpectationSuite, column: &str) {
        suite.add(Expectation::new(ExpectationType::NotNull, Some(column)));
    }

    pub fn add_unique(suite: &mut ExpectationSuite, column: &str) {
        suite.add(Expectation::new(ExpectationType::Unique, Some(column)));
    }

    pub fn add_between(suite: &mut ExpectationSuite, column: &str, min: f64, max: f64) {
        suite.add(Expectation::new(
            ExpectationType::Between(min, max),
            Some(column),
        ));
    }

    pub fn add_match_regex(suite: &mut ExpectationSuite, column: &str, pattern: &str) {
        suite.add(Expectation::new(
            ExpectationType::MatchRegex(pattern.to_string()),
            Some(column),
        ));
    }

    pub fn add_not_match_regex(suite: &mut ExpectationSuite, column: &str, pattern: &str) {
        suite.add(Expectation::new(
            ExpectationType::NotMatchRegex(pattern.to_string()),
            Some(column),
        ));
    }

    pub fn add_row_count_between(suite: &mut ExpectationSuite, min: u64, max: u64) {
        suite.add(Expectation::new(
            ExpectationType::RowCountBetween(min, max),
            None,
        ));
    }

    pub fn add_column_mean_between(suite: &mut ExpectationSuite, column: &str, min: f64, max: f64) {
        suite.add(Expectation::new(
            ExpectationType::ColumnMeanBetween(min, max),
            Some(column),
        ));
    }

    pub fn add_column_stddev_between(
        suite: &mut ExpectationSuite,
        column: &str,
        min: f64,
        max: f64,
    ) {
        suite.add(Expectation::new(
            ExpectationType::ColumnStddevBetween(min, max),
            Some(column),
        ));
    }

    pub fn add_column_min_between(suite: &mut ExpectationSuite, column: &str, min: f64, max: f64) {
        suite.add(Expectation::new(
            ExpectationType::ColumnMinBetween(min, max),
            Some(column),
        ));
    }

    pub fn add_column_max_between(suite: &mut ExpectationSuite, column: &str, min: f64, max: f64) {
        suite.add(Expectation::new(
            ExpectationType::ColumnMaxBetween(min, max),
            Some(column),
        ));
    }

    pub fn add_distinct_values_equal_set(
        suite: &mut ExpectationSuite,
        column: &str,
        values: &[&str],
    ) {
        let vals: Vec<String> = values.iter().map(|s| s.to_string()).collect();
        suite.add(Expectation::new(
            ExpectationType::DistinctValuesEqualSet(vals),
            Some(column),
        ));
    }

    pub fn add_distinct_values_contained_in_set(
        suite: &mut ExpectationSuite,
        column: &str,
        values: &[&str],
    ) {
        let vals: Vec<String> = values.iter().map(|s| s.to_string()).collect();
        suite.add(Expectation::new(
            ExpectationType::DistinctValuesContainedInSet(vals),
            Some(column),
        ));
    }

    pub fn add_values_in_set(suite: &mut ExpectationSuite, column: &str, values: &[&str]) {
        let vals: Vec<String> = values.iter().map(|s| s.to_string()).collect();
        suite.add(Expectation::new(
            ExpectationType::ColumnValuesToBeInSet(vals),
            Some(column),
        ));
    }

    pub fn add_columns_match_ordered(suite: &mut ExpectationSuite, columns: &[&str]) {
        let cols: Vec<String> = columns.iter().map(|s| s.to_string()).collect();
        suite.add(Expectation::new(
            ExpectationType::TableColumnsMatchOrderedList(cols),
            None,
        ));
    }

    pub fn add_quantile_between(
        suite: &mut ExpectationSuite,
        column: &str,
        quantile: f64,
        low: f64,
        high: f64,
    ) {
        suite.add(Expectation::new(
            ExpectationType::ColumnQuantileBetween(quantile, low, high),
            Some(column),
        ));
    }

    pub fn add_kl_divergence_less_than(
        suite: &mut ExpectationSuite,
        column: &str,
        max_divergence: f64,
    ) {
        suite.add(Expectation::new(
            ExpectationType::ColumnKLDivergenceLessThan(max_divergence),
            Some(column),
        ));
    }
}

// Re-export ExpectationType for the helper methods
use crate::types::ExpectationType;

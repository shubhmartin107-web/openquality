use crate::IntegrationError;
use openquality_core::types::ExpectationType;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GeSuite {
    #[serde(rename = "expectation_suite_name")]
    pub name: Option<String>,
    pub expectations: Vec<GeExpectation>,
    pub meta: Option<Value>,
    #[serde(rename = "data_asset_type")]
    pub data_asset_type: Option<String>,
}

use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct GeExpectation {
    #[serde(rename = "expectation_type")]
    pub expectation_type: String,
    pub kwargs: GeKwargs,
    pub meta: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeKwargs {
    pub column: Option<String>,
    pub value: Option<Value>,
    #[serde(rename = "min_value")]
    pub min_value: Option<Value>,
    #[serde(rename = "max_value")]
    pub max_value: Option<Value>,
    #[serde(rename = "mostly")]
    pub mostly: Option<f64>,
    #[serde(rename = "value_set")]
    pub value_set: Option<Vec<Value>>,
    #[serde(rename = "threshold")]
    pub threshold: Option<f64>,
    #[serde(rename = "method")]
    pub method: Option<String>,
    #[serde(rename = "result_format")]
    pub result_format: Option<String>,
    #[serde(rename = "column_list")]
    pub column_list: Option<Vec<String>>,
    #[serde(rename = "value_pairs")]
    pub value_pairs: Option<Vec<String>>,
    #[serde(rename = "parse_strings_as_datetimes")]
    pub parse_strings_as_datetimes: Option<bool>,
    #[serde(rename = "mostly_pct")]
    pub mostly_pct: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranslatedExpectation {
    pub expectation_type: ExpectationType,
    pub column: Option<String>,
    pub kwargs: HashMap<String, Value>,
}

impl GeSuite {
    pub fn from_json(json: &str) -> Result<Self, IntegrationError> {
        serde_json::from_str(json).map_err(|e| IntegrationError::Parse(e.to_string()))
    }

    pub fn translate(&self) -> Vec<TranslatedExpectation> {
        self.expectations
            .iter()
            .filter_map(translate_expectation)
            .collect()
    }
}

fn translate_expectation(ge: &GeExpectation) -> Option<TranslatedExpectation> {
    let oe_type = match ge.expectation_type.as_str() {
        "expect_column_to_exist" => ExpectationType::NotNull,
        "expect_table_row_count_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as u64;
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as u64;
            ExpectationType::RowCountBetween(min, max)
        }
        "expect_column_values_to_not_be_null" => ExpectationType::NotNull,
        "expect_column_values_to_be_unique" => ExpectationType::Unique,
        "expect_column_values_to_be_in_set" => {
            let set = ge
                .kwargs
                .value_set
                .as_ref()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            ExpectationType::ColumnValuesToBeInSet(set)
        }
        "expect_column_values_to_not_be_in_set" => {
            let set = ge
                .kwargs
                .value_set
                .as_ref()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            ExpectationType::DistinctValuesContainedInSet(set)
        }
        "expect_column_distinct_values_to_equal_set" => {
            let set = ge
                .kwargs
                .value_set
                .as_ref()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            ExpectationType::DistinctValuesEqualSet(set)
        }
        "expect_column_distinct_values_to_be_contained_in_set" => {
            let set = ge
                .kwargs
                .value_set
                .as_ref()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            ExpectationType::DistinctValuesContainedInSet(set)
        }
        "expect_column_values_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            ExpectationType::Between(min, max)
        }
        "expect_column_min_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            ExpectationType::ColumnMinBetween(min, max)
        }
        "expect_column_max_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            ExpectationType::ColumnMaxBetween(min, max)
        }
        "expect_column_mean_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            ExpectationType::ColumnMeanBetween(min, max)
        }
        "expect_column_quantile_values_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            ExpectationType::ColumnQuantileBetween(min, max, 0.5)
        }
        "expect_column_stdev_to_be_between" => {
            let min = ge
                .kwargs
                .min_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let max = ge
                .kwargs
                .max_value
                .as_ref()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            ExpectationType::ColumnStddevBetween(min, max)
        }
        "expect_column_kl_divergence_to_be_less_than" => {
            let threshold = ge.kwargs.threshold.unwrap_or(0.1);
            ExpectationType::ColumnKLDivergenceLessThan(threshold)
        }
        "expect_column_values_to_match_regex" => {
            let pattern = ge
                .kwargs
                .value
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ExpectationType::MatchRegex(pattern)
        }
        "expect_column_values_to_not_match_regex" => {
            let pattern = ge
                .kwargs
                .value
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ExpectationType::NotMatchRegex(pattern)
        }
        "expect_table_columns_to_match_set" => {
            let cols = ge.kwargs.column_list.clone().unwrap_or_default();
            ExpectationType::TableColumnsMatchOrderedList(cols)
        }
        "expect_column_values_to_be_of_type" => {
            let type_name = ge
                .kwargs
                .value
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            ExpectationType::Custom("column_type".into(), serde_json::json!({"type": type_name}))
        }
        _ => return None,
    };

    let column = ge.kwargs.column.clone();

    Some(TranslatedExpectation {
        expectation_type: oe_type,
        column,
        kwargs: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_suite() {
        let json = r#"{"expectation_suite_name":"empty","expectations":[],"meta":{}}"#;
        let suite = GeSuite::from_json(json).unwrap();
        assert!(suite.translate().is_empty());
    }

    #[test]
    fn test_translate_not_null() {
        let json = r#"{
            "expectation_suite_name": "test",
            "expectations": [
                {
                    "expectation_type": "expect_column_values_to_not_be_null",
                    "kwargs": {"column": "id"},
                    "meta": {}
                }
            ],
            "meta": {}
        }"#;
        let suite = GeSuite::from_json(json).unwrap();
        let translated = suite.translate();
        assert_eq!(translated.len(), 1);
        assert_eq!(translated[0].column, Some("id".into()));
        assert!(matches!(
            translated[0].expectation_type,
            ExpectationType::NotNull
        ));
    }

    #[test]
    fn test_translate_row_count_between() {
        let json = r#"{
            "expectation_suite_name": "test",
            "expectations": [
                {
                    "expectation_type": "expect_table_row_count_to_be_between",
                    "kwargs": {"min_value": 100, "max_value": 10000},
                    "meta": {}
                }
            ],
            "meta": {}
        }"#;
        let suite = GeSuite::from_json(json).unwrap();
        let translated = suite.translate();
        assert_eq!(translated.len(), 1);
        assert!(matches!(
            translated[0].expectation_type,
            ExpectationType::RowCountBetween(100, 10000)
        ));
    }

    #[test]
    fn test_translate_multiple() {
        let json = r#"{
            "expectation_suite_name": "multi",
            "expectations": [
                {
                    "expectation_type": "expect_column_to_exist",
                    "kwargs": {"column": "email"},
                    "meta": {}
                },
                {
                    "expectation_type": "expect_column_values_to_be_unique",
                    "kwargs": {"column": "user_id"},
                    "meta": {}
                },
                {
                    "expectation_type": "expect_column_values_to_be_in_set",
                    "kwargs": {"column": "status", "value_set": ["active", "inactive"]},
                    "meta": {}
                },
                {
                    "expectation_type": "expect_column_values_to_match_regex",
                    "kwargs": {"column": "email", "value": "^[a-z@.]+$"},
                    "meta": {}
                }
            ],
            "meta": {}
        }"#;
        let suite = GeSuite::from_json(json).unwrap();
        let translated = suite.translate();
        assert_eq!(translated.len(), 4);
    }

    #[test]
    fn test_translate_unknown_is_skipped() {
        let json = r#"{
            "expectation_suite_name": "skip",
            "expectations": [
                {
                    "expectation_type": "expect_unknown_custom_thing",
                    "kwargs": {"column": "x"},
                    "meta": {}
                }
            ],
            "meta": {}
        }"#;
        let suite = GeSuite::from_json(json).unwrap();
        assert!(suite.translate().is_empty());
    }
}

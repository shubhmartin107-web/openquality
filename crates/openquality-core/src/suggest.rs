use crate::types::*;

pub struct SuggestionEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Suggestion {
    pub expectation_type: ExpectationType,
    pub column: Option<String>,
    pub confidence: f64,
    pub reason: String,
}

impl SuggestionEngine {
    pub fn suggest(profiles: &[ColumnProfile]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        for profile in profiles {
            Self::suggest_for_column(profile, &mut suggestions);
        }

        suggestions
    }

    fn suggest_for_column(profile: &ColumnProfile, suggestions: &mut Vec<Suggestion>) {
        let name = &profile.name;

        if profile.null_count > 0 {
            let null_pct = profile.null_count as f64 / profile.row_count.max(1) as f64;
            if null_pct < 0.05 {
                suggestions.push(Suggestion {
                    expectation_type: ExpectationType::NotNull,
                    column: Some(name.clone()),
                    confidence: 0.9 - (null_pct * 5.0).min(0.8),
                    reason: format!(
                        "{:.1}% null values — consider not_null expectation",
                        null_pct * 100.0
                    ),
                });
            }
        }

        if profile.distinct_count == profile.row_count && profile.row_count > 1 {
            suggestions.push(Suggestion {
                expectation_type: ExpectationType::Unique,
                column: Some(name.clone()),
                confidence: 0.8,
                reason: format!(
                    "All {} values are unique — consider unique expectation",
                    profile.row_count
                ),
            });
        }

        if let (Some(min), Some(max)) = (profile.min, profile.max) {
            let range = max - min;
            if range > 0.0 {
                suggestions.push(Suggestion {
                    expectation_type: ExpectationType::Between(min, max),
                    column: Some(name.clone()),
                    confidence: 0.6,
                    reason: format!(
                        "Values range [{:.2}, {:.2}] — consider between expectation",
                        min, max
                    ),
                });
            }

            if let Some(quantiles) = profile.quantiles {
                let iqr = quantiles[3] - quantiles[1];
                if iqr > 0.0 {
                    let low = quantiles[1] - 1.5 * iqr;
                    let high = quantiles[3] + 1.5 * iqr;
                    suggestions.push(Suggestion {
                        expectation_type: ExpectationType::Between(low, high),
                        column: Some(name.clone()),
                        confidence: 0.5,
                        reason: format!(
                            "IQR-based bounds [{:.2}, {:.2}] — consider between expectation",
                            low, high
                        ),
                    });
                }
            }

            if let Some(mean) = profile.mean {
                suggestions.push(Suggestion {
                    expectation_type: ExpectationType::ColumnMeanBetween(mean * 0.8, mean * 1.2),
                    column: Some(name.clone()),
                    confidence: 0.4,
                    reason: format!(
                        "Mean is {:.2} — consider column_mean_between(mean*0.8, mean*1.2)",
                        mean
                    ),
                });
            }
        }

        if profile.null_count == 0 && profile.distinct_count > 0 && profile.distinct_count <= 20 {
            if let (Some(_min), Some(_max)) = (profile.min, profile.max) {
                suggestions.push(Suggestion {
                    expectation_type: ExpectationType::ColumnValuesToBeInSet(Vec::new()),
                    column: Some(name.clone()),
                    confidence: 0.3,
                    reason: format!(
                        "Only {} distinct values — consider values_in_set expectation",
                        profile.distinct_count
                    ),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiler::Profiler;
    use polars::prelude::*;

    #[test]
    fn test_suggest_from_profiles() {
        let df = df!(
            "id" => &[1, 2, 3, 4, 5],
            "name" => &["a", "b", "c", "d", "e"],
            "salary" => &[50000.0, 60000.0, 70000.0, 80000.0, 90000.0],
        )
        .unwrap();
        let profiles = Profiler::profile(&df).unwrap();
        let suggestions = SuggestionEngine::suggest(&profiles);
        assert!(!suggestions.is_empty());

        let id_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.column.as_deref() == Some("id"))
            .collect();
        let has_unique = id_suggestions
            .iter()
            .any(|s| matches!(s.expectation_type, ExpectationType::Unique));
        assert!(has_unique);
    }
}

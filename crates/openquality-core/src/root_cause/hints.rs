use crate::types::*;

pub struct RootCauseAnalyzer;

impl RootCauseAnalyzer {
    pub fn analyze(result: &MonitorResult, context: &RootCauseContext) -> Vec<String> {
        let mut hints = Vec::new();
        match &result.monitor_type {
            MonitorType::Volume { .. } => Self::volume_hints(result, context, &mut hints),
            MonitorType::Freshness { .. } => Self::freshness_hints(result, context, &mut hints),
            MonitorType::Schema => Self::schema_hints(result, context, &mut hints),
            MonitorType::Distribution { .. } => {
                Self::distribution_hints(result, context, &mut hints)
            }
            MonitorType::Correlation { .. } => Self::correlation_hints(result, context, &mut hints),
            MonitorType::Uniqueness { .. } => Self::uniqueness_hints(result, context, &mut hints),
            MonitorType::ReferentialIntegrity { .. } => {
                Self::referential_hints(result, context, &mut hints)
            }
            MonitorType::CustomSQL { .. } => Self::custom_sql_hints(result, context, &mut hints),
            MonitorType::MLDrift { .. } => Self::ml_drift_hints(result, context, &mut hints),
            MonitorType::Cost { .. } => Self::cost_hints(result, context, &mut hints),
        }
        Self::causal_inference(result, context, &mut hints);
        hints
    }

    fn causal_inference(_result: &MonitorResult, ctx: &RootCauseContext, hints: &mut Vec<String>) {
        if ctx.historical_results.len() < 3 {
            return;
        }
        let scores: Vec<f64> = ctx.historical_results.iter().map(|r| r.score).collect();
        if let Some(granger) = Self::granger_causality(&scores, 2) {
            hints.push(format!(
                "[Granger causality] Past values predict current anomaly (F={:.4})",
                granger
            ));
        }
        if let Some(pca) = Self::pca_contribution(&scores) {
            hints.push(format!(
                "[PCA] Top dimension contributed {:.1}% to anomaly score",
                pca * 100.0
            ));
        }
        if let Some(dim) = &ctx.anomaly_dimension {
            hints.push(format!(
                "[Dimension isolation] Segment '{}' drove the change",
                dim
            ));
        }
        if let Some(deploy) = &ctx.recent_deployment {
            hints.push(format!(
                "[Deployment correlation] '{}' deployed at {} — within anomaly window",
                deploy.name, deploy.timestamp
            ));
        }
    }

    fn granger_causality(series: &[f64], lag: usize) -> Option<f64> {
        if series.len() < lag + 3 {
            return None;
        }
        let n = series.len();
        let mut ssr_restricted = 0.0;
        let mut ssr_unrestricted = 0.0;
        for i in lag..n {
            let pred_restricted = series[i - 1];
            let pred_unrestricted = series[i - lag.max(1)];
            ssr_restricted += (series[i] - pred_restricted).powi(2);
            ssr_unrestricted += (series[i] - pred_unrestricted).powi(2);
        }
        let f_stat = ((ssr_restricted - ssr_unrestricted) / lag as f64)
            / (ssr_unrestricted / (n - lag) as f64);
        if f_stat.is_finite() && f_stat > 0.0 {
            Some(f_stat)
        } else {
            None
        }
    }

    fn pca_contribution(series: &[f64]) -> Option<f64> {
        if series.len() < 3 {
            return None;
        }
        let mean: f64 = series.iter().sum::<f64>() / series.len() as f64;
        let variance: f64 =
            series.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / series.len() as f64;
        if variance <= 0.0 {
            return None;
        }
        let last = *series.last().unwrap_or(&mean);
        let contribution = (last - mean).abs() / variance.sqrt().max(1e-10);
        Some(contribution.min(1.0))
    }

    fn volume_hints(result: &MonitorResult, ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let row_count = result
            .details
            .get("row_count")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let pct_change = result
            .details
            .get("pct_change")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        if row_count == 0.0 {
            hints.push(format!(
                "Table '{}' has 0 rows — possible upstream pipeline failure or source table empty",
                result.table_name
            ));
        } else if pct_change < -50.0 {
            hints.push(format!("Volume dropped {:.0}% — check for upstream filter changes, partition pruning, or source disconnection", pct_change));
        } else if pct_change > 100.0 {
            hints.push(format!("Volume surged {:.0}% — possible data duplication, backfill, or upstream row explosion", pct_change));
        }
        if let Some(upstream) = &ctx.upstream_pipeline {
            hints.push(format!(
                "Last successful run of upstream pipeline '{}' was at {}",
                upstream.name, upstream.last_success
            ));
        }
    }

    fn freshness_hints(result: &MonitorResult, ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let age = result
            .details
            .get("age_seconds")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let age_hours = age / 3600.0;
        hints.push(format!(
            "Data in '{}' is {:.1}h old — expected max {:.0}s",
            result.table_name, age_hours, result.threshold
        ));
        if let Some(upstream) = &ctx.upstream_pipeline {
            hints.push(format!(
                "Pipeline '{}' last ran at {} — may be stalled or failed",
                upstream.name, upstream.last_success
            ));
            hints.push(format!(
                "Check scheduler logs for pipeline '{}' around {}",
                upstream.name, upstream.last_success
            ));
        }
        hints.push(
            "Verify that upstream source is still producing data and the connection is active"
                .to_string(),
        );
    }

    fn schema_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        if let Some(changes) = result.details.get("changes").and_then(|v| v.as_array()) {
            for change in changes {
                if let Some(s) = change.as_str() {
                    if s.starts_with('+') {
                        hints.push(format!("New column detected: {} — possible schema migration or new data source version", s));
                    } else if s.starts_with('-') {
                        hints.push(format!(
                            "Column removed: {} — possible schema migration or column deprecation",
                            s
                        ));
                    } else if s.starts_with('~') {
                        hints.push(format!("Column type changed: {} — possible schema migration or data type update", s));
                    }
                }
            }
        }
        hints.push(
            "Schema changes often correlate with application deployments or ETL code changes"
                .to_string(),
        );
    }

    fn distribution_hints(
        result: &MonitorResult,
        _ctx: &RootCauseContext,
        hints: &mut Vec<String>,
    ) {
        let metric = result
            .details
            .get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let score = result
            .details
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        hints.push(format!(
            "Distribution drift detected via {} (score={:.4}, threshold={:.4})",
            metric, score, result.threshold
        ));
        hints.push("Possible causes: upstream business logic changed, new data source, seasonality effect, or data corruption".to_string());
        hints.push("Compare the failing distribution against a known-good reference period to isolate which values changed most".to_string());
    }

    fn correlation_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let observed = result
            .details
            .get("correlation")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let threshold = result
            .details
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let col_x = result
            .details
            .get("column_x")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let col_y = result
            .details
            .get("column_y")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        hints.push(format!(
            "Correlation between '{}' and '{}' changed to {:.4} (threshold: {:.4})",
            col_x, col_y, observed, threshold
        ));
        hints.push("Possible causes: business logic change affecting one column, data quality issue in join key, or shifted data distribution".to_string());
    }

    fn uniqueness_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let dup_ratio = result
            .details
            .get("duplicate_ratio")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        hints.push(format!("Duplicate ratio {:.2}% exceeds threshold — check for upstream dedup logic failures or repeated data loads", dup_ratio * 100.0));
        hints.push(
            "Common causes: missing DISTINCT in pipeline, repeated ingestion, or join explosion"
                .to_string(),
        );
    }

    fn referential_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let orphan_count = result
            .details
            .get("orphan_count")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let total = result
            .details
            .get("total_rows")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let pct = orphan_count / total.max(1.0) * 100.0;
        hints.push(format!(
            "Referential integrity violation: {:.0} orphan rows ({:.1}%)",
            orphan_count, pct
        ));
        hints.push("Possible causes: source table deleted rows without cascade, ETL order-of-operations changed, or data cleanup scripts running".to_string());
    }

    fn custom_sql_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let observed = result
            .details
            .get("observed")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        hints.push(format!(
            "Custom SQL check returned {:.4} — review query logic for upstream changes",
            observed
        ));
        hints.push(
            "Check if source schema changed, data volume shifted, or business rules were updated"
                .to_string(),
        );
    }

    fn ml_drift_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let model = result
            .details
            .get("model_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let metric = result
            .details
            .get("drift_metric")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        hints.push(format!(
            "ML drift detected for model '{}' on metric '{}' (score={:.4})",
            model, metric, result.score
        ));
        hints.push("Check for training-serving skew, data distribution shift in upstream features, or model staleness".to_string());
        hints.push(
            "Consider retraining the model or reviewing feature engineering pipeline for changes"
                .to_string(),
        );
    }

    fn cost_hints(result: &MonitorResult, _ctx: &RootCauseContext, hints: &mut Vec<String>) {
        let resource = result
            .details
            .get("resource_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        hints.push(format!(
            "Cost anomaly for resource '{}': actual={:.2}, budget={:.2}",
            resource, result.score, result.threshold
        ));
        hints.push("Possible causes: query pattern changed, data volume increased, inefficient joins, or pricing model changes".to_string());
        hints.push("Review recent query performance metrics and consider adding resource limits or query optimization".to_string());
    }
}

pub struct RootCauseContext {
    pub upstream_pipeline: Option<PipelineStatus>,
    pub historical_results: Vec<MonitorResult>,
    pub anomaly_dimension: Option<String>,
    pub recent_deployment: Option<DeploymentInfo>,
}

pub struct PipelineStatus {
    pub name: String,
    pub last_success: String,
    pub status: String,
}

pub struct DeploymentInfo {
    pub name: String,
    pub timestamp: String,
}

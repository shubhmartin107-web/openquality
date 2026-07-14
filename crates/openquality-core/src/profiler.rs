use polars::prelude::*;

use crate::error::Result;
use crate::types::*;

type NumericStats = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<[f64; 5]>,
);

pub struct Profiler;

impl Profiler {
    pub fn profile(df: &DataFrame) -> Result<Vec<ColumnProfile>> {
        let mut profiles = Vec::new();
        for name in df.get_column_names() {
            let col = df.column(name)?;
            let profile = Self::profile_column(name, col)?;
            profiles.push(profile);
        }
        Ok(profiles)
    }

    pub fn profile_column(name: &str, col: &Column) -> Result<ColumnProfile> {
        let null_count = col.null_count();
        let row_count = col.len();
        let distinct_count = col.n_unique().unwrap_or(0);

        let dtype = format!("{:?}", col.dtype());

        let (min_val, max_val, mean_val, stddev_val, quantiles) = if let Ok(ca) = col.f64() {
            let vals: Vec<f64> = ca.into_iter().flatten().collect();
            Self::numeric_stats(&vals)
        } else if let Ok(ca) = col.i32() {
            let vals: Vec<f64> = ca.into_iter().flatten().map(|v| v as f64).collect();
            Self::numeric_stats(&vals)
        } else if let Ok(ca) = col.i64() {
            let vals: Vec<f64> = ca.into_iter().flatten().map(|v| v as f64).collect();
            Self::numeric_stats(&vals)
        } else if let Ok(ca) = col.u32() {
            let vals: Vec<f64> = ca.into_iter().flatten().map(|v| v as f64).collect();
            Self::numeric_stats(&vals)
        } else {
            (None, None, None, None, None)
        };

        Ok(ColumnProfile {
            name: name.to_string(),
            dtype,
            null_count,
            distinct_count,
            row_count,
            min: min_val,
            max: max_val,
            mean: mean_val,
            stddev: stddev_val,
            quantiles,
        })
    }

    fn numeric_stats(vals: &[f64]) -> NumericStats {
        if vals.is_empty() {
            return (None, None, None, None, None);
        }
        let n = vals.len() as f64;
        let min = vals.iter().cloned().fold(f64::MAX, f64::min);
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = vals.iter().sum::<f64>() / n;
        let variance = if vals.len() > 1 {
            vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (vals.len() - 1) as f64
        } else {
            0.0
        };
        let stddev = variance.sqrt();

        let mut sorted = vals.to_vec();
        sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let q = |p: f64| -> f64 {
            let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
            sorted[idx.min(sorted.len() - 1)]
        };
        let quantiles = [q(0.0), q(0.25), q(0.5), q(0.75), q(1.0)];

        (
            Some(min),
            Some(max),
            Some(mean),
            Some(stddev),
            Some(quantiles),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_basic() {
        let df = df!(
            "name" => &["alice", "bob", "charlie", "diana", "eve"],
            "age" => &[25.0, 30.0, 35.0, 40.0, 45.0],
            "salary" => &[50000.0, 60000.0, 70000.0, 80000.0, 90000.0],
        )
        .unwrap();
        let profiles = Profiler::profile(&df).unwrap();
        assert_eq!(profiles.len(), 3);

        let age = &profiles[1];
        assert_eq!(age.name, "age");
        assert_eq!(age.row_count, 5);
        assert_eq!(age.null_count, 0);
        assert_eq!(age.distinct_count, 5);
        assert!((age.mean.unwrap() - 35.0).abs() < 0.01);
        assert!((age.min.unwrap() - 25.0).abs() < 0.01);
        assert!((age.max.unwrap() - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_profiler_with_nulls() {
        let df = df!(
            "x" => &[Some(1.0), None, Some(3.0), None, Some(5.0)],
        )
        .unwrap();
        let profiles = Profiler::profile(&df).unwrap();
        assert_eq!(profiles[0].null_count, 2);
        assert_eq!(profiles[0].row_count, 5);
        assert!(profiles[0].distinct_count >= 3);
    }
}

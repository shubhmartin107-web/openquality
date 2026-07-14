use crate::error::{OpenQualityError, Result};
use std::collections::HashMap;

pub struct ChiSquareResult {
    pub statistic: f64,
    pub degrees_of_freedom: usize,
    pub critical_value: f64,
    pub p_value: f64,
    pub significant: bool,
}

fn chi_square_cdf(x: f64, _k: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut term = (-x / 2.0).exp();
    sum += term;
    for i in 1..200 {
        term *= x / (2.0 * i as f64);
        sum += term;
        if term < 1e-15 {
            break;
        }
    }
    sum.min(1.0)
}

pub fn chi_square_test(
    observed: &HashMap<String, usize>,
    expected: &HashMap<String, usize>,
) -> Result<ChiSquareResult> {
    if observed.is_empty() {
        return Err(OpenQualityError::Stats("Chi-square: empty observed".into()));
    }
    let mut statistic = 0.0;
    let total_observed: usize = observed.values().sum();
    let total_expected: usize = expected.values().sum();

    let mut all_categories: Vec<String> = Vec::new();
    for k in observed.keys() {
        if !all_categories.contains(k) {
            all_categories.push(k.clone());
        }
    }
    for k in expected.keys() {
        if !all_categories.contains(k) {
            all_categories.push(k.clone());
        }
    }
    let dof = all_categories.len().max(1) - 1;

    for cat in &all_categories {
        let obs_count = *observed.get(cat).unwrap_or(&0);
        let exp_count = *expected.get(cat).unwrap_or(&0);
        let exp_ratio = if total_expected > 0 {
            exp_count as f64 / total_expected as f64
        } else {
            0.0
        };
        let exp_scaled = exp_ratio * total_observed as f64;
        if exp_scaled > 0.0 {
            let diff = obs_count as f64 - exp_scaled;
            statistic += diff * diff / exp_scaled;
        }
    }

    let critical = 3.841;
    let p = 1.0 - chi_square_cdf(statistic, dof as f64);
    Ok(ChiSquareResult {
        statistic,
        degrees_of_freedom: dof,
        critical_value: critical,
        p_value: p,
        significant: statistic > critical,
    })
}

pub fn chi_square_from_slices(reference: &[&str], target: &[&str]) -> Result<ChiSquareResult> {
    let mut freq_ref: HashMap<String, usize> = HashMap::new();
    let mut freq_tgt: HashMap<String, usize> = HashMap::new();
    for v in reference {
        *freq_ref.entry(v.to_string()).or_insert(0) += 1;
    }
    for v in target {
        *freq_tgt.entry(v.to_string()).or_insert(0) += 1;
    }
    chi_square_test(&freq_tgt, &freq_ref)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_chi_square_identical() {
        let ref_data = vec!["a", "b", "a", "b", "a"];
        let tgt_data = vec!["a", "b", "a", "b", "a"];
        let result = chi_square_from_slices(&ref_data, &tgt_data).unwrap();
        assert!(result.statistic < 0.01);
    }
    #[test]
    fn test_chi_square_different() {
        let ref_data = vec!["a", "a", "a", "a", "a"];
        let tgt_data = vec!["b", "b", "b", "b", "b"];
        let result = chi_square_from_slices(&ref_data, &tgt_data).unwrap();
        assert!(
            result.significant,
            "chi-sq stat={}, critical={}",
            result.statistic, result.critical_value
        );
    }
}

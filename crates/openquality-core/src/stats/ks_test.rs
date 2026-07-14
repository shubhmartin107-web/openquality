use crate::error::{OpenQualityError, Result};

pub struct KsResult {
    pub statistic: f64,
    pub p_value: f64,
}

fn group_sorted(data: &[f64]) -> Vec<(f64, usize)> {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut groups: Vec<(f64, usize)> = Vec::new();
    for v in sorted {
        match groups.last_mut() {
            Some((last_val, count)) if *last_val == v => *count += 1,
            _ => groups.push((v, 1)),
        }
    }
    groups
}

pub fn ks_test(reference: &[f64], target: &[f64]) -> Result<KsResult> {
    if reference.is_empty() || target.is_empty() {
        return Err(OpenQualityError::Stats("Empty sample for KS test".into()));
    }

    let n1 = reference.len() as f64;
    let n2 = target.len() as f64;
    let groups1 = group_sorted(reference);
    let groups2 = group_sorted(target);

    let mut i = 0;
    let mut j = 0;
    let mut max_diff = 0.0;
    let mut cdf1 = 0.0;
    let mut cdf2 = 0.0;

    while i < groups1.len() || j < groups2.len() {
        if i < groups1.len() && j < groups2.len() && groups1[i].0 == groups2[j].0 {
            cdf1 += groups1[i].1 as f64 / n1;
            cdf2 += groups2[j].1 as f64 / n2;
            i += 1;
            j += 1;
        } else if i < groups1.len() && (j >= groups2.len() || groups1[i].0 < groups2[j].0) {
            cdf1 += groups1[i].1 as f64 / n1;
            i += 1;
        } else {
            cdf2 += groups2[j].1 as f64 / n2;
            j += 1;
        }
        let diff = (cdf1 - cdf2).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    let ne = (n1 * n2) / (n1 + n2);
    let p_value = 2.0 * (-2.0 * ne.sqrt() * max_diff).exp().min(0.5);
    Ok(KsResult {
        statistic: max_diff,
        p_value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ks_identical() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ks_test(&data, &data).unwrap();
        assert!(result.statistic < 1e-10, "statistic={}", result.statistic);
    }
    #[test]
    fn test_ks_different() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let result = ks_test(&a, &b).unwrap();
        assert!(result.statistic > 0.5);
    }
}

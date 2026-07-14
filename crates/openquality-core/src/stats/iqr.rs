pub struct IqrResult {
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub lower_fence: f64,
    pub upper_fence: f64,
    pub outliers: Vec<(usize, f64)>,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len();
    let idx = (p / 100.0) * (n - 1) as f64;
    let floor = idx.floor() as usize;
    let ceil = idx.ceil() as usize;
    if floor == ceil || floor >= n - 1 {
        sorted[floor.min(n - 1)]
    } else {
        let frac = idx - floor as f64;
        sorted[floor] * (1.0 - frac) + sorted[ceil] * frac
    }
}

pub fn iqr_outliers(data: &[f64], multiplier: f64) -> IqrResult {
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q1 = percentile(&sorted, 25.0);
    let q3 = percentile(&sorted, 75.0);
    let iqr = q3 - q1;
    let lower_fence = q1 - multiplier * iqr;
    let upper_fence = q3 + multiplier * iqr;
    let outliers: Vec<(usize, f64)> = data
        .iter()
        .enumerate()
        .filter(|&(_, v)| *v < lower_fence || *v > upper_fence)
        .map(|(i, &v)| (i, v))
        .collect();
    IqrResult {
        q1,
        q3,
        iqr,
        lower_fence,
        upper_fence,
        outliers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_iqr_no_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = iqr_outliers(&data, 1.5);
        assert!(result.outliers.is_empty());
    }
    #[test]
    fn test_iqr_with_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 100.0];
        let result = iqr_outliers(&data, 1.5);
        assert!(!result.outliers.is_empty());
    }
}

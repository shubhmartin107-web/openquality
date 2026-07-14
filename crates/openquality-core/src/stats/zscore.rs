pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

pub fn stddev(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let m = mean(data);
    let variance = data.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

pub fn zscore(value: f64, data: &[f64]) -> f64 {
    let m = mean(data);
    let s = stddev(data);
    if s == 0.0 {
        return 0.0;
    }
    (value - m) / s
}

pub fn median(data: &mut [f64]) -> f64 {
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = data.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 0 {
        (data[n / 2 - 1] + data[n / 2]) / 2.0
    } else {
        data[n / 2]
    }
}

pub fn mad(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    let med = median(&mut sorted);
    let deviations: Vec<f64> = data.iter().map(|v| (v - med).abs()).collect();
    let mut dev_sorted = deviations;
    median(&mut dev_sorted)
}

pub fn modified_zscore(value: f64, data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    let med = median(&mut sorted);
    let m = mad(data);
    if m == 0.0 {
        return 0.0;
    }
    0.6745 * (value - med) / m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mean() {
        assert!((mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_zscore_normal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let z = zscore(100.0, &data);
        assert!(z > 5.0);
    }
    #[test]
    fn test_modified_zscore() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mz = modified_zscore(100.0, &data);
        assert!(mz > 0.5);
    }
}

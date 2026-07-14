use crate::error::Result;

#[derive(Debug, Clone, Copy)]
pub enum ThresholdMethod {
    ThreeSigma,
    IQR,
    Mad,
}

pub struct AutoThreshold {
    pub threshold: f64,
    pub method: ThresholdMethod,
    pub values_used: usize,
}

pub fn auto_threshold(
    history: &[f64],
    method: ThresholdMethod,
    sensitivity: f64,
    min_val: f64,
    max_val: f64,
) -> Result<AutoThreshold> {
    if history.is_empty() {
        return Ok(AutoThreshold {
            threshold: min_val.max(0.001),
            method,
            values_used: 0,
        });
    }
    let n = history.len();
    let threshold = match method {
        ThresholdMethod::ThreeSigma => {
            let mu = history.iter().sum::<f64>() / n as f64;
            let variance = history.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / n as f64;
            let sigma = variance.sqrt();
            (mu + sensitivity * sigma).max(min_val).min(max_val)
        }
        ThresholdMethod::Mad => {
            let mut sorted = history.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let med = if n % 2 == 0 {
                (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
            } else {
                sorted[n / 2]
            };
            let deviations: Vec<f64> = history.iter().map(|v| (v - med).abs()).collect();
            let mut dev_sorted = deviations;
            dev_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let m = if dev_sorted.len() % 2 == 0 {
                (dev_sorted[dev_sorted.len() / 2 - 1] + dev_sorted[dev_sorted.len() / 2]) / 2.0
            } else {
                dev_sorted[dev_sorted.len() / 2]
            };
            (med + sensitivity * 1.4826 * m).max(min_val).min(max_val)
        }
        ThresholdMethod::IQR => {
            let mut sorted = history.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q1_idx = (n as f64 * 0.25) as usize;
            let q3_idx = (n as f64 * 0.75) as usize;
            let q1 = sorted[q1_idx.min(n - 1)];
            let q3 = sorted[q3_idx.min(n - 1)];
            let iqr = q3 - q1;
            (q3 + sensitivity * iqr).max(min_val).min(max_val)
        }
    };
    Ok(AutoThreshold {
        threshold,
        method,
        values_used: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_auto_threshold_mad() {
        let history = vec![1.0, 2.0, 1.5, 2.5, 2.0, 1.8, 2.2, 1.0, 3.0, 2.5];
        let result = auto_threshold(&history, ThresholdMethod::Mad, 3.0, 0.001, 100.0).unwrap();
        assert!(result.threshold > 0.0);
    }
}

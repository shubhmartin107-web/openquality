use crate::error::{OpenQualityError, Result};

pub fn histogram_from(
    values: &[f64],
    bins: usize,
    global_min: f64,
    global_max: f64,
) -> Result<Vec<f64>> {
    if values.is_empty() {
        return Err(OpenQualityError::Stats("Empty data for histogram".into()));
    }
    if (global_max - global_min).abs() < f64::EPSILON {
        let mut h = vec![0.0; bins];
        h[0] = values.len() as f64;
        return Ok(h);
    }
    let bin_width = (global_max - global_min) / bins as f64;
    let mut hist = vec![0.0_f64; bins];
    for v in values {
        let mut idx = ((v - global_min) / bin_width) as usize;
        if idx >= bins {
            idx = bins - 1;
        }
        hist[idx] += 1.0;
    }
    let total: f64 = hist.iter().sum();
    if total > 0.0 {
        for h in &mut hist {
            *h /= total;
        }
    }
    Ok(hist)
}

fn kl_divergence(p: &[f64], q: &[f64]) -> Result<f64> {
    if p.len() != q.len() {
        return Err(OpenQualityError::Stats("KL: mismatched lengths".into()));
    }
    let mut kl = 0.0;
    for i in 0..p.len() {
        if p[i] > 0.0 {
            if q[i] <= 0.0 {
                kl += p[i] * (p[i] / (q[i] + 1e-10)).ln();
            } else {
                kl += p[i] * (p[i] / q[i]).ln();
            }
        }
    }
    Ok(kl)
}

pub fn js_divergence(p: &[f64], q: &[f64]) -> Result<f64> {
    if p.len() != q.len() {
        return Err(OpenQualityError::Stats("JS: mismatched lengths".into()));
    }
    if p.is_empty() {
        return Err(OpenQualityError::Stats("JS: empty input".into()));
    }
    let m: Vec<f64> = p
        .iter()
        .zip(q.iter())
        .map(|(pi, qi)| (pi + qi) / 2.0)
        .collect();
    let kl_pm = kl_divergence(p, &m)?;
    let kl_qm = kl_divergence(q, &m)?;
    Ok((kl_pm + kl_qm) / 2.0)
}

pub fn js_divergence_samples(reference: &[f64], target: &[f64], bins: usize) -> Result<f64> {
    if reference.is_empty() || target.is_empty() {
        return Err(OpenQualityError::Stats(
            "Empty sample for JS divergence".into(),
        ));
    }
    let global_min = reference
        .iter()
        .chain(target.iter())
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let global_max = reference
        .iter()
        .chain(target.iter())
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let p = histogram_from(reference, bins, global_min, global_max)?;
    let q = histogram_from(target, bins, global_min, global_max)?;
    js_divergence(&p, &q)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_js_identical() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let js = js_divergence_samples(&a, &b, 5).unwrap();
        assert!(js < 0.01);
    }
    #[test]
    fn test_js_different() {
        let a = vec![1.0; 100];
        let b = vec![100.0; 100];
        let js = js_divergence_samples(&a, &b, 10).unwrap();
        assert!(js > 0.1, "JS divergence was {js}, expected > 0.1");
    }
}

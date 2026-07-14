import numpy as np
from typing import Optional


def ks_test(sample1: np.ndarray, sample2: np.ndarray) -> float:
    if len(sample1) == 0 or len(sample2) == 0:
        return 1.0
    all_values = np.concatenate([sample1, sample2])
    labels = np.array([0] * len(sample1) + [1] * len(sample2))
    idx = np.argsort(all_values)
    all_values = all_values[idx]
    labels = labels[idx]
    cdf1 = 0.0
    cdf2 = 0.0
    n1 = len(sample1)
    n2 = len(sample2)
    max_diff = 0.0
    i = 0
    while i < len(all_values):
        val = all_values[i]
        j = i
        while j < len(all_values) and all_values[j] == val:
            if labels[j] == 0:
                cdf1 += 1.0 / n1
            else:
                cdf2 += 1.0 / n2
            j += 1
        max_diff = max(max_diff, abs(cdf1 - cdf2))
        i = j
    return max_diff


def js_divergence(p: np.ndarray, q: np.ndarray) -> float:
    p = np.asarray(p, dtype=float)
    q = np.asarray(q, dtype=float)
    m = (p + q) / 2.0
    def kl(a, b):
        a = np.clip(a, 1e-10, 1.0)
        b = np.clip(b, 1e-10, 1.0)
        return np.sum(a * np.log(a / b))
    return (kl(p, m) + kl(q, m)) / 2.0


def histogram_from(values: np.ndarray, bins: int, vmin: Optional[float] = None, vmax: Optional[float] = None) -> np.ndarray:
    if vmin is None or vmax is None:
        vmin, vmax = values.min(), values.max()
    if vmax == vmin:
        hist = np.zeros(bins)
        hist[0] = len(values)
        return hist / hist.sum()
    hist, _ = np.histogram(values, bins=bins, range=(vmin, vmax), density=False)
    return hist.astype(float) / hist.sum()


def js_divergence_samples(reference: np.ndarray, target: np.ndarray, bins: int = 20) -> float:
    vmin = min(reference.min(), target.min())
    vmax = max(reference.max(), target.max())
    p = histogram_from(reference, bins, vmin, vmax)
    q = histogram_from(target, bins, vmin, vmax)
    return js_divergence(p, q)


def zscore(value: float, data: np.ndarray) -> float:
    if len(data) < 2:
        return 0.0
    mu, sigma = np.mean(data), np.std(data, ddof=1)
    if sigma == 0:
        return 0.0
    return (value - mu) / sigma


def modified_zscore(value: float, data: np.ndarray) -> float:
    if len(data) == 0:
        return 0.0
    med = np.median(data)
    mad = np.median(np.abs(data - med))
    if mad == 0:
        return 0.0
    return 0.6745 * (value - med) / mad


def iqr_outliers(data: np.ndarray, multiplier: float = 1.5):
    q1, q3 = np.percentile(data, [25, 75])
    iqr = q3 - q1
    lower = q1 - multiplier * iqr
    upper = q3 + multiplier * iqr
    mask = (data < lower) | (data > upper)
    return {
        "q1": q1, "q3": q3, "iqr": iqr,
        "lower_fence": lower, "upper_fence": upper,
        "outlier_count": int(mask.sum()),
        "outlier_indices": np.where(mask)[0].tolist(),
        "outlier_values": data[mask].tolist(),
    }


def auto_threshold(history: np.ndarray, method: str = "mad", sensitivity: float = 3.0) -> float:
    if len(history) == 0:
        return 0.001
    if method == "three_sigma":
        mu, sigma = np.mean(history), np.std(history, ddof=1)
        return mu + sensitivity * sigma
    elif method == "iqr":
        q1, q3 = np.percentile(history, [25, 75])
        return q3 + sensitivity * (q3 - q1)
    else:
        med = np.median(history)
        mad = np.median(np.abs(history - med))
        return med + sensitivity * 1.4826 * mad

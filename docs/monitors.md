# Monitors Reference

## FreshnessMonitor

Detects stale data by comparing the last-updated timestamp against the current time.

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_age_seconds` | — | Maximum allowed age before alerting |
| `auto_threshold` | `true` | Whether to use dynamic thresholds from history |
| `sensitivity` | `3.0` | Multiplier for threshold calculation |

**Root-cause hints**: pipeline stall, upstream failure, connection issue

## VolumeMonitor

Detects anomalous row counts using a rolling window of historical counts.

| Parameter | Default | Description |
|-----------|---------|-------------|
| `sensitivity` | `3.0` | Multiplier for threshold calculation |
| `auto_threshold` | `true` | MAD-based dynamic thresholding |
| `window_size` | `20` | Number of historical values to keep |

**Root-cause hints**: upstream pipeline failure, source empty, data duplication

## SchemaMonitor

Detects schema drift by comparing column names and types against a baseline.

| Parameter | Default | Description |
|-----------|---------|-------------|
| (none) | — | Automatically captures baseline on first check |

**Root-cause hints**: schema migration, app deployment, ETL code change

## DistributionMonitor

Detects statistical distribution drift using KS test, JS divergence, or chi-squared.

| Parameter | Default | Description |
|-----------|---------|-------------|
| `metric` | `ks_test` | Statistical test: `ks_test`, `js_divergence`, `chi_squared` |
| `auto_threshold` | `true` | MAD-based dynamic thresholding |
| `sensitivity` | `3.0` | Multiplier for threshold calculation |

**Root-cause hints**: business logic change, new data source, seasonality, corruption

## Auto-Threshold Methods

| Method | Formula | Best For |
|--------|---------|----------|
| MAD (default) | `med + k * 1.4826 * mad` | Robust to outliers |
| IQR | `Q3 + k * IQR` | Skewed distributions |
| 3-Sigma | `mean + k * std` | Normal distributions |

## Incident Severity

| Condition | Severity |
|-----------|----------|
| Score < threshold | No alert |
| Threshold ≤ Score < 2× threshold | WARNING |
| Score ≥ 2× threshold or row count = 0 | CRITICAL |

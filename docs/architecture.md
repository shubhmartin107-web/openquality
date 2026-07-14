# OpenQuality Architecture

## Overview

OpenQuality is composed of two tiers:

1. **Rust Core** (`openquality-core`) — the statistical engine, expectation runner,
   monitor implementations, and alerting framework
2. **Python Layer** — a pure-Python reference implementation that mirrors the Rust
   core, plus PyO3 bindings for direct Rust calls from Python

## Data Flow

```
User / MCP / CLI / API
        │
        ▼
  ExpectationSuite ──► ExpectationRunner ──► SuiteResult
  (YAML/API-defined)      │                      │
                          │  │                    │ pass/fail stats
                          │  ▼                    ▼
                          │  Polars DataFrame
                          │
  MonitorConfig ─────────► Monitor
  (freshness/volume/       │
   schema/distribution)     │
                            ▼
                      MonitorResult
                            │
                      ┌─────┴──────┐
                      ▼            ▼
               IncidentManager  RootCauseAnalyzer
                      │            │
                      ▼            ▼
               AlertChannel    RootCauseHints
               (stdout/file)   (contextual)
```

## Monitor Auto-Thresholding

Each monitor maintains a rolling window of historical scores. When
`auto_threshold` is enabled, the monitor computes:

- **MAD (default)**: `threshold = median + sensitivity * 1.4826 * mad`
- **IQR**: `threshold = Q3 + sensitivity * IQR`
- **3-Sigma**: `threshold = mean + sensitivity * std`

The sensitivity is configurable (default 3.0). Higher values = fewer alerts.

## Statistical Tests

| Test | Use Case | Output Range | Interpretation |
|------|----------|-------------|----------------|
| KS Test | Continuous distribution comparison | [0, 1] | Higher = more drift |
| JS Divergence | Distribution histogram comparison | [0, ln 2] | Higher = more drift |
| Chi-Squared | Categorical distribution comparison | [0, ∞) | Higher = more drift |
| Modified Z-Score | Outlier detection | [0, ∞) | Higher = more anomalous |
| IQR | Robust outlier detection | N/A | Points outside fences |

## Incident Severity

- **WARNING**: The monitor's threshold is exceeded by < 2x
- **CRITICAL**: The monitor's threshold is exceeded by >= 2x (or row count = 0)

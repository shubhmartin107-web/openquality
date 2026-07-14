# OpenQuality

Declarative data tests (Great Expectations-style) with anomaly-based data observability
(Monte Carlo-style). Provides expectation suites; freshness/volume/schema/distribution
monitors with auto-thresholds; incident alerting; and root-cause hints.

## Quick Start

```bash
cargo build --workspace
cargo test --workspace
```

## Demos

```bash
pip install pandas numpy
python demo/drift_detection.py
python demo/broken_pipeline.py
```

## Benchmarks

```bash
python benchmarks/injected_anomalies.py
```

## Python Tests

```bash
pip install pandas numpy pytest
pytest python/tests/
```

## Project Structure

```
openquality/
├── Cargo.toml                     # Workspace root
├── crates/
│   ├── openquality-core/          # Foundation: types, stats, expectations, monitors, alerts
│   ├── openquality-python/        # PyO3 bindings to Rust core
│   ├── openquality-mcp/           # MCP server tools
│   ├── openquality-server/        # Axum REST API
│   └── openquality-cli/           # CLAP command-line interface
├── python/                        # Pure Python reference implementation
│   └── src/openquality/
│       ├── expectations/          # Great Expectations-style assertions
│       ├── monitors/              # Freshness/volume/schema/distribution monitors
│       ├── stats/                 # KS test, JS divergence, z-score, IQR
│       ├── alerting/              # Incident management + alert channels
│       └── root_cause/            # Root-cause hint generation
├── demo/                          # Drift detection + broken pipeline demos
├── benchmarks/                    # Injected anomaly detection accuracy benchmarks
└── docs/                          # Architecture and getting-started guides
```

## Architecture

```
Data Source (CSV/Parquet/Pandas)
    │
    ├── ExpectationRunner ── ExpectationSuite ── SuiteResult (pass/fail)
    │
    └── Monitors:
         ├── FreshnessMonitor   (staleness detection + auto-threshold)
         ├── VolumeMonitor      (row count anomalies + auto-threshold)
         ├── SchemaMonitor      (column drift detection)
         └── DistributionMonitor (KS/JS/chi-squared + auto-threshold)
                │
                ▼
         IncidentManager ── AlertChannel (stdout/file)
                │
                ▼
         RootCauseAnalyzer (hints: pipeline failure, schema migration, data corruption)
```

## Code Conventions

- `cargo fmt` + `cargo clippy` must pass
- `thiserror` for error enums
- `tracing` for logging with structured fields
- Inline tests with `#[cfg(test)] mod tests`
- `snake_case` for functions/vars, `CamelCase` for types
- Python: type hints, dataclasses for state, pandas for data access

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `polars` | DataFrame operations |
| `serde` / `serde_json` | Serialization |
| `clap` 4 | CLI |
| `axum` 0.8 | REST server |
| `chrono` | Timestamps |
| `thiserror` / `anyhow` | Error handling |

## Adding a Feature

1. Define types in `openquality-core/src/types.rs`
2. Implement logic in the appropriate module
3. Add Rust tests inline (`#[cfg(test)]`)
4. Add Python equivalent in `python/src/openquality/`
5. Add Python tests in `python/tests/`
6. Add demo in `demo/` if user-facing
7. `cargo test --workspace` + `pytest python/tests/`

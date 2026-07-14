# OpenQuality Demos

## Prerequisites

```bash
pip install pandas numpy
```

## Demo 1: Distribution Drift Detection

```bash
python demo/drift_detection.py
```

Demonstrates:
- Reference vs. drifted distribution generation
- KS test and JS divergence computation
- Distribution monitor with auto-thresholds
- Alerting via IncidentManager with root-cause hints

## Demo 2: Broken Pipeline Detection

```bash
python demo/broken_pipeline.py
```

Demonstrates:
- Freshness monitor (stale data detection)
- Volume monitor (zero-row detection with auto-threshold)
- Schema monitor (no change — narrows root cause)
- Expectation suite (Great Expectations-style assertions)
- Combined root-cause analysis and alerting

#!/usr/bin/env python3
"""
OpenQuality Demo 1: Distribution Drift Detection

Generates a reference distribution (normal), then a drifted distribution,
runs the distribution monitor with auto-thresholds, and produces alerts
with root-cause hints.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python", "src"))

import numpy as np
import pandas as pd
from openquality.monitors.core import MonitorConfig, DistributionMonitor
from openquality.alerting.core import IncidentManager, StdoutAlertChannel
from openquality.stats.core import ks_test, js_divergence_samples

np.random.seed(42)
print("=" * 65)
print("  OpenQuality Demo 1: Distribution Drift Detection")
print("=" * 65)

# Step 1: Generate reference data (normal distribution)
n = 1000
reference = np.random.normal(loc=50, scale=10, size=n)
ref_df = pd.DataFrame({"value": reference, "category": np.random.choice(["A", "B", "C"], n)})
print(f"\n[1] Reference data: {n} rows, mean={reference.mean():.2f}, std={reference.std():.2f}")

# Step 2: Generate drifted data
print("\n[2] Injecting drift: shifting mean by +15 and increasing variance...")
drifted = np.random.normal(loc=65, scale=15, size=n)
tgt_df = pd.DataFrame({"value": drifted, "category": np.random.choice(["A", "B", "C"], n)})
print(f"    Target data: {n} rows, mean={drifted.mean():.2f}, std={drifted.std():.2f}")

# Step 3: Compute drift statistics
print("\n[3] Running statistical tests...")
ks_stat = ks_test(reference, drifted)
js = js_divergence_samples(reference, drifted, bins=20)
print(f"    KS statistic: {ks_stat:.4f}")
print(f"    JS divergence: {js:.4f}")

# Step 4: Run distribution monitor
print("\n[4] Running distribution monitor with auto-thresholds...")
config = MonitorConfig.distribution("demo-dist-mon", "synthetic_data", metric="ks_test")
monitor = DistributionMonitor(config)
monitor.set_reference(reference)

result = monitor.check(pd.Series(drifted))
print(f"    Alert: {result.alert}")
print(f"    Severity: {result.severity}")
print(f"    Score: {result.score:.4f}")
print(f"    Auto-threshold: {result.threshold:.4f}")
print(f"    Message: {result.message}")

# Step 5: Generate root-cause hints
print("\n[5] Root-cause analysis:")
if result.alert:
    hints = [
        f"Reference mean={reference.mean():.2f} vs target mean={drifted.mean():.2f} (shift={drifted.mean() - reference.mean():.2f})",
        f"Reference std={reference.std():.2f} vs target std={drifted.std():.2f} (variance changed)",
        f"KS statistic={ks_stat:.4f} exceeds threshold={result.threshold:.4f}",
        "Possible causes: upstream business logic changed, new data source, seasonality effect, or data corruption",
        "Check the pipeline code for changes around the drift onset time",
    ]
    for hint in hints:
        print(f"  └─ {hint}")

# Step 6: Alert via incident manager
print("\n[6] Alerting...")
incident_mgr = IncidentManager()
incident_mgr.add_channel(StdoutAlertChannel())
incident = incident_mgr.register(result, hints if result.alert else [])
print(f"\n  Incident created: id={incident.id}")
print(f"  Resolved: {incident.resolved}")

print("\n" + "=" * 65)
print("  Demo complete. Drift detected and alerted successfully.")
print("=" * 65)

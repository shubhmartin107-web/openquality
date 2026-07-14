#!/usr/bin/env python3
"""
OpenQuality Demo 2: Broken Pipeline Detection

Simulates a pipeline that fails, then uses freshness, volume, and schema
monitors to detect the failure and generate root-cause hints.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python", "src"))

import numpy as np
import pandas as pd
from datetime import datetime, timezone, timedelta
from openquality.monitors.core import MonitorConfig, FreshnessMonitor, VolumeMonitor, SchemaMonitor
from openquality.alerting.core import IncidentManager, StdoutAlertChannel
from openquality.expectations.core import (ExpectationSuite, expect_row_count_between,
                                            expect_not_null, expect_between, ExpectationRunner)

np.random.seed(42)
print("=" * 65)
print("  OpenQuality Demo 2: Broken Pipeline Detection")
print("=" * 65)

# Step 1: Historical normal data
print("\n[1] Generating historical normal pipeline data...")
normal_rows = 10_000
normal_df = pd.DataFrame({
    "order_id": range(1, normal_rows + 1),
    "amount": np.random.exponential(scale=100, size=normal_rows).round(2),
    "status": np.random.choice(["completed", "pending", "cancelled"], normal_rows, p=[0.8, 0.15, 0.05]),
    "created_at": pd.date_range(end=datetime.now(timezone.utc) - timedelta(hours=1),
                                 periods=normal_rows, freq="1s"),
})
print(f"    Normal data: {len(normal_df)} rows")

# Step 2: Pipeline breaks — no new data
print("\n[2] Simulating pipeline failure...")
print("    Pipeline 'orders_etl' last successful run: 6 hours ago")
last_data_time = datetime.now(timezone.utc) - timedelta(hours=6)
broken_df = pd.DataFrame()  # Empty — pipeline produced nothing

# Step 3: Freshness monitor
print("\n[3] Freshness monitor check:")
fresh_config = MonitorConfig.freshness("demo-fresh-mon", "orders", max_age_seconds=7200)
fresh_mon = FreshnessMonitor(fresh_config)
fresh_result = fresh_mon.check(last_data_time)
print(f"    Age: {fresh_result.details['age_seconds']:.0f}s")
print(f"    Alert: {fresh_result.alert}")
print(f"    Severity: {fresh_result.severity}")
print(f"    Message: {fresh_result.message}")

# Step 4: Volume monitor
print("\n[4] Volume monitor check:")
vol_config = MonitorConfig.volume("demo-vol-mon", "orders", sensitivity=3.0)
vol_mon = VolumeMonitor(vol_config)
vol_mon.history = [9500.0, 10200.0, 9800.0, 10500.0, 9900.0, 10100.0, 9700.0, 10400.0, 9600.0, 10000.0]
vol_result = vol_mon.check(0)
print(f"    Row count: {vol_result.details['row_count']}")
print(f"    Alert: {vol_result.alert}")
print(f"    Severity: {vol_result.severity}")
print(f"    Message: {vol_result.message}")

# Step 5: Schema monitor
print("\n[5] Schema monitor check:")
schema_config = MonitorConfig.schema("demo-schema-mon", "orders")
schema_mon = SchemaMonitor(schema_config)
schema_mon.check(normal_df)  # Establish baseline
# Broken pipeline produces no data — schema is N/A
schema_result = schema_mon.check(normal_df)  # No change
print(f"    Alert: {schema_result.alert}")
print(f"    Message: {schema_result.message}")

# Step 6: Expectation suite
print("\n[6] Running expectation suite:")
suite = ExpectationSuite("orders_suite")
suite.add(expect_row_count_between(5000, 15000))
suite.add(expect_not_null("order_id"))
suite.add(expect_between("amount", min_val=0, max_val=1000))
result = ExpectationRunner.run(suite, normal_df)
print(f"    {result}")

# Step 7: Combined root-cause analysis
print("\n[7] Combined root-cause analysis:")
combined = [
    ("Freshness", fresh_result.alert, [
        f"Data in 'orders' is {fresh_result.score:.0f}s old — expected max {fresh_result.threshold:.0f}s",
        "Pipeline 'orders_etl' last ran at " + last_data_time.strftime("%Y-%m-%d %H:%M:%S UTC"),
        "Check scheduler logs for pipeline 'orders_etl' around that time",
    ]),
    ("Volume", vol_result.alert, [
        f"Row count dropped from ~10,000 to 0 ({vol_result.details.get('pct_change', 0):.1f}%)",
        "Possible upstream source failure or connection issue",
        "Verify that the source database is accessible and producing data",
    ]),
    ("Schema", schema_result.alert, [
        "Schema is unchanged — this narrows the root cause to a pipeline/logical failure rather than a schema migration",
    ]),
]

for name, alerted, hints in combined:
    if alerted:
        print(f"\n  [{name}] ALERT:")
        for hint in hints:
            print(f"    └─ {hint}")
    else:
        print(f"\n  [{name}] OK — no issues detected")

# Step 8: Alert
print("\n[8] Firing alerts via IncidentManager:")
incident_mgr = IncidentManager()
incident_mgr.add_channel(StdoutAlertChannel())

if fresh_result.alert:
    hints = [
        "Pipeline 'orders_etl' may be stalled or failed",
        f"Last data point at {last_data_time.strftime('%Y-%m-%d %H:%M:%S UTC')}",
        "Check the ETL scheduler and upstream database connectivity",
    ]
    incident_mgr.register(fresh_result, hints)

if vol_result.alert:
    hints = [
        f"Volume dropped to 0 from ~10,000 rows (100% drop)",
        "Possible upstream pipeline failure or source table empty",
        "Check if the source query returned results",
    ]
    incident_mgr.register(vol_result, hints)

print("\n" + "=" * 65)
print("  Demo complete. Pipeline failure detected and alerted.")
print("  Root cause: stalled pipeline or upstream source failure.")
print("=" * 65)

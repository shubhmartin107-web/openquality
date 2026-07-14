import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

import pandas as pd
import numpy as np
from datetime import datetime, timezone, timedelta
from openquality.monitors.core import MonitorConfig, FreshnessMonitor, VolumeMonitor, SchemaMonitor, DistributionMonitor


def test_freshness_ok():
    config = MonitorConfig.freshness("test", "t", max_age_seconds=3600)
    monitor = FreshnessMonitor(config)
    result = monitor.check(datetime.now(timezone.utc) - timedelta(minutes=30))
    assert not result.alert


def test_freshness_alert():
    config = MonitorConfig.freshness("test", "t", max_age_seconds=3600)
    monitor = FreshnessMonitor(config)
    result = monitor.check(datetime.now(timezone.utc) - timedelta(hours=12))
    assert result.alert


def test_volume_ok():
    config = MonitorConfig.volume("test", "t")
    monitor = VolumeMonitor(config)
    for _ in range(10):
        monitor.check(10000)
    result = monitor.check(9500)
    assert not result.alert


def test_volume_zero():
    config = MonitorConfig.volume("test", "t")
    monitor = VolumeMonitor(config)
    for _ in range(10):
        monitor.check(10000)
    result = monitor.check(0)
    assert result.alert
    assert result.severity == "CRITICAL"


def test_schema_no_change():
    config = MonitorConfig.schema("test", "t")
    monitor = SchemaMonitor(config)
    df = pd.DataFrame({"a": [1], "b": [2]})
    r1 = monitor.check(df)
    r2 = monitor.check(df)
    assert not r1.alert
    assert not r2.alert


def test_schema_add_column():
    config = MonitorConfig.schema("test", "t")
    monitor = SchemaMonitor(config)
    monitor.check(pd.DataFrame({"a": [1], "b": [2]}))
    result = monitor.check(pd.DataFrame({"a": [1], "b": [2], "c": [3]}))
    assert result.alert
    assert any("+c" in c for c in result.details.get("changes", []))


def test_distribution_no_drift():
    config = MonitorConfig.distribution("test", "t", metric="ks_test")
    monitor = DistributionMonitor(config)
    ref = np.random.normal(50, 10, 500)
    monitor.set_reference(ref)
    result = monitor.check(pd.Series(np.random.normal(50, 10, 500)))
    assert not result.alert


def test_distribution_drift():
    config = MonitorConfig.distribution("test", "t", metric="ks_test")
    monitor = DistributionMonitor(config)
    ref = np.random.normal(50, 10, 500)
    monitor.set_reference(ref)
    result = monitor.check(pd.Series(np.random.normal(80, 10, 500)))
    assert result.alert

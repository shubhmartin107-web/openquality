#!/usr/bin/env python3
"""
OpenQuality Benchmark: Detection Accuracy on Injected Anomalies

Injects controlled anomalies into synthetic data and measures:
  - True Positive Rate (detection rate)
  - False Positive Rate
  - Precision, Recall, F1
  - Threshold sensitivity
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python", "src"))

import json
import time
import numpy as np
from datetime import datetime, timezone, timedelta
from collections import defaultdict

from openquality.monitors.core import MonitorConfig, FreshnessMonitor, VolumeMonitor, SchemaMonitor, DistributionMonitor
from openquality.expectations.core import (ExpectationSuite, expect_not_null, expect_between,
                                            expect_row_count_between, ExpectationRunner)

np.random.seed(42)


def generate_normal_data(n=5000):
    return {
        "amount": np.random.exponential(scale=100, size=n),
        "age": np.random.normal(loc=35, scale=12, size=n),
        "category": np.random.choice(["A", "B", "C", "D"], n),
        "email": [f"user{i}@example.com" for i in range(n)],
    }


def benchmark_expectation_not_null():
    print("\n  [Expectation] not_null:")
    results = []
    for null_pct in [0.0, 0.01, 0.05, 0.10, 0.25, 0.50]:
        n = 1000
        data = {"value": np.random.normal(50, 10, n).tolist()}
        n_nulls = int(n * null_pct)
        for i in range(n_nulls):
            data["value"][i] = None
        df = __import__("pandas").DataFrame(data)
        suite = ExpectationSuite("bench")
        suite.add(expect_not_null("value", tolerance=0.02))
        result = ExpectationRunner.run(suite, df)
        detected = not result.results[0].success
        tp = detected and null_pct > 0.02
        fp = detected and null_pct <= 0.02
        results.append({"null_pct": null_pct, "detected": detected, "tp": tp, "fp": fp})
        print(f"    null_pct={null_pct:.2f} detected={detected}")
    return results


def benchmark_volume():
    print("\n  [Monitor] volume:")
    results = []
    config = MonitorConfig.volume("bench-vol", "t", sensitivity=3.0)
    from openquality.monitors.core import VolumeMonitor
    for anomaly_type in ["normal", "drop_50", "drop_90", "drop_100", "spike_2x", "spike_5x"]:
        monitor = VolumeMonitor(config)
        for _ in range(15):
            monitor.check(np.random.randint(8000, 12000))
        n = 10000
        if anomaly_type == "normal":
            n = np.random.randint(8000, 12000)
        elif anomaly_type == "drop_50":
            n = 5000
        elif anomaly_type == "drop_90":
            n = 1000
        elif anomaly_type == "drop_100":
            n = 0
        elif anomaly_type == "spike_2x":
            n = 20000
        elif anomaly_type == "spike_5x":
            n = 50000
        result = monitor.check(n)
        expected_alert = anomaly_type != "normal"
        correct = result.alert == expected_alert
        results.append({"anomaly": anomaly_type, "alert": result.alert, "expected": expected_alert, "correct": correct, "score": result.score, "threshold": result.threshold})
        print(f"    {anomaly_type}: alert={result.alert} expected={expected_alert} correct={correct} score={result.score:.0f} thresh={result.threshold:.0f}")
    return results


def benchmark_freshness():
    print("\n  [Monitor] freshness:")
    results = []
    config = MonitorConfig.freshness("bench-fresh", "t", max_age_seconds=7200)
    from openquality.monitors.core import FreshnessMonitor
    for age_hours in [0.5, 1, 2, 3, 6, 12, 24]:
        monitor = FreshnessMonitor(config)
        last_time = datetime.now(timezone.utc) - timedelta(hours=age_hours)
        result = monitor.check(last_time)
        expected_alert = age_hours > 2
        correct = result.alert == expected_alert
        results.append({"age_hours": age_hours, "alert": result.alert, "expected": expected_alert, "correct": correct})
        print(f"    age={age_hours}h: alert={result.alert} expected={expected_alert} correct={correct}")
    return results


def benchmark_distribution():
    import pandas as pd
    print("\n  [Monitor] distribution:")
    results = []
    config = MonitorConfig.distribution("bench-dist", "t", metric="ks_test")
    from openquality.monitors.core import DistributionMonitor
    reference = np.random.normal(50, 10, 1000)
    for shift in [0, 1, 3, 5, 10, 20]:
        monitor = DistributionMonitor(config)
        monitor.set_reference(reference)
        target = np.random.normal(50 + shift, 10, 1000)
        result = monitor.check(pd.Series(target))
        expected_alert = shift >= 5
        correct = result.alert == expected_alert
        results.append({"shift": shift, "alert": result.alert, "expected": expected_alert, "correct": correct, "score": result.score, "threshold": result.threshold})
        print(f"    shift={shift}: alert={result.alert} expected={expected_alert} correct={correct} score={result.score:.4f} thresh={result.threshold:.4f}")
    return results


def benchmark_schema():
    print("\n  [Monitor] schema:")
    import pandas as pd
    results = []
    config = MonitorConfig.schema("bench-schema", "t")
    from openquality.monitors.core import SchemaMonitor
    baseline = pd.DataFrame({"a": [1], "b": [2], "c": [3]})
    for change_type in ["none", "add_col", "drop_col", "rename_col"]:
        monitor = SchemaMonitor(config)
        monitor.check(baseline)  # establish baseline
        if change_type == "none":
            current = baseline
        elif change_type == "add_col":
            current = baseline.copy()
            current["d"] = [4]
        elif change_type == "drop_col":
            current = baseline.drop(columns=["b"])
        elif change_type == "rename_col":
            current = baseline.rename(columns={"a": "a_renamed"})
        result = monitor.check(current)
        expected_alert = change_type != "none"
        correct = result.alert == expected_alert
        results.append({"change": change_type, "alert": result.alert, "expected": expected_alert, "correct": correct})
        print(f"    {change_type}: alert={result.alert} expected={expected_alert} correct={correct}")
    return results


def compute_metrics(all_results):
    tps = sum(1 for r in all_results if r.get("tp", r.get("correct", False)))
    fps = sum(1 for r in all_results if r.get("fp", False))
    total_positives = sum(1 for r in all_results if r.get("expected", True))
    total_negatives = sum(1 for r in all_results if not r.get("expected", False))
    tp = sum(1 for r in all_results if r.get("expected", False) and r.get("alert", r.get("detected", False)))
    fp = sum(1 for r in all_results if not r.get("expected", True) and r.get("alert", r.get("detected", False)))
    fn = sum(1 for r in all_results if r.get("expected", False) and not r.get("alert", r.get("detected", False)))
    tn = sum(1 for r in all_results if not r.get("expected", True) and not r.get("alert", r.get("detected", False)))
    tpr = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    fpr = fp / (fp + tn) if (fp + tn) > 0 else 0.0
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tpr
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0
    accuracy = (tp + tn) / (tp + tn + fp + fn) if (tp + tn + fp + fn) > 0 else 0.0
    return {"tpr": tpr, "fpr": fpr, "precision": precision, "recall": recall, "f1": f1, "accuracy": accuracy, "tp": tp, "fp": fp, "fn": fn, "tn": tn}


def main():
    import pandas as pd
    print("=" * 65)
    print("  OpenQuality Benchmark: Detection Accuracy")
    print("  Injected Anomalies — 5 Monitor Types")
    print("=" * 65)

    start = time.time()
    all_results = []
    results_by_type = {}

    results_by_type["expectation_not_null"] = benchmark_expectation_not_null()
    all_results.extend(results_by_type["expectation_not_null"])

    results_by_type["volume"] = benchmark_volume()
    all_results.extend(results_by_type["volume"])

    results_by_type["freshness"] = benchmark_freshness()
    all_results.extend(results_by_type["freshness"])

    results_by_type["distribution"] = benchmark_distribution()
    all_results.extend(results_by_type["distribution"])

    results_by_type["schema"] = benchmark_schema()
    all_results.extend(results_by_type["schema"])

    elapsed = time.time() - start

    print("\n" + "=" * 65)
    print("  Results Summary")
    print("=" * 65)
    metrics = compute_metrics(all_results)
    print(f"  True Positive Rate:  {metrics['tpr']:.1%}")
    print(f"  False Positive Rate: {metrics['fpr']:.1%}")
    print(f"  Precision:           {metrics['precision']:.1%}")
    print(f"  Recall:              {metrics['recall']:.1%}")
    print(f"  F1 Score:            {metrics['f1']:.1%}")
    print(f"  Accuracy:            {metrics['accuracy']:.1%}")
    print(f"  TP={metrics['tp']} FP={metrics['fp']} FN={metrics['fn']} TN={metrics['tn']}")
    print(f"  Time: {elapsed:.2f}s")

    report = {"metrics": metrics, "by_type": {k: v for k, v in results_by_type.items()}, "elapsed_seconds": elapsed}
    os.makedirs(os.path.join(os.path.dirname(__file__), "results"), exist_ok=True)
    report_path = os.path.join(os.path.dirname(__file__), "results", "benchmark_report.json")
    with open(report_path, "w") as f:
        json.dump(report, f, indent=2, default=str)
    print(f"\n  Full report saved to {report_path}")

    print("\n" + "=" * 65)
    print("  Benchmark complete.")
    print("=" * 65)


if __name__ == "__main__":
    main()

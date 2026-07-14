import uuid
from datetime import datetime, timezone, timedelta
from typing import Optional
from dataclasses import dataclass, field

import pandas as pd
import numpy as np


@dataclass
class MonitorConfig:
    monitor_id: str
    monitor_type: str
    table_name: str
    auto_threshold: bool = True
    sensitivity: float = 3.0
    window_size: int = 20
    enabled: bool = True
    meta: dict = field(default_factory=dict)
    max_age_seconds: Optional[float] = None
    baseline_period_seconds: Optional[float] = None

    @staticmethod
    def freshness(monitor_id: str, table_name: str, max_age_seconds: float) -> "MonitorConfig":
        return MonitorConfig(monitor_id, "freshness", table_name, max_age_seconds=max_age_seconds)

    @staticmethod
    def volume(monitor_id: str, table_name: str, sensitivity: float = 3.0) -> "MonitorConfig":
        return MonitorConfig(monitor_id, "volume", table_name, sensitivity=sensitivity)

    @staticmethod
    def schema(monitor_id: str, table_name: str) -> "MonitorConfig":
        return MonitorConfig(monitor_id, "schema", table_name)

    @staticmethod
    def distribution(monitor_id: str, table_name: str, metric: str = "ks_test", column: Optional[str] = None) -> "MonitorConfig":
        return MonitorConfig(monitor_id, "distribution", table_name, meta={"metric": metric, "column": column or ""})


@dataclass
class MonitorResult:
    monitor_id: str
    monitor_type: str
    table_name: str
    alert: bool
    severity: str
    score: float
    threshold: float
    message: str
    details: dict
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))


class FreshnessMonitor:
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.history: list[float] = []

    def check(self, last_data_time: datetime) -> MonitorResult:
        age_secs = (datetime.now(timezone.utc) - last_data_time).total_seconds()
        max_age = self.config.max_age_seconds or 3600
        threshold = max_age
        if self.config.auto_threshold and len(self.history) > 5:
            arr = np.array(self.history)
            med = np.median(arr)
            mad = np.median(np.abs(arr - med))
            threshold = max(med + self.config.sensitivity * 1.4826 * mad, max_age)
        alert = age_secs > threshold
        severity = "CRITICAL" if alert and age_secs > threshold * 2 else "WARNING" if alert else "INFO"
        self.history.append(age_secs)
        if len(self.history) > self.config.window_size:
            self.history.pop(0)
        return MonitorResult(
            monitor_id=self.config.monitor_id,
            monitor_type="freshness",
            table_name=self.config.table_name,
            alert=alert,
            severity=severity,
            score=age_secs,
            threshold=threshold,
            message=f"Freshness {'alert' if alert else 'OK'}: table '{self.config.table_name}' last updated {age_secs:.0f}s ago (threshold: {threshold:.0f}s)",
            details={"age_seconds": age_secs, "threshold": threshold, "last_data_time": last_data_time.isoformat()},
        )


class VolumeMonitor:
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.history: list[float] = []

    def check(self, row_count: int) -> MonitorResult:
        count = float(row_count)
        threshold = count * 0.5
        if self.config.auto_threshold and len(self.history) > 3:
            arr = np.array(self.history)
            med = np.median(arr)
            mad = np.median(np.abs(arr - med))
            threshold = med + self.config.sensitivity * 1.4826 * mad
        self.history.append(count)
        if len(self.history) > self.config.window_size:
            self.history.pop(0)
        alert = count > threshold * 1.5 or count < threshold * 0.1
        severity = "CRITICAL" if count == 0 else "CRITICAL" if alert and (count > threshold * 2 or count < threshold * 0.05) else "WARNING" if alert else "INFO"
        pct_change = 0.0
        if len(self.history) > 1:
            baseline = np.mean(self.history[:-1])
            if baseline > 0:
                pct_change = (count - baseline) / baseline * 100
        return MonitorResult(
            monitor_id=self.config.monitor_id,
            monitor_type="volume",
            table_name=self.config.table_name,
            alert=alert,
            severity=severity,
            score=count,
            threshold=threshold,
            message=f"Volume {'alert' if alert else 'OK'}: table '{self.config.table_name}' has {count} rows (change: {pct_change:.1f}%)",
            details={"row_count": count, "threshold": threshold, "pct_change": pct_change},
        )


class SchemaMonitor:
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.last_schema: Optional[dict] = None

    def check(self, df: pd.DataFrame) -> MonitorResult:
        current = {col: str(df[col].dtype) for col in df.columns}
        changes = []
        if self.last_schema is not None:
            for col, dtype in current.items():
                if col not in self.last_schema:
                    changes.append(f"+{col} ({dtype})")
                elif self.last_schema[col] != dtype:
                    changes.append(f"~{col}: {self.last_schema[col]} -> {dtype}")
            for col in self.last_schema:
                if col not in current:
                    changes.append(f"-{col} ({self.last_schema[col]})")
        alert = len(changes) > 0
        self.last_schema = current
        return MonitorResult(
            monitor_id=self.config.monitor_id,
            monitor_type="schema",
            table_name=self.config.table_name,
            alert=alert,
            severity="WARNING" if alert else "INFO",
            score=float(len(changes)),
            threshold=0.0,
            message=f"Schema {'alert: change detected' if alert else 'OK'}: table '{self.config.table_name}'",
            details={"changes": changes, "current_schema": current},
        )


class DistributionMonitor:
    def __init__(self, config: MonitorConfig):
        self.config = config
        self.metric = config.meta.get("metric", "ks_test")
        self.column_name = config.meta.get("column", "")
        self.reference: Optional[np.ndarray] = None
        self.history: list[float] = []

    def set_reference(self, values: np.ndarray):
        self.reference = values

    def check(self, column_data: pd.Series) -> MonitorResult:
        from ..stats.core import ks_test as _ks, js_divergence as _js
        target = column_data.dropna().values.astype(float)
        ref = self.reference if self.reference is not None else target
        if self.metric == "ks_test":
            score = _ks(ref, target)
            threshold = 0.3
        elif self.metric == "js_divergence":
            score = _js(ref, target, bins=20)
            threshold = 0.1
        else:
            score = _ks(ref, target)
            threshold = 0.3
        if self.config.auto_threshold and len(self.history) > 5:
            arr = np.array(self.history)
            med = np.median(arr)
            mad = np.median(np.abs(arr - med))
            threshold = med + self.config.sensitivity * 1.4826 * mad
        alert = score > threshold
        severity = "CRITICAL" if alert and score > threshold * 2 else "WARNING" if alert else "INFO"
        self.history.append(score)
        if len(self.history) > self.config.window_size:
            self.history.pop(0)
        return MonitorResult(
            monitor_id=self.config.monitor_id,
            monitor_type="distribution",
            table_name=self.config.table_name,
            alert=alert,
            severity=severity,
            score=score,
            threshold=threshold,
            message=f"Distribution {'drift detected' if alert else 'OK'} on '{self.config.table_name}': {self.metric}={score:.4f} (threshold={threshold:.4f})",
            details={"metric": self.metric, "score": score, "threshold": threshold, "column": self.column_name},
        )

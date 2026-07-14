import uuid
import json
from datetime import datetime, timezone
from typing import Any, Optional

import pandas as pd
import numpy as np


class Expectation:
    def __init__(self, expectation_type: str, column: Optional[str] = None,
                 tolerance: float = 0.0, **kwargs):
        self.id = str(uuid.uuid4())
        self.expectation_type = expectation_type
        self.column = column
        self.tolerance = tolerance
        self.kwargs = kwargs

    def to_dict(self):
        return {
            "id": self.id,
            "expectation_type": self.expectation_type,
            "column": self.column,
            "tolerance": self.tolerance,
            "kwargs": self.kwargs,
        }


class ExpectationSuite:
    def __init__(self, name: str):
        self.name = name
        self.expectations: list[Expectation] = []

    def add(self, expectation: Expectation):
        self.expectations.append(expectation)
        return self

    def __len__(self):
        return len(self.expectations)


class ExpectationResult:
    def __init__(self, expectation_id: str, expectation_type: str,
                 column: Optional[str], success: bool,
                 observed_value: Any, expected_value: Any,
                 exception_info: Optional[str] = None):
        self.expectation_id = expectation_id
        self.expectation_type = expectation_type
        self.column = column
        self.success = success
        self.observed_value = observed_value
        self.expected_value = expected_value
        self.exception_info = exception_info


class SuiteResult:
    def __init__(self, suite_name: str, results: list[ExpectationResult]):
        self.suite_name = suite_name
        self.results = results
        self.total = len(results)
        self.passed = sum(1 for r in results if r.success)
        self.failed = self.total - self.passed
        self.success = self.failed == 0
        self.run_id = str(uuid.uuid4())
        self.timestamp = datetime.now(timezone.utc)

    def summary(self):
        return {
            "suite": self.suite_name,
            "total": self.total,
            "passed": self.passed,
            "failed": self.failed,
            "success_pct": round(self.passed / self.total * 100, 2) if self.total else 100.0,
        }

    def __repr__(self):
        s = self.summary()
        return f"<SuiteResult {s['suite']}: {s['passed']}/{s['total']} passed ({s['success_pct']}%)>"


class ExpectationRunner:
    @staticmethod
    def run(suite: ExpectationSuite, df: pd.DataFrame) -> SuiteResult:
        results = []
        for exp in suite.expectations:
            result = ExpectationRunner._run_single(exp, df)
            results.append(result)
        return SuiteResult(suite.name, results)

    @staticmethod
    def _run_single(exp: Expectation, df: pd.DataFrame) -> ExpectationResult:
        col = exp.column
        try:
            if exp.expectation_type == "not_null":
                if col not in df.columns:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"error": "column not found"}, {})
                null_count = int(df[col].isna().sum())
                total = len(df)
                threshold = int(exp.tolerance * total)
                success = null_count <= threshold
                return ExpectationResult(
                    exp.id, exp.expectation_type, col, success,
                    {"null_count": null_count, "total": total, "null_pct": round(null_count / total * 100, 2) if total else 0},
                    {"tolerance": exp.tolerance, "max_nulls": threshold},
                )

            elif exp.expectation_type == "unique":
                if col not in df.columns:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False, {"error": "column not found"}, {})
                distinct = int(df[col].nunique())
                total = len(df)
                success = distinct == total
                return ExpectationResult(
                    exp.id, exp.expectation_type, col, success,
                    {"distinct": distinct, "total": total, "duplicates": total - distinct},
                    {"distinct": total},
                )

            elif exp.expectation_type == "between":
                min_val, max_val = exp.kwargs.get("min", float("-inf")), exp.kwargs.get("max", float("inf"))
                if col not in df.columns:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False, {"error": "column not found"}, {})
                vals = df[col].dropna()
                out_of_range = int(((vals < min_val) | (vals > max_val)).sum())
                success = out_of_range == 0
                return ExpectationResult(
                    exp.id, exp.expectation_type, col, success,
                    {"out_of_range": out_of_range, "total": len(vals)}, {"min": min_val, "max": max_val},
                )

            elif exp.expectation_type == "row_count_between":
                min_rows, max_rows = exp.kwargs.get("min", 0), exp.kwargs.get("max", float("inf"))
                n = len(df)
                success = min_rows <= n <= max_rows
                return ExpectationResult(exp.id, exp.expectation_type, None, success,
                                         {"row_count": n}, {"min": min_rows, "max": max_rows})

            elif exp.expectation_type == "column_mean_between":
                min_val, max_val = exp.kwargs.get("min", float("-inf")), exp.kwargs.get("max", float("inf"))
                vals = df[col].dropna()
                if len(vals) == 0:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"mean": None}, {"min": min_val, "max": max_val})
                mean = float(vals.mean())
                success = min_val <= mean <= max_val
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"mean": mean, "count": len(vals)}, {"min": min_val, "max": max_val})

            elif exp.expectation_type == "column_stddev_between":
                min_val, max_val = exp.kwargs.get("min", float("-inf")), exp.kwargs.get("max", float("inf"))
                vals = df[col].dropna()
                if len(vals) < 2:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"stddev": None}, {"min": min_val, "max": max_val})
                stddev = float(vals.std(ddof=1))
                success = min_val <= stddev <= max_val
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"stddev": stddev, "count": len(vals)}, {"min": min_val, "max": max_val})

            elif exp.expectation_type == "column_min_between":
                min_val, max_val = exp.kwargs.get("min", float("-inf")), exp.kwargs.get("max", float("inf"))
                vals = df[col].dropna()
                if len(vals) == 0:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"min": None}, {"min": min_val, "max": max_val})
                col_min = float(vals.min())
                success = min_val <= col_min <= max_val
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"min": col_min, "count": len(vals)}, {"expected_min": min_val, "expected_max": max_val})

            elif exp.expectation_type == "column_max_between":
                min_val, max_val = exp.kwargs.get("min", float("-inf")), exp.kwargs.get("max", float("inf"))
                vals = df[col].dropna()
                if len(vals) == 0:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"max": None}, {"min": min_val, "max": max_val})
                col_max = float(vals.max())
                success = min_val <= col_max <= max_val
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"max": col_max, "count": len(vals)}, {"expected_min": min_val, "expected_max": max_val})

            elif exp.expectation_type == "match_regex":
                pattern = exp.kwargs.get("pattern", "")
                vals = df[col].dropna().astype(str)
                mismatches = int((~vals.str.match(pattern, na=False)).sum())
                success = mismatches == 0
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"mismatches": mismatches, "total": len(vals)}, {"pattern": pattern})

            elif exp.expectation_type == "not_match_regex":
                pattern = exp.kwargs.get("pattern", "")
                vals = df[col].dropna().astype(str)
                matches = int(vals.str.match(pattern, na=False).sum())
                success = matches == 0
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"matches": matches, "total": len(vals)}, {"pattern": pattern})

            elif exp.expectation_type == "columns_match_ordered":
                expected = exp.kwargs.get("columns", [])
                actual = list(df.columns)
                success = actual == expected
                return ExpectationResult(exp.id, exp.expectation_type, None, success,
                                         {"actual_columns": actual}, {"expected_columns": expected})

            elif exp.expectation_type == "values_in_set":
                allowed = set(exp.kwargs.get("values", []))
                vals = df[col].dropna().astype(str)
                outside = int((~vals.isin(allowed)).sum())
                success = outside == 0
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"outside_set": outside, "total": len(vals)}, {"values": list(allowed)})

            elif exp.expectation_type == "distinct_values_equal_set":
                expected = set(exp.kwargs.get("values", []))
                actual = set(df[col].dropna().astype(str).unique())
                success = actual == expected
                missing = list(expected - actual)
                extra = list(actual - expected)
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"actual_distinct": list(actual), "missing": missing, "extra": extra},
                                         {"expected_set": list(expected)})

            elif exp.expectation_type == "distinct_values_contained_in_set":
                allowed = set(exp.kwargs.get("values", []))
                actual = set(df[col].dropna().astype(str).unique())
                outside = list(actual - allowed)
                success = len(outside) == 0
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"actual_distinct": list(actual), "outside_set": outside},
                                         {"allowed_set": list(allowed)})

            elif exp.expectation_type == "column_quantile_between":
                quantile = exp.kwargs.get("quantile", 0.5)
                low = exp.kwargs.get("low", float("-inf"))
                high = exp.kwargs.get("high", float("inf"))
                vals = df[col].dropna()
                if len(vals) == 0:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"quantile_value": None}, {"quantile": quantile, "low": low, "high": high})
                q_val = float(vals.quantile(quantile, interpolation="higher"))
                success = low <= q_val <= high
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"quantile_value": q_val, "quantile": quantile}, {"low": low, "high": high})

            elif exp.expectation_type == "kl_divergence_less_than":
                max_divergence = exp.kwargs.get("max_divergence", 1.0)
                vals = df[col].dropna()
                if len(vals) == 0:
                    return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                             {"kl_divergence": None}, {"max_divergence": max_divergence})
                n = len(vals)
                mean = float(vals.mean())
                stddev = float(vals.std(ddof=0)) or 1e-10
                nbins = max(10, min(100, int(np.sqrt(n))))
                bin_width = (4.0 * stddev) / nbins
                center = mean
                bins = [center - 2.0 * stddev + (i + 0.5) * bin_width for i in range(nbins)]
                observed, _ = np.histogram(vals, bins=nbins, range=(center - 2.0 * stddev, center + 2.0 * stddev))
                observed = observed.astype(float)
                o_sum = observed.sum()
                observed_norm = observed / o_sum if o_sum > 0 else observed
                expected_norm = np.array([
                    np.exp(-0.5 * ((b - mean) / stddev) ** 2) / (stddev * np.sqrt(2 * np.pi)) * bin_width
                    for b in bins
                ])
                e_sum = expected_norm.sum()
                expected_norm = expected_norm / e_sum if e_sum > 0 else expected_norm
                kl = np.sum([
                    o * np.log(o / e) if o > 0 and e > 0 else 0.0
                    for o, e in zip(observed_norm, expected_norm)
                ])
                success = float(kl) <= max_divergence
                return ExpectationResult(exp.id, exp.expectation_type, col, success,
                                         {"kl_divergence": float(kl)}, {"max_divergence": max_divergence})

            else:
                return ExpectationResult(exp.id, exp.expectation_type, col, True,
                                         {"note": f"expectation '{exp.expectation_type}' not implemented"},
                                         {"type": exp.expectation_type})

        except Exception as e:
            return ExpectationResult(exp.id, exp.expectation_type, col, False,
                                     {}, {}, exception_info=str(e))


def expect_not_null(column: str, tolerance: float = 0.0) -> Expectation:
    return Expectation("not_null", column, tolerance=tolerance)


def expect_unique(column: str) -> Expectation:
    return Expectation("unique", column)


def expect_between(column: str, min_val: float, max_val: float) -> Expectation:
    return Expectation("between", column, min=min_val, max=max_val)


def expect_row_count_between(min_rows: int, max_rows: int) -> Expectation:
    return Expectation("row_count_between", None, min=min_rows, max=max_rows)


def expect_column_mean_between(column: str, min_val: float, max_val: float) -> Expectation:
    return Expectation("column_mean_between", column, min=min_val, max=max_val)


def expect_column_stddev_between(column: str, min_val: float, max_val: float) -> Expectation:
    return Expectation("column_stddev_between", column, min=min_val, max=max_val)


def expect_column_min_between(column: str, min_val: float, max_val: float) -> Expectation:
    return Expectation("column_min_between", column, min=min_val, max=max_val)


def expect_column_max_between(column: str, min_val: float, max_val: float) -> Expectation:
    return Expectation("column_max_between", column, min=min_val, max=max_val)


def expect_match_regex(column: str, pattern: str) -> Expectation:
    return Expectation("match_regex", column, pattern=pattern)


def expect_not_match_regex(column: str, pattern: str) -> Expectation:
    return Expectation("not_match_regex", column, pattern=pattern)


def expect_columns_match_ordered(columns: list[str]) -> Expectation:
    return Expectation("columns_match_ordered", None, columns=columns)


def expect_values_in_set(column: str, values: list) -> Expectation:
    return Expectation("values_in_set", column, values=values)


def expect_distinct_values_equal_set(column: str, values: list) -> Expectation:
    return Expectation("distinct_values_equal_set", column, values=values)


def expect_distinct_values_contained_in_set(column: str, values: list) -> Expectation:
    return Expectation("distinct_values_contained_in_set", column, values=values)


def expect_column_quantile_between(column: str, quantile: float, low: float, high: float) -> Expectation:
    return Expectation("column_quantile_between", column, quantile=quantile, low=low, high=high)


def expect_kl_divergence_less_than(column: str, max_divergence: float = 1.0) -> Expectation:
    return Expectation("kl_divergence_less_than", column, max_divergence=max_divergence)


builtin_expectations = {
    "not_null": expect_not_null,
    "unique": expect_unique,
    "between": expect_between,
    "row_count_between": expect_row_count_between,
    "column_mean_between": expect_column_mean_between,
    "column_stddev_between": expect_column_stddev_between,
    "column_min_between": expect_column_min_between,
    "column_max_between": expect_column_max_between,
    "match_regex": expect_match_regex,
    "not_match_regex": expect_not_match_regex,
    "columns_match_ordered": expect_columns_match_ordered,
    "values_in_set": expect_values_in_set,
    "distinct_values_equal_set": expect_distinct_values_equal_set,
    "distinct_values_contained_in_set": expect_distinct_values_contained_in_set,
    "column_quantile_between": expect_column_quantile_between,
    "kl_divergence_less_than": expect_kl_divergence_less_than,
}

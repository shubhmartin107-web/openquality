import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

import pandas as pd
import numpy as np
from openquality.expectations.core import (
    ExpectationSuite, ExpectationRunner,
    expect_not_null, expect_unique, expect_between,
    expect_row_count_between, expect_column_mean_between,
    expect_column_stddev_between, expect_column_min_between,
    expect_column_max_between,
    expect_values_in_set, expect_columns_match_ordered,
    expect_match_regex, expect_not_match_regex,
    expect_distinct_values_equal_set,
    expect_distinct_values_contained_in_set,
    expect_column_quantile_between,
    expect_kl_divergence_less_than,
)


def test_not_null_passes():
    df = pd.DataFrame({"x": [1, 2, 3]})
    suite = ExpectationSuite("test").add(expect_not_null("x"))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_not_null_fails():
    df = pd.DataFrame({"x": [1, None, 3]})
    suite = ExpectationSuite("test").add(expect_not_null("x"))
    result = ExpectationRunner.run(suite, df)
    assert not result.success


def test_unique_passes():
    df = pd.DataFrame({"x": [1, 2, 3]})
    suite = ExpectationSuite("test").add(expect_unique("x"))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_unique_fails():
    df = pd.DataFrame({"x": [1, 1, 2]})
    suite = ExpectationSuite("test").add(expect_unique("x"))
    result = ExpectationRunner.run(suite, df)
    assert not result.success


def test_between_passes():
    df = pd.DataFrame({"x": [1, 2, 3]})
    suite = ExpectationSuite("test").add(expect_between("x", 0, 10))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_between_fails():
    df = pd.DataFrame({"x": [1, 2, 100]})
    suite = ExpectationSuite("test").add(expect_between("x", 0, 10))
    result = ExpectationRunner.run(suite, df)
    assert not result.success


def test_row_count_passes():
    df = pd.DataFrame({"x": range(50)})
    suite = ExpectationSuite("test").add(expect_row_count_between(10, 100))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_row_count_fails():
    df = pd.DataFrame({"x": range(200)})
    suite = ExpectationSuite("test").add(expect_row_count_between(10, 100))
    result = ExpectationRunner.run(suite, df)
    assert not result.success


def test_column_mean_between_passes():
    df = pd.DataFrame({"x": [1, 2, 3, 4, 5]})
    suite = ExpectationSuite("test").add(expect_column_mean_between("x", 2, 4))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_column_stddev_between():
    df = pd.DataFrame({"x": [1.0, 2.0, 3.0, 4.0, 5.0]})
    suite = ExpectationSuite("test").add(expect_column_stddev_between("x", 0.5, 3.0))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_column_min_between():
    df = pd.DataFrame({"x": [5, 10, 15]})
    suite = ExpectationSuite("test").add(expect_column_min_between("x", 0, 10))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_column_max_between():
    df = pd.DataFrame({"x": [5, 10, 15]})
    suite = ExpectationSuite("test").add(expect_column_max_between("x", 10, 20))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_values_in_set():
    df = pd.DataFrame({"x": ["a", "b", "c"]})
    suite = ExpectationSuite("test").add(expect_values_in_set("x", ["a", "b", "c"]))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_columns_match():
    df = pd.DataFrame({"a": [1], "b": [2], "c": [3]})
    suite = ExpectationSuite("test").add(expect_columns_match_ordered(["a", "b", "c"]))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_match_regex():
    df = pd.DataFrame({"x": ["abc", "def", "ghi"]})
    suite = ExpectationSuite("test").add(expect_match_regex("x", r"^[a-z]+$"))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_not_match_regex():
    df = pd.DataFrame({"x": ["abc", "def", "ghi"]})
    suite = ExpectationSuite("test").add(expect_not_match_regex("x", r"^\d+$"))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_distinct_values_equal_set():
    df = pd.DataFrame({"x": ["a", "b", "c"]})
    suite = ExpectationSuite("test").add(expect_distinct_values_equal_set("x", ["a", "b", "c"]))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_distinct_values_contained_in_set():
    df = pd.DataFrame({"x": ["a", "b"]})
    suite = ExpectationSuite("test").add(expect_distinct_values_contained_in_set("x", ["a", "b", "c"]))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_column_quantile_between():
    df = pd.DataFrame({"x": [1.0, 2.0, 3.0, 4.0, 5.0]})
    suite = ExpectationSuite("test").add(expect_column_quantile_between("x", 0.5, 2.0, 4.0))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_kl_divergence_less_than():
    np.random.seed(42)
    df = pd.DataFrame({"x": np.random.normal(0, 1, 100)})
    suite = ExpectationSuite("test").add(expect_kl_divergence_less_than("x", 1.0))
    result = ExpectationRunner.run(suite, df)
    assert result.success


def test_mixed_suite():
    df = pd.DataFrame({
        "id": [1, 2, 3, 4, 5],
        "val": [10, 20, 30, 40, 50],
        "status": ["ok", "ok", "fail", "ok", "ok"],
    })
    suite = ExpectationSuite("mixed")
    suite.add(expect_not_null("id"))
    suite.add(expect_unique("id"))
    suite.add(expect_between("val", 0, 100))
    suite.add(expect_row_count_between(1, 10))
    result = ExpectationRunner.run(suite, df)
    assert result.success
    assert result.summary()["passed"] == 4

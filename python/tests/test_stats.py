import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

import numpy as np
from openquality.stats.core import ks_test, js_divergence, modified_zscore, iqr_outliers, auto_threshold


def test_ks_identical():
    data = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
    stat = ks_test(data, data)
    assert stat < 1e-10


def test_ks_different():
    a = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
    b = np.array([100.0, 200.0, 300.0, 400.0, 500.0])
    stat = ks_test(a, b)
    assert stat > 0.5


def test_js_identical():
    p = np.array([0.5, 0.5])
    q = np.array([0.5, 0.5])
    js = js_divergence(p, q)
    assert js < 0.01


def test_js_different():
    p = np.array([1.0, 0.0, 0.0])
    q = np.array([0.0, 0.0, 1.0])
    js = js_divergence(p, q)
    assert js > 0.1


def test_modified_zscore():
    data = np.array([1, 2, 3, 4, 5, 100])
    mz = modified_zscore(100, data)
    assert mz > 0.5


def test_iqr():
    data = np.array([1, 2, 3, 4, 5, 6, 7, 8, 9, 100])
    result = iqr_outliers(data)
    assert result["outlier_count"] > 0


def test_auto_threshold_mad():
    data = np.array([1, 2, 1.5, 2.5, 2, 1.8, 2.2, 1, 3, 2.5])
    threshold = auto_threshold(data, method="mad", sensitivity=3.0)
    assert threshold > 0

//! # OpenQuality Python Bindings
//!
//! PyO3 bindings exposing core statistical functions to Python.
//!
//! Functions: `ks_test`, `js_divergence`, `modified_zscore`

use pyo3::prelude::*;

#[pyfunction]
fn ks_test_py(reference: Vec<f64>, target: Vec<f64>) -> PyResult<f64> {
    let result = openquality_core::stats::ks_test::ks_test(&reference, &target)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    Ok(result.statistic)
}

#[pyfunction]
fn js_divergence_py(reference: Vec<f64>, target: Vec<f64>, bins: usize) -> PyResult<f64> {
    let result =
        openquality_core::stats::js_divergence::js_divergence_samples(&reference, &target, bins)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
    Ok(result)
}

#[pyfunction]
fn modified_zscore_py(value: f64, data: Vec<f64>) -> PyResult<f64> {
    Ok(openquality_core::stats::zscore::modified_zscore(
        value, &data,
    ))
}

#[pymodule]
fn openquality(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ks_test_py, m)?)?;
    m.add_function(wrap_pyfunction!(js_divergence_py, m)?)?;
    m.add_function(wrap_pyfunction!(modified_zscore_py, m)?)?;
    Ok(())
}

use alloc::vec::Vec;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};

use crate::{compress_chunk, decompress_chunk, tensor};

#[pyfunction]
#[pyo3(signature = (data, predictor_id=0, weights=None))]
fn encode_bytes(
    py: Python<'_>,
    data: &[u8],
    predictor_id: u8,
    weights: Option<&[u8]>,
) -> PyResult<Py<PyBytes>> {
    // Allocate a conservative buffer (input size + header/neural weights margin)
    let overhead = 4096 + weights.map_or(0, |w| w.len());
    let capacity = data.len().saturating_add(overhead);
    let mut buffer = Vec::with_capacity(capacity);
    buffer.resize(capacity, 0);

    let compressed_len = compress_chunk(data, predictor_id, weights, None, &mut buffer)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

    buffer.truncate(compressed_len);
    Ok(PyBytes::new_bound(py, &buffer).unbind())
}

#[pyfunction]
#[pyo3(signature = (data, predictor_id=0, weights=None))]
fn decode_bytes(
    py: Python<'_>,
    data: &[u8],
    predictor_id: u8,
    weights: Option<&[u8]>,
) -> PyResult<Py<PyBytes>> {
    let decompressed = decompress_chunk(data, predictor_id, weights)
        .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
    Ok(PyBytes::new_bound(py, &decompressed).unbind())
}

#[pyfunction]
#[pyo3(signature = (data, predictor_id=0, weights=None))]
fn compress_adaptive(
    py: Python<'_>,
    data: &[u8],
    predictor_id: u8,
    weights: Option<&[u8]>,
) -> PyResult<Py<PyBytes>> {
    encode_bytes(py, data, predictor_id, weights)
}

#[pyfunction]
#[pyo3(signature = (_data, _predictor_id=0, _weights=None))]
fn get_residuals_py(_data: &[u8], _predictor_id: u8, _weights: Option<&[u8]>) -> PyResult<Vec<i8>> {
    // Placeholder: analytics hook for future residual inspection
    Ok(Vec::new())
}

#[pyfunction]
fn compress_matrix_v1(
    data: Vec<f64>,
    rows: usize,
    cols: usize,
    threshold: f64,
) -> PyResult<Vec<f64>> {
    let compressor = tensor::MpsCompressor::new(10, threshold);
    let cores = compressor.compress_matrix(&data, rows, cols);
    if let Some(first_core) = cores.first() {
        Ok(first_core.clone())
    } else {
        Ok(alloc::vec![])
    }
}

/// QRES Rust extension module exported to Python.
#[pymodule]
fn qres_rust(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(decode_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(compress_adaptive, m)?)?;
    m.add_function(wrap_pyfunction!(get_residuals_py, m)?)?;
    m.add_function(wrap_pyfunction!(compress_matrix_v1, m)?)?;

    // Expose module version to Python
    m.add("__version__", "18.0.0")?;

    Ok(())
}

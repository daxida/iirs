use ::iirs as _iirs;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::str::FromStr;

// Note: this has no setters/getters
#[pyclass]
pub struct SearchParams {
    inner: _iirs::SearchParams,
}

#[pymethods]
impl SearchParams {
    /// `repeat_type` is one of "inverted", "mirror", "direct" or "complement".
    #[new]
    #[pyo3(signature = (min_len, max_len, max_gap, mismatches, repeat_type = "inverted"))]
    pub fn new(
        min_len: usize,
        max_len: usize,
        max_gap: usize,
        mismatches: usize,
        repeat_type: &str,
    ) -> PyResult<Self> {
        let repeat_type = _iirs::RepeatType::from_str(repeat_type)
            .map_err(|e| PyErr::new::<PyValueError, _>(format!("{}", e)))?;

        match _iirs::SearchParams::new(min_len, max_len, max_gap, mismatches) {
            Ok(inner) => Ok(Self {
                inner: inner.with_repeat_type(repeat_type),
            }),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!(
                "Invalid search parameters: {:?}",
                e
            ))),
        }
    }
}

#[pyfunction]
pub fn find_repeats(params: &SearchParams, seq: &str) -> PyResult<Vec<(usize, usize, usize)>> {
    match _iirs::find_repeats(&params.inner, seq.as_bytes()) {
        Ok(result) => Ok(result),
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {:?}", e))),
    }
}

#[pymodule]
fn iirs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SearchParams>()?;
    m.add_function(wrap_pyfunction!(find_repeats, m)?)?;
    Ok(())
}

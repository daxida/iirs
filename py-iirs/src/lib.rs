use ::iirs as _iirs;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// Note: this has no setters/getters
#[pyclass]
pub struct SearchParams {
    inner: _iirs::SearchParams,
}

#[pymethods]
impl SearchParams {
    #[new]
    pub fn new(
        min_len: usize,
        max_len: usize,
        max_gap: usize,
        mismatches: usize,
    ) -> PyResult<Self> {
        match _iirs::SearchParams::new(min_len, max_len, max_gap, mismatches) {
            Ok(inner) => Ok(Self { inner }),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!(
                "Invalid search parameters: {:?}",
                e
            ))),
        }
    }
}

#[pyfunction]
pub fn find_irs(params: &SearchParams, seq: &str) -> PyResult<Vec<(usize, usize, usize)>> {
    match _iirs::find_irs(&params.inner, seq.as_bytes()) {
        Ok(result) => Ok(result),
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {:?}", e))),
    }
}

#[pymodule]
fn iirs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SearchParams>()?;
    m.add_function(wrap_pyfunction!(find_irs, m)?)?;
    Ok(())
}

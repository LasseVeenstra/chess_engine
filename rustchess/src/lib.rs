use pyo3::prelude::*;

mod bitboard_helper;
mod chessboard_helper;
mod chessboard;
mod lookuptables;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn create_lookup(search_number: usize) -> PyResult<()> {
    let mut lookup = lookuptables::CreateLookUpTables::new();
    lookup.create_all(search_number as u32).unwrap();
    Ok(())
}

#[pyclass]
pub struct PyChessboard {
    cb: chessboard::Chessboard,
    value: i32
}

#[pymethods]
impl PyChessboard {
    #[new]
    pub fn new() -> Self {
        PyChessboard { cb: chessboard::Chessboard::new_start(), value: 5 }
    }
    pub fn value(&self) -> i32 {
        self.value
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn rustchess(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(create_lookup, m)?)?;
    m.add_class::<PyChessboard>()?;
    m.add_class::<chessboard::Chessboard>()?;
    Ok(())
}
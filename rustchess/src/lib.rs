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

/// A Python module implemented in Rust.
#[pymodule]
fn rustchess(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}

fn main() {
    let mut lookup = lookuptables::CreateLookUpTables::new();
    lookup.create_all(1000);
}
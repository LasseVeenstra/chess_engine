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

/// A Python module implemented in Rust.
#[pymodule]
fn rustchess(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(create_lookup, m)?)?;
    Ok(())
}


fn main() {
    
}
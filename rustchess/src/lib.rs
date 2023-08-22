mod bitboard_helper;
mod chessboard_helper;
mod chessboard;
mod lookuptables;
mod chess_computer;
use pyo3::prelude::*;


#[pyfunction]
fn create_lookup(search_number: usize) -> PyResult<()> {
    let mut lookup = lookuptables::CreateLookUpTables::new();
    lookup.create_all(search_number as u32).unwrap();
    Ok(())
}

#[pymodule]
fn RustEngine(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_lookup, m)?)?;
    m.add_class::<chessboard::Chessboard>()?;
    m.add_class::<chess_computer::RandomComputer>()?;
    
    Ok(())
}
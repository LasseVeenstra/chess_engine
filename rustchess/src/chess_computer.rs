use pyo3::prelude::*;
use crate::chessboard::*;

#[pyclass]
pub struct RandomComputer {
    chessboard: Chessboard
}

impl RandomComputer {
    
}
#[pymethods]
impl RandomComputer {
    #[new]
    pub fn new() -> RandomComputer {
        RandomComputer { chessboard: Chessboard::new_start() }
    }
    pub fn get_move_and_postion(&self) -> String {
        "asd".to_string()
    }
}
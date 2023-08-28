use pyo3::prelude::*;
use crate::chessboard::*;
use crate::chessboard_helper::*;
use rand::seq::SliceRandom;

pub trait RecieveAndReturnMove {
    // recieves a mutable reference to the current chessboard and then returns a new move 
    fn return_move(&mut self, chessboard: &mut Chessboard) -> Move;
}

#[pyclass]
pub struct RandomComputer {
}

impl RecieveAndReturnMove for RandomComputer {
    fn return_move(&mut self, chessboard: &mut Chessboard) -> Move {
        // now we get all legal moves
        let moves = chessboard.all_moves();
        // choose a random move
        *moves.choose(&mut rand::thread_rng()).expect("No moves available.")
    }
}
#[pymethods]
impl RandomComputer {
    #[new]
    pub fn new() -> RandomComputer {
        RandomComputer {}
    }
}
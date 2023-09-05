use pyo3::prelude::*;
use crate::chessboard;
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

pub struct BasicTreeSearchComputer {
}

impl RecieveAndReturnMove for BasicTreeSearchComputer {
    fn return_move(&mut self, chessboard: &mut Chessboard) -> Move {
        
    }
}

impl BasicTreeSearchComputer {
    pub fn new() -> BasicTreeSearchComputer {
        BasicTreeSearchComputer {}
    }
    pub fn evaluate(position: &Position) -> i32 {
        // evaluate the position it is given, positive evaluation means good for white
        // while negative evaluation means good for black

    }

    pub fn minimax(chessboard: &mut Chessboard, depth: u8, maximizing_player: bool) -> i32 {
        // depth is how far ahead we want to search, maximizing_player deals with either white to move or black
        // TODO add game over as well in this if statement
        if depth == 0 {
            return BasicTreeSearchComputer::evaluate(chessboard.get_position())
        }

        if maximizing_player {
            let mut max_eval = -100000;
            for new_move in chessboard.all_moves().iter() {
                chessboard.move_piece(&new_move);
                let eval = BasicTreeSearchComputer::minimax(chessboard, depth, maximizing_player);
                max_eval = 
                chessboard.undo();
            }
        }
    }
}
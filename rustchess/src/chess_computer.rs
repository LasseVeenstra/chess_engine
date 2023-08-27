use pyo3::prelude::*;
use crate::chessboard::*;
use crate::chessboard_helper::*;
use rand::seq::SliceRandom;

pub trait RecieveAndReturnMove {
    // This function should recieve a move and then return the move the computer(or human) wants to make
    fn recieve_and_return_move(&mut self, new_move: Move) -> Move;
}

pub struct RandomComputer {
    chessboard: Chessboard
}

impl RecieveAndReturnMove for RandomComputer {
    fn recieve_and_return_move(&mut self, new_move: Move) -> Move {
        self.chessboard.move_piece(new_move);
        // now we get all legal moves
        let moves = self.chessboard.all_moves();
        // choose a random move
        *moves.choose(&mut rand::thread_rng()).expect("No moves available.")
    }
}

impl RandomComputer {
    pub fn new() -> RandomComputer {
        RandomComputer { chessboard: Chessboard::new_start() }
    }
}
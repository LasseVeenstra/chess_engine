use pyo3::prelude::*;
use serde::de;
use crate::chessboard;
use crate::chessboard::*;
use crate::chessboard_helper::*;
use crate::bitboard_helper::*;
use rand::seq::SliceRandom;
use std::cmp;

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
    best_move: Option<Move>,
    depth: u8
}

impl RecieveAndReturnMove for BasicTreeSearchComputer {
    fn return_move(&mut self, chessboard: &mut Chessboard) -> Move {
        let depth = 4;
        self.depth = depth;
        match chessboard.get_to_move() {
            ToMove::White => self.minimax(chessboard, depth, true),
            ToMove::Black => self.minimax(chessboard, depth, false)
        };
        self.best_move.unwrap()
    }
}

impl BasicTreeSearchComputer {
    pub fn new() -> BasicTreeSearchComputer {
        BasicTreeSearchComputer {best_move: None, depth: 4}
    }
    pub fn static_evaluate(position: &Position) -> i32 {
        // evaluate the position it is given, positive evaluation means good for white
        // while negative evaluation means good for black
        let mut eval = 0;
        // subtract the value of all black pieces on the board
        let black_pieces = &position.black_pieces;
        eval -= bb_to_vec(black_pieces.get_bb_bishops()).len() as i32 * 30;
        eval -= bb_to_vec(black_pieces.get_bb_knights()).len() as i32 * 30;
        eval -= bb_to_vec(black_pieces.get_bb_rooks()).len() as i32 * 50;
        eval -= bb_to_vec(black_pieces.get_bb_queens()).len() as i32 * 90;
        eval -= bb_to_vec(black_pieces.get_bb_pawns()).len() as i32 * 10;
        // subtract the value of all white pieces on the board
        let white_pieces = &position.white_pieces;
        eval += bb_to_vec(white_pieces.get_bb_bishops()).len() as i32 * 30;
        eval += bb_to_vec(white_pieces.get_bb_knights()).len() as i32 * 30;
        eval += bb_to_vec(white_pieces.get_bb_rooks()).len() as i32 * 50;
        eval += bb_to_vec(white_pieces.get_bb_queens()).len() as i32 * 90;
        eval += bb_to_vec(white_pieces.get_bb_pawns()).len() as i32 * 10;

        eval
    }

    pub fn minimax(&mut self, chessboard: &mut Chessboard, depth: u8, maximizing_player: bool) -> i32 {
        // depth is how far ahead we want to search, maximizing_player deals with either white to move or black
        if depth == 0 {
            return BasicTreeSearchComputer::static_evaluate(chessboard.get_position())
        }

        if maximizing_player {
            let mut max_eval = -100000;
            for new_move in chessboard.all_moves().iter() {
                chessboard.move_piece(new_move).unwrap();
                let eval = self.minimax(chessboard, depth - 1, false);
                max_eval = cmp::max(max_eval, eval);
                if depth == self.depth && max_eval == eval {
                    self.best_move = Some(*new_move);
                }
                chessboard.undo();
                return max_eval
            }
        }
        else {
            let mut min_eval = 100000;
            for new_move in chessboard.all_moves().iter() {
                chessboard.move_piece(new_move).unwrap();
                let eval = self.minimax(chessboard, depth - 1, true);
                min_eval = cmp::min(min_eval, eval);
                if depth == self.depth && min_eval == eval {
                    self.best_move = Some(*new_move);
                }
                chessboard.undo();
                return min_eval
            }
        }
        // we will reach this whenever the are no legal moves that we can loop over
        return BasicTreeSearchComputer::static_evaluate(chessboard.get_position())
    }
}
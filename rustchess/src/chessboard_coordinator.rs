use pyo3::prelude::*;
use crate::chessboard::*;
use crate::chessboard_helper::*;
use crate::chess_computer::*;

pub struct Coordinator<T: RecieveAndReturnMove> {
    // The coordinator is able to coordinate a game between two humans, two computers or human vs computer
    // The two players
    player1: T,
    player2: T,
    // The main chessboard
    chessboard: Chessboard,
}

impl<T: RecieveAndReturnMove> Coordinator<T> {
    pub fn new_human_vs_human() -> Coordinator<T> {
        Coordinator { player1: (), player2: (), chessboard: () }
    }
}
use pyo3::prelude::*;
use crate::chessboard::*;
use crate::chessboard_helper::*;
use crate::chess_computer::*;

pub struct Coordinator<T: RecieveAndReturnMove> {
    // The coordinator is able to coordinate a game between two humans, two computers or human vs computer
    // The two computers, None indicates that we want the human to make the move instead of a computer, Some(T) is the computer
    computer1: Option<T>,
    computer2: Option<T>,
    // The main chessboard
    chessboard: Chessboard,
}

impl<T: RecieveAndReturnMove> Coordinator<T> {
    pub fn new_human_vs_human() -> Coordinator<T> {
        Coordinator { computer1: None, computer2: None, chessboard: Chessboard::new_start() }
    }
    pub fn new_computer_vs_computer(comp1: T, comp2: T) -> Coordinator<T> {
        Coordinator { computer1: Some(comp1), computer2: Some(comp2), chessboard: Chessboard::new_start() }
    }
    pub fn new_human_vs_computer(comp2: T) -> Coordinator<T> {
        Coordinator { computer1: None, computer2: Some(comp2), chessboard: Chessboard::new_start() }
    }
    pub fn new_computer_vs_human(comp1: T) -> Coordinator<T> {
        Coordinator { computer1: Some(comp1), computer2: None, chessboard: Chessboard::new_start() }
    }
}
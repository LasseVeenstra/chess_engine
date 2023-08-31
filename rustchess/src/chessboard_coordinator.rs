use pyo3::prelude::*;
use crate::bitboard_helper::to_stringboard;
use crate::chessboard::*;
use crate::chessboard_helper::*;
use crate::chess_computer::*;

// converts a string into an option<computer>
pub fn computer_from_string(name: &str) -> Option<Box<dyn RecieveAndReturnMove + Send>> {
    // keep adding newer and different versions of computers
    match name {
        "human" => None,
        "random" => Some(Box::new(RandomComputer::new())),
        _ => None
    }
}

#[pyclass]
pub struct Coordinator {
    // The coordinator is able to coordinate a game between two humans, two computers or human vs computer
    // The two computers, None indicates that we want the human to make the move instead of a computer, Some(T) is the computer.
    // Computer1 always plays as white whereas computer2 will always play as black.
    computer1: Option<Box<dyn RecieveAndReturnMove + Send>>,
    computer2: Option<Box<dyn RecieveAndReturnMove + Send>>,
    // The main chessboard
    chessboard: Chessboard,
    // allow to select a square for user input
    selected: Selected,

}

impl Coordinator {
    pub fn next_move(&mut self, new_move: Option<Move>) -> Result<(), NoLegalMoveInputError> {
        // This function will make the next move on the board. If Some(new_move) is not legal we will return a NoLegalMoveInputError.
        // Note that Some(new_move) will only be used whenever the player that has to move, is a human, i.e. computer1/2 is a None.
        // If computer1 has to move and computer1 is Some(T) then any value Some value passed into new_move will be ignored, since
        // the computer1 will make a move on his own.
        let to_move = self.chessboard.get_to_move();
        
        match to_move {
            // white to move
            ToMove::White => {
                match &mut self.computer1 {
                    // Black to move and a computer is playing black
                    Some(computer) => {
                        let computer_move = computer.return_move(&mut self.chessboard);
                        self.chessboard.move_piece(computer_move)?;
                    },
                    // Black to move but a human is playing black
                    None => {
                        match new_move {
                            // play the move provided
                            Some(new_move) => self.chessboard.move_piece(new_move)?,
                            // We want the human to move but there was no move provided
                            None => return Err(NoLegalMoveInputError)
                        }
                    }
                }
            },
            // black to move
            ToMove::Black => {
                match &mut self.computer2 {
                    // Black to move and a computer is playing black
                    Some(computer) => {
                        let computer_move = computer.return_move(&mut self.chessboard);
                        self.chessboard.move_piece(computer_move)?;
                    },
                    // Black to move but a human is playing black
                    None => {
                        match new_move {
                            // play the move provided
                            Some(new_move) => self.chessboard.move_piece(new_move)?,
                            // We want the human to move but there was no move provided
                            None => return Err(NoLegalMoveInputError)
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn select_new(&mut self, index: u8) {
        let w_pieces = self.chessboard.get_white_pieces();
        let b_pieces = self.chessboard.get_black_pieces();

        match self.chessboard.get_to_move() {
            // check if it is a valid new select
            ToMove::White => {
                if (w_pieces >> index) & 1 == 1 {
                    self.selected = Selected::White(index)
                }
            }
            // check if it is a valid new select
            ToMove::Black => {
                if (b_pieces >> index) & 1 == 1 {
                    self.selected = Selected::Black(index)
                }
            }
        }
    }
}

#[pymethods]
impl Coordinator {
    #[new]
    pub fn new() -> Coordinator {
        Coordinator { computer1: None, computer2: None, chessboard: Chessboard::new_start(), selected: Selected::None}
    }
    #[staticmethod]
    pub fn new_human_vs_human() -> Coordinator {
        Coordinator { computer1: None, computer2: None, chessboard: Chessboard::new_start(), selected: Selected::None}
    }
    #[staticmethod]
    pub fn new_computer_vs_computer(comp1: &str, comp2: &str) -> Coordinator {
        Coordinator { computer1: computer_from_string(comp1), computer2: computer_from_string(comp2), chessboard: Chessboard::new_start(), selected: Selected::None}
    }
    #[staticmethod]
    pub fn new_human_vs_computer(comp2: &str) -> Coordinator {
        Coordinator { computer1: None, computer2: computer_from_string(comp2), chessboard: Chessboard::new_start(), selected: Selected::None}
    }
    #[staticmethod]
    pub fn new_computer_vs_human(comp1: &str) -> Coordinator {
        Coordinator { computer1: computer_from_string(comp1), computer2: None, chessboard: Chessboard::new_start(), selected: Selected::None}
    }
    pub fn set_player1(&mut self, name: &str) {
        self.computer1 = computer_from_string(name);
    }
    pub fn set_player2(&mut self, name: &str) {
        self.computer2 = computer_from_string(name);
    }
    pub fn load_fen(&mut self, fen: String) {
        self.chessboard.load_fen(fen);
    }

    pub fn to_string(&self) -> String {
        self.chessboard.to_string()
    }

    pub fn undo(&mut self) {
        self.chessboard.undo();
    }

    pub fn get_selected(&self) -> i32 {
        // return -1 in case we have no square selected, 
        // note that Option<> is not available because this function
        // is available to python, so we must strictly use integers
        match self.selected {
            Selected::None => -1,
            Selected::Black(i) => i as i32,
            Selected::White(i) => i as i32
        }
    }
    pub fn next_computer_move(&mut self) {
        // tries to make a new computer move
        match self.next_move(None) {
            Ok(_) => {},
            Err(_) => {}
        }
    }

    pub fn input_select(&mut self, index: u8) {
        match self.selected {
            // in case we have not selected anything
            Selected::None => {
                self.select_new(index);
            }
            // in case we have already selected something
            Selected::White(old_index) => {
                match self.chessboard.get_to_move() {
                    // might be possible
                    ToMove::White => {
                        match self.next_move(Some(Move { from: old_index, to: index, on_promotion: Some(PiecePromotes::Queen)})) {
                            Ok(_) => {},
                            Err(_) => {}
                        };
                        // remove the highlight
                        self.selected = Selected::None;
                    }
                    // can't move a white piece when black to move
                    ToMove::Black => {}
                    }
                }
            Selected::Black(old_index) => {
                match self.chessboard.get_to_move() {
                    // might be possible
                    ToMove::Black => {
                        match self.next_move(Some(Move { from: old_index, to: index, on_promotion: Some(PiecePromotes::Queen)})) {
                            Ok(_) => {},
                            Err(_) => {}
                        };
                        self.selected = Selected::None;
                    }
                    ToMove::White => {}
                }
            }
        }
    }
    pub fn reset_position(&mut self) {
        self.chessboard = Chessboard::new_start();
    }
    pub fn empty_position(&mut self) {
        self.chessboard = Chessboard::new();
    }

    pub fn test_positions(&mut self) {
        self.chessboard.test_position_depth();
    }
    pub fn get_legal_captures(&mut self, index: u8) -> Vec<u8> {
        self.chessboard.get_legal_captures(index)
    }
    pub fn get_legal_non_captures(&mut self, index: u8) -> Vec<u8> {
        self.chessboard.get_legal_non_captures(index)
    }
}
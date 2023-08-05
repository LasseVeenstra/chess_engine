use pyo3::prelude::*;
use crate::bitboard_helper::*;
use crate::lookuptables::LoadMoves;


const BLACK_PAWN_STARTING_BB: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
const BLACK_ROOK_STARTING_BB: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001;
const BLACK_BISHOP_STARTING_BB: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100;
const BLACK_KNIGHT_STARTING_BB: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010;
const BLACK_QUEEN_STARTING_BB: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000;
const BLACK_KING_STARTING_BB: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000;
const WHITE_PAWN_STARTING_BB: u64 = 0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_ROOK_STARTING_BB: u64 = 0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_BISHOP_STARTING_BB: u64 = 0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_KNIGHT_STARTING_BB: u64 = 0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_QUEEN_STARTING_BB: u64 = 0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_KING_STARTING_BB: u64 = 0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

const FILE_A_BB: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
const FILE_B_BB: u64 = 0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
const FILE_C_BB: u64 = 0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000; 
const FILE_D_BB: u64 = 0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
const FILE_E_BB: u64 = 0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
const FILE_F_BB: u64 = 0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
const FILE_G_BB: u64 = 0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
const FILE_H_BB: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;

enum ToMove {
    White,
    Black
}

pub enum PieceColor {
    White,
    Black,
    None
}

#[derive(Debug)]
enum Selected {
    None,
    White(u8),
    Black(u8)
}

#[derive(Debug)]
pub enum PieceType {
    WhiteRook,
    BlackRook,
    WhiteKnight,
    BlackKnight,
    WhiteBishop,
    BlackBishop,
    WhiteQueen,
    BlackQueen,
    WhiteKing,
    BlackKing,
    WhitePawn,
    BlackPawn,
    EmptySquare
}

impl PieceType {
    pub fn to_char(&self) -> char {
        match *self {
            PieceType::WhitePawn => 'P',
            PieceType::BlackPawn => 'p',
            PieceType::WhiteRook => 'R',
            PieceType::BlackRook => 'r',
            PieceType::WhiteBishop => 'B',
            PieceType::BlackBishop => 'b',
            PieceType::WhiteKnight => 'N',
            PieceType::BlackKnight => 'n',
            PieceType::WhiteKing => 'K',
            PieceType::BlackKing => 'k',
            PieceType::WhiteQueen => 'Q',
            PieceType::BlackQueen => 'q',
            PieceType::EmptySquare => ' '
        }
    }
}

pub struct Position {
    bb_wp: u64, // BitBoard White Pawn
    bb_wr: u64, // bitBoard White Rook
    bb_wb: u64,
    bb_wn: u64,
    bb_wq: u64,
    bb_wk: u64,
    bb_bp: u64,
    bb_br: u64,
    bb_bb: u64,
    bb_bn: u64, // BitBoard Black Knight
    bb_bq: u64, // BitBoard Black Queen
    bb_bk: u64, // BitBoard Black King
    es_target: Option<u8>, // en passant target square can be either None or some index of the target
    white_castle: bool,
    black_caslte: bool,
    to_move: ToMove
}

impl Position {
    pub fn new() -> Position {
        Position { 
            bb_wp: 0, 
            bb_wr: 0, 
            bb_wb: 0, 
            bb_wn: 0, 
            bb_wq: 0, 
            bb_wk: 0, 
            bb_bp: 0, 
            bb_br: 0, 
            bb_bb: 0, 
            bb_bn: 0, 
            bb_bq: 0, 
            bb_bk: 0, 
            es_target: None,
            white_castle: true,
            black_caslte: true,
            to_move: ToMove::White
        }
    }

    pub fn new_start() -> Position {
        Position {
            bb_wp: WHITE_PAWN_STARTING_BB, 
            bb_wr: WHITE_ROOK_STARTING_BB, 
            bb_wb: WHITE_BISHOP_STARTING_BB, 
            bb_wn: WHITE_KNIGHT_STARTING_BB, 
            bb_wq: WHITE_QUEEN_STARTING_BB, 
            bb_wk: WHITE_KING_STARTING_BB, 
            bb_bp: BLACK_PAWN_STARTING_BB, 
            bb_br: BLACK_ROOK_STARTING_BB, 
            bb_bb: BLACK_BISHOP_STARTING_BB, 
            bb_bn: BLACK_KNIGHT_STARTING_BB, 
            bb_bq: BLACK_QUEEN_STARTING_BB, 
            bb_bk: BLACK_KING_STARTING_BB, 
            es_target: None,
            white_castle: true,
            black_caslte: true,
            to_move: ToMove::White
        }
    }

    pub fn black_pieces(&mut self) -> u64 {
        self.bb_bp | self.bb_br | self.bb_bn | self.bb_bb | self.bb_bk | self.bb_bq
    }
    pub fn white_pieces(&mut self) -> u64 {
        self.bb_wp | self.bb_wr | self.bb_wn | self.bb_wb | self.bb_wk | self.bb_wq
    }

    pub fn detect_piece_type(&self, piece_index: u8) -> PieceType {
        // Detects the piece type of the current piece location. Piece index must
        // be an integer between 0 and 63, not a bitboard with one bit!
        if (self.bb_wp >> piece_index) & 1 == 1 {
            PieceType::WhitePawn
        }
        else if (self.bb_bp >> piece_index) & 1 == 1 {
            PieceType::BlackPawn
        }
        else if (self.bb_wr >> piece_index) & 1 == 1 {
            PieceType::WhiteRook
        }
        else if (self.bb_br >> piece_index) & 1 == 1 {
            PieceType::BlackRook
        }
        else if (self.bb_wn >> piece_index) & 1 == 1 {
            PieceType::WhiteKnight
        }
        else if (self.bb_bn >> piece_index) & 1 == 1 {
            PieceType::BlackKnight
        }
        else if (self.bb_wb >> piece_index) & 1 == 1 {
            PieceType::WhiteBishop
        }
        else if (self.bb_bb >> piece_index) & 1 == 1 {
            PieceType::BlackBishop
        }
        else if (self.bb_wk >> piece_index) & 1 == 1 {
            PieceType::WhiteKing
        }
        else if (self.bb_bk >> piece_index) & 1 == 1 {
            PieceType::BlackKing
        }
        else if (self.bb_wq >> piece_index) & 1 == 1 {
            PieceType::WhiteQueen
        }
        else if (self.bb_bq >> piece_index) & 1 == 1 {
            PieceType::BlackQueen
        }
        else {
            PieceType::EmptySquare
        }
    }

    pub fn detect_piece_color(&self, index: u8) -> PieceColor {
        if self.detect_piece_type(index).to_char().is_uppercase() {
            PieceColor::White
        }
        else if !self.detect_piece_type(index).to_char().is_whitespace() {
            PieceColor::Black
        }
        else {
            PieceColor::None
        }
    }

    pub fn to_string(&self) -> String {
        (0..64).map(|i| self.detect_piece_type(i).to_char().to_string()).collect::<Vec<String>>().join("")
    }

}

#[pyclass]
pub struct Chessboard {
    current_position: Position,
    selected: Selected,
    pseudo_moves: LoadMoves,
}

impl Chessboard {
    fn get_legal_moves(&mut self, index: u8, piece_color: &PieceColor) -> u64 {
        // index is the index of the piece of which we want to find the legal moves.
        // This function already assumes that the correct player is making the move
        // and that he is not trying to move to his own piece, it also assumes that 
        // on old index is indeed a piece of the player that has to move.
        let blockers = self.current_position.white_pieces() | self.current_position.black_pieces();
        let mut legal_moves = match self.current_position.detect_piece_type(index) {
            PieceType::WhitePawn  => self.pseudo_moves.white_pawn(index as usize),
            PieceType::BlackPawn => self.pseudo_moves.black_pawn(index as usize),
            PieceType::BlackKnight | PieceType::WhiteKnight => self.pseudo_moves.knight(index as usize),
            PieceType::BlackBishop | PieceType::WhiteBishop => *self.pseudo_moves.bishop(index as usize, blockers).unwrap(),
            PieceType::BlackRook | PieceType::WhiteRook => *self.pseudo_moves.rook(index as usize, blockers).unwrap(),
            PieceType::BlackKing | PieceType::WhiteKing => self.pseudo_moves.king(index as usize),
            PieceType::WhiteQueen | PieceType::BlackQueen => self.pseudo_moves.queen(index as usize, blockers).unwrap(),
            _ => 0
        };
        // remove the ability to capture own pieces
        legal_moves = match piece_color {
            PieceColor::White => subtract_bb(legal_moves, self.current_position.white_pieces()),
            PieceColor::Black => subtract_bb(legal_moves, self.current_position.black_pieces()),
            _ => 0
        };

        // return final moves
        legal_moves
    }

    fn check_move_for_legal(&mut self, old_index: u8, index: u8, piece_color: &PieceColor) -> bool {
        let legal_moves = self.get_legal_moves(old_index, piece_color);
        // if after all the constraints the bit is still available the the move must be legal
        if (legal_moves >> old_index) & 1 == 1 {
            return true
        }
        else {
            false
        }
    }

    fn move_piece(&mut self, old_index: u8, index: u8, piece_color: &PieceColor) {
        // when possible move piece from old index to new index
        println!("Moving {} to {}", old_index, index);
    }

    fn select_new(&mut self, index: u8) {
        let w_pieces = self.current_position.white_pieces();
        let b_pieces = self.current_position.black_pieces();

        match self.current_position.to_move {
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
impl Chessboard {
    #[staticmethod]
    pub fn new_start() -> Chessboard {
        Chessboard { current_position: Position::new_start(),
        selected: Selected::None,
        pseudo_moves: LoadMoves::new() }
    }
    #[new]
    pub fn new() -> Chessboard {
        Chessboard { current_position: Position::new(),
        selected: Selected::None,
        pseudo_moves: LoadMoves::new() }
    }
    pub fn to_string(&self) -> String {
        self.current_position.to_string()
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

    pub fn get_legal_captures(&mut self, index: u8) -> Vec<u8> {
        // index must be the index of the piece of which we want to get legal captures
        let piece_color = self.current_position.detect_piece_color(index);
        let legal_moves = self.get_legal_moves(index, &piece_color);
        let legal_captures = match piece_color {
            PieceColor::White => legal_moves & self.current_position.black_pieces(),
            PieceColor::Black => legal_moves & self.current_position.white_pieces(),
            _ => 0
        };
        bb_to_vec(legal_captures)
    }

    pub fn get_legal_non_capture_moves(&mut self, index: u8) -> Vec<u8> {
        // index must be the index of the piece of which we want to get legal captures
        let piece_color = self.current_position.detect_piece_color(index);
        let legal_moves = self.get_legal_moves(index, &piece_color);
        let legal_non_capture = match piece_color {
            PieceColor::White => subtract_bb(legal_moves, self.current_position.black_pieces()),
            PieceColor::Black => subtract_bb(legal_moves, self.current_position.white_pieces()),
            _ => 0
        };
        bb_to_vec(legal_non_capture)
    }

    pub fn input_select(&mut self, index: u8) {
        let w_pieces = self.current_position.white_pieces();
        let b_pieces = self.current_position.black_pieces();

        match self.selected {
            // in case we have not selected anything
            Selected::None => {
                self.select_new(index);
            }
            // in case we have already selected something
            Selected::White(old_index) => {
                match self.current_position.to_move {
                    // might be possible
                    ToMove::White => {
                        // if newly selected piece is not white we can move or capture there
                        if !((w_pieces >> index) & 1 == 1) {
                            self.move_piece(old_index, index, &PieceColor::White);
                        }
                        // remove the highlight
                        self.selected = Selected::None;
                    }
                    // can't move a white piece when black to move
                    ToMove::Black => {}
                    }
                }
            Selected::Black(old_index) => {
                match self.current_position.to_move {
                    // might be possible
                    ToMove::Black => {
                        // if newly selected piece is not black we can move or capture there
                        if !((b_pieces >> index) & 1 == 1) {
                            self.move_piece(old_index, index, &PieceColor::Black);
                        }
                        self.selected = Selected::None;
                    }
                    ToMove::White => {}
                }
            }
        }
    }

}
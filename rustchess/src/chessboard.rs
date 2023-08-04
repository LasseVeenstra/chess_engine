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

enum ToMove {
    White,
    Black
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

pub struct Position {
    pseudo_moves: LoadMoves,
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
    pub fn new_empty() -> Position {
        Position { 
            pseudo_moves: LoadMoves::new(),
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
            pseudo_moves: LoadMoves::new(),
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

    pub fn detect_piece_type(&self, piece_index: u64) -> PieceType {
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

}


pub struct Chessboard {
    current_position: Position
}

impl Chessboard {
    fn new_start() -> Chessboard {
        Chessboard { current_position: Position::new_start() }
    }
}
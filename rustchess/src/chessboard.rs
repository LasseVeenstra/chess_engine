use pyo3::prelude::*;
use crate::{bitboard_helper::*, lookuptables};
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

const WHITE_KINGSIDE_CASTLE_MAP: u64 = 0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_QUEENSIDE_CASTLE_MAP: u64 = 0b00001100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BLACK_KINGSIDE_CASTLE_MAP: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000;
const BLACK_QUEENSIDE_CASTLE_MAP: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001100;
const WHITE_KINGSIDE_CASTLE_GOAL: u64 = 0b01000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const WHITE_QUEENSIDE_CASTLE_GOAL: u64 = 0b00000100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BLACK_KINGSIDE_CASTLE_GOAL: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000000;
const BLACK_QUEENSIDE_CASTLE_GOAL: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100;

const FILE_H_BB: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
const FILE_G_BB: u64 = 0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
const FILE_F_BB: u64 = 0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000; 
const FILE_E_BB: u64 = 0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
const FILE_D_BB: u64 = 0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
const FILE_C_BB: u64 = 0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
const FILE_B_BB: u64 = 0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
const FILE_A_BB: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;

// array that converts piece index to file
const INDEX2FILE: [u64; 64] = [FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB];

enum ToMove {
    White,
    Black
}
#[derive(Debug)]
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
    white_kingside_castle: bool,
    black_kingside_castle: bool,
    white_queenside_castle: bool,
    black_queenside_castle: bool,
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
            white_kingside_castle: true,
            black_kingside_castle: true,
            white_queenside_castle: true,
            black_queenside_castle: true,
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
            white_kingside_castle: true,
            black_kingside_castle: true,
            white_queenside_castle: true,
            black_queenside_castle: true,
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

    pub fn piece_type2bb(&self, piece_type: &PieceType) -> u64 {
        // note that the returned bitboard is a copy of the stored bitboard,
        // so it is not a mutable reference!
        match piece_type {
            PieceType::WhitePawn => self.bb_wp,
            PieceType::BlackPawn => self.bb_bp,
            PieceType::WhiteRook => self.bb_wr,
            PieceType::BlackRook => self.bb_br,
            PieceType::WhiteBishop => self.bb_wb,
            PieceType::BlackBishop => self.bb_bb,
            PieceType::WhiteKnight => self.bb_wn,
            PieceType::BlackKnight => self.bb_bn,
            PieceType::WhiteKing => self.bb_wk,
            PieceType::BlackKing => self.bb_bk,
            PieceType::WhiteQueen => self.bb_wq,
            PieceType::BlackQueen => self.bb_bq,
            PieceType::EmptySquare => 0
        }
    }

    pub fn set_bb_of_piece_type(&mut self,bb: u64, piece_type: &PieceType) {
        match piece_type {
            PieceType::WhitePawn => self.bb_wp = bb,
            PieceType::BlackPawn => self.bb_bp = bb,
            PieceType::WhiteRook => self.bb_wr = bb,
            PieceType::BlackRook => self.bb_br = bb,
            PieceType::WhiteBishop => self.bb_wb = bb,
            PieceType::BlackBishop => self.bb_bb = bb,
            PieceType::WhiteKnight => self.bb_wn = bb,
            PieceType::BlackKnight => self.bb_bn = bb,
            PieceType::WhiteKing => self.bb_wk = bb,
            PieceType::BlackKing => self.bb_bk = bb,
            PieceType::WhiteQueen => self.bb_wq = bb,
            PieceType::BlackQueen => self.bb_bq = bb,
            PieceType::EmptySquare => {}
        };
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
    // every time we make a move this will need to be cleared
    legal_moves_cache: [Option<u64>; 64],
    enemy_heat_cache: Option<u64>
}

impl Chessboard {
    fn remove_piece_run_add_piece<F, T>(&mut self, pieces: u64, piece_type: &PieceType, f: F) -> T
    where
        F: FnOnce() -> T
        {
            // first we remove the pieces from the board
            let board_pieces = self.current_position.piece_type2bb(piece_type);
            self.current_position.set_bb_of_piece_type(subtract_bb(board_pieces, pieces), piece_type);
            // now run the closure that has been passed
            let res = f();
            // now add the pieces back onto the board
            self.current_position.set_bb_of_piece_type(board_pieces, piece_type);

            res
        }

    fn get_heatmap(&mut self, color: &PieceColor) -> u64 {
        // returns all squares that the color can move to in a bitboard,

        // check if we have already calculated the enemy heat before
        match self.enemy_heat_cache {
            Some(heat) => return heat,
            None => {}
        }
        // calculate the heatmap
        let mut heat = 0;
        let all_pieces = match color {
            PieceColor::White => self.current_position.white_pieces(),
            PieceColor::Black => self.current_position.black_pieces(),
            _ => 0
        };
        // loop over all indices of black squares
        for index in bb_to_vec(all_pieces) {
            heat |= self.get_legal_moves(index, &color, true);
        }

        // store the heat in the cache
        self.enemy_heat_cache = Some(heat);
        heat
    }

    fn get_checking_pieces(&mut self, king_index: usize, color: &PieceColor) -> u64 {
        // returns a bitboard with all pieces that are currently checking the king
        let blockers = self.current_position.black_pieces() | self.current_position.white_pieces();
        let mut attackers: u64 = 0;
        match color {
            PieceColor::White => {
                // add possible knight checks
                attackers |= self.pseudo_moves.knight(king_index) & self.current_position.bb_bn;
                // add possible rook, bishop or queen checks
                attackers |= self.pseudo_moves.queen(king_index, blockers).unwrap() & self.current_position.bb_bq;
                attackers |= self.pseudo_moves.rook(king_index, blockers).unwrap() & self.current_position.bb_br;
                attackers |= self.pseudo_moves.bishop(king_index, blockers).unwrap() & self.current_position.bb_bb;
                // add possible pawn checks
                attackers |= subtract_bb(self.pseudo_moves.white_pawn(king_index), INDEX2FILE[king_index]) & self.current_position.bb_bp;
            }
            PieceColor::Black => {
                // add possible knight checks
                attackers |= self.pseudo_moves.knight(king_index) & self.current_position.bb_wn;
                // add possible rook, bishop or queen checks
                attackers |= self.pseudo_moves.queen(king_index, blockers).unwrap() & self.current_position.bb_wq;
                attackers |= self.pseudo_moves.rook(king_index, blockers).unwrap() & self.current_position.bb_wr;
                attackers |= self.pseudo_moves.bishop(king_index, blockers).unwrap() & self.current_position.bb_wb;
                // add possible pawn checks
                attackers |= subtract_bb(self.pseudo_moves.black_pawn(king_index), INDEX2FILE[king_index]) & self.current_position.bb_wp;
            }
            _ => {}
        }
        attackers
    }

    fn get_legal_moves(&mut self, index: u8, piece_color: &PieceColor, heat_only: bool) -> u64 {
        // index is the index of the piece of which we want to find the legal moves.
        // This function already assumes that the correct player is making the move
        // and that he is not trying to move to his own piece, it also assumes that 
        // on old index is indeed a piece of the player that has to move.

        // Check if we have already calculated the moves before
        match self.legal_moves_cache[index as usize] {
            Some(legal_moves) => if !heat_only {return legal_moves},
            None => {}
        }

        // get the king index
        let king_index: usize = match piece_color {
            PieceColor::White => bb_to_vec(self.current_position.bb_wk)[0] as usize,
            PieceColor::Black => bb_to_vec(self.current_position.bb_bk)[0] as usize,
            _ => 0
        };

        let piece_type = self.current_position.detect_piece_type(index);

        // if the piece we want to move is not the king and if we are in double check, only king moves
        // are allowed so we return no legal moves
        let pieces_giving_check = self.get_checking_pieces(king_index, piece_color);
        match piece_type {
            PieceType::BlackKing | PieceType::WhiteKing => {},
            _ => {
                if bb_to_vec(pieces_giving_check).len() > 1 {
                    return 0
                }
            }
        }
        // set friendly and enemy pieces and enemy color
        let (friendly_pieces, enemy_pieces, enemy_color, enemy_king) = match piece_color {
            PieceColor::White => (self.current_position.white_pieces(), self.current_position.black_pieces(), &PieceColor::Black, self.current_position.bb_bk),
            PieceColor::Black => (self.current_position.black_pieces(), self.current_position.white_pieces(), &PieceColor::White, self.current_position.bb_wk),
            _ => (0, 0, &PieceColor::None, 0)
        };
        // Get all blockers.
        let blockers = friendly_pieces | enemy_pieces;
        let mut legal_moves = match piece_type {
            PieceType::WhitePawn | PieceType::BlackPawn => {
                let piece_file = INDEX2FILE[index as usize];
                let pawn_moves = match piece_color {
                    PieceColor::White => self.pseudo_moves.white_pawn(index as usize),
                    PieceColor::Black => self.pseudo_moves.black_pawn(index as usize),
                    _ => 0
                };
                // moves that go directly forwards
                let front = if !heat_only {
                    subtract_bb(pawn_moves & piece_file, blockers) & *self.pseudo_moves.rook(index as usize, blockers).expect("Couldn't get rook moves")  
                } else {0};
                // captures and pieces that block our way
                let captures = match self.current_position.es_target {
                    //_ => subtract_bb(enemy_pieces & pawn_moves, piece_file),
                    Some(target) => subtract_bb(set_bit(enemy_pieces, target) & pawn_moves, piece_file),
                    None => {
                        if heat_only {
                            subtract_bb(pawn_moves, piece_file)
                        } else {subtract_bb(enemy_pieces & pawn_moves, piece_file)}
                    }
                };
                front | captures
            }
            PieceType::BlackKnight | PieceType::WhiteKnight => self.pseudo_moves.knight(index as usize),
            PieceType::BlackBishop | PieceType::WhiteBishop => {
                if heat_only {
                    *self.pseudo_moves.bishop(index as usize, subtract_bb(blockers, enemy_king)).expect("Couldn't get bishop moves")
                } else {*self.pseudo_moves.bishop(index as usize, blockers).expect("Couldn't get bishop moves")}
            },
            PieceType::BlackRook | PieceType::WhiteRook => {
                if heat_only {
                    *self.pseudo_moves.rook(index as usize, subtract_bb(blockers, enemy_king)).expect("Couldn't get rook moves")    
                } else {*self.pseudo_moves.rook(index as usize, blockers).expect("Couldn't get rook moves")}
            },
            PieceType::BlackKing | PieceType::WhiteKing => {
                if !heat_only {
                    subtract_bb(self.pseudo_moves.king(index as usize), self.get_heatmap(enemy_color) | self.get_defended(enemy_color)) | self.get_castling_squares(king_index, piece_color)
                }
                else {
                    self.pseudo_moves.king(index as usize)
                }
            },
            PieceType::WhiteQueen | PieceType::BlackQueen => {
                if heat_only {
                    self.pseudo_moves.queen(index as usize, subtract_bb(blockers, enemy_king)).expect("Couldn't get queen moves")    
                } else {self.pseudo_moves.queen(index as usize, blockers).expect("Couldn't get queen moves")}
            },
            _ => 0
        };

        // remove the ability to capture own pieces
        legal_moves = subtract_bb(legal_moves, friendly_pieces);

        // now we restrict the legal moves if we are in single check
        if bb_to_vec(pieces_giving_check).len() == 1 {
            match piece_type {
                PieceType::BlackKing | PieceType::WhiteKing => {},
                _ => {
                    // split the legal moves in capture and non capture
                    let mut legal_captures = legal_moves & enemy_pieces;
                    let mut legal_non_captures = subtract_bb(legal_moves, enemy_pieces);
                    // restrict captures to only captures of the pieces that are giving check
                    legal_captures &= pieces_giving_check;

                    // the ray to the king from the piece giving check
                    let index_of_checking_piece = bb_to_vec(pieces_giving_check)[0];
                    let piece_type_of_checking_piece = self.current_position.detect_piece_type(index_of_checking_piece);
                    match piece_type_of_checking_piece {
                        PieceType::BlackPawn | PieceType::WhitePawn => {
                            // we might still be able to capture the checking pawn with en passant
                            match self.current_position.es_target {
                                Some(target) => {
                                    // if the piece giving check is right below or above the en passant target square then it must be 
                                    // that that piece just move two up and therefore we still allow en passant capture
                                    if set_bit(set_bit(0, target - 8), target + 8) & pieces_giving_check != 0 {
                                        match piece_type {
                                            PieceType::BlackPawn | PieceType::WhitePawn => {legal_non_captures = set_bit(0, target);},
                                            _ => {}
                                        }
                                    }
                                },
                                None => {legal_non_captures = 0;}
                            }
                        }
                        PieceType::BlackKnight | PieceType::WhiteKnight => {
                            legal_non_captures = 0;
                        }
                        _ => {
                            let ray2king = self.pseudo_moves.queen(king_index, blockers).unwrap() & self.pseudo_moves.queen(index_of_checking_piece as usize, blockers).unwrap();
                            // only allow moves that block the check
                            legal_non_captures &= ray2king;
                        }
                    }
                    legal_moves = legal_captures | legal_non_captures;
                }
            }
        }
        // finally we take into account the possibilty that our piece is currently pinned to the king (an absolute pin)
        match piece_type {
            // kings cannot be pinned
            PieceType::WhiteKing | PieceType::BlackKing => {},
            _ => {

            }
        }


        // store the result in the cache
        if !heat_only {self.legal_moves_cache[index as usize] = Some(legal_moves);}

        // return final moves
        legal_moves
    }

    fn get_pinned(&mut self, color: &PieceColor) -> u64 {
        // returns a bitboard with bits on all pieces that are currently pinned to the king

        let blockers = self.current_position.black_pieces() & self.current_position.white_pieces();

        let king_index = match color {
            PieceColor::White => bb_to_vec(self.current_position.bb_wk)[0] as usize,
            PieceColor::Black => bb_to_vec(self.current_position.bb_bk)[0] as usize,
            _ => 0
        };

        // from the king we slide in every direction to check if there is a pin on that direction
        for direction_index in 0..8 {
            // a ray in a certain direction ignoring all pieces on the board
            let ray = self.pseudo_moves.direction_ray(king_index, direction_index);

            let possible_pinned = self.pseudo_moves.queen(king_index, blockers).unwrap() & ray;
            // now 
        }


        0
    }

    fn get_defended(&mut self, color: &PieceColor) -> u64 {
        // returns a bitboard with bits on all pieces that are defended by one of his own pieces
        let mut defended = 0;
        let all_pieces = match color {
            PieceColor::White => self.current_position.white_pieces(),
            PieceColor::Black => self.current_position.black_pieces(),
            _ => 0
        };
        // loop over all indices of squares
        for index in bb_to_vec(all_pieces) {
            defended |= self.get_defended_by_piece(index, color);
        }
        defended
    }

    fn get_defended_by_piece(&mut self, index: u8, piece_color: &PieceColor) -> u64 {
        let piece_type = self.current_position.detect_piece_type(index);

        // set friendly and enemy pieces and enemy color
        let (friendly_pieces, enemy_pieces) = match piece_color {
            PieceColor::White => (self.current_position.white_pieces(), self.current_position.black_pieces()),
            PieceColor::Black => (self.current_position.black_pieces(), self.current_position.white_pieces()),
            _ => (0, 0)
        };
        // Get all blockers.
        let blockers = friendly_pieces | enemy_pieces;
        match piece_type {
            PieceType::WhitePawn | PieceType::BlackPawn => {
                let piece_file = INDEX2FILE[index as usize];
                let pawn_moves = match piece_color {
                    PieceColor::White => self.pseudo_moves.white_pawn(index as usize),
                    PieceColor::Black => self.pseudo_moves.black_pawn(index as usize),
                    _ => 0
                };
                subtract_bb(pawn_moves, piece_file) & friendly_pieces
            }
            PieceType::BlackKnight | PieceType::WhiteKnight => self.pseudo_moves.knight(index as usize) & friendly_pieces,
            PieceType::BlackBishop | PieceType::WhiteBishop => {
                *self.pseudo_moves.bishop(index as usize, blockers).expect("Couldn't get bishop moves") & friendly_pieces
            },
            PieceType::BlackRook | PieceType::WhiteRook => {
                *self.pseudo_moves.rook(index as usize, blockers).expect("Couldn't get rook moves") & friendly_pieces
            },
            PieceType::BlackKing | PieceType::WhiteKing => {
                    self.pseudo_moves.king(index as usize) & friendly_pieces
            },
            PieceType::WhiteQueen | PieceType::BlackQueen => {
                self.pseudo_moves.queen(index as usize, blockers).expect("Couldn't get queen moves") & friendly_pieces
            },
            _ => 0
        }
    }

    fn get_castling_squares(&mut self, king_index: usize, king_color: &PieceColor) -> u64 {
        // takes in the color of the king and return a bitboard where the castle square is either a one or zero.

        // if we are in check we can't castle
        if self.get_checking_pieces(king_index, king_color) != 0 {
            return 0
        }
        let mut castle_squares = 0;
        let all_pieces = self.current_position.white_pieces() | self.current_position.black_pieces();
        match king_color {
            PieceColor::White => {
                let heat = self.get_heatmap(&PieceColor::Black);
                if ((heat | all_pieces) & WHITE_KINGSIDE_CASTLE_MAP == 0) && self.current_position.white_kingside_castle {
                    castle_squares |= WHITE_KINGSIDE_CASTLE_GOAL;
                }
                if ((heat | all_pieces) & WHITE_QUEENSIDE_CASTLE_MAP == 0) && self.current_position.white_queenside_castle {
                    castle_squares |= WHITE_QUEENSIDE_CASTLE_GOAL;
                }
            }
            PieceColor::Black => {
                let heat = self.get_heatmap(&PieceColor::White);
                if ((heat | all_pieces) & BLACK_KINGSIDE_CASTLE_MAP == 0) && self.current_position.black_kingside_castle {
                    castle_squares |= BLACK_KINGSIDE_CASTLE_GOAL;
                }
                if ((heat | all_pieces) & BLACK_QUEENSIDE_CASTLE_MAP == 0) && self.current_position.black_queenside_castle {
                    castle_squares |= BLACK_QUEENSIDE_CASTLE_GOAL;
                }
            }
            _ => {}
        }
        castle_squares
    }

    fn check_move_for_legal(&mut self, old_index: u8, index: u8, piece_color: &PieceColor) -> bool {
        // get all legal moves
        let legal_moves = self.get_legal_moves(old_index, piece_color, false);
        // if there is a bit on the legal moves bitboard at index, the move is legal
        if (legal_moves >> index) & 1 == 1 {
            return true
        }
        else {
            false
        }
    }

    fn move_piece(&mut self, old_index: u8, index: u8, piece_color: &PieceColor) {
        // when possible move piece from old index to new index
        
        // return if the move is not legal
        if !self.check_move_for_legal(old_index, index, piece_color) {
            return
        }
        // get the piece type of the piece we want to move and the piece type
        // of the piece we want to capture (note that these are not by reference but a copy, so we must
        // replace them later.)
        let moving_piece_type = self.current_position.detect_piece_type(old_index);
        let captured_piece_type = self.current_position.detect_piece_type(index);
        let mut bb_moving_piece = self.current_position.piece_type2bb(&moving_piece_type);
        let mut bb_captured_piece = self.current_position.piece_type2bb(&captured_piece_type);
        match moving_piece_type {
            // detect en-passant for capture or new es-target
            PieceType::WhitePawn | PieceType::BlackPawn => {
                // detect en-passant capture
                match self.current_position.es_target {
                    Some(target) => {                
                        if index == target {
                            match piece_color {
                                PieceColor::White => self.current_position.bb_bp = subtract_bb(self.current_position.bb_bp, set_bit(0, index + 8)),
                                PieceColor::Black => self.current_position.bb_bp = subtract_bb(self.current_position.bb_wp, set_bit(0, index - 8)),
                                _ => {}
                            }
                        }
                    }
                    None => {}
                }
                // update en-passant target
                // if we move up two squares (or down).
                if (old_index as i16 - index as i16).abs() == 16 {
                    self.current_position.es_target = Some((old_index + index) / 2);
                }
                else {self.current_position.es_target = None;}
            }
            // detect castle move
            PieceType::WhiteKing | PieceType::BlackKing => {
                // detect if a castling move has been made

                // white kingside castling
                if index == 62 {
                    self.current_position.bb_wr = subtract_bb(self.current_position.bb_wr, set_bit(0, 63));
                    self.current_position.bb_wr = set_bit(self.current_position.bb_wr, 61);
                }
                // white queenside castling
                if index == 58 {
                    self.current_position.bb_wr = subtract_bb(self.current_position.bb_wr, set_bit(0, 56));
                    self.current_position.bb_wr = set_bit(self.current_position.bb_wr, 59);
                }
                // black kingside castling
                if index == 6 {
                    self.current_position.bb_br = subtract_bb(self.current_position.bb_br, set_bit(0, 7));
                    self.current_position.bb_br = set_bit(self.current_position.bb_br, 5);
                }
                // black queenside castling
                if index == 2 {
                    self.current_position.bb_br = subtract_bb(self.current_position.bb_br, set_bit(0, 0));
                    self.current_position.bb_br = set_bit(self.current_position.bb_br, 3);
                }
                // update castling rights
                match piece_color {
                    PieceColor::Black => {if old_index != 4 {
                        self.current_position.black_kingside_castle = false;
                        self.current_position.black_queenside_castle = false;
                    }},
                    PieceColor::White => {if old_index != 60 {
                        self.current_position.white_kingside_castle = false;
                        self.current_position.white_queenside_castle = false;
                    }},
                    _ => {}
                }

            }
            // detect moving of rook to update castling rights
            PieceType::WhiteRook | PieceType::BlackRook => {
                // black queenside castling
                if old_index == 0 || index == 0 {
                    self.current_position.black_queenside_castle = false;
                }
                // black kingside castling
                else if old_index == 7 || index == 7 {
                    self.current_position.black_kingside_castle = false;
                }
                // white kingside castling
                else if old_index == 63 || index == 63 {
                    self.current_position.white_kingside_castle = false;
                }
                // white queenside castling
                else if old_index == 56 || index == 56 {
                    self.current_position.white_queenside_castle = false;
                }
                
            }
            _ => {self.current_position.es_target = None;}
        };
        
        // make the move on the bitboards
        bb_moving_piece = subtract_bb(bb_moving_piece, set_bit(0, old_index));
        bb_moving_piece = set_bit(bb_moving_piece, index);
        bb_captured_piece = subtract_bb(bb_captured_piece, set_bit(0, index));
        
        // place the new bitboards on the place of the old ones
        self.current_position.set_bb_of_piece_type(bb_moving_piece, &moving_piece_type);
        self.current_position.set_bb_of_piece_type(bb_captured_piece, &captured_piece_type);

        // update meta data
        self.current_position.to_move = match self.current_position.to_move {
            ToMove::White => ToMove::Black,
            _ => ToMove::White
        };
        self.legal_moves_cache = [None; 64];
        self.enemy_heat_cache = None;

        
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
        pseudo_moves: LoadMoves::new(),
        legal_moves_cache: [None; 64],
        enemy_heat_cache: None }
    }
    #[new]
    pub fn new() -> Chessboard {
        Chessboard { current_position: Position::new(),
        selected: Selected::None,
        pseudo_moves: LoadMoves::new(),
        legal_moves_cache: [None; 64],
        enemy_heat_cache: None }
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
        let legal_moves = self.get_legal_moves(index, &piece_color, false);
        // get enemy pieces
        let enemy_pieces = match piece_color {
            PieceColor::White => self.current_position.black_pieces(),
            PieceColor::Black => self.current_position.white_pieces(),
            _ => 0
        };
        // add en-passant possibly
        let legal_captures = match self.current_position.es_target {
            Some(target) => legal_moves & set_bit(enemy_pieces, target),
            None => legal_moves & enemy_pieces
        };
        bb_to_vec(legal_captures)
    }

    pub fn get_legal_non_captures(&mut self, index: u8) -> Vec<u8> {
        // index must be the index of the piece of which we want to get legal captures
        let piece_color = self.current_position.detect_piece_color(index);
        let legal_moves = self.get_legal_moves(index, &piece_color, false);
        // get enemy pieces
        let enemy_pieces = match piece_color {
            PieceColor::White => self.current_position.black_pieces(),
            PieceColor::Black => self.current_position.white_pieces(),
            _ => 0
        };
        let legal_non_captures = match self.current_position.es_target {
            Some(target) => subtract_bb(legal_moves, set_bit(enemy_pieces, target)),
            None => subtract_bb(legal_moves, enemy_pieces)
        };
        bb_to_vec(legal_non_captures)
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
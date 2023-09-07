use crate::{bitboard_helper::*, lookuptables::LoadMoves};
use std::cmp;

const BOARD_EDGE_UP: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;
const BOARD_EDGE_DOWN: u64 = 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BOARD_EDGE_RIGHT: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
const BOARD_EDGE_LEFT: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;

#[derive(Debug, Clone, Copy)]
pub struct MoveCalculator {
    occupied: u64,
    result: u64,
    piece_index: u8
}
impl MoveCalculator {
    pub fn new(occupied: u64, piece_index: u8) -> MoveCalculator{
        MoveCalculator{ occupied, result: 0, piece_index}
    }

    pub fn remove_redundant_board_edges(&mut self) -> u64 {
        let (rank, file) = index2rank_file(self.piece_index).unwrap();
        // remove the edges
        if rank > 1 {
            self.result = subtract_bb(self.result, BOARD_EDGE_DOWN);
        }
        if rank < 8 {
            self.result = subtract_bb(self.result, BOARD_EDGE_UP);
        }
        if file > 1 {
            self.result = subtract_bb(self.result, BOARD_EDGE_LEFT);
        }
        if file < 8 {
            self.result = subtract_bb(self.result, BOARD_EDGE_RIGHT);
        }
        self.result
    }

    fn iter_direction(&mut self, iter: impl Iterator<Item=(u8, u8)>) {
        for (x, y) in iter {
            let i = rank_file2index(x, y).unwrap();
            self.result = set_bit(self.result, i);
            if (self.occupied >> i) & 1 == 1 {
                break
            }
        }
    }
    pub fn calculate_rook_moves(&mut self) -> Result<u64, BitBoardError> {
        let (rank, file) = index2rank_file(self.piece_index)?;

        let up = ((rank+1)..9).map(|n| (n, file));
        let below = (1..(rank)).map(|n| (rank-n, file));
        let right = ((file+1)..9).map(|n| (rank, n));
        let left = (1..(file)).map(|n| (rank, file-n));
        self.iter_direction(up);
        self.iter_direction(below);
        self.iter_direction(right);
        self.iter_direction(left);
        Ok(self.result)
    }

    pub fn calculate_bishop_moves(&mut self) -> Result<u64, BitBoardError> {
        let (rank, file) = index2rank_file(self.piece_index)?;

        let below_right = (1..cmp::min(9-rank, 9-file)).map(|n| (rank+n,file+n));
        let below_left = (1..cmp::min(9-rank, file)).map(|n| (rank+n,file-n));
        let up_right = (1..cmp::min(rank, 9-file)).map(|n| (rank-n,file+n));
        let up_left = (1..cmp::min(rank, file)).map(|n| (rank-n, file-n));
        self.iter_direction(below_right);
        self.iter_direction(below_left);
        self.iter_direction(up_right);
        self.iter_direction(up_left);
        Ok(self.result)
    }

    pub fn calculate_knight_moves(&mut self) -> Result<u64, BitBoardError> {
        let (rank, file) = index2rank_file(self.piece_index)?;
        let rank = rank as i32;
        let file = file as i32;
        let possible_moves = [(rank + 2, file + 1), (rank + 2, file - 1), (rank - 2, file + 1),
                                               (rank - 2, file - 1), (rank - 1, file + 2), (rank + 1, file + 2),
                                               (rank + 1, file - 2), (rank - 1, file - 2)];
        let iter = (0..8).map(|n| possible_moves[n])
                                            .filter(|(a, b)| a>&0 && a<&9 && b>&0 && b<&9)
                                            .map(|(a, b)| rank_file2index(a as u8, b as u8).unwrap());
        for i in iter {
            self.result = set_bit(self.result, i);
        }
        Ok(self.result)
    }
    
    pub fn calculate_queen_moves(&mut self) -> Result<u64, BitBoardError> {
        self.calculate_rook_moves()?;
        self.calculate_bishop_moves()
    }
    pub fn calculate_king_moves(&mut self) -> Result<u64, BitBoardError> {
        let (rank, file) = index2rank_file(self.piece_index)?;
        let rank = rank as i32;
        let file = file as i32;
        let possible_moves = [(rank + 1, file), (rank + 1, file + 1), (rank + 1, file - 1), 
                                               (rank, file + 1), (rank, file - 1), (rank - 1, file), (rank - 1, file - 1),
                                               (rank - 1, file + 1)];
        // make an iterator over the neighbours of the king                                                   
        let iter = (0..8).map(|n| possible_moves[n])
                                                .filter(|(a, b)| a>&0 && a<&9 && b>&0 && b<&9)
                                                .map(|(a, b)| rank_file2index(a as u8, b as u8).unwrap());
        // set all possible bits to one
        for i in iter {
            self.result = set_bit(self.result, i);
        }
        Ok(self.result)
    }
    pub fn calculate_white_pawn_moves(&mut self) -> Result<u64, BitBoardError> {
        let (rank, file) = index2rank_file(self.piece_index)?;
        // move up one square
        if rank < 8 {
            self.result = set_bit(self.result, self.piece_index - 8);
            //capture diagonally
            if file > 1 {
                self.result = set_bit(self.result, self.piece_index - 9);
            }
            if file < 8 {
                self.result = set_bit(self.result, self.piece_index - 7);
            }
        }
        // starting pawns can move up two squares
        if rank == 2 {
            self.result = set_bit(self.result, self.piece_index - 16);
        }
        Ok(self.result)
    }
    pub fn calculate_black_pawn_moves(&mut self) -> Result<u64, BitBoardError> {
        let (rank, file) = index2rank_file(self.piece_index)?;
        // move up one square
        if rank > 1 {
            self.result = set_bit(self.result, self.piece_index + 8);
            //capture diagonally
            if file > 1 {
                self.result = set_bit(self.result, self.piece_index + 7);
            }
            if file < 8 {
                self.result = set_bit(self.result, self.piece_index + 9);
            }
        }
        // starting pawns can move up two squares
        if rank == 7 {
            self.result = set_bit(self.result, self.piece_index + 16);
        }
        Ok(self.result)
    }
}

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

pub const WHITE_KINGSIDE_CASTLE_MAP: u64 = 0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const WHITE_QUEENSIDE_CASTLE_MAP: u64 = 0b00001100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const BLACK_KINGSIDE_CASTLE_MAP: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000;
pub const BLACK_QUEENSIDE_CASTLE_MAP: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001100;
pub const WHITE_KINGSIDE_CASTLE_GOAL: u64 = 0b01000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const WHITE_QUEENSIDE_CASTLE_GOAL: u64 = 0b00000100_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
pub const BLACK_KINGSIDE_CASTLE_GOAL: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000000;
pub const BLACK_QUEENSIDE_CASTLE_GOAL: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000100;

const FILE_H_BB: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
const FILE_G_BB: u64 = 0b01000000_01000000_01000000_01000000_01000000_01000000_01000000_01000000;
const FILE_F_BB: u64 = 0b00100000_00100000_00100000_00100000_00100000_00100000_00100000_00100000; 
const FILE_E_BB: u64 = 0b00010000_00010000_00010000_00010000_00010000_00010000_00010000_00010000;
const FILE_D_BB: u64 = 0b00001000_00001000_00001000_00001000_00001000_00001000_00001000_00001000;
const FILE_C_BB: u64 = 0b00000100_00000100_00000100_00000100_00000100_00000100_00000100_00000100;
const FILE_B_BB: u64 = 0b00000010_00000010_00000010_00000010_00000010_00000010_00000010_00000010;
const FILE_A_BB: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;

// array that converts piece index to file
pub const INDEX2FILE: [u64; 64] = [FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB,
                               FILE_A_BB, FILE_B_BB, FILE_C_BB, FILE_D_BB, FILE_E_BB, FILE_F_BB, FILE_G_BB, FILE_H_BB];

#[derive(Clone, Debug, Copy)]
pub enum ToMove {
    White,
    Black
}

#[derive(Debug, Clone, Copy)]
pub enum PieceColor {
    White,
    Black,
    None
}
#[derive(Debug, Clone, Copy)]
pub struct NoLegalMoveInputError;

#[derive(Debug, Clone, Copy)]
pub enum PiecePromotes {
    Rook,
    Knight,
    Bishop,
    Queen,
}

impl PiecePromotes {
    pub fn to_piece_type(&self) -> PieceType {
        match self {
            PiecePromotes::Bishop => PieceType::Bishop,
            PiecePromotes::Knight => PieceType::Knight,
            PiecePromotes::Queen => PieceType::Queen,
            PiecePromotes::Rook => PieceType::Rook
        }
    }
}

pub fn board_notation2index(square: &str) -> Option<u8> {
    // sets the board notation like e7, b8 to the corresponding index on the board.
    let mut index = 0;
    if square.contains('a') {
        index += 0;
    }
    else if square.contains('b') {
        index += 1;
    }
    else if square.contains('c') {
        index += 2;
    }
    else if square.contains('d') {
        index += 3;
    }
    else if square.contains('e') {
        index += 4;
    }
    else if square.contains('f') {
        index += 5;
    }
    else if square.contains('g') {
        index += 6;
    }
    else if square.contains('h') {
        index += 7;
    }
    else {return None}

    let num = square.chars().nth(1)?.to_digit(10)?;
    index += (8-num)*8;

    Some(index as u8)
}

pub fn index2board_notation(index: u8) -> String {
    let (rank, file) = index2rank_file(index).unwrap();
    let mut notation = String::new();
    if file == 1 {
        notation.push('a')
    }
    else if file == 2 {
        notation.push('b')
    }
    else if file == 3 {
        notation.push('c')
    }
    else if file == 4 {
        notation.push('d')
    }
    else if file == 5 {
        notation.push('e')
    }
    else if file == 6 {
        notation.push('f')
    }
    else if file == 7 {
        notation.push('g')
    }
    else if file == 8 {
        notation.push('h')
    }
    notation.push(char::from_digit(rank as u32, 10).unwrap());
    notation
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub on_promotion: Option<PiecePromotes>
}

impl Move {
    pub fn to_string(&self) -> String {
        let mut res = index2board_notation(self.from);
        res.push_str(index2board_notation(self.to).as_str());
        match self.on_promotion {
            Some(promote) => match promote {
                PiecePromotes::Queen => res.push('q'),
                PiecePromotes::Bishop => res.push('b'),
                PiecePromotes::Rook => res.push('r'),
                PiecePromotes::Knight => res.push('n')
            }
            None => {}
        }
        res
    }
}


#[derive(Debug, Clone, Copy)]
pub enum Selected {
    None,
    White(u8),
    Black(u8)
}

#[derive(Debug, Clone, Copy)]
pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
    EmptySquare
}

impl PieceType {
    pub fn to_char(&self) -> char {
        match *self {
            PieceType::Pawn => 'p',
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            PieceType::King => 'k',
            PieceType::Queen => 'q',
            PieceType::EmptySquare => ' '
        }
    }
    pub fn from_char(character: char) -> PieceType {
        if character == 'p' {
            PieceType::Pawn
        }
        else if character == 'r' {
            PieceType::Rook
        }
        else if character == 'b' {
            PieceType::Bishop
        }
        else if character == 'n' {
            PieceType::Knight
        }
        else if character == 'q' {
            PieceType::Queen
        }
        else if character == 'k' {
            PieceType::King
        }
        else {PieceType::EmptySquare}
    }
}
#[derive(Clone, Debug, Copy)]
pub struct Pieces {
    bb_pawns: u64,
    bb_rooks: u64,
    bb_knights: u64,
    bb_bishops: u64,
    bb_queens: u64,
    bb_king: u64,
    king_index: Option<usize>,
    all: Option<u64>, // all pieces in one bitboard, the option allows us to cache results,
    color: PieceColor
}

impl Pieces {
    pub fn new(piece_color: PieceColor) -> Pieces {
        Pieces { 
            bb_pawns: 0, 
            bb_rooks: 0, 
            bb_knights: 0, 
            bb_bishops: 0, 
            bb_queens: 0, 
            bb_king: 0,
            king_index: None,
            all: None,
            color: piece_color}
    }
    pub fn new_white() -> Pieces {
        let mut pieces = Pieces { 
            bb_pawns: WHITE_PAWN_STARTING_BB, 
            bb_rooks: WHITE_ROOK_STARTING_BB, 
            bb_knights: WHITE_KNIGHT_STARTING_BB, 
            bb_bishops: WHITE_BISHOP_STARTING_BB, 
            bb_queens: WHITE_QUEEN_STARTING_BB, 
            bb_king: WHITE_KING_STARTING_BB,
            king_index: Some(60),
            all: None,
            color: PieceColor::White};
        pieces.get_all();
        pieces
    }
    pub fn new_black() -> Pieces {
        let mut pieces = Pieces { 
            bb_pawns: BLACK_PAWN_STARTING_BB, 
            bb_rooks: BLACK_ROOK_STARTING_BB, 
            bb_knights: BLACK_KNIGHT_STARTING_BB, 
            bb_bishops: BLACK_BISHOP_STARTING_BB, 
            bb_queens: BLACK_QUEEN_STARTING_BB, 
            bb_king: BLACK_KING_STARTING_BB,
            king_index: Some(4),
            all: None,
        color: PieceColor::Black};
        pieces.get_all();
        pieces
    }
    pub fn get_all(&mut self) -> u64 {
        // returns all the pieces in one bitboard
        match self.all {
            Some(all) => all,
            None => {
                let all = self.bb_pawns|self.bb_rooks|self.bb_knights|
                self.bb_bishops|self.bb_queens|self.bb_king;
                // store value for later use
                self.all = Some(all);
                all
            }
        }
    }
    // #[inline]
    pub fn get_color(&self) -> &PieceColor {
        &self.color
    }
    // get all the pieces bitboards
    // #[inline]
    pub fn get_bb_pawns(&self) -> u64 {
        self.bb_pawns
    }
    // #[inline]
    pub fn get_bb_rooks(&self) -> u64 {
        self.bb_rooks
    }
    // #[inline]
    pub fn get_bb_knights(&self) -> u64 {
        self.bb_knights
    }
    // #[inline]
    pub fn get_bb_bishops(&self) -> u64 {
        self.bb_bishops
    }
    // #[inline]
    pub fn get_bb_queens(&self) -> u64 {
        self.bb_queens
    }
    // #[inline]
    pub fn get_bb_king(&self) -> u64 {
        self.bb_king
    }
    // #[inline]
    pub fn get_king_index(&mut self) -> usize {
        match self.king_index {
            Some(i) => i,
            None => {
                let i = (self.bb_king as f64).log2() as usize;
                self.king_index = Some(i);
                i
            }
        }
    }
    // set all the pieces bitboards
    // #[inline]
    pub fn set_bb_pawns(&mut self, new_bb: u64){
        self.bb_pawns = new_bb;
        self.all = None;
    }
    // #[inline]
    pub fn set_bb_rooks(&mut self, new_bb: u64){
        self.bb_rooks = new_bb;
        self.all = None;
    }
    // #[inline]
    pub fn set_bb_knights(&mut self, new_bb: u64){
        self.bb_knights = new_bb;
        self.all = None;
    }
    // #[inline]
    pub fn set_bb_bishops(&mut self, new_bb: u64){
        self.bb_bishops = new_bb;
        self.all = None;
    }
    // #[inline]
    pub fn set_bb_queens(&mut self, new_bb: u64){
        self.bb_queens = new_bb;
        self.all = None;
    }
    // #[inline]
    pub fn set_bb_king(&mut self, new_bb: u64){
        self.bb_king = new_bb;
        self.all = None;
        self.king_index = None;
    }
    pub fn detect_piece_type(&self, piece_index: u8) -> PieceType {
        // Detects the piece type of the current piece location. Piece index must
        // be an integer between 0 and 63, not a bitboard with one bit!
        if (self.bb_pawns >> piece_index) & 1 == 1 {
            PieceType::Pawn
        }
        else if (self.bb_rooks >> piece_index) & 1 == 1 {
            PieceType::Rook
        }
        else if (self.bb_knights >> piece_index) & 1 == 1 {
            PieceType::Knight
        }
        else if (self.bb_bishops >> piece_index) & 1 == 1 {
            PieceType::Bishop
        }
        else if (self.bb_king >> piece_index) & 1 == 1 {
            PieceType::King
        }
        else if (self.bb_queens >> piece_index) & 1 == 1 {
            PieceType::Queen
        }
        else {
            PieceType::EmptySquare
        }
    }
    // #[inline]
    pub fn piece_type2bb(&self, piece_type: &PieceType) -> u64 {
        // note that the returned bitboard is a copy of the stored bitboard,
        // so it is not a mutable reference!
        match piece_type {
            PieceType::Pawn => self.bb_pawns,
            PieceType::Rook => self.bb_rooks,
            PieceType::Bishop => self.bb_bishops,
            PieceType::Knight => self.bb_knights,
            PieceType::King => self.bb_king,
            PieceType::Queen => self.bb_queens,
            PieceType::EmptySquare => 0
        }
    }

    // #[inline]
    pub fn set_bb_of_piece_type(&mut self,bb: u64, piece_type: &PieceType) {
        match piece_type {
            PieceType::Pawn => self.set_bb_pawns(bb),
            PieceType::Rook => self.set_bb_rooks(bb),
            PieceType::Bishop => self.set_bb_bishops(bb),
            PieceType::Knight => self.set_bb_knights(bb),
            PieceType::King => self.set_bb_king(bb),
            PieceType::Queen => self.set_bb_queens(bb),
            PieceType::EmptySquare => {}
        };
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Position {
    pub white_pieces: Pieces,
    pub black_pieces: Pieces,
    pub es_target: Option<u8>, // en passant target square can be either None or some index of the target
    pub white_kingside_castle: bool,
    pub black_kingside_castle: bool,
    pub white_queenside_castle: bool,
    pub black_queenside_castle: bool,
    pub to_move: ToMove,
    pub halfmove_clock: u8,
    pub fullmove_clock: u8
}

impl Position {
    pub fn new() -> Position {
        Position {
            white_pieces: Pieces::new(PieceColor::White),
            black_pieces: Pieces::new(PieceColor::Black),
            es_target: None,
            white_kingside_castle: true,
            black_kingside_castle: true,
            white_queenside_castle: true,
            black_queenside_castle: true,
            to_move: ToMove::White,
            halfmove_clock: 0,
            fullmove_clock: 1
        }
    }
    pub fn new_start() -> Position {
        Position {
            white_pieces: Pieces::new_white(),
            black_pieces: Pieces::new_black(),
            es_target: None,
            white_kingside_castle: true,
            black_kingside_castle: true,
            white_queenside_castle: true,
            black_queenside_castle: true,
            to_move: ToMove::White,
            halfmove_clock: 0,
            fullmove_clock: 1
        }
    }
    // #[inline]
    pub fn get_all(&mut self) -> u64 {
        // returns a bitboard with all pieces, black and white
        self.black_pieces.get_all() | self.white_pieces.get_all()
    }
    // #[inline]
    pub fn piece_type_color2bb(&self, piece_type: &PieceType, piece_color: &PieceColor) -> u64 {
        // takes in the piecetype and piececolor and returns the bitboard
        match piece_color {
            PieceColor::White => self.white_pieces.piece_type2bb(piece_type),
            PieceColor::Black => self.black_pieces.piece_type2bb(piece_type),
            _ => 0
        }
    }
    // #[inline]
    pub fn set_bb_of_piece_type_color(&mut self, bb: u64, piece_type: &PieceType, piece_color: &PieceColor) {
        // takes in the piecetype and piececolor and sets the bb to that values
        match piece_color {
            PieceColor::White => self.white_pieces.set_bb_of_piece_type(bb, piece_type),
            PieceColor::Black => self.black_pieces.set_bb_of_piece_type(bb, piece_type),
            _ => {}
        }
    }
    // #[inline]
    fn index2char(&self, index: u8) -> char {
        match self.black_pieces.detect_piece_type(index) {
            PieceType::EmptySquare => match self.white_pieces.detect_piece_type(index) {
                PieceType::EmptySquare => ' ',
                other => other.to_char().to_ascii_uppercase()
            },
            other => other.to_char()
        }
    }
    // #[inline]
    pub fn to_string(&self) -> String {
        (0..64).map(|i| self.index2char(i).to_string()).collect::<Vec<String>>().join("")
    }
    pub fn detect_piece_color(&mut self, piece_index: u8) -> PieceColor {
        if (self.white_pieces.get_all() >> piece_index) & 1 == 1 {
            PieceColor::White
        }
        else if (self.black_pieces.get_all() >> piece_index) & 1 == 1 {
            PieceColor::Black
        }
        else {PieceColor::None}

    }
}
use crate::bitboard_helper::*;
use std::cmp;

const BOARD_EDGE_UP: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_11111111;
const BOARD_EDGE_DOWN: u64 = 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BOARD_EDGE_RIGHT: u64 = 0b10000000_10000000_10000000_10000000_10000000_10000000_10000000_10000000;
const BOARD_EDGE_LEFT: u64 = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001;

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
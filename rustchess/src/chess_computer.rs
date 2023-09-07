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

const PAWN_SCORE_MASK: [i32; 64] = [
0,  0,  0,  0,  0,  0,  0,  0,
50, 50, 50, 50, 50, 50, 50, 50,
10, 10, 20, 30, 30, 20, 10, 10,
 5,  5, 10, 25, 25, 10,  5,  5,
 0,  0,  0, 20, 20,  0,  0,  0,
 5, -5,-10,  0,  0,-10, -5,  5,
 5, 10, 10,-20,-20, 10, 10,  5,
 0,  0,  0,  0,  0,  0,  0,  0];

const KNIGHT_SCORE_MASK: [i32; 64] = [
-50,-40,-30,-30,-30,-30,-40,-50,
-40,-20,  0,  0,  0,  0,-20,-40,
-30,  0, 10, 15, 15, 10,  0,-30,
-30,  5, 15, 20, 20, 15,  5,-30,
-30,  0, 15, 20, 20, 15,  0,-30,
-30,  5, 10, 15, 15, 10,  5,-30,
-40,-20,  0,  5,  5,  0,-20,-40,
-50,-40,-30,-30,-30,-30,-40,-50];

const BISHOP_SCORE_MASK: [i32; 64] = [
-20,-10,-10,-10,-10,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5, 10, 10,  5,  0,-10,
-10,  5,  5, 10, 10,  5,  5,-10,
-10,  0, 10, 10, 10, 10,  0,-10,
-10, 10, 10, 10, 10, 10, 10,-10,
-10,  5,  0,  0,  0,  0,  5,-10,
-20,-10,-10,-10,-10,-10,-10,-20];

const ROOK_SCORE_MASK: [i32; 64] = [  
0,  0,  0,  0,  0,  0,  0,  0,
5, 10, 10, 10, 10, 10, 10,  5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
0,  0,  0,  5,  5,  0,  0,  0];

const QUEEN_SCORE_MASK: [i32; 64] = [
-20,-10,-10, -5, -5,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5,  5,  5,  5,  0,-10,
 -5,  0,  5,  5,  5,  5,  0, -5,
  0,  0,  5,  5,  5,  5,  0, -5,
-10,  5,  5,  5,  5,  5,  0,-10,
-10,  0,  5,  0,  0,  0,  0,-10,
-20,-10,-10, -5, -5,-10,-10,-20];

const KING_SCORE_MIDDLE_GAME: [i32; 64] = [
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-30,-40,-40,-50,-50,-40,-40,-30,
-20,-30,-30,-40,-40,-30,-30,-20,
-10,-20,-20,-20,-20,-20,-20,-10,
 20, 20,  0,  0,  0,  0, 20, 20,
 20, 30, 10,  0,  0, 10, 30, 20];

 const KING_SCORE_END_GAME: [i32; 64] = [
-50,-40,-30,-20,-20,-30,-40,-50,
 -30,-20,-10,  0,  0,-10,-20,-30,
 -30,-10, 20, 30, 30, 20,-10,-30,
 -30,-10, 30, 40, 40, 30,-10,-30,
 -30,-10, 30, 40, 40, 30,-10,-30,
 -30,-10, 20, 30, 30, 20,-10,-30,
 -30,-30,  0,  0,  0,  0,-30,-30,
 -50,-30,-30,-30,-30,-30,-30,-50];

pub struct BasicTreeSearchComputer {
    best_move: Option<Move>,
    depth: u8
}

impl RecieveAndReturnMove for BasicTreeSearchComputer {
    fn return_move(&mut self, chessboard: &mut Chessboard) -> Move {
        let depth = 4;
        self.depth = depth;
        let eval = match chessboard.get_to_move() {
            ToMove::White => self.minimax(chessboard, depth, -100000, 100000, true),
            ToMove::Black => self.minimax(chessboard, depth, -100000, 100000, false)
        };
        println!("current evaluation: {}", eval);
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
        // get all bitboards as vecs
        let black_pieces = &position.black_pieces;
        let white_pieces = &position.white_pieces;
        let bb = bb_to_vec(black_pieces.get_bb_bishops());
        let bn = bb_to_vec(black_pieces.get_bb_knights());
        let br = bb_to_vec(black_pieces.get_bb_rooks());
        let bq = bb_to_vec(black_pieces.get_bb_queens());
        let bp = bb_to_vec(black_pieces.get_bb_pawns());
        let bk = bb_to_vec(black_pieces.get_bb_king());
        let wb = bb_to_vec(white_pieces.get_bb_bishops());
        let wn = bb_to_vec(white_pieces.get_bb_knights());
        let wr = bb_to_vec(white_pieces.get_bb_rooks());
        let wq = bb_to_vec(white_pieces.get_bb_queens());
        let wp = bb_to_vec(white_pieces.get_bb_pawns());
        let wk = bb_to_vec(white_pieces.get_bb_king());

        // subtract value of black pieces
        eval -= bb.len() as i32 * 330;
        eval -= bn.len() as i32 * 320;
        eval -= br.len() as i32 * 500;
        eval -= bq.len() as i32 * 900;
        eval -= bp.len() as i32 * 100;
        // subtract the value of all white pieces on the board
        eval += wb.len() as i32 * 330;
        eval += wn.len() as i32 * 320;
        eval += wr.len() as i32 * 500;
        eval += wq.len() as i32 * 900;
        eval += wp.len() as i32 * 100;

        let mut total_board_score = 0;
        total_board_score += bb.len() as i32 * 330;
        total_board_score += bn.len() as i32 * 320;
        total_board_score += br.len() as i32 * 500;
        total_board_score += bq.len() as i32 * 900;
        total_board_score += bp.len() as i32 * 100;
        total_board_score += wb.len() as i32 * 330;
        total_board_score += wn.len() as i32 * 320;
        total_board_score += wr.len() as i32 * 500;
        total_board_score += wq.len() as i32 * 900;
        total_board_score += wp.len() as i32 * 100;
        // 8000 is the total board score at the starting position
        let game_progress = 1.0 - (total_board_score as f64 / 8000.0);

        eval += bb.iter().map(|i| BISHOP_SCORE_MASK[63 - *i as usize]).sum::<i32>();
        eval += bn.iter().map(|i| KNIGHT_SCORE_MASK[63 - *i as usize]).sum::<i32>();
        eval += br.iter().map(|i| ROOK_SCORE_MASK[63 - *i as usize]).sum::<i32>();
        eval += bq.iter().map(|i| QUEEN_SCORE_MASK[63 - *i as usize]).sum::<i32>();
        eval += bp.iter().map(|i| PAWN_SCORE_MASK[63 - *i as usize]).sum::<i32>();
        eval += bk.iter().map(|i| (((1.0-game_progress) * KING_SCORE_MIDDLE_GAME[63 - *i as usize] as f64)
         + (game_progress * KING_SCORE_END_GAME[63 - *i as usize] as f64)) as i32).sum::<i32>();

        eval += wb.iter().map(|i| BISHOP_SCORE_MASK[*i as usize]).sum::<i32>();
        eval += wn.iter().map(|i| KNIGHT_SCORE_MASK[*i as usize]).sum::<i32>();
        eval += wr.iter().map(|i| ROOK_SCORE_MASK[*i as usize]).sum::<i32>();
        eval += wq.iter().map(|i| QUEEN_SCORE_MASK[*i as usize]).sum::<i32>();
        eval += wp.iter().map(|i| PAWN_SCORE_MASK[*i as usize]).sum::<i32>();
        eval += wk.iter().map(|i| (((1.0-game_progress) * KING_SCORE_MIDDLE_GAME[*i as usize] as f64)
         + (game_progress * KING_SCORE_END_GAME[*i as usize] as f64)) as i32).sum::<i32>();
        eval
    }

    pub fn minimax(&mut self, chessboard: &mut Chessboard, depth: u8, mut alpha: i32, mut beta: i32, maximizing_player: bool) -> i32 {
        // depth is how far ahead we want to search, maximizing_player deals with either white to move or black
        if depth == 0 {
            return BasicTreeSearchComputer::static_evaluate(chessboard.get_position())
        }

        if maximizing_player {
            let mut max_eval = -100000;
            for new_move in chessboard.all_moves().iter() {
                chessboard.move_piece(new_move).unwrap();
                let eval = self.minimax(chessboard, depth - 1, alpha, beta, false);
                max_eval = cmp::max(max_eval, eval);
                alpha = cmp::max(alpha, eval);
                if beta <= alpha {
                    chessboard.undo();
                    break
                }
                if depth == self.depth && max_eval == eval {
                    self.best_move = Some(*new_move);
                }
                chessboard.undo();
            }
            return max_eval
        }
        else {
            let mut min_eval = 100000;
            for new_move in chessboard.all_moves().iter() {
                chessboard.move_piece(new_move).unwrap();
                let eval = self.minimax(chessboard, depth - 1, alpha, beta, true);
                min_eval = cmp::min(min_eval, eval);
                beta = cmp::min(beta, eval);
                if beta <= alpha {
                    chessboard.undo();
                    break
                }
                if depth == self.depth && min_eval == eval {
                    self.best_move = Some(*new_move);
                }
                chessboard.undo();
            }
            return min_eval
        }
    }
}
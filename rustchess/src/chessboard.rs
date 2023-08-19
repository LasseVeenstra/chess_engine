use pyo3::prelude::*;
use crate::bitboard_helper::*;
use crate::lookuptables::LoadMoves;
use crate::chessboard_helper::*;

#[pyclass]
pub struct Chessboard {
    pos: Position,
    selected: Selected,
    pseudo_moves: LoadMoves,
    // every time we make a move this will need to be cleared
    legal_moves_cache: [Option<u64>; 64],
    enemy_heat_cache: Option<u64>,
    pinned_pieces_cache: Option<u64>
}

impl Chessboard {
    fn remove_piece_run_add_piece<F, T>(&mut self, pieces: u64, piece_type: &PieceType, piece_color: &PieceColor, f: F) -> T
    // pieces is a bitboard of the pieces we want to temporary remove from the board
    where
        F: FnOnce() -> T
        {
            // first we remove the pieces from the board
            let board_pieces = self.pos.piece_type_color2bb(piece_type, piece_color);
            self.pos.set_bb_of_piece_type_color(subtract_bb(board_pieces, pieces), piece_type, piece_color);
            // now run the closure that has been passed
            let res = f();
            // now add the pieces back onto the board
            self.pos.set_bb_of_piece_type_color(board_pieces, piece_type, piece_color);

            res
        }

    fn get_heatmap(&mut self, color: &PieceColor) -> u64 {
        // returns all squares that the color can move to in a bitboard,
        // check if we have already calculated the enemy heat before
        match self.enemy_heat_cache {
            Some(heat) => return heat,
            None => {
                // calculate the heatmap
                let mut heat = 0;
                let all_pieces = match color {
                    PieceColor::White => self.pos.white_pieces.get_all(),
                    PieceColor::Black => self.pos.black_pieces.get_all(),
                    _ => 0
                };
                // loop over all indices of black squares
                for index in bb_to_vec(all_pieces) {
                    heat |= self.get_legal_moves(index, true);
                }

                // store the heat in the cache
                self.enemy_heat_cache = Some(heat);
                heat
            }
        }
    }

    fn get_checking_pieces(&mut self, king_index: usize, color: &PieceColor) -> u64 {
        // returns a bitboard with all pieces that are currently checking the king
        let blockers = self.pos.get_all();
        let mut attackers: u64 = 0;
        let enemy = match color {
            PieceColor::White => &self.pos.black_pieces,
            PieceColor::Black => &self.pos.white_pieces,
            _ => return 0
        };
        // add possible knight checks
        attackers |= self.pseudo_moves.knight(king_index) & enemy.get_bb_knights();
        // add possible rook, bishop or queen checks
        attackers |= self.pseudo_moves.queen(king_index, blockers).unwrap() & enemy.get_bb_queens();
        attackers |= self.pseudo_moves.rook(king_index, blockers).unwrap() & enemy.get_bb_rooks();
        attackers |= self.pseudo_moves.bishop(king_index, blockers).unwrap() & enemy.get_bb_bishops();
        // add possible pawn checks
        attackers |= subtract_bb(self.pseudo_moves.white_pawn(king_index), INDEX2FILE[king_index]) & enemy.get_bb_pawns();
        attackers
    }

    fn pieces(&mut self, color: &PieceColor) -> &mut Pieces {
        match color {
            PieceColor::White => &mut self.pos.white_pieces,
            PieceColor::Black => &mut self.pos.black_pieces,
            _ => {panic!("Cannot get pieces of no color!")}
        }
    }

    fn get_pseudo_legal_moves(&mut self, index: usize, piece_type: &PieceType, friendly_color: &PieceColor, enemy_color: &PieceColor, heat_only: bool) -> u64 {
        // returns pseudo legal moves, that is, legal moves ignoring checks and pins
        // some information we use later
        let enemy_king_bb = self.pieces(&enemy_color).get_bb_king();
        let blockers = self.pos.get_all();

        match piece_type {
            PieceType::Pawn => {
                let piece_file = INDEX2FILE[index as usize];
                let pawn_moves = match self.pieces(&friendly_color).get_color() {
                    PieceColor::White => self.pseudo_moves.white_pawn(index as usize),
                    PieceColor::Black => self.pseudo_moves.black_pawn(index as usize),
                    _ => 0
                };
                // moves that go directly forwards
                let front = if !heat_only {
                    subtract_bb(pawn_moves & piece_file, blockers) & *self.pseudo_moves.rook(index as usize, blockers).expect("Couldn't get rook moves")  
                } else {0};
                // captures and pieces that block our way
                let captures = match self.pos.es_target {
                    //_ => subtract_bb(enemy_pieces & pawn_moves, piece_file),
                    Some(target) => subtract_bb(set_bit(self.pieces(&enemy_color).get_all(), target) & pawn_moves, piece_file),
                    None => {
                        if heat_only {
                            subtract_bb(pawn_moves, piece_file)
                        } else {subtract_bb(self.pieces(&enemy_color).get_all() & pawn_moves, piece_file)}
                    }
                };
                front | captures
            }
            PieceType::Knight => self.pseudo_moves.knight(index as usize),
            PieceType::Bishop => {
                if heat_only {
                    *self.pseudo_moves.bishop(index, subtract_bb(blockers, enemy_king_bb)).expect("Couldn't get bishop moves")
                } else {*self.pseudo_moves.bishop(index, blockers).expect("Couldn't get bishop moves")}
            },
            PieceType::Rook => {
                if heat_only {
                    *self.pseudo_moves.rook(index, subtract_bb(blockers, enemy_king_bb)).expect("Couldn't get rook moves")    
                } else {*self.pseudo_moves.rook(index, blockers).expect("Couldn't get rook moves")}
            },
            PieceType::Queen => {
                if heat_only {
                    self.pseudo_moves.queen(index, subtract_bb(blockers, enemy_king_bb)).expect("Couldn't get queen moves")    
                } else {self.pseudo_moves.queen(index, blockers).expect("Couldn't get queen moves")}
            },
            PieceType::King => {
                if !heat_only {
                    {subtract_bb(self.pseudo_moves.king(index), self.get_heatmap(&enemy_color)
                     | self.get_defended(&friendly_color))
                      | self.get_castling_squares(index, &friendly_color)}
                }
                else {
                    self.pseudo_moves.king(index)
                }
            },
            _ => 0
        }
    }

    fn add_check_moves(&mut self, legal_moves: u64, piece_type: &PieceType, enemy_color: &PieceColor, king_index: usize, pieces_giving_check: u64) -> u64 {
        // assumes that there is a single check present in the position. Then restricts the legal moves given to only moves legal due to the check
        
        let blockers = self.pos.get_all();
        match piece_type {
            PieceType::King => legal_moves,
            _ => {
                // split the legal moves in capture and non capture
                let mut legal_captures = legal_moves & self.pieces(&enemy_color).get_all();
                let mut legal_non_captures = subtract_bb(legal_moves, self.pieces(&enemy_color).get_all());
                // restrict captures to only captures of the pieces that are giving check
                legal_captures &= pieces_giving_check;

                // the ray to the king from the piece giving check
                let index_of_checking_piece = bb_to_vec(pieces_giving_check)[0];
                let piece_type_of_checking_piece = self.pieces(&enemy_color).detect_piece_type(index_of_checking_piece);
                match piece_type_of_checking_piece {
                    PieceType::Pawn => {
                        // we might still be able to capture the checking pawn with en passant
                        match self.pos.es_target {
                            Some(target) => {
                                // if the piece giving check is right below or above the en passant target square then it must be 
                                // that that piece just move two up and therefore we still allow en passant capture
                                if set_bit(set_bit(0, target - 8), target + 8) & pieces_giving_check != 0 {
                                    match piece_type {
                                        PieceType::Pawn => {legal_non_captures = set_bit(0, target);},
                                        _ => {}
                                    }
                                }
                            },
                            None => {legal_non_captures = 0;}
                        }
                    }
                    PieceType::Knight => {
                        legal_non_captures = 0;
                    }
                    _ => {
                        let ray2king = self.pseudo_moves.queen(king_index, blockers).unwrap() & 
                            self.pseudo_moves.queen(index_of_checking_piece as usize, blockers).unwrap();
                        // only allow moves that block the check
                        legal_non_captures &= ray2king;
                    }
                }
                legal_captures | legal_non_captures
            }
        }
    }

    fn get_legal_moves(&mut self, index: u8, heat_only: bool) -> u64 {
        // index is the index of the piece of which we want to find the legal moves.
        // This function already assumes that the correct player is making the move
        // and that he is not trying to move to his own piece, it also assumes that 
        // on old index is indeed a piece of the player that has to move.

        // Check if we have already calculated the moves before
        match self.legal_moves_cache[index as usize] {
            Some(legal_moves) => if !heat_only {return legal_moves},
            None => {}
        }
        let (friendly_color, enemy_color) = match self.pos.detect_piece_color(index) {
            PieceColor::White => (PieceColor::White, PieceColor::Black),
            PieceColor::Black => (PieceColor::Black, PieceColor::White),
            PieceColor::None => {return 0}
        };

        let piece_type = self.pieces(&friendly_color).detect_piece_type(index);
        let king_bb = self.pieces(&friendly_color).get_bb_king();
        let king_index = bb_to_vec(king_bb)[0] as usize;

        // if the piece we want to move is not the king and if we are in double check, only king moves
        // are allowed so we return no legal moves
        let pieces_giving_check = self.get_checking_pieces(king_index, &friendly_color);
        match piece_type {
            PieceType::King => {},
            _ => {
                if bb_to_vec(pieces_giving_check).len() > 1 {
                    return 0
                }
            }
        }
        // get blockers
        let blockers = self.pos.get_all();
        let mut legal_moves = self.get_pseudo_legal_moves(index as usize, &piece_type, &friendly_color, &enemy_color, heat_only);

        // remove the ability to capture own pieces
        legal_moves = subtract_bb(legal_moves, self.pieces(&friendly_color).get_all());

        // now we restrict the legal moves if we are in single check
        if bb_to_vec(pieces_giving_check).len() == 1 {
            legal_moves = self.add_check_moves(legal_moves, &piece_type, &enemy_color, king_index, pieces_giving_check);
        }
        // finally we take into account the possibilty that our piece is currently pinned to the king (an absolute pin)
        let pinned = self.get_pinned(&friendly_color, &enemy_color);
        if (pinned >> index) & 1 == 1 {
            legal_moves = match piece_type {
                // kings cannot be pinned
                PieceType::King => legal_moves,
                PieceType::Knight => 0,
                PieceType::Bishop => legal_moves & self.pseudo_moves.bishop(king_index, subtract_bb(blockers, set_bit(0, index))).unwrap(),
                PieceType::Rook => legal_moves & self.pseudo_moves.rook(king_index, subtract_bb(blockers, set_bit(0, index))).unwrap(),
                PieceType::Queen => legal_moves & self.pseudo_moves.queen(king_index, subtract_bb(blockers, set_bit(0, index))).unwrap(),
                PieceType::Pawn => legal_moves & self.pseudo_moves.queen(king_index, subtract_bb(blockers, set_bit(0, index))).unwrap(),
                _ => 0
            }
        }


        // store the result in the cache
        if !heat_only {self.legal_moves_cache[index as usize] = Some(legal_moves);}

        // return final moves
        legal_moves
    }

    fn get_pinned(&mut self, friendly_color: &PieceColor, enemy_color: &PieceColor) -> u64 {
        // returns a bitboard with bits on all pieces that are currently pinned to the king

        // check if we still have the pinned pieces stored
        match self.pinned_pieces_cache {
            Some(pinned) => return pinned,
            None => {}
        }

        let mut pinned = 0;

        let blockers = self.pos.get_all();
        let king_index = bb_to_vec(self.pieces(&friendly_color).get_bb_king())[0] as usize;

        // loop over all enemy sliding pieces
        let enemy = self.pieces(&enemy_color);
        for index in bb_to_vec(enemy.get_bb_rooks() | enemy.get_bb_queens()) {
            let enemy_moves = self.pseudo_moves.rook(index as usize, blockers).unwrap();
            // now for each direction we want to go from the king into the opposite direction and check the overlap
            for direction in [0, 2, 4, 6] {
                let enemy_ray = enemy_moves & self.pseudo_moves.direction_ray(index as usize, direction);
                let king_ray = self.pseudo_moves.rook(king_index, blockers).unwrap() & self.pseudo_moves.direction_ray(king_index, (direction + 4) % 8);
                pinned |= enemy_ray & king_ray;
            }
        }
        let enemy = self.pieces(&enemy_color);
        for index in bb_to_vec(enemy.get_bb_bishops() | enemy.get_bb_queens()) {
            let enemy_moves = self.pseudo_moves.bishop(index as usize, blockers).unwrap();
            // now for each direction we want to go from the king into the opposite direction and check the overlap
            for direction in [1, 3, 5, 7] {
                let enemy_ray = enemy_moves & self.pseudo_moves.direction_ray(index as usize, direction);
                let king_ray = self.pseudo_moves.bishop(king_index, blockers).unwrap() & self.pseudo_moves.direction_ray(king_index, (direction + 4) % 8);
                pinned |= enemy_ray & king_ray;
            }
        }

        self.pinned_pieces_cache = Some(pinned);
        pinned

    }

    fn get_defended(&mut self, friendly_color: &PieceColor) -> u64 {
        // returns a bitboard with bits on all pieces that are defended by one of his own pieces
        let mut defended = 0;

        // loop over all indices of squares
        for index in bb_to_vec(self.pieces(&friendly_color).get_all()) {
            defended |= self.get_defended_by_piece(index, friendly_color);
        }
        defended
    }

    fn get_defended_by_piece(&mut self, index: u8, friendly_color: &PieceColor) -> u64 {
        // get all pieces that are defended by the piece on index

        // get the piece type
        let piece_type = self.pieces(friendly_color).detect_piece_type(index);
        let blockers = self.pos.get_all();

        let all_friendly = self.pieces(friendly_color).get_all();

        match piece_type {
            PieceType::Pawn => {
                let piece_file = INDEX2FILE[index as usize];
                let pawn_moves = match friendly_color {
                    PieceColor::White => self.pseudo_moves.white_pawn(index as usize),
                    PieceColor::Black => self.pseudo_moves.black_pawn(index as usize),
                    _ => 0
                };
                subtract_bb(pawn_moves, piece_file) & all_friendly
            }
            PieceType::Knight => self.pseudo_moves.knight(index as usize) & all_friendly,
            PieceType::Bishop => {
                *self.pseudo_moves.bishop(index as usize, blockers).expect("Couldn't get bishop moves") & all_friendly
            },
            PieceType::Rook => {
                *self.pseudo_moves.rook(index as usize, blockers).expect("Couldn't get rook moves") & all_friendly
            },
            PieceType::King => {
                    self.pseudo_moves.king(index as usize) & all_friendly
            },
            PieceType::Queen => {
                self.pseudo_moves.queen(index as usize, blockers).expect("Couldn't get queen moves") & all_friendly
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
        let all_pieces = self.pos.get_all();
        match king_color {
            PieceColor::White => {
                let heat = self.get_heatmap(&PieceColor::Black);
                if ((heat | all_pieces) & WHITE_KINGSIDE_CASTLE_MAP == 0) && self.pos.white_kingside_castle {
                    castle_squares |= WHITE_KINGSIDE_CASTLE_GOAL;
                }
                if ((heat | all_pieces) & WHITE_QUEENSIDE_CASTLE_MAP == 0) && self.pos.white_queenside_castle {
                    castle_squares |= WHITE_QUEENSIDE_CASTLE_GOAL;
                }
            }
            PieceColor::Black => {
                let heat = self.get_heatmap(&PieceColor::White);
                if ((heat | all_pieces) & BLACK_KINGSIDE_CASTLE_MAP == 0) && self.pos.black_kingside_castle {
                    castle_squares |= BLACK_KINGSIDE_CASTLE_GOAL;
                }
                if ((heat | all_pieces) & BLACK_QUEENSIDE_CASTLE_MAP == 0) && self.pos.black_queenside_castle {
                    castle_squares |= BLACK_QUEENSIDE_CASTLE_GOAL;
                }
            }
            _ => {}
        }
        castle_squares
    }

    fn check_move_for_legal(&mut self, old_index: u8, index: u8, piece_color: &PieceColor) -> bool {
        match piece_color {
            PieceColor::None => return false,
            _ => {}
        }
        // get all legal moves
        let legal_moves = self.get_legal_moves(old_index, false);
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

        let (friendly_color, enemy_color) = match self.pos.detect_piece_color(old_index) {
            PieceColor::White => (PieceColor::White, PieceColor::Black),
            PieceColor::Black => (PieceColor::Black, PieceColor::White),
            PieceColor::None => {return}
        };

        // get the piece type of the piece we want to move and the piece type
        // of the piece we want to capture (note that these are not by reference but a copy, so we must
        // replace them later.)
        let moving_piece_type = self.pieces(&friendly_color).detect_piece_type(old_index);
        let captured_piece_type = self.pieces(&enemy_color).detect_piece_type(index);
        let mut bb_moving_piece = self.pieces(&friendly_color).piece_type2bb(&moving_piece_type);
        let mut bb_captured_piece = self.pieces(&enemy_color).piece_type2bb(&captured_piece_type);
        match moving_piece_type {
            // detect en-passant for capture or new es-target
            PieceType::Pawn => {
                // detect en-passant capture
                match self.pos.es_target {
                    Some(target) => {                
                        if index == target {
                            let friendly_pawns = self.pieces(&friendly_color).get_bb_pawns();
                            self.pieces(&friendly_color).set_bb_pawns(subtract_bb(friendly_pawns, set_bit(0, index + 8)));
                        }
                    }
                    None => {}
                }
                // update en-passant target
                // if we move up two squares (or down).
                if (old_index as i16 - index as i16).abs() == 16 {
                    self.pos.es_target = Some((old_index + index) / 2);
                }
                else {self.pos.es_target = None;}
            }
            // detect castle move
            PieceType::King => {
                // detect if a castling move has been made

                // white kingside castling
                let mut friendly_rooks = self.pieces(&friendly_color).get_bb_rooks();
                if index == 62 {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 63));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 61));
                }
                // white queenside castling
                if index == 58 {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 56));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 59));
                }
                // black kingside castling
                if index == 6 {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 7));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 5));
                }
                // black queenside castling
                if index == 2 {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 0));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 3));
                }
                // update castling rights
                match piece_color {
                    PieceColor::Black => {if old_index != 4 {
                        self.pos.black_kingside_castle = false;
                        self.pos.black_queenside_castle = false;
                    }},
                    PieceColor::White => {if old_index != 60 {
                        self.pos.white_kingside_castle = false;
                        self.pos.white_queenside_castle = false;
                    }},
                    _ => {}
                }

            }
            // detect moving of rook to update castling rights
            PieceType::Rook => {
                // black queenside castling
                if old_index == 0 || index == 0 {
                    self.pos.black_queenside_castle = false;
                }
                // black kingside castling
                else if old_index == 7 || index == 7 {
                    self.pos.black_kingside_castle = false;
                }
                // white kingside castling
                else if old_index == 63 || index == 63 {
                    self.pos.white_kingside_castle = false;
                }
                // white queenside castling
                else if old_index == 56 || index == 56 {
                    self.pos.white_queenside_castle = false;
                }
                
            }
            _ => {self.pos.es_target = None;}
        };
        
        // make the move on the bitboards
        bb_moving_piece = subtract_bb(bb_moving_piece, set_bit(0, old_index));
        bb_moving_piece = set_bit(bb_moving_piece, index);
        bb_captured_piece = subtract_bb(bb_captured_piece, set_bit(0, index));
        
        // place the new bitboards on the place of the old ones
        self.pieces(&friendly_color).set_bb_of_piece_type(bb_moving_piece, &moving_piece_type);
        self.pieces(&enemy_color).set_bb_of_piece_type(bb_captured_piece, &captured_piece_type);

        // update meta data
        self.pos.to_move = match self.pos.to_move {
            ToMove::White => ToMove::Black,
            _ => ToMove::White
        };
        self.legal_moves_cache = [None; 64];
        self.enemy_heat_cache = None;
        self.pinned_pieces_cache = None;

        
    }

    fn select_new(&mut self, index: u8) {
        let w_pieces = self.pos.white_pieces.get_all();
        let b_pieces = self.pos.black_pieces.get_all();

        match self.pos.to_move {
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
        Chessboard { pos: Position::new_start(),
        selected: Selected::None,
        pseudo_moves: LoadMoves::new(),
        legal_moves_cache: [None; 64],
        enemy_heat_cache: None,
        pinned_pieces_cache: None }
    }
    #[new]
    pub fn new() -> Chessboard {
        Chessboard { pos: Position::new(),
        selected: Selected::None,
        pseudo_moves: LoadMoves::new(),
        legal_moves_cache: [None; 64],
        enemy_heat_cache: None,
        pinned_pieces_cache: None }
    }
    pub fn to_string(&self) -> String {
        self.pos.to_string()
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
        let piece_color = self.pos.detect_piece_color(index);
        // return early if we have selected an empty square
        match piece_color {
            PieceColor::None => return vec![],
            _ => {}
        }
        let legal_moves = self.get_legal_moves(index, false);
        // get enemy pieces
        let enemy_pieces = match piece_color {
            PieceColor::White => self.pos.black_pieces.get_all(),
            PieceColor::Black => self.pos.white_pieces.get_all(),
            _ => 0
        };
        // add en-passant possibly
        let legal_captures = match self.pos.es_target {
            Some(target) => legal_moves & set_bit(enemy_pieces, target),
            None => legal_moves & enemy_pieces
        };
        bb_to_vec(legal_captures)
    }

    pub fn get_legal_non_captures(&mut self, index: u8) -> Vec<u8> {
        // index must be the index of the piece of which we want to get legal captures
        let piece_color = self.pos.detect_piece_color(index);
        // return early if we have selected an empty square
        match piece_color {
            PieceColor::None => return vec![],
            _ => {}
        }
        let legal_moves = self.get_legal_moves(index, false);
        // get enemy pieces
        let enemy_pieces = match piece_color {
            PieceColor::White => self.pos.black_pieces.get_all(),
            PieceColor::Black => self.pos.white_pieces.get_all(),
            _ => 0
        };
        let legal_non_captures = match self.pos.es_target {
            Some(target) => subtract_bb(legal_moves, set_bit(enemy_pieces, target)),
            None => subtract_bb(legal_moves, enemy_pieces)
        };
        bb_to_vec(legal_non_captures)
    }

    pub fn input_select(&mut self, index: u8) {
        let w_pieces = self.pos.white_pieces.get_all();
        let b_pieces = self.pos.black_pieces.get_all();

        match self.selected {
            // in case we have not selected anything
            Selected::None => {
                self.select_new(index);
            }
            // in case we have already selected something
            Selected::White(old_index) => {
                match self.pos.to_move {
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
                match self.pos.to_move {
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
    pub fn loadFEN(&mut self, FEN: String) {
        
    }

}






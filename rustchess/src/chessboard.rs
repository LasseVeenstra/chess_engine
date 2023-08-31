use pyo3::prelude::*;
use crate::bitboard_helper::*;
use crate::lookuptables::LoadMoves;
use crate::chessboard_helper::*;

#[pyclass]
pub struct Chessboard {
    pos: Position,
    history: Vec<Position>,
    pseudo_moves: LoadMoves,
    // every time we make a move this will need to be cleared
    legal_moves_cache: [Option<u64>; 64],
    enemy_heat_cache: Option<u64>,
    pinned_pieces_cache: Option<u64>,
    checking_pieces_cache: Option<u64>,
    defended_cache: Option<u64>,
    pinned_masks_cache: [u64; 64]
}

impl Chessboard {
    fn get_heatmap(&mut self, color: &PieceColor) -> u64 {
        // returns all squares that the color can move to in a bitboard,
        // check if we have already calculated the enemy heat before
        match self.enemy_heat_cache {
            Some(heat) => return heat,
            None => {}
        }
        // calculate the heatmap
        let mut heat = 0;
        let (friendly_color, enemy_color) = match color {
            PieceColor::White => (PieceColor::White, PieceColor::Black),
            PieceColor::Black => (PieceColor::Black, PieceColor::White),
            PieceColor::None => {return 0}
        };
        let all_pieces = self.pieces(&friendly_color).get_all();
        // loop over all indices of black squares
        for index in bb_to_vec(all_pieces) {
            let piece_type = self.pieces(&friendly_color).detect_piece_type(index);
            heat |= self.get_pseudo_heat_moves(index as usize, &piece_type, &friendly_color, &enemy_color);
        }

        // store the heat in the cache
        self.enemy_heat_cache = Some(heat);
        heat
    }

    fn get_checking_pieces(&mut self, king_index: usize, color: &PieceColor) -> u64 {
        // returns a bitboard with all pieces that are currently checking the king

        // check if we still have the checking pieces stored
        match self.checking_pieces_cache {
            Some(attackers) => return attackers,
            None => {}
        }

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
        match color {
            PieceColor::White => {
                attackers |= subtract_bb(self.pseudo_moves.white_pawn(king_index), INDEX2FILE[king_index]) & enemy.get_bb_pawns();
            }
            PieceColor::Black => {
                attackers |= subtract_bb(self.pseudo_moves.black_pawn(king_index), INDEX2FILE[king_index]) & enemy.get_bb_pawns();
            }
            _ => {}
        }
        
        // store and return result
        self.checking_pieces_cache = Some(attackers);
        attackers
    }

    fn pieces(&mut self, color: &PieceColor) -> &mut Pieces {
        match color {
            PieceColor::White => &mut self.pos.white_pieces,
            PieceColor::Black => &mut self.pos.black_pieces,
            _ => {panic!("Cannot get pieces of no color!")}
        }
    }
    fn add_special_pawn_moves(&mut self, index: usize, mut legal_moves: u64, enemy_color: &PieceColor, friendly_color: &PieceColor) -> u64 {
        // this function assumes that there is a pawn present on index

        // this extra situation is needed because of this postion: 8/8/8/8/2k2p1R/8/2K1P3/8 w - - 0 1 after e4.
        match self.pos.es_target {
            None => {},
            Some(target) => {
                let blockers = self.pos.get_all();
                let king_index = self.pieces(&friendly_color).get_king_index();
                let invisable_pawn = match enemy_color {
                    PieceColor::White => set_bit(0, target - 8),
                    PieceColor::Black => set_bit(0, target + 8),
                    _ => 0
                };
                let blockers_without_invis = subtract_bb(blockers, invisable_pawn);

                let enemy = self.pieces(enemy_color);
                let rook_sliders = enemy.get_bb_queens() | enemy.get_bb_rooks();
                let enemy_pieces2consider = (self.pseudo_moves.direction_ray(index, 2) |  
                    self.pseudo_moves.direction_ray(index, 6)) & rook_sliders;
                for enemy_index in bb_to_vec(enemy_pieces2consider) {
                    let enemy_moves = *self.pseudo_moves.rook(enemy_index as usize, blockers_without_invis).unwrap();
                    for direction in [2, 6] {
                        let enemy_ray = self.pseudo_moves.direction_ray(enemy_index as usize, direction) & enemy_moves;
                        let king_ray = self.pseudo_moves.rook(king_index, blockers_without_invis).unwrap()
                             & self.pseudo_moves.direction_ray(king_index, (direction + 4) % 8);
                        // so there is a single piece between the king and a rook type piece, if it a pawn it cannot capture en passant
                        if enemy_ray & king_ray == set_bit(0, index as u8) {
                            legal_moves = subtract_bb(legal_moves, set_bit(0, target));
                        }
                    }
                }
            }
        }
        legal_moves
    }
    fn get_pseudo_heat_moves(&mut self, index: usize, piece_type: &PieceType, friendly_color: &PieceColor, enemy_color: &PieceColor) -> u64 {
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
                // captures and pieces that block our way
                subtract_bb(pawn_moves, piece_file)
            }
            PieceType::Knight => self.pseudo_moves.knight(index as usize),
            PieceType::Bishop => *self.pseudo_moves.bishop(index, subtract_bb(blockers, enemy_king_bb)).expect("Couldn't get bishop moves"),
            PieceType::Rook => *self.pseudo_moves.rook(index, subtract_bb(blockers, enemy_king_bb)).expect("Couldn't get rook moves"),
            PieceType::Queen => self.pseudo_moves.queen(index, subtract_bb(blockers, enemy_king_bb)).expect("Couldn't get queen moves"),
            PieceType::King => self.pseudo_moves.king(index),
            _ => 0
        }
    }
    
    fn get_pseudo_legal_moves(&mut self, index: usize, piece_type: &PieceType, friendly_color: &PieceColor, enemy_color: &PieceColor) -> u64 {
        // returns pseudo legal moves, that is, legal moves ignoring checks and pins
        // some information we use later
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
                let front = subtract_bb(pawn_moves & piece_file, blockers) & *self.pseudo_moves.rook(index as usize, blockers).expect("Couldn't get rook moves");
                // captures and pieces that block our way
                let captures = match self.pos.es_target {
                    Some(target) => subtract_bb(set_bit(self.pieces(&enemy_color).get_all(), target) & pawn_moves, piece_file),
                    None => {
                        subtract_bb(self.pieces(&enemy_color).get_all() & pawn_moves, piece_file)
                    }
                };
                front | captures
            }
            PieceType::Knight => self.pseudo_moves.knight(index as usize),
            PieceType::Bishop => *self.pseudo_moves.bishop(index, blockers).expect("Couldn't get bishop moves"),
            PieceType::Rook => *self.pseudo_moves.rook(index, blockers).expect("Couldn't get rook moves"),
            PieceType::Queen => self.pseudo_moves.queen(index, blockers).expect("Couldn't get queen moves"),
            PieceType::King => {
                    subtract_bb(self.pseudo_moves.king(index), self.get_heatmap(&enemy_color)
                     | self.get_defended(&enemy_color))
                      | self.get_castling_squares(index, &friendly_color)
            }
            ,
            _ => 0
        }
    }
    fn add_check_moves(&mut self, legal_moves: u64, piece_type: &PieceType, friendly_color: &PieceColor, enemy_color: &PieceColor, pieces_giving_check: u64) -> u64 {
        // assumes that there is a single check present in the position. Then restricts the legal moves given to only moves legal due to the check
        
        match piece_type {
            PieceType::King => legal_moves,
            _ => {
                // split the legal moves in capture and non capture
                let mut legal_captures = legal_moves & self.pieces(&enemy_color).get_all();
                let mut legal_non_captures = subtract_bb(legal_moves, self.pieces(&enemy_color).get_all());
                // restrict captures to only captures of the pieces that are giving check
                legal_captures &= pieces_giving_check;

                // the ray to the king from the piece giving check
                let index_of_checking_piece = get_lsb_index(pieces_giving_check);
                let piece_type_of_checking_piece = self.pieces(&enemy_color).detect_piece_type(index_of_checking_piece as u8);
                match piece_type_of_checking_piece {
                    PieceType::Pawn => {
                        // we might still be able to capture the checking pawn with en passant
                        match self.pos.es_target {
                            Some(target) => {
                                // if the piece giving check is right below or above the en passant target square then it must be 
                                // that that piece just move two up and therefore we still allow en passant capture
                                if set_bit(set_bit(0, target - 8), target + 8) & pieces_giving_check != 0 {
                                    match piece_type {
                                        PieceType::Pawn => {legal_non_captures = set_bit(0, target) & legal_moves;},
                                        _ => {legal_non_captures = 0;}
                                    }
                                }
                                else {legal_non_captures = 0;}
                            },
                            None => {legal_non_captures = 0;}
                        }
                    }
                    PieceType::Knight => {
                        legal_non_captures = 0;
                    }
                    PieceType::Bishop => {
                        let bb_king = self.pieces(friendly_color).get_bb_king();
                        let king_index = (bb_king as f64).log2() as usize;
                        for direction_index in [1, 3, 5, 7] {
                            let ray = self.pseudo_moves.direction_ray(index_of_checking_piece as usize, direction_index);
                            if ray & bb_king != 0 {
                                legal_non_captures &= ray & self.pseudo_moves.direction_ray(king_index, (direction_index + 4)%8);
                                break
                            }
                        }
                    }
                    PieceType::Rook => {
                        let bb_king = self.pieces(friendly_color).get_bb_king();
                        let king_index = (bb_king as f64).log2() as usize;
                        for direction_index in [0, 2, 4, 6] {
                            let ray = self.pseudo_moves.direction_ray(index_of_checking_piece as usize, direction_index);
                            if ray & bb_king != 0 {
                                legal_non_captures &= ray & self.pseudo_moves.direction_ray(king_index, (direction_index + 4)%8);
                                break
                            }
                        }
                    }
                    PieceType::Queen => {
                        let bb_king = self.pieces(friendly_color).get_bb_king();
                        let king_index = (bb_king as f64).log2() as usize;
                        for direction_index in 0..8 {
                            let ray = self.pseudo_moves.direction_ray(king_index, direction_index);
                            if ray & bb_king != 0 {
                                legal_non_captures &= ray & self.pseudo_moves.direction_ray(bb_to_vec(bb_king)[0] as usize, (direction_index + 4)%8);
                                break
                            }
                        }
                    }
                    _ => {}
                }
                legal_captures | legal_non_captures
            }
        }
    }

    fn get_legal_moves(&mut self, index: u8) -> u64 {
        // index is the index of the piece of which we want to find the legal moves.
        // This function already assumes that the correct player is making the move
        // and that he is not trying to move to his own piece, it also assumes that 
        // on old index is indeed a piece of the player that has to move.

        // Check if we have already calculated the moves before
        match self.legal_moves_cache[index as usize] {
            Some(legal_moves) => return legal_moves,
            None => {}
        }
        let (friendly_color, enemy_color) = match self.pos.detect_piece_color(index) {
            PieceColor::White => (PieceColor::White, PieceColor::Black),
            PieceColor::Black => (PieceColor::Black, PieceColor::White),
            PieceColor::None => {return 0}
        };

        let piece_type = self.pieces(&friendly_color).detect_piece_type(index);
        let king_bb = self.pieces(&friendly_color).get_bb_king();
        let king_index = (king_bb as f64).log2() as usize;

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
        let mut legal_moves = self.get_pseudo_legal_moves(index as usize, &piece_type, &friendly_color, &enemy_color);

        // remove the ability to capture own pieces
        legal_moves = subtract_bb(legal_moves, self.pieces(&friendly_color).get_all());
        // now we restrict the legal moves if we are in single check
        if bb_to_vec(pieces_giving_check).len() == 1 {
            legal_moves = self.add_check_moves(legal_moves, &piece_type, &friendly_color, &enemy_color, pieces_giving_check);
        }

        // we must also add some special moves if certain situations for the pawns
        match piece_type {
            PieceType::Pawn => {legal_moves = self.add_special_pawn_moves(index as usize, legal_moves, &enemy_color, &friendly_color);}
            _ => {}
        }

        // finally we take into account the possibilty that our piece is currently pinned to the king (an absolute pin)
        let pinned = self.get_pinned(&friendly_color, &enemy_color, king_index);
        if (pinned >> index) & 1 == 1 {
            legal_moves = match piece_type {
                // kings cannot be pinned
                PieceType::King => legal_moves,
                PieceType::Knight => 0,
                PieceType::Bishop => legal_moves & self.pinned_masks_cache[index as usize],
                PieceType::Rook => legal_moves & self.pinned_masks_cache[index as usize],
                PieceType::Queen => legal_moves & self.pinned_masks_cache[index as usize],
                PieceType::Pawn => legal_moves & self.pinned_masks_cache[index as usize],
                _ => 0
            }
        }


        // store the result in the cache
        self.legal_moves_cache[index as usize] = Some(legal_moves);

        // return final moves
        legal_moves
    }

    fn get_pinned(&mut self, friendly_color: &PieceColor, enemy_color: &PieceColor, king_index: usize) -> u64 {
        // returns a bitboard with bits on all pieces that are currently pinned to the king
        // check if we still have the pinned pieces stored
        match self.pinned_pieces_cache {
            Some(pinned) => return pinned,
            None => {}
        }

        let mut pinned = 0;
        let blockers = self.pos.get_all();

        // loop over all enemy sliding pieces
        let enemy = self.pieces(&enemy_color);
        for index in bb_to_vec((enemy.get_bb_rooks() | enemy.get_bb_queens()) & *self.pseudo_moves.rook(king_index, 0).unwrap()) {
            let enemy_moves = *self.pseudo_moves.rook(index as usize, blockers).unwrap();
            // now for each direction we want to go from the king into the opposite direction and check the overlap
            for direction in [0, 2, 4, 6] {
                let enemy_ray = enemy_moves & self.pseudo_moves.direction_ray(index as usize, direction);
                let king_ray = self.pseudo_moves.rook(king_index, blockers).unwrap() & self.pseudo_moves.direction_ray(king_index, (direction + 4) % 8);
                let pinned_piece = enemy_ray & king_ray;
                // store the mask for later use
                if pinned_piece != 0 {
                    pinned |= pinned_piece;
                    let pinned_index = get_lsb_index(pinned_piece);
                    self.pinned_masks_cache[pinned_index] = self.pseudo_moves.direction_ray(pinned_index, direction) | 
                    self.pseudo_moves.direction_ray(pinned_index, (direction + 4) % 8);
                    break;
                }
            }
        }
        let enemy = self.pieces(&enemy_color);
        for index in bb_to_vec((enemy.get_bb_bishops() | enemy.get_bb_queens()) & *self.pseudo_moves.bishop(king_index, 0).unwrap()) {
            let enemy_moves = *self.pseudo_moves.bishop(index as usize, blockers).unwrap();
            // now for each direction we want to go from the king into the opposite direction and check the overlap
            for direction in [1, 3, 5, 7] {
                let enemy_ray = enemy_moves & self.pseudo_moves.direction_ray(index as usize, direction);
                let king_ray = self.pseudo_moves.bishop(king_index, blockers).unwrap() & self.pseudo_moves.direction_ray(king_index, (direction + 4) % 8);
                let pinned_piece = enemy_ray & king_ray;
                // store the mask for later use
                if pinned_piece != 0 {
                    pinned |= pinned_piece;
                    let pinned_index = get_lsb_index(pinned_piece);
                    self.pinned_masks_cache[pinned_index] = self.pseudo_moves.direction_ray(pinned_index, direction) | 
                    self.pseudo_moves.direction_ray(pinned_index, (direction + 4) % 8);
                    break;
                }
            }
        }

        self.pinned_pieces_cache = Some(pinned);
        pinned

    }

    fn get_defended(&mut self, friendly_color: &PieceColor) -> u64 {
        // returns a bitboard with bits on all pieces that are defended by one of his own pieces
        match self.defended_cache {
            Some(defended) => return defended,
            None => {}
        }
        let mut defended = 0;

        // loop over all indices of squares
        for index in bb_to_vec(self.pieces(&friendly_color).get_all()) {
            defended |= self.get_defended_by_piece(index, friendly_color);
        }
        self.defended_cache = Some(defended);
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
                if (heat & WHITE_QUEENSIDE_CASTLE_MAP == 0) && (all_pieces & set_bit(WHITE_QUEENSIDE_CASTLE_MAP, 57) == 0) && self.pos.white_queenside_castle {
                    castle_squares |= WHITE_QUEENSIDE_CASTLE_GOAL;
                }
            }
            PieceColor::Black => {
                let heat = self.get_heatmap(&PieceColor::White);
                if ((heat | all_pieces) & BLACK_KINGSIDE_CASTLE_MAP == 0) && self.pos.black_kingside_castle {
                    castle_squares |= BLACK_KINGSIDE_CASTLE_GOAL;
                }
                if (heat & BLACK_QUEENSIDE_CASTLE_MAP == 0) && (all_pieces & set_bit(BLACK_QUEENSIDE_CASTLE_MAP, 1) == 0) && self.pos.black_queenside_castle {
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
        let legal_moves = self.get_legal_moves(old_index);
        // if there is a bit on the legal moves bitboard at index, the move is legal
        if (legal_moves >> index) & 1 == 1 {
            return true
        }
        else {
            false
        }
    }

    pub fn move_piece(&mut self, new_move: Move) -> Result<(), NoLegalMoveInputError> {
        // make a clone of the current position that we add in the end in case the move was legally made
        let cloned = self.pos.clone();

        // extract all relevant data
        let old_index = new_move.from;
        let index = new_move.to;
        let piece_color = match self.pos.to_move {
            ToMove::White => &PieceColor::White,
            ToMove::Black => &PieceColor::Black
        };     
        // old_index:    the index of the piece we want to move
        // index:        index of where we want to move the piece
        // piece_color:  color of the piece we want to move
        
        // when possible move piece from old index to new index

        // return if the move is not legal
        if !self.check_move_for_legal(old_index, index, piece_color) {
            return Err(NoLegalMoveInputError)
        }

        let (friendly_color, enemy_color) = match self.pos.detect_piece_color(old_index) {
            PieceColor::White => (PieceColor::White, PieceColor::Black),
            PieceColor::Black => (PieceColor::Black, PieceColor::White),
            PieceColor::None => {return Err(NoLegalMoveInputError)}
        };

        // get the piece type of the piece we want to move and the piece type
        // of the piece we want to capture (note that these are not by reference but a copy, so we must
        // replace them later.)
        let moving_piece_type = self.pieces(&friendly_color).detect_piece_type(old_index);
        let mut captured_piece_type = self.pieces(&enemy_color).detect_piece_type(index);
        let mut bb_moving_piece = self.pieces(&friendly_color).piece_type2bb(&moving_piece_type);
        let mut bb_captured_piece = self.pieces(&enemy_color).piece_type2bb(&captured_piece_type);
        let mut promoted: bool = false;
        match moving_piece_type {
            // detect en-passant for capture or new es-target
            PieceType::Pawn => {
                // detect en-passant capture
                match self.pos.es_target {
                    Some(target) => {                
                        if index == target {
                            captured_piece_type = PieceType::Pawn;
                            bb_captured_piece = self.pieces(&enemy_color).get_bb_pawns();
                            match enemy_color {
                                PieceColor::White => bb_captured_piece = subtract_bb(bb_captured_piece, set_bit(0, index - 8)),
                                PieceColor::Black => bb_captured_piece = subtract_bb(bb_captured_piece, set_bit(0, index + 8)),
                                _ => return Err(NoLegalMoveInputError)
                            };
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

                // we finally also detect promotions
                let (rank, _) = index2rank_file(index).expect("Index is too high.");
                if rank == 8 || rank == 1 {
                    let on_promotion = new_move.on_promotion.expect("There was no promotion type specified.");
                    promoted = true;
                    // now set the promoted piece
                    let mut promoted_pieces = self.pieces(&friendly_color).piece_type2bb(&on_promotion.to_piece_type());
                    promoted_pieces = set_bit(promoted_pieces, index);
                    self.pieces(&friendly_color).set_bb_of_piece_type(promoted_pieces, &on_promotion.to_piece_type());
                }
            }
            // detect castle move
            PieceType::King => {
                // detect if a castling move has been made
                // white kingside castling
                let mut friendly_rooks = self.pieces(&friendly_color).get_bb_rooks();
                if index == 62 && self.pos.white_kingside_castle {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 63));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 61));
                }
                // white queenside castling
                else if index == 58 && self.pos.white_queenside_castle {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 56));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 59));
                }
                // black kingside castling
                if index == 6 && self.pos.black_kingside_castle {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 7));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 5));
                }
                // black queenside castling
                if index == 2 && self.pos.black_queenside_castle {
                    friendly_rooks = subtract_bb(friendly_rooks, set_bit(0, 0));
                    self.pieces(&friendly_color).set_bb_rooks(set_bit(friendly_rooks, 3));
                }
                // update castling rights
                match piece_color {
                    PieceColor::Black => {
                        self.pos.black_kingside_castle = false;
                        self.pos.black_queenside_castle = false;
                    },
                    PieceColor::White => {
                        self.pos.white_kingside_castle = false;
                        self.pos.white_queenside_castle = false;
                    },
                    _ => {}
                }
                self.pos.es_target = None;

            }
            _ => {self.pos.es_target = None;}
        };
        // detect capturing or moving of a rook
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
        
        // make the move on the bitboards
        bb_moving_piece = subtract_bb(bb_moving_piece, set_bit(0, old_index));
        if !promoted {
            bb_moving_piece = set_bit(bb_moving_piece, index);
        }
        bb_captured_piece = subtract_bb(bb_captured_piece, set_bit(0, index));
        
        // place the new bitboards on the place of the old ones
        self.pieces(&friendly_color).set_bb_of_piece_type(bb_moving_piece, &moving_piece_type);
        self.pieces(&enemy_color).set_bb_of_piece_type(bb_captured_piece, &captured_piece_type);

        // update meta data
        self.pos.to_move = match self.pos.to_move {
            ToMove::White => ToMove::Black,
            _ => ToMove::White
        };
        // update halfmove clock
        match moving_piece_type {
            PieceType::Pawn => self.pos.halfmove_clock = 0,
            _ => {
                match captured_piece_type {
                    PieceType::EmptySquare => self.pos.halfmove_clock += 1,
                    _ => self.pos.halfmove_clock = 0
                }
            }
        }
        // update fullmove counter
        match self.pos.to_move {
            ToMove::White => self.pos.fullmove_clock += 1,
            _ => {}
        }
        self.clear_cache();
        self.history.push(cloned);
        Ok(())
    }

    pub fn all_moves(&mut self) -> Vec<Move> {
        // this function returns all the legal moves the current player can make in the position
        let mut all_moves: Vec<Move> = Vec::new();
        let color = match self.pos.to_move {
            ToMove::White => PieceColor::White,
            ToMove::Black => PieceColor::Black
        };
        // loop over all pieces that can move
        let all_pieces = self.pieces(&color).get_all();
        for piece_index in bb_to_vec(all_pieces) {
            let piece_type = self.pieces(&color).detect_piece_type(piece_index);
            // get all legal moves of current piece
            let legal_moves = self.get_legal_moves(piece_index);
            // now for each possible legal move want to add all moves to the vec
            for to_index in bb_to_vec(legal_moves) {
                // notice that when we are going to promote we have more options!
                match piece_type {
                    PieceType::Pawn => {
                        let (rank, _) = index2rank_file(to_index).unwrap();
                        // we are promoting
                        if rank == 8 || rank == 1 {
                            all_moves.push(Move {from: piece_index, to: to_index, on_promotion: Some(PiecePromotes::Rook)});
                            all_moves.push(Move {from: piece_index, to: to_index, on_promotion: Some(PiecePromotes::Bishop)});
                            all_moves.push(Move {from: piece_index, to: to_index, on_promotion: Some(PiecePromotes::Knight)});
                            all_moves.push(Move {from: piece_index, to: to_index, on_promotion: Some(PiecePromotes::Queen)});
                        }
                        else {
                            all_moves.push(Move {from: piece_index, to: to_index, on_promotion: None});
                        }
                    },
                    _ => {all_moves.push(Move {from: piece_index, to: to_index, on_promotion: None});}
                };
            }
        }
        // return the final moves
        all_moves
    }

    pub fn get_to_move(&self) -> &ToMove {
        &self.pos.to_move
    }
}

#[pymethods]
impl Chessboard {
    #[staticmethod]
    pub fn new_start() -> Chessboard {
        Chessboard { pos: Position::new_start(),
        history: Vec::new(),
        pseudo_moves: LoadMoves::new(),
        legal_moves_cache: [None; 64],
        enemy_heat_cache: None,
        pinned_pieces_cache: None,
        checking_pieces_cache: None,
        defended_cache: None,
        pinned_masks_cache: [0; 64] }
    }
    #[new]
    pub fn new() -> Chessboard {
        Chessboard { pos: Position::new(),
            history: Vec::new(),
        pseudo_moves: LoadMoves::new(),
        legal_moves_cache: [None; 64],
        enemy_heat_cache: None,
        pinned_pieces_cache: None,
        checking_pieces_cache: None,
        defended_cache: None,
        pinned_masks_cache: [0; 64] }
    }
    pub fn to_string(&self) -> String {
        self.pos.to_string()
    }

    pub fn clear(&mut self) {
        self.pos = Position::new();
        self.clear_cache();
    }

    pub fn clear_cache(&mut self) {
        self.legal_moves_cache = [None; 64];
        self.enemy_heat_cache = None;
        self.pinned_pieces_cache = None;
        self.checking_pieces_cache = None;
        self.defended_cache = None;
    }

    pub fn undo(&mut self) {
        if self.history.len() > 0 {
            self.pos = self.history.pop().expect("couldn't get previous position.");
            self.clear_cache();
        }
    }
    pub fn legal_positions_on_depth(&mut self, depth: u8) -> u128 {
        // gets the number of legal positions in the current position within a certain depth
        if depth == 0 {
            return 1
        }
        let moves = self.all_moves();
        let mut num_positions = 0;
        // now loop over all moves
        for current_move in moves {
            self.move_piece(current_move).unwrap();
            num_positions += self.legal_positions_on_depth(depth - 1);
            self.undo();
        }
        num_positions
    }
    pub fn time_legal_positions_on_depth(&mut self, depth: u8) {
        let now = std::time::Instant::now();
        let result = self.legal_positions_on_depth(depth);
        let elapsed = now.elapsed();
        println!("Depth {} ply  Calculated result: {} positions  Time: {:.2?}", depth, result, elapsed);
    }

    pub fn debug_depth(&mut self, depth: u8) {
        let moves = self.all_moves();
        for current_move in moves {
            self.move_piece(current_move).unwrap();
            println!("{}: {}", current_move.to_string(), self.legal_positions_on_depth(depth - 1));
            self.undo();
        }
    }

    pub fn test_position_depth(&mut self) {
        // first we test the standard position
        self.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string());
        println!("\nTESTING ON 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'");
        println!("\nDepth 1 ply  Actual:            20");
        self.time_legal_positions_on_depth(1);
        println!("\nDepth 2 ply  Actual:            400");
        self.time_legal_positions_on_depth(2);
        println!("\nDepth 3 ply  Actual:            8902");
        self.time_legal_positions_on_depth(3);
        println!("\nDepth 4 ply  Actual:            197281");
        self.time_legal_positions_on_depth(4);
        println!("\nDepth 5 ply  Actual:            4865609");
        self.time_legal_positions_on_depth(5);
        println!("\nDepth 6 ply  Actual:            119060324");
        self.time_legal_positions_on_depth(6);

        self.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string());
        println!("\nTESTING ON 'r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1'");
        println!("\nDepth 1 ply  Actual:            48");
        self.time_legal_positions_on_depth(1);
        println!("\nDepth 2 ply  Actual:            2039");
        self.time_legal_positions_on_depth(2);
        println!("\nDepth 3 ply  Actual:            97862");
        self.time_legal_positions_on_depth(3);
        println!("\nDepth 4 ply  Actual:            4085603");
        self.time_legal_positions_on_depth(4);
        println!("\nDepth 5 ply  Actual:            193690690");
        self.time_legal_positions_on_depth(5);

        self.load_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string());
        println!("\nTESTING ON '8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1'");
        println!("\nDepth 1 ply  Actual:            14");
        self.time_legal_positions_on_depth(1);
        println!("\nDepth 2 ply  Actual:            191");
        self.time_legal_positions_on_depth(2);
        println!("\nDepth 3 ply  Actual:            2812");
        self.time_legal_positions_on_depth(3);
        println!("\nDepth 4 ply  Actual:            43238");
        self.time_legal_positions_on_depth(4);
        println!("\nDepth 5 ply  Actual:            674624");
        self.time_legal_positions_on_depth(5);
        println!("\nDepth 6 ply  Actual:            11030083");
        self.time_legal_positions_on_depth(6);

        self.load_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string());
        println!("\nTESTING ON 'r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1'");
        println!("\nDepth 1 ply  Actual:            6");
        self.time_legal_positions_on_depth(1);
        println!("\nDepth 2 ply  Actual:            264");
        self.time_legal_positions_on_depth(2);
        println!("\nDepth 3 ply  Actual:            9467");
        self.time_legal_positions_on_depth(3);
        println!("\nDepth 4 ply  Actual:            422333");
        self.time_legal_positions_on_depth(4);
        println!("\nDepth 5 ply  Actual:            15833292");
        self.time_legal_positions_on_depth(5);

        self.load_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string());
        println!("\nTESTING ON 'rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8'");
        println!("\nDepth 1 ply  Actual:            44");
        self.time_legal_positions_on_depth(1);
        println!("\nDepth 2 ply  Actual:            1486");
        self.time_legal_positions_on_depth(2);
        println!("\nDepth 3 ply  Actual:            62379");
        self.time_legal_positions_on_depth(3);
        println!("\nDepth 4 ply  Actual:            2103487");
        self.time_legal_positions_on_depth(4);
        println!("\nDepth 5 ply  Actual:            89941194");
        self.time_legal_positions_on_depth(5);

        self.load_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".to_string());
        println!("\nTESTING ON 'r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10'");
        println!("\nDepth 1 ply  Actual:            46");
        self.time_legal_positions_on_depth(1);
        println!("\nDepth 2 ply  Actual:            2079");
        self.time_legal_positions_on_depth(2);
        println!("\nDepth 3 ply  Actual:            89890");
        self.time_legal_positions_on_depth(3);
        println!("\nDepth 4 ply  Actual:            3894594");
        self.time_legal_positions_on_depth(4);
        println!("\nDepth 5 ply  Actual:            164075551");
        self.time_legal_positions_on_depth(5);



    }


    pub fn load_fen(&mut self, fen: String) {
        // first clear the board
        self.clear();
        // get the parts of the FEN format
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board = match parts.get(0) {
            Some(board) => board,
            None => return
        };
        let filtered_board = board.replace("/", "");
        // now load the board
        let mut index: u32 = 0;
        for ch in filtered_board.chars() {
            if ch.is_digit(10) {
                index += ch.to_digit(10).unwrap();
            }
            else if ch.is_ascii_uppercase() {
                let piece_type = PieceType::from_char(ch.to_ascii_lowercase());
                let new_bb = set_bit(self.pos.white_pieces.piece_type2bb(&piece_type), index as u8);
                self.pos.white_pieces.set_bb_of_piece_type(new_bb, &piece_type);
                index += 1;
            }
            else if ch.is_ascii_lowercase() {
                let piece_type = PieceType::from_char(ch);
                let new_bb = set_bit(self.pos.black_pieces.piece_type2bb(&piece_type), index as u8);
                self.pos.black_pieces.set_bb_of_piece_type(new_bb, &piece_type);
                index += 1;
            }
        }
        // set the person to move
        self.pos.to_move = match parts.get(1) {
            None => return,
            Some(c) => {
                if c == &"w" {
                    ToMove::White
                }
                else if c == &"b" {
                    ToMove::Black
                }
                else {return}
            }
        };
        // set castling ability
        match parts.get(2) {
            None => return,
            Some(rights) => {
                if rights.contains('K') {
                    self.pos.white_kingside_castle = true;
                } else {self.pos.white_kingside_castle = false;}
                if rights.contains('Q') {
                    self.pos.white_queenside_castle = true;
                } else {self.pos.white_queenside_castle = false;}
                if rights.contains('k') {
                    self.pos.black_kingside_castle = true;
                } else {self.pos.black_kingside_castle = false;}
                if rights.contains('q') {
                    self.pos.black_queenside_castle = true;
                } else {self.pos.black_queenside_castle = false;}
            }
        }
        // set en passant target square
        self.pos.es_target = match parts.get(3) {
            None => return,
            Some(target) => board_notation2index(target)
        };

        // set half and full move clock
        self.pos.halfmove_clock = match parts.get(4) {
            None => 0,
            Some(num) => match num.chars().nth(0) {
                None => 0,
                Some(i) => i.to_digit(10).unwrap() as u8
            }
        };
        self.pos.fullmove_clock = match parts.get(5) {
            None => 1,
            Some(num) => match num.chars().nth(0) {
                None => 1,
                Some(i) => i.to_digit(10).unwrap() as u8
            }
        };
        self.clear_cache();
    }

    pub fn get_legal_captures(&mut self, index: u8) -> Vec<u8> {
        // index must be the index of the piece of which we want to get legal captures
        let piece_color = self.pos.detect_piece_color(index);
        // return early if we have selected an empty square
        match piece_color {
            PieceColor::None => return vec![],
            _ => {}
        }
        let legal_moves = self.get_legal_moves(index);
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
        let legal_moves = self.get_legal_moves(index);
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

    pub fn get_white_pieces(&mut self) -> u64 {
        self.pos.white_pieces.get_all()
    }
    pub fn get_black_pieces(&mut self) -> u64 {
        self.pos.black_pieces.get_all()
    }

}
use crate::bitboard_helper::*;
use crate::chessboard_helper::MoveCalculator;
use rand::Rng;
use std::io::{BufWriter, Write, BufReader};
use std::{fs, path};
use std::convert::TryInto;
use serde::{Serialize, Deserialize};
use serde;
use serde_big_array::BigArray;
use std::thread;
use std::sync::{Mutex, Arc};
use std::cmp;

// Basic struct that is used for writing and parsing the json files.
#[derive(Serialize, Deserialize, Debug)]
struct Array64 {
    values: Vec<u64>
}

#[derive(Serialize, Deserialize, Debug)]
struct DirectionTable {
    #[serde(with = "BigArray")]
    values: [[u64; 8]; 64]
}

enum FailedMagicNumberError {
    ConflictingIndexError
}

#[derive(Clone)]
pub enum SlidePieceType {
    Rook,
    Bishop
}

impl SlidePieceType {
    fn to_string(&self) -> &str {
        match self {
            SlidePieceType::Rook => "Rook",
            SlidePieceType::Bishop => "Bishop"
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MagicLookUp {
    #[serde(with = "BigArray")]
    shifts: [u8; 64],
    #[serde(with = "BigArray")]
    magic_numbers: [u64; 64],
    #[serde(with = "BigArray")]
    magic_masks: [Vec<u64>; 64] // for each square we have a vector where after multiplying the blocker by
    // the corresponding magic number and then shifting by the corresponding 
    // shift we find on that index the legal moves for that given piece and blocker combination.
}
impl MagicLookUp {
    pub fn new() -> MagicLookUp {
        MagicLookUp { shifts: [0; 64], magic_numbers: [0; 64], magic_masks: std::array::from_fn(|_| Vec::new())}
    }
}

pub struct CreateLookUpTables {
    // Note that we use a Vec here because we do not yet know the size.
    // later when loading the lookup table one should use an array since that is faster.
    knight_masks: Vec<u64>,
    white_pawn_masks: Vec<u64>,
    black_pawn_masks: Vec<u64>,
    king_masks: Vec<u64>,
    bishop_magic_lookup: MagicLookUp,
    rook_magic_lookup: MagicLookUp,
    // masks for where possible blockers can be for a given piece location. 
    // Is basically the moves for an empty board while ignoring the outer ranks and files
    rook_pre_masks: Vec<u64>,
    rook_blocker_patterns: [Vec<u64>; 64],
    bishop_pre_masks: Vec<u64>,
    bishop_blocker_patterns: [Vec<u64>; 64],
    // all directions on an empty board in order [n, ne, e, se, s, sw, w, nw] for every square
    direction_masks: [[u64; 8]; 64]
}

impl CreateLookUpTables {
    pub fn new() -> CreateLookUpTables {
        let rook_magic = match LoadMoves::parse_slide_masks("rook_magic_lookup.json") {
            Ok(res) => res,
            Err(_) => MagicLookUp::new()
        };
        let bishop_magic = match LoadMoves::parse_slide_masks("bishop_magic_lookup.json") {
            Ok(res) => res,
            Err(_) => MagicLookUp::new()
        };
        CreateLookUpTables { knight_masks: Vec::new(),
             white_pawn_masks: Vec::new(), 
             black_pawn_masks: Vec::new(), 
             king_masks: Vec::new(),
             bishop_magic_lookup: bishop_magic, 
             rook_magic_lookup: rook_magic, 
             rook_pre_masks: Vec::new(),
             rook_blocker_patterns: std::array::from_fn(|_| Vec::new()),
             bishop_pre_masks: Vec::new(),
             bishop_blocker_patterns: std::array::from_fn(|_| Vec::new()),
             direction_masks: [[0; 8]; 64]}
    }
    fn setup_directory(filepath: &str) {
        // setup into the correct directory
        let result = std::env::set_current_dir(filepath);
        match result {
            Ok(_) => {},
            Err(_) => {
                fs::create_dir(filepath).expect("Couldn't make directory");
                std::env::set_current_dir(filepath).expect("Couldn't set directory to lookuptables");
            }
        };
    }

    fn write2file<T: Serialize>(filepath: &str, data: &T) -> Result<(), std::io::Error> {
        let file = fs::File::create(path::Path::new(filepath))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, data)?;
        writer.flush()?;
        Ok(())
    }

    fn get_specific_blocker_pattern(movement_mask: u64) -> Vec<u64> {
        // Takes in a movement mask to where a piece can move. In practice this will be the bishop and rook.
        // Then the function returns a vector with all possible blockers on this movement mask. So all possible 
        // ways we can have pieces blocking the possible moves of the piece. 

        // Create a list of the indices of the bits that are set in the movement mask.
        let mut move_square_indices: Vec<u64> = Vec::new();
        for i in 0..64 {
            if (movement_mask >> i) & 1 == 1 {
                move_square_indices.push(i);
            }
        }
        // Create all possible bitboards
        let num_blockers: u64 = 1 << move_square_indices.len(); // is just 2^n
        let mut blocker_bitboards: Vec<u64> = vec![0; num_blockers as usize];
        for pattern_index in 0..num_blockers {
            // Shift all the bits of the pattern index into the movement mask
            for bit_index in 0..move_square_indices.len() {
                // Bit is often 0
                let bit = pattern_index >> bit_index & 1;
                blocker_bitboards[pattern_index as usize] |= bit << move_square_indices[bit_index];
            }
        }
        blocker_bitboards
    }

    fn create_rook_pre_table(&mut self) -> Result<(), std::io::Error> {
        // Creates the pre masks for the rook, that is, for a given index,
        // all possible places where blocker might be. Here we ignore the outer squares because
        // we can either move there, or capture there.

        // reset the vector
        self.rook_pre_masks = Vec::new();
        // loop over the board
        for i in 0..64 {
            // get moves and remove the redundant edges
            let mut movecalc = MoveCalculator::new(0, i);
            movecalc.calculate_rook_moves().expect("Couldn't get rook moves");
            let moves_for_index = movecalc.remove_redundant_board_edges();
            self.rook_pre_masks.push(moves_for_index);
        }
        // save the result
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("rook_pre_masks.json", &Array64{values: self.rook_pre_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }
    fn get_all_rook_blocker_patterns(&mut self) {
        // This function gets all the possible rook blocker patterns for each possible rook location.
        // Returns an array of size 64 corresponding to each of the 64 rook locations on the board. For
        // each location there is a vector containing all possible blockers for that rook placement.
        self.rook_blocker_patterns = std::array::from_fn(|_| Vec::new());

        // Loop over all squares
        for i in 0..64 {
            let movement_mask: u64 = self.rook_pre_masks[i];
            self.rook_blocker_patterns[i] = CreateLookUpTables::get_specific_blocker_pattern(movement_mask);
        }
    }

    fn try_slide_magic_number(blockers: &[Vec<u64>; 64], piece_index: usize, magic_number: u64, shift: u8, slide_type: &SlidePieceType) -> Result<Vec<u64>, FailedMagicNumberError> {
        let mut new_magic_masks: Vec<u64> = Vec::new();

        // Check all possible blocker patterns and keep placing them as long as there are no duplicates
        for blocker in &blockers[piece_index] {
            let lookup_index = blocker.wrapping_mul(magic_number) >> shift;
            // Handle if we already have a value on the index or if the vector is not large enough
            match new_magic_masks.get(lookup_index as usize) {
                // Detect duplicate indices and move on to the next magic number
                Some(old) => {if *old != 0 {return Err(FailedMagicNumberError::ConflictingIndexError)}},
                None => new_magic_masks.resize(lookup_index as usize + 1, 0)
            }
            // Calculate the moves and place them on the correct index
            let mut move_calculator = MoveCalculator::new(*blocker, piece_index as u8);
            let moves = match slide_type {
                SlidePieceType::Rook => move_calculator.calculate_rook_moves().expect("Error in calculating rook moves for rook table."),
                SlidePieceType::Bishop => move_calculator.calculate_bishop_moves().expect("Error in calculating bishop moves for bishop table.")
            };
            new_magic_masks[lookup_index as usize] = moves;
        }

        Ok(new_magic_masks)
    }

    fn thread_slide_magic_search(search_number: u32, blockers: &[Vec<u64>; 64], magic_lookup: Arc<Mutex<MagicLookUp>>, slide_type: SlidePieceType) {
        // get the shift range
        let shift_range = match slide_type {
            SlidePieceType::Rook => 50..55,
            SlidePieceType::Bishop => 53..63
        };
        for piece_index in 0..64 {
            // count how many times we went through everything
            let mut i = 0;
            // we keep going untill we find at least one magic number
            let mut found_magic = false;
            // Find the best magic number in combination with shift
            let number2search = ((32.0 - piece_index as f64).abs()*0.1*search_number as f64) as u32 + search_number;
            while i <= number2search || !found_magic {
                // note the new scope so that way the lock will be dropped afterwards
                {
                    // every 100 try's we update if we have already found a magic number in any other thread.
                    let m = magic_lookup.lock().expect("Couldn't get lock on magic lookup");
                    if m.magic_numbers[piece_index] != 0 {
                        found_magic = true;
                    }

                }

                // Make new magic
                let magic_number: u64 = rand::thread_rng().gen_range(0..(1 << 63));
                for shift in shift_range.clone() {
                    // Send the found magic number to the recieving thread
                    match CreateLookUpTables::try_slide_magic_number(&blockers, piece_index, magic_number, shift, &slide_type) {
                        Ok(magic_masks) => {
                            // We found a magic number!
                            found_magic = true;
                            // If the new lookuptable is smaller than the old one we will replace it.
                            let mut m = magic_lookup.lock().expect("Couldn't get lock on magic lookup");
                            if m.magic_masks[piece_index].len() > magic_masks.len() || m.magic_masks[piece_index].len() == 0 {
                                println!("{}: magic number {} for piece index {} with shift {} and lenght {}", slide_type.to_string(),
                                magic_number, piece_index, shift, magic_masks.len());
                                m.magic_masks[piece_index] = magic_masks;
                                m.magic_numbers[piece_index] = magic_number;
                                m.shifts[piece_index] = shift;
                            }
                        },
                        Err(_) => {}
                    };
                }
                i += 1;
            }
        }
    }

    pub fn create_slide_piece_table(&mut self, search_number: u32, slide_type: SlidePieceType) -> Result<(), std::io::Error> {
        // search_number: the number of magic numbers to try for each board index.
        // Store the shift numbers
        let blockers = match slide_type {
            SlidePieceType::Rook => {
                self.create_rook_pre_table()?;
                self.get_all_rook_blocker_patterns();
                self.rook_blocker_patterns.clone()
            }
            SlidePieceType::Bishop => {
                self.create_bishop_pre_table()?;
                self.get_all_bishop_blocker_patterns();
                self.bishop_blocker_patterns.clone()
            }
        };
        let magic_lookup = match slide_type {
            SlidePieceType::Rook => self.rook_magic_lookup.clone(),
            SlidePieceType::Bishop => self.bishop_magic_lookup.clone()
        };
        // create a version of self that can have multiple owners, namely all threads will own self.
        let shared_magic_lookup = Arc::new(Mutex::new(magic_lookup));
        // store handles
        let mut handles = vec![];

        // create all threads
        for _ in 0..7 {
            let cloned_magic = shared_magic_lookup.clone();
            let clone_blockers = blockers.clone();
            let clone_slide_type = slide_type.clone();
            let handle = thread::spawn(move || {
                CreateLookUpTables::thread_slide_magic_search(search_number, &clone_blockers, cloned_magic, clone_slide_type);
            });
            handles.push(handle);
        }

        // make sure all handles have finished
        for handle in handles {
            handle.join().unwrap();
        }

        let magic_lookup = Arc::try_unwrap(shared_magic_lookup).unwrap().into_inner().unwrap();
        // save the new magic lookup tables.
        CreateLookUpTables::setup_directory("lookuptables");
        match slide_type {
            SlidePieceType::Rook => {
                self.rook_magic_lookup = magic_lookup;
                CreateLookUpTables::write2file("rook_magic_lookup.json", &self.rook_magic_lookup)?;
            },
            SlidePieceType::Bishop => {
                self.bishop_magic_lookup = magic_lookup;
                CreateLookUpTables::write2file("bishop_magic_lookup.json", &self.bishop_magic_lookup)?;
            }
        }
        std::env::set_current_dir("../")?;

        Ok(())
    }

    fn create_bishop_pre_table(&mut self) -> Result<(), std::io::Error> {
        // Creates the pre masks for the bishop, that is, for a given index,
        // all possible places where blocker might be. Here we ignore the outer squares because
        // we can either move there, or capture there.

        // reset the vector
        self.bishop_pre_masks = Vec::new();
        // loop over the board
        for i in 0..64 {
            // get moves and remove the redundant edges
            let mut movecalc = MoveCalculator::new(0, i);
            movecalc.calculate_bishop_moves().expect("Couldn't get bishop moves");
            let moves_for_index = movecalc.remove_redundant_board_edges();
            self.bishop_pre_masks.push(moves_for_index);
        }
        // save the result
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("bishop_pre_masks.json", &Array64{values: self.bishop_pre_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }
    fn get_all_bishop_blocker_patterns(&mut self) {
        // This function gets all the possible bishop blocker patterns for each possible bishop location.
        // Returns an array of size 64 corresponding to each of the 64 bishop locations on the board. For
        // each location there is a vector containing all possible blockers for that bishop placement.
        self.bishop_blocker_patterns = std::array::from_fn(|_| Vec::new());

        // Loop over all squares
        for i in 0..64 {
            let movement_mask: u64 = self.bishop_pre_masks[i];
            self.bishop_blocker_patterns[i] = CreateLookUpTables::get_specific_blocker_pattern(movement_mask);
        }
    }

    pub fn create_knight_table(&mut self) -> Result<(), std::io::Error> {
        // This function will use as index the knight piece index,
        // since for each knight position there will exactly one way the
        // knight is able to move
        self.knight_masks = Vec::new();
        for i in 0..64 {
            let mut movecalc = MoveCalculator::new(0, i as u8);
            self.knight_masks.push(movecalc.calculate_knight_moves().unwrap());
        }
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("knight_masks.json", &Array64{values: self.knight_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }
    pub fn create_king_table(&mut self) -> Result<(), std::io::Error> {
        // This function will use as index the king piece index,
        // since for each king position there will exactly one way the
        // king is able to move
        self.king_masks = Vec::new();
        for i in 0..64 {
            let mut movecalc = MoveCalculator::new(0, i as u8);
            self.king_masks.push(movecalc.calculate_king_moves().unwrap());
        }
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("king_masks.json", &Array64{values: self.king_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }
    pub fn create_white_pawn_table(&mut self) -> Result<(), std::io::Error> {
        self.white_pawn_masks = Vec::new();
        for i in 0..64 {
            let mut movecalc = MoveCalculator::new(0, i as u8);
            self.white_pawn_masks.push(movecalc.calculate_white_pawn_moves().unwrap());
        }
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("white_pawn_masks.json", &Array64{values: self.white_pawn_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }
    pub fn create_black_pawn_table(&mut self) -> Result<(), std::io::Error> {
        self.black_pawn_masks = Vec::new();
        for i in 0..64 {
            let mut movecalc = MoveCalculator::new(0, i as u8);
            self.black_pawn_masks.push(movecalc.calculate_black_pawn_moves().unwrap());
        }
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("black_pawn_masks.json", &Array64{values: self.black_pawn_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }

    pub fn create_direction_table(&mut self) -> Result<(), std::io::Error>{
        self.direction_masks = [[0; 8]; 64];
        for i in 0..64 {
            let (rank, file) = index2rank_file(i).unwrap();
            // get all directions
            let up: Vec<u8> = ((rank+1)..9).map(|n|rank_file2index(n, file).unwrap()).collect();
            let below: Vec<u8> = (1..(rank)).map(|n|rank_file2index(rank-n, file).unwrap()).collect();
            let right: Vec<u8>  = ((file+1)..9).map(|n|rank_file2index(rank, n).unwrap()).collect();
            let left: Vec<u8> = (1..(file)).map(|n|rank_file2index(rank, file-n).unwrap()).collect();
            let below_right: Vec<u8> = (1..cmp::min(9-rank, 9-file)).map(|n|rank_file2index(rank+n,file+n).unwrap()).collect();
            let below_left: Vec<u8> = (1..cmp::min(9-rank, file)).map(|n|rank_file2index(rank+n,file-n).unwrap()).collect();
            let up_right: Vec<u8> = (1..cmp::min(rank, 9-file)).map(|n|rank_file2index(rank-n,file+n).unwrap()).collect();
            let up_left: Vec<u8> = (1..cmp::min(rank, file)).map(|n|rank_file2index(rank-n, file-n).unwrap()).collect();
            
            let directions = [up, up_right, right, below_right, below, below_left, left, up_left];
            for (direction_index, direction) in directions.iter().enumerate() {
                let mut bb: u64 = 0;
                for square in direction {
                    bb = set_bit(bb, *square);
                }
                // save the direction in the correct index
                self.direction_masks[i as usize][direction_index] = bb;
            }
        }
        CreateLookUpTables::setup_directory("lookuptables");
        CreateLookUpTables::write2file("direction_lookup.json", &DirectionTable{values: self.direction_masks.clone()})?;
        std::env::set_current_dir("../")?;
        Ok(())
    }

    pub fn create_all(&mut self, search_number: u32) -> Result<(), std::io::Error> {
        self.create_slide_piece_table(search_number, SlidePieceType::Rook)?;
        self.create_slide_piece_table(search_number, SlidePieceType::Bishop)?;
        self.create_knight_table()?;
        self.create_king_table()?;
        self.create_white_pawn_table()?;
        self.create_black_pawn_table()?;
        self.create_direction_table()
    }
}



pub struct LoadMoves {
    knight_masks: [u64; 64],
    white_pawn_masks: [u64; 64],
    black_pawn_masks: [u64; 64],
    king_masks: [u64; 64],
    bishop_magic_lookup: MagicLookUp,
    rook_magic_lookup: MagicLookUp,
    // masks for where possible blockers can be for a given piece location. 
    // Is basically the moves for an empty board while ignoring the outer ranks and files
    rook_pre_masks: [u64; 64],
    bishop_pre_masks: [u64; 64],
    direction_masks: [[u64; 8]; 64]
}

impl LoadMoves {
    pub fn new() -> LoadMoves {
        LoadMoves { knight_masks: LoadMoves::parse_non_slide_masks("knight_masks.json").expect("No knight table found"), 
            white_pawn_masks: LoadMoves::parse_non_slide_masks("white_pawn_masks.json").expect("No white pawn table found"), 
            black_pawn_masks: LoadMoves::parse_non_slide_masks("black_pawn_masks.json").expect("No black pawn table found"), 
            king_masks: LoadMoves::parse_non_slide_masks("king_masks.json").expect("No king table found"), 
            bishop_magic_lookup: LoadMoves::parse_slide_masks("bishop_magic_lookup.json").expect("No bishop magic table found"), 
            rook_magic_lookup: LoadMoves::parse_slide_masks("rook_magic_lookup.json").expect("No rook magic table found"), 
            rook_pre_masks: LoadMoves::parse_non_slide_masks("rook_pre_masks.json").expect("No rook pre masks table found"), 
            bishop_pre_masks: LoadMoves::parse_non_slide_masks("bishop_pre_masks.json").expect("No bishop pre masks table found"),
        direction_masks: LoadMoves::parse_direction_masks("direction_lookup.json").expect("No direction masks table found") }
    }
    #[inline]
    pub fn rook(&self, piece_index: usize, blockers: u64) -> Option<&u64> {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        // blockers is the bitboard with bits on enemy and friendly pieces combined, note that 
        // this function will return a bitboard where friendly piece can be captured.
        let viewed_blockers = self.rook_pre_masks[piece_index] & blockers;
        let i = viewed_blockers.wrapping_mul(self.rook_magic_lookup.magic_numbers[piece_index]) >> self.rook_magic_lookup.shifts[piece_index];
        self.rook_magic_lookup.magic_masks[piece_index].get(i as usize)
    }
    #[inline]
    pub fn bishop(&self, piece_index: usize, blockers: u64) -> Option<&u64> {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        // blockers is the bitboard with bits on enemy and friendly pieces combined, note that 
        // this function will return a bitboard where friendly piece can be captured.
        let viewed_blockers = self.bishop_pre_masks[piece_index] & blockers;
        let i = viewed_blockers.wrapping_mul(self.bishop_magic_lookup.magic_numbers[piece_index]) >> self.bishop_magic_lookup.shifts[piece_index];
        self.bishop_magic_lookup.magic_masks[piece_index].get(i as usize)
    }
    #[inline]
    pub fn queen(&self, piece_index: usize, blockers: u64) -> Option<u64> {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        // blockers is the bitboard with bits on enemy and friendly pieces combined, note that 
        // this function will return a bitboard where friendly piece can be captured.
        Some(self.rook(piece_index, blockers)? | self.bishop(piece_index, blockers)?)
    }
    #[inline]
    pub fn king(&self, piece_index: usize) -> u64 {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        self.king_masks[piece_index]
    }
    #[inline]
    pub fn knight(&self, piece_index: usize) -> u64 {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        self.knight_masks[piece_index]
    }
    #[inline]
    pub fn white_pawn(&self, piece_index: usize) -> u64 {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        self.white_pawn_masks[piece_index]
    }
    #[inline]
    pub fn black_pawn(&self, piece_index: usize) -> u64 {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        self.black_pawn_masks[piece_index]
    }
    #[inline]
    pub fn direction_ray(&self, piece_index: usize, direction: usize) -> u64 {
        // piece_index must be a number between 0 and 63, not a bitboard with one bit!
        // directions is index as [north, northeast, east, southeast, south, southwest, west, northwest]
        self.direction_masks[piece_index][direction]
    }

    fn parse_non_slide_masks(filepath: &str) -> Result<[u64; 64], std::io::Error>{
        CreateLookUpTables::setup_directory("lookuptables");
        // Before propagating the error we first go back to the home directory.
        let f = match fs::File::open(filepath) {
            Ok(res) => res,
            Err(err) => {std::env::set_current_dir("../")?; return Err(err)}
        };
        let reader = BufReader::new(f);
        let u: Array64 = serde_json::from_reader(reader).expect("File that is being read has no struct with Array64.");
        let result: [u64; 64] = u.values.try_into().expect("Could not convert vec to array while parsing json.");
        std::env::set_current_dir("../")?;
        Ok(result)
    }

    fn parse_slide_masks(filepath: &str) -> Result<MagicLookUp, std::io::Error>{
        CreateLookUpTables::setup_directory("lookuptables");
        // Before propagating the error we first go back to the home directory.
        let f = match fs::File::open(filepath) {
            Ok(res) => res,
            Err(err) => {std::env::set_current_dir("../")?; return Err(err)}
        };
        let reader = BufReader::new(f);
        let u: MagicLookUp = serde_json::from_reader(reader).expect("File that is being read has no struct with MagicLookUp.");
        // Return to the home directory
        std::env::set_current_dir("../")?;
        Ok(u)
    }

    fn parse_direction_masks(filepath: &str) -> Result<[[u64; 8]; 64], std::io::Error> {
        CreateLookUpTables::setup_directory("lookuptables");
        // Before propagating the error we first go back to the home directory.
        let f = match fs::File::open(filepath) {
            Ok(res) => res,
            Err(err) => {std::env::set_current_dir("../")?; return Err(err)}
        };
        let reader = BufReader::new(f);
        let u: DirectionTable = serde_json::from_reader(reader).expect("File that is being read has no struct with DirectionTable.");
        let result: [[u64; 8]; 64] = u.values.try_into().expect("Could not convert vec to array while parsing json.");
        std::env::set_current_dir("../")?;
        Ok(result)
    }
}
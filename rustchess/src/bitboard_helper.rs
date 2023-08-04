// Rank and File will be indexed from the white pov. and the bitboard index
// as follows:
//   R    8   0  1  2  3  4  5  6  7  
//   A    7   8  9  10 11 12 13 14 15 
//   N    6   16 17 18 19 20 21 22 23
//   K    5   24 25 26 27 28 29 30 31
//        4   32 33 34 35 36 37 38 39
//        3   40 41 42 43 44 45 46 47 
//        2   48 49 50 51 52 53 54 55 
//        1   56 57 58 59 60 61 62 63 
//
//            1  2  3  4  5  6  7  8
//               F I L E
//

#[derive(Debug)]
pub enum BitBoardError {
    IndexOutOfBoard,
    FileOutOfBoard,
    RankOutOfBoard
}

pub fn to_string(bb: u64) -> String {
    (0..64).map(|n| ((bb >> n) & 1).to_string()).collect::<Vec<String>>().join("")
}
pub fn to_stringboard(bb: u64) -> String {
    (0..64).map(|n| ((bb >> n) & 1).
    to_string() + {if (n+1) % 8 == 0 {"\n"} else {""}}).
    collect::<Vec<String>>().join("")
}

pub fn subtract_bb(bb: u64, subtract: u64) -> u64 {
    (bb & subtract) ^ bb
}

pub fn get_lsb(bb: u64) -> u64 {
    // gets the Least Significant bit
    let bb = bb as i64;
    (bb & -bb) as u64
}

pub fn rank_file2index(rank: u8, file:u8) -> Result<u8, BitBoardError> {
    if rank == 0 || rank > 8 {
        Err(BitBoardError::RankOutOfBoard)
    }
    else if file == 0 || file > 8 {
        Err(BitBoardError::FileOutOfBoard)
    }
    else {
        Ok(((8-rank)*8) + file - 1)
    }
}
pub fn index2rank_file(i: u8) -> Result<(u8, u8), BitBoardError> {
    if i > 63 {Err(BitBoardError::IndexOutOfBoard)} else {Ok((8-(i/8), (i%8)+1))}
}
pub fn set_bit(bb: u64, i: u8) -> u64 {
    bb | (1 << i)
}
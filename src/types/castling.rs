//! Logic and constants for castling rights management.

use crate::types::bitboard::Bitboard;
use crate::types::color::Color;
use crate::types::square::Square;

/// Returns true if `side` has castling rights and the path is not blocked. Does not check
/// if castling squares are attacked.
pub const fn can_castle(occupancy: Bitboard, current_rights: u8, side: Color, kingside: bool) -> bool {
    has_castling_rights(current_rights, side, kingside) && is_path_clear(occupancy, side, kingside)
}

/// Pre-calculated masks to update castling rights via bitwise AND.
/// 
/// How it works:
/// When a piece moves from 'sq1' to 'sq2', we do: `rights &= MASK[sq1] & MASK[sq2]`.
/// 
/// Values:
/// - 0xFF: No effect on castling rights.
/// - 0xFC: !(WHITE_OO | WHITE_OOO) -> White King moved, loses all white rights.
/// - 0xFD: !(WHITE_OOO)            -> White A-Rook moved/captured.
/// - 0xFE: !(WHITE_OO)             -> White H-Rook moved/captured.
/// - 0xF3: !(BLACK_OO | BLACK_OOO) -> Black King moved, loses all black rights.
/// - 0xF7: !(BLACK_OOO)            -> Black A-Rook moved/captured.
/// - 0xFB: !(BLACK_OO)             -> Black H-Rook moved/captured.
#[rustfmt::skip]
pub const RIGHTS_UPDATE_MASK: [u8; 64] = [
    0xFD, 0xFF, 0xFF, 0xFF, 0xFC, 0xFF, 0xFF, 0xFE, // A1, ..., E1, ..., H1
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xF7, 0xFF, 0xFF, 0xFF, 0xF3, 0xFF, 0xFF, 0xFB, // A8, ..., E8, ..., H8
];

const RIGHTS_WHITE_OO: u8 = 0b0001; // Kingside
const RIGHTS_WHITE_OOO: u8 = 0b0010; // Queenside
const RIGHTS_BLACK_OO: u8 = 0b0100;
const RIGHTS_BLACK_OOO: u8 = 0b1000;

/// Checks if the specific castling bit is set in the rights byte.
#[inline(always)]
const fn has_castling_rights(current_rights: u8, side: Color, kingside: bool) -> bool {
    let mask = match (side, kingside) {
        (Color::White, true) => RIGHTS_WHITE_OO,
        (Color::White, false) => RIGHTS_WHITE_OOO,
        (Color::Black, true) => RIGHTS_BLACK_OO,
        (Color::Black, false) => RIGHTS_BLACK_OOO,
    };
    (current_rights & mask) != 0
}

/// Squares F1 and G1.
const PATH_WHITE_OO: Bitboard = Bitboard(Square::F1.as_u64() | Square::G1.as_u64());

/// Squares B1, C1, and D1.
const PATH_WHITE_OOO: Bitboard = Bitboard(Square::B1.as_u64() | Square::C1.as_u64() | Square::D1.as_u64());

/// Squares F8 and G8.
const PATH_BLACK_OO: Bitboard = Bitboard(Square::F8.as_u64() | Square::G8.as_u64());

/// Squares B8, C8, and D8.
const PATH_BLACK_OOO: Bitboard = Bitboard(Square::B8.as_u64() | Square::C8.as_u64() | Square::D8.as_u64());

/// Checks if the path between king and rook is clear of pieces.
#[inline(always)]
const fn is_path_clear(occupancy: Bitboard, side: Color, kingside: bool) -> bool {
    let path = match (side, kingside) {
        (Color::White, true) => PATH_WHITE_OO,
        (Color::White, false) => PATH_WHITE_OOO,
        (Color::Black, true) => PATH_BLACK_OO,
        (Color::Black, false) => PATH_BLACK_OOO,
    };
    (occupancy.0 & path.0) == 0
}

//! Logic and constants for castling rights management.

use crate::types::bitboard::Bitboard;
use crate::types::color::Color;
use crate::types::square::Square;

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
const RIGHTS_UPDATE_MASK: [u8; 64] = [
    0xFD, 0xFF, 0xFF, 0xFF, 0xFC, 0xFF, 0xFF, 0xFE, // A1, ..., E1, ..., H1
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xF7, 0xFF, 0xFF, 0xFF, 0xF3, 0xFF, 0xFF, 0xFB, // A8, ..., E8, ..., H8
];

pub const RIGHTS_WHITE_OO: u8 = 0b0001;
pub const RIGHTS_WHITE_OOO: u8 = 0b0010;
pub const RIGHTS_BLACK_OO: u8 = 0b0100;
pub const RIGHTS_BLACK_OOO: u8 = 0b1000;

/// Squares F1 and G1.
const PATH_WHITE_OO: Bitboard = Square::F1.bb().or(Square::G1.bb());

/// Squares B1, C1, and D1.
const PATH_WHITE_OOO: Bitboard = Square::B1.bb().or(Square::C1.bb()).or(Square::D1.bb());

/// Squares F8 and G8.
const PATH_BLACK_OO: Bitboard = Square::F8.bb().or(Square::G8.bb());

/// Squares B8, C8, and D8.
const PATH_BLACK_OOO: Bitboard = Square::B8.bb().or(Square::C8.bb()).or(Square::D8.bb());

#[repr(transparent)] // Treat the struct exactly like a u8
#[derive(Copy, Clone)]
pub struct CastlingRights {
    encoding: u8,
}

impl CastlingRights {
    /// Returns a CastlingRights objects with all rights active.
    pub fn new() -> CastlingRights {
        Self { encoding: 0b1111 }
    }

    // Returns the internal u8 encoding.
    #[inline(always)]
    pub fn encoding(&self) -> u8 {
        self.encoding
    }

    /// Removes all castling rights.
    #[inline(always)]
    pub fn zero(&mut self) {
        self.encoding = 0;
    }

    /// Returns true if `side` has castling rights and the path is not blocked. Does not check
    /// if castling squares are attacked.
    #[inline(always)]
    pub fn can_castle(&self, occupancy: Bitboard, side: Color, kingside: bool) -> bool {
        self.has_castling_rights(side, kingside) && Self::is_path_clear(occupancy, side, kingside)
    }

    /// Given a move, updates castling rights accordingly.
    #[inline(always)]
    pub fn update_rights(&mut self, from: Square, to: Square) {
        self.encoding &= RIGHTS_UPDATE_MASK[from] & RIGHTS_UPDATE_MASK[to];
    }

    #[inline(always)]
    pub fn add_white_oo(&mut self) {
        self.encoding |= RIGHTS_WHITE_OO;
    }

    #[inline(always)]
    pub fn add_white_ooo(&mut self) {
        self.encoding |= RIGHTS_WHITE_OOO;
    }

    #[inline(always)]
    pub fn add_black_oo(&mut self) {
        self.encoding |= RIGHTS_BLACK_OO;
    }

    #[inline(always)]
    pub fn add_black_ooo(&mut self) {
        self.encoding |= RIGHTS_BLACK_OOO;
    }

    /// Checks if the specific castling bit is set in the rights byte.
    #[inline(always)]
    const fn has_castling_rights(&self, side: Color, kingside: bool) -> bool {
        let mask = match (side, kingside) {
            (Color::White, true) => RIGHTS_WHITE_OO,
            (Color::White, false) => RIGHTS_WHITE_OOO,
            (Color::Black, true) => RIGHTS_BLACK_OO,
            (Color::Black, false) => RIGHTS_BLACK_OOO,
        };
        (self.encoding & mask) != 0
    }

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
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self { encoding: 0 }
    }
}

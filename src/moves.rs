//! Low-level move encoding.
//!
//! This module contains the logic to encode a move in 16 bits.

use crate::types::PieceType;

/// 16-bit encoded move.
/// 0-5: from square (0 to 63)
/// 6-11: to square (0 to 63)
/// 12-15: flags
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move {
    encoding: u16,
}

/// Move type encoding as found in https://www.chessprogramming.org/Encoding_Moves#From-To_Based.
/// These are only the most-significant bits in the 16-bit move encoding.
#[repr(u8)]
#[rustfmt::skip]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MoveType {
    //Quiet
    Normal            = 0b0000,
    DoublePush        = 0b0001,
    KingCastle        = 0b0010,
    QueenCastle       = 0b0011,

    //Noisy
    Capture           = 0b0100,
    EnPassant         = 0b0101,
    PromotionN        = 0b1000,
    PromotionB        = 0b1001, 
    PromotionR        = 0b1010,
    PromotionQ        = 0b1011,
    PromotionCaptureN = 0b1100,
    PromotionCaptureB = 0b1101,
    PromotionCaptureR = 0b1110,
    PromotionCaptureQ = 0b1111,
}

impl Move {
    /// Null-move definition required for null-move pruning.
    pub const NULL_MOVE: Move = Move { encoding: 0 };

    /// Encodes a "normal" move.
    pub const fn new_normal(from_square: usize, to_square: usize) -> Self {
        Self { encoding: (from_square as u16) | ((to_square as u16) << 6) }
    }

    /// Encodes a "special" move (double push, castling, capture, en passant, promotion).
    pub const fn new_special(from_square: usize, to_square: usize, movetype: MoveType) -> Self {
        Self {
            encoding: (from_square as u16 | ((to_square as u16) << 6) | ((movetype as u16) << 12)),
        }
    }

    /// Returns the origin square.
    #[inline(always)]
    pub const fn from_square(self) -> u8 {
        (self.encoding & 0x3F) as u8
    }

    /// Returns the destination square.
    #[inline(always)]
    pub const fn to_square(self) -> u8 {
        ((self.encoding >> 6) & 0x3F) as u8
    }

    /// Checks whether the move is a capture.
    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self.encoding & 0x4000) != 0
    }

    /// Checks whether the move is a promotion.
    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.encoding & 0x8000) != 0
    }

    /// Makes the enum self-aware, returns the move type.
    #[inline(always)]
    pub const fn get_type(self) -> MoveType {
        unsafe { core::mem::transmute((self.encoding >> 12) as u8) }
    }

    /// Checks whether the move is quiet.
    #[inline(always)]
    pub const fn is_quiet(self) -> bool {
        (self.encoding & 0xF000) == 0
    }

    /// Checks whether the move is noisy.
    #[inline(always)]
    pub const fn is_noisy(self) -> bool {
        (self.encoding & 0xC000) != 0
    }

    /// Checks whether the move is castling.
    #[inline(always)]
    pub const fn is_castling(self) -> bool {
        (self.encoding & 0xE000) == 0x2000
    }

    /// Checks whether the move is en-passant.
    #[inline(always)]
    pub const fn is_enpassant(self) -> bool {
        (self.encoding & 0xF000) == 0x5000
    }

    /// Checks whether the move is a double pawn push.
    #[inline(always)]
    pub const fn is_double_push(self) -> bool {
        (self.encoding & 0xF000) == 0x1000
    }

    /// Returns the piece you get after a pawn promotion.
    #[inline(always)]
    pub const fn get_promotion_piece(self) -> PieceType {
        debug_assert!(self.is_promotion());
        match (self.encoding >> 12) & 0b11 {
            0 => PieceType::Knight,
            1 => PieceType::Bishop,
            2 => PieceType::Rook,
            3 => PieceType::Queen,
            _ => unreachable!(),
        }
    }
}

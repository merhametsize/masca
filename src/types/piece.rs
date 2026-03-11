//! Definitions for chess pieces and piece kinds.
//! Contains the `Piece` (color + kind) and `PieceKind` (pawn, knight, etc.) enums used in board representation.

use crate::types::color::Color;

use std::ops::{Index, IndexMut};

/// Identifies the type of a piece regardless of its color (e.g., Knight, Rook).
/// Used for indexing bitboards and piece-square tables.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PieceKind {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    /// Total number of piece kinds, used for array sizing (e.g., bitboards).
    pub const NUM: usize = 6;

    /// An array containing all piece kinds for easy iteration.
    pub const ALL: [PieceKind; 6] =
        [PieceKind::Pawn, PieceKind::Knight, PieceKind::Bishop, PieceKind::Rook, PieceKind::Queen, PieceKind::King];

    /// Creates a PieceKind from a raw integer.
    /// # Safety: Caller must ensure encoding is within [0, 5].
    pub unsafe fn new(encoding: u8) -> PieceKind {
        debug_assert!(encoding < Self::NUM as u8);
        unsafe { core::mem::transmute(encoding) }
    }
}

impl<T> Index<PieceKind> for [T] {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: PieceKind) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}
impl<T> IndexMut<PieceKind> for [T] {
    #[inline(always)]
    fn index_mut(&mut self, index: PieceKind) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

/// Returns the value of a piece for MVV-LVA. Should be optimized by the compiler.
#[inline(always)]
pub fn piece_value(piece_type: PieceKind) -> i32 {
    match piece_type {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0, // Dummy
    }
}

/// A specific piece on the board, combining a `Color` and a `PieceKind`.
/// Encodes both identity and side for mailbox representation and move generation.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Piece {
    //White
    WhitePawn = 0,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    //Black
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Piece {
    /// Builds a Piece from a Color and a PieceType.
    #[inline(always)]
    pub const fn new(color: Color, piece_type: PieceKind) -> Self {
        let encoding = (color as u8) * 6 + (piece_type as u8);
        debug_assert!(encoding < 12);
        unsafe { core::mem::transmute(encoding) }
    }

    /// Returns the color of the piece.
    #[inline(always)]
    pub const fn color(self) -> Color {
        let color_index = (self as u8) / 6;
        debug_assert!(color_index <= 1);
        unsafe { core::mem::transmute(color_index) }
    }

    /// Makes the enum self-aware, returns the piece-type.
    #[inline(always)]
    pub const fn kind(self) -> PieceKind {
        match (self as u8) % 6 {
            0 => PieceKind::Pawn,
            1 => PieceKind::Knight,
            2 => PieceKind::Bishop,
            3 => PieceKind::Rook,
            4 => PieceKind::Queen,
            5 => PieceKind::King,
            _ => unreachable!(), // optional safety
        }
    }

    /// Converts Piece to a character.
    #[rustfmt::skip]
    pub const fn to_char(self) -> char {
        match self {
            Piece::WhitePawn   => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook   => 'R',
            Piece::WhiteQueen  => 'Q',
            Piece::WhiteKing   => 'K',
            Piece::BlackPawn   => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook   => 'r',
            Piece::BlackQueen  => 'q',
            Piece::BlackKing   => 'k',
        }
    }

    /// Creates Piece from a character.
    #[rustfmt::skip]
    pub const fn from_char(ch: char) -> Self {
        match ch {
            'P' => Piece::WhitePawn,
            'N' => Piece::WhiteKnight,
            'B' => Piece::WhiteBishop,
            'R' => Piece::WhiteRook,
            'Q' => Piece::WhiteQueen,
            'K' => Piece::WhiteKing,
            'p' => Piece::BlackPawn,
            'n' => Piece::BlackKnight,
            'b' => Piece::BlackBishop,
            'r' => Piece::BlackRook,
            'q' => Piece::BlackQueen,
            'k' => Piece::BlackKing,
            _   => unreachable!(),
        }
    }
}

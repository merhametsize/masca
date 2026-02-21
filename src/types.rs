use std::ops::{Index, IndexMut, Not};

use crate::bitboard::Bitboard;

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

/// Allows for array indexing without explicit conversion of Color to usize.
/// Example: `array[Color::White]`
impl<T> Index<Color> for [T] {
    type Output = T;
    fn index(&self, index: Color) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Color> for [T] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[rustfmt::skip]
    pub const ALL: [Square; 64] = [
        Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
        Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
        Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
        Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
        Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
        Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
        Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
        Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
    ];

    pub const fn new(index: u8) -> Self {
        debug_assert!(index < 64);
        unsafe { std::mem::transmute(index) }
    }

    /// Returns the rank index of the square
    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self as u8 >> 3 // Same as /8
    }

    /// Returns the file index of the square
    #[inline(always)]
    pub const fn file(self) -> u8 {
        self as u8 & 0b0000_0111 // Same as %8
    }

    #[inline(always)]
    pub const fn north(self) -> Square {
        let s = self as u8;
        debug_assert!(s < 56); // not on rank 8
        unsafe { std::mem::transmute(s + 8) }
    }

    #[inline(always)]
    pub const fn south(self) -> Square {
        let s = self as u8;
        debug_assert!(s >= 8); // not on rank 1
        unsafe { std::mem::transmute(s - 8) }
    }

    #[inline(always)]
    pub const fn north_east(self) -> Square {
        let s = self as u8;
        debug_assert!(s < 56 && (s & 7) != 7); // not rank 8, not file H
        unsafe { std::mem::transmute(s + 9) }
    }

    #[inline(always)]
    pub const fn north_west(self) -> Square {
        let s = self as u8;
        debug_assert!(s < 56 && (s & 7) != 0); // not rank 8, not file A
        unsafe { std::mem::transmute(s + 7) }
    }

    #[inline(always)]
    pub const fn south_east(self) -> Square {
        let s = self as u8;
        debug_assert!(s >= 8 && (s & 7) != 7); // not rank 1, not file H
        unsafe { std::mem::transmute(s - 7) }
    }

    #[inline(always)]
    pub const fn south_west(self) -> Square {
        let s = self as u8;
        debug_assert!(s >= 8 && (s & 7) != 0); // not rank 1, not file A
        unsafe { std::mem::transmute(s - 9) }
    }

    /// Turns the square into a bitboard
    #[inline(always)]
    pub fn bb(self) -> Bitboard {
        Bitboard(1u64 << (self as u8))
    }
}

/// Allows for array indexing without explicit conversion of Color to usize.
/// Example: `array[Color::White]`
impl<T> Index<Square> for [T] {
    type Output = T;
    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Square> for [T] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PieceType {
    Pawn = 0,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub const NUM: usize = 6;

    pub fn new(encoding: u8) -> PieceType {
        debug_assert!(encoding < Self::NUM as u8);
        unsafe { core::mem::transmute(encoding) }
    }
}

/// Allows for array indexing without explicit conversion of Color to usize.
/// Example: `array[PieceType::YaasssQueeeeeen]`
impl<T> Index<PieceType> for [T] {
    type Output = T;
    fn index(&self, index: PieceType) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}
impl<T> IndexMut<PieceType> for [T] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

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
    pub const fn new(color: Color, piece_type: PieceType) -> Self {
        let encoding = (color as u8) * 6 + (piece_type as u8);
        debug_assert!(encoding < 12);
        unsafe { core::mem::transmute(encoding) }
    }

    /// Returns the color of the piece.
    #[inline(always)]
    pub const fn get_color(self) -> Color {
        let color_index = (self as u8) / 6;
        debug_assert!(color_index <= 1);
        unsafe { core::mem::transmute(color_index) }
    }

    /// Makes the enum self-aware, returns the piece-type.
    #[inline(always)]
    pub const fn get_type(self) -> PieceType {
        match (self as u8) % 6 {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            5 => PieceType::King,
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

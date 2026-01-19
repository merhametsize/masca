use std::ops::{Index, IndexMut};

pub const NULL_SQUARE: u8 = 64; //For en-passant

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
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

#[repr(u8)]
#[derive(Copy, Clone)]
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
}

/// Allows for array indexing without explicit conversion of Color to usize.
/// Example: `array[Color::White]`
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
#[derive(Copy, Clone)]
pub enum Piece {
    WhitePawn = 0,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Piece {
    pub const fn get_color(self) -> Color {
        if (self as u8) & 1 == 0 { Color::White } else { Color::Black }
    }

    /// Makes the enum self-aware, returns the piece-type.
    pub const fn get_type(self) -> PieceType {
        match (self as u8) >> 1 {
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

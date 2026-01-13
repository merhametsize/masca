#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
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

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

    None, //No use of Option<Piece>
}

#[rustfmt::skip]
pub const PIECE_2_CHAR: &[(Piece, char)] = &[
    (Piece::WhitePawn,   'P'),
    (Piece::WhiteKnight, 'N'),
    (Piece::WhiteBishop, 'B'),
    (Piece::WhiteRook,   'R'),
    (Piece::WhiteQueen,  'Q'),
    (Piece::WhiteKing,   'K'),
    (Piece::BlackPawn,   'p'),
    (Piece::BlackKnight, 'n'),
    (Piece::BlackBishop, 'b'),
    (Piece::BlackRook,   'r'),
    (Piece::BlackQueen,  'q'),
    (Piece::BlackKing,   'k'),
];

pub const NULL_SQUARE: u8 = 64; //For en-passant

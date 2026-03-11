//! Defines the fundamental types used in the Masca chess engine.
//! This module groups core chess objects like colors, squares and pieces.
pub mod bitboard;
pub mod castling;
pub mod color;
pub mod moves;
pub mod piece;
pub mod square;

pub use bitboard::Bitboard;
pub use color::Color;
pub use moves::{Move, MoveKind};
pub use piece::{Piece, PieceKind, piece_value};
pub use square::Square;

pub const MAX_PLY: usize = 128;

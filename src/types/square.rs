//! Square representation and coordinate logic for the chessboard.
//! Defines the `Square` enum (A1 to H8) and methods for bitboard conversion, rank/file extraction, and mirroring.

use crate::types::bitboard::Bitboard;

use std::fmt;
use std::ops::{Index, IndexMut};

/// Represents one of the 64 squares on a chessboard, indexed from A1 (0) to H8 (63).
/// Includes utilities for coordinate conversion, rank/file extraction, and bitboard mapping.
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

    /// Creates a square from a raw index (0-63).
    ///
    /// # Safety
    /// The caller must ensure that `index` is less than 64.
    pub const fn new(index: u8) -> Self {
        debug_assert!(index < 64);
        unsafe { std::mem::transmute(index) }
    }

    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self as u8 >> 3 // Same as /8
    }

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

    /// Returns the bitboard representation of the square.
    #[inline(always)]
    pub const fn bb(self) -> Bitboard {
        Bitboard(1u64 << (self as u8))
    }

    #[inline(always)]
    pub const fn mirror_rank(self) -> Square {
        unsafe { std::mem::transmute((self as u8) ^ 56) }
    }

    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self as u64
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = *self as u8;

        let file = (idx % 8) as u8;
        let rank = (idx / 8) as u8;

        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;

        write!(f, "{}{}", file_char, rank_char)
    }
}

impl<T> Index<Square> for [T] {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: Square) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Square> for [T] {
    #[inline(always)]
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

//! Bitboard object definition
//!
//! This module contains the definition of the Bitboard object, and the overloading of bitwise operators.
//! Using a Bitboard object makes for more type-safety as it cannot be used with other u64 variables of
//! the engine, such as masks, squares, attacks and more.

use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// Bitboard object defined as a struct with unnamed u64 field.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub fn from_square(sq: usize) -> Self {
        Self(1u64 << sq)
    }

    /// Returns rank 1 as a bitboard
    #[inline(always)]
    pub fn rank_1() -> Self {
        Self(0x0000_0000_0000_00FFu64)
    }

    /// Returns rank 8 as a bitboard
    #[inline(always)]
    pub fn rank_8() -> Self {
        Self(0xFF00_0000_0000_0000u64)
    }

    /// Returns file A as a bitboard
    #[inline(always)]
    pub fn file_A() -> Self {
        Self(0x0101_0101_0101_0101u64)
    }

    /// Returns file H as a bitboard
    #[inline(always)]
    pub fn file_H() -> Self {
        Self(0x8080_8080_8080_8080u64)
    }

    /// Returns the square's corresponding rank as a bitboard
    #[inline(always)]
    pub fn square_to_rank(sq: usize) -> Self {
        let rank_index = sq / 8;
        Self(0x0000_0000_0000_00FFu64 << (rank_index * 8))
    }

    /// Returns the square's corresponding file as a bitboard
    #[inline(always)]
    pub fn square_to_file(sq: usize) -> Self {
        let file_index = sq % 8;
        Self(0x0101_0101_0101_0101u64 << file_index)
    }

    /// Returns the LSB from the bitboard
    #[inline(always)]
    pub fn lsb(&mut self) -> usize {
        self.0.trailing_zeros() as usize
    }

    /// Pops the LSB from the bitboard, in-place
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> usize {
        let lsb = self.0.trailing_zeros() as usize;
        self.0 &= self.0 - 1;
        lsb
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                let bit = (self.0 >> sq) & 1;
                write!(f, "{} ", if bit == 1 { '1' } else { '.' })?;
            }
            writeln!(f)?;
        }
        writeln!(f)
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0u64)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

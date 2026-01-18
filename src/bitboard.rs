use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, Not};

/// Bitboard object overloading logical operators.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub fn from_square(sq: usize) -> Self {
        Bitboard(1u64 << sq)
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

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

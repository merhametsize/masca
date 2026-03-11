//! Representation of the two players in a chess game.
//! Provides the `Color` enum and utilities for switching sides and indexing arrays by player.

use std::ops::{Index, IndexMut, Not};

/// Represents the side to move or the color of a piece.
/// Provides logical operations like `!color` for switching sides and array indexing support.
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
    #[inline(always)]
    fn index(&self, index: Color) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Color> for [T] {
    #[inline(always)]
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl Not for Color {
    type Output = Self;

    /// Returns the opposite color.
    #[inline(always)]
    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

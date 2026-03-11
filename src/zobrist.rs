// zobrist.rs
//! Zobrist hashing for fast incremental board hash updates.
//!
//! Generates 64-bit random keys for all pieces, castling rights, en-passant files, and side to move.
//! Provides a `Zobrist` struct holding the current key.

#![allow(dead_code)]

use crate::types::{Color, PieceKind, Square};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

/// Number of castling rights (16 possibilities)
pub const NUM_CASTLING: usize = 16;

/// Number of en-passant files
pub const NUM_EP_FILES: usize = 8;

/// Zobrist tables
pub struct ZobristTables {
    pub pieces: [[[u64; 64]; 2]; PieceKind::NUM], // [piece][color][square]
    pub castling: [u64; NUM_CASTLING],            // 16 castling rights
    pub en_passant: [u64; NUM_EP_FILES],          // en-passant files
    pub side: u64,                                // side to move
}

impl ZobristTables {
    /// Generates random keys with a fixed seed for reproducibility.
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(0xd10caaaa); // 🥚
        let mut pieces = [[[0u64; 64]; 2]; PieceKind::NUM];
        let mut castling = [0u64; NUM_CASTLING];
        let mut en_passant = [0u64; NUM_EP_FILES];
        let side;

        // Fill pieces table
        for piece_kind in PieceKind::ALL {
            for color in [Color::Black, Color::White] {
                for sq in Square::ALL {
                    pieces[piece_kind][color][sq] = rng.next_u64();
                }
            }
        }

        // Castling
        for i in 0..NUM_CASTLING {
            castling[i] = rng.next_u64();
        }

        // En-passant
        for i in 0..NUM_EP_FILES {
            en_passant[i] = rng.next_u64();
        }

        // Side
        side = rng.next_u64();

        Self { pieces, castling, en_passant, side }
    }
}

/// Current board hash.
#[derive(Copy, Clone, Default)]
pub struct Zobrist {
    pub key: u64,
}

impl Zobrist {
    /// Creates an empty hash.
    pub fn new() -> Self {
        Self { key: 0 }
    }

    /// XOR a value into the hash.
    #[inline(always)]
    pub fn xor(&mut self, val: u64) {
        self.key ^= val;
    }
}

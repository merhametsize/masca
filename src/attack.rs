//! Attack table generation.
//!
//! This module generates attack look-up tables for knights, kings and pawns.
//! Tables provide possible moves for a given piece type and square, queried via `[square]` or `[color][square]`.
//!
//! Sliding pieces are handled differently (in magics.rs) because of board occupancy.

use crate::bitboard::Bitboard;
use crate::types::Color;

use std::io::Write;
use std::sync::OnceLock;

/// Contains the attack look-up tables per piece.
pub struct AttackTables {
    pub knight: [Bitboard; 64],
    pub king: [Bitboard; 64],
    pub pawn_capture: [[Bitboard; 64]; 2],
    pub pawn_push: [[Bitboard; 64]; 2],
    pub pawn_double_push: [[Bitboard; 64]; 2],
}

const KNIGHT_DELTAS: [(i8, i8); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
const KING_DELTAS: [(i8, i8); 8] = [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)];

//// Global attack tables, initialized once at startup.
///
/// This `OnceLock` holds the precomputed `AttackTables` and ensures
/// thread-safe, one-time initialization. Access the tables with:
pub static ATTACK_TABLES: OnceLock<AttackTables> = OnceLock::new();

/// Computes the attack tables and stores them into the global variable ATTACK_TABLES.
pub fn init_attack_tables() -> &'static AttackTables {
    ATTACK_TABLES.get_or_init(AttackTables::new)
}

impl AttackTables {
    pub fn new() -> Self {
        let mut knight = [Bitboard(0); 64];
        let mut king = [Bitboard(0); 64];
        let mut pawn_capture = [[Bitboard(0); 64]; 2]; //Color-dependent
        let mut pawn_push = [[Bitboard(0); 64]; 2]; //Color-dependent
        let mut pawn_double_push = [[Bitboard(0); 64]; 2]; //Color-dependent

        // Initializes the attack table for each square
        for sq in 0..64 {
            let from_rank = (sq / 8) as i8;
            let from_file = (sq % 8) as i8;

            // ****************** KNIGHT ******************
            for (delta_rank, delta_file) in KNIGHT_DELTAS {
                let to_rank = from_rank + delta_rank;
                let to_file = from_file + delta_file;

                if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                    knight[sq] |= Bitboard::from_square((to_rank * 8 + to_file) as usize);
                }
            }

            // ****************** KING ******************
            for (delta_rank, delta_file) in KING_DELTAS {
                let to_rank = from_rank + delta_rank;
                let to_file = from_file + delta_file;

                if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                    king[sq] |= Bitboard::from_square((to_rank * 8 + to_file) as usize);
                }
            }

            // ****************** PAWN CAPTURE ******************
            //White
            if from_rank < 7 {
                if from_file > 0 {
                    pawn_capture[Color::White][sq] |= Bitboard::from_square(sq + 7);
                }
                if from_file < 7 {
                    pawn_capture[Color::White][sq] |= Bitboard::from_square(sq + 9);
                }
            }
            //Black
            if from_rank > 0 {
                if from_file > 0 {
                    pawn_capture[Color::Black][sq] |= Bitboard::from_square(sq - 9);
                }
                if from_file < 7 {
                    pawn_capture[Color::Black][sq] |= Bitboard::from_square(sq - 7);
                }
            }

            // ****************** PAWN PUSH ******************
            if from_rank < 7 {
                pawn_push[Color::White][sq] = Bitboard::from_square(sq + 8);
            }
            if from_rank > 0 {
                pawn_push[Color::Black][sq] = Bitboard::from_square(sq - 8);
            }

            // ****************** DOUBLE PAWN PUSH ******************
            if from_rank == 1 {
                pawn_double_push[Color::White][sq] = Bitboard::from_square(sq + 16);
            }
            if from_rank == 6 {
                pawn_double_push[Color::Black][sq] = Bitboard::from_square(sq - 16);
            }
        }

        Self { knight, king, pawn_capture, pawn_push, pawn_double_push }
    }

    /// Writes the attack tables on a buffer for debug purposes.
    #[cfg(debug_assertions)]
    pub fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        writeln!(out, "*********************************KNIGHT*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.knight[sq])?;
        }
        writeln!(out, "*********************************KING*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.king[sq])?;
        }
        writeln!(out, "*********************************PAWN CAPTURE (WHITE)*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.pawn_capture[Color::White][sq])?;
        }
        writeln!(out, "*********************************PAWN PUSH (WHITE)*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.pawn_push[Color::White][sq])?;
        }
        writeln!(out, "*********************************PAWN DOUBLE PUSH (WHITE)*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.pawn_double_push[Color::White][sq])?;
        }
        writeln!(out, "*********************************PAWN CAPTURE (BLACK)*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.pawn_capture[Color::Black][sq])?;
        }
        writeln!(out, "*********************************PAWN PUSH (BLACK)*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.pawn_push[Color::Black][sq])?;
        }
        writeln!(out, "*********************************PAWN DOUBLE PUSH (BLACK)*********************************")?;
        for sq in 0..64 {
            writeln!(out, "{}", self.pawn_double_push[Color::Black][sq])?;
        }
        Ok(())
    }
}

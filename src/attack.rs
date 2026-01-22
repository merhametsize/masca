//! Attack table generation.
//!
//! This module generates attack look-up tables for knights, kings and pawns.
//! Tables provide possible moves for a given piece type and square, queried via `[square]` or `[color][square]`.
//!
//! Sliding pieces are handled differently (in magics.rs) because of board occupancy.

use crate::bitboard::Bitboard;
use crate::magics::MagicTables;
use crate::types::{Color, Square};

/// Contains the attack look-up tables per piece.
pub struct AttackTables {
    pub knight: [Bitboard; 64],
    pub king: [Bitboard; 64],
    pub pawn_capture: [[Bitboard; 64]; 2],
    pub pawn_push: [[Bitboard; 64]; 2],
    pub pawn_double_push: [[Bitboard; 64]; 2],

    pub magic_tables: MagicTables,
}

const KNIGHT_DELTAS: [(i8, i8); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
const KING_DELTAS: [(i8, i8); 8] = [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)];

impl AttackTables {
    pub fn new() -> Self {
        let mut knight = [Bitboard(0); 64];
        let mut king = [Bitboard(0); 64];
        let mut pawn_capture = [[Bitboard(0); 64]; 2]; //Color-dependent
        let mut pawn_push = [[Bitboard(0); 64]; 2]; //Color-dependent
        let mut pawn_double_push = [[Bitboard(0); 64]; 2]; //Color-dependent

        // Initializes the attack table for each square
        for sq in Square::ALL {
            let from_rank = sq.rank() as i8;
            let from_file = sq.file() as i8;

            // ****************** KNIGHT ******************
            for (delta_rank, delta_file) in KNIGHT_DELTAS {
                let to_rank = from_rank + delta_rank;
                let to_file = from_file + delta_file;

                if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                    let to = Square::new((to_rank * 8 + to_file) as u8);
                    knight[sq] |= to.bb();
                }
            }

            // ****************** KING ******************
            for (delta_rank, delta_file) in KING_DELTAS {
                let to_rank = from_rank + delta_rank;
                let to_file = from_file + delta_file;

                if (0..8).contains(&to_rank) && (0..8).contains(&to_file) {
                    let to = Square::new((to_rank * 8 + to_file) as u8);
                    king[sq] |= to.bb();
                }
            }

            // ****************** PAWN CAPTURE ******************
            //White
            if from_rank < 7 {
                if from_file > 0 {
                    pawn_capture[Color::White][sq] |= sq.north_west().bb();
                }
                if from_file < 7 {
                    pawn_capture[Color::White][sq] |= sq.north_east().bb();
                }
            }
            //Black
            if from_rank > 0 {
                if from_file > 0 {
                    pawn_capture[Color::Black][sq] |= sq.south_west().bb();
                }
                if from_file < 7 {
                    pawn_capture[Color::Black][sq] |= sq.south_east().bb();
                }
            }

            // ****************** PAWN PUSH ******************
            if from_rank < 7 {
                pawn_push[Color::White][sq] = sq.north().bb();
            }
            if from_rank > 0 {
                pawn_push[Color::Black][sq] = sq.south().bb();
            }

            // ****************** DOUBLE PAWN PUSH ******************
            if from_rank == 1 {
                pawn_double_push[Color::White][sq] = sq.north().north().bb();
            }
            if from_rank == 6 {
                pawn_double_push[Color::Black][sq] = sq.south().south().bb();
            }
        }

        // Generates sliding piece attacks
        let mut magic_tables = MagicTables::new();
        magic_tables.generate_magics();

        Self {
            knight,
            king,
            pawn_capture,
            pawn_push,
            pawn_double_push,
            magic_tables,
        }
    }

    /// Writes the attack tables on a buffer for debug purposes.
    pub fn print(&self) {
        use std::io::{Write, stdout};
        let mut out = stdout();

        fn print_section<W: Write>(out: &mut W, title: &str, boards: &[Bitboard]) -> std::io::Result<()> {
            writeln!(out, "\n=== {} ===", title)?;
            // Print 4 bitboards per row
            let per_row = 4;
            for row in (0..boards.len()).step_by(per_row) {
                for i in 0..per_row {
                    if row + i < boards.len() {
                        write!(out, "{:2}: {:016X}  ", row + i, boards[row + i].0)?;
                    }
                }
                writeln!(out)?;
            }
            Ok(())
        }

        // Knights
        print_section(&mut out, "KNIGHT", &self.knight).unwrap();

        // Kings
        print_section(&mut out, "KING", &self.king).unwrap();

        // Pawns
        print_section(&mut out, "PAWN CAPTURE (WHITE)", &self.pawn_capture[Color::White]).unwrap();
        print_section(&mut out, "PAWN PUSH (WHITE)", &self.pawn_push[Color::White]).unwrap();
        print_section(&mut out, "PAWN DOUBLE PUSH (WHITE)", &self.pawn_double_push[Color::White]).unwrap();

        print_section(&mut out, "PAWN CAPTURE (BLACK)", &self.pawn_capture[Color::Black]).unwrap();
        print_section(&mut out, "PAWN PUSH (BLACK)", &self.pawn_push[Color::Black]).unwrap();
        print_section(&mut out, "PAWN DOUBLE PUSH (BLACK)", &self.pawn_double_push[Color::Black]).unwrap();

        println!("");
        self.magic_tables.print();
    }
}

//! Magics generation
//!

use crate::bitboard::Bitboard;

const MAX_ROOK_ENTRIES: usize = 4096; // maximum 2^12 relevant bits
const MAX_BISHOP_ENTRIES: usize = 512; // maximum 2^9 relevant bits

const ROOK_DELTAS: [(i8, i8); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
const BISHOP_DELTAS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

pub struct MagicTables {
    pub rook_masks: [Bitboard; 64],
    pub bishop_masks: [Bitboard; 64],

    pub rook_magics: [u64; 64],
    pub bishop_magics: [u64; 64],

    pub rook_attacks: [Bitboard; MAX_ROOK_ENTRIES],
    pub bishop_attacks: [Bitboard; MAX_BISHOP_ENTRIES],
    pub rook_offsets: [usize; 64],
    pub bishop_offsets: [usize; 64],
}

impl MagicTables {
    pub fn new() -> Self {
        let mut rook_masks = [Bitboard(0); 64];
        let mut bishop_masks = [Bitboard(0); 64];

        Self {
            rook_masks,
            bishop_masks,
            rook_magics: [0; 64],
            bishop_magics: [0; 64],
            rook_attacks: [Bitboard(0); MAX_ROOK_ENTRIES],
            bishop_attacks: [Bitboard(0); MAX_BISHOP_ENTRIES],
            rook_offsets: [0; 64],
            bishop_offsets: [0; 64],
        }
    }

    // Generic relevant occupancy mask generator for sliding pieces. Excludes edge squares (rank/file 0 or 7).
    #[inline(always)]
    fn relevant_occupancy_mask(square: usize, deltas: &[(i8, i8)]) -> Bitboard {
        let mut mask = Bitboard(0);
        let from_rank = (square / 8) as i8;
        let from_file = (square % 8) as i8;

        for &(delta_rank, delta_file) in deltas {
            let mut to_rank = from_rank + delta_rank;
            let mut to_file = from_file + delta_file;

            while (1..=6).contains(&to_rank) && (1..=6).contains(&to_file) {
                mask |= Bitboard::from_square((to_rank * 8 + to_file) as usize);
                to_rank += delta_rank;
                to_file += delta_file;
            }
        }
        mask
    }

    /// Initializes relevant occupancy masks in the MagicTables struct.
    pub fn init_relevant_occupancy_masks(&mut self) {
        for sq in 0..64 {
            self.rook_masks[sq] = Self::relevant_occupancy_mask(sq, &ROOK_DELTAS);
            self.bishop_masks[sq] = Self::relevant_occupancy_mask(sq, &BISHOP_DELTAS);
        }
    }

    /// Enumerates all possible occupancies for a given relevant mask.
    fn enumerate_occupancies(mask: Bitboard) -> Vec<Bitboard> {
        let num_relevant_bits = mask.0.count_ones() as usize;
        let mut occupancies = Vec::with_capacity(1 << num_relevant_bits); //1<<n = 2^n

        // Gathers the indices of all bits that are 1 in the mask
        let mut relevant_square_indices = Vec::with_capacity(num_relevant_bits);
        for square in 0..64 {
            if mask & Bitboard::from_square(square) != Bitboard(0) {
                relevant_square_indices.push(square);
            }
        }

        // Enumerates all 2^num_relevant_bits combinations
        // num_relevant_bits=3 --> iterate on 0b000, 0b001, 0b010, 0b011 etc.
        for subset in 0..(1 << num_relevant_bits) {
            let mut occ = 0u64;

            for (i, &square) in relevant_square_indices.iter().enumerate() {
                if subset & (1 << i) != 0 {
                    occ |= 1u64 << square;
                }
            }
            occupancies.push(Bitboard(occ));
        }
        occupancies
    }

    // Computes sliding attacks given a square and an occupancy
    fn sliding_attack(square: usize, deltas: &[(i8, i8)], occupancy: Bitboard) -> Bitboard {
        let mut attacks = Bitboard(0);
        let from_rank = (square / 8) as i8;
        let from_file = (square % 8) as i8;

        for &(delta_rank, delta_file) in deltas {
            let mut to_rank = from_rank + delta_rank;
            let mut to_file = from_file + delta_file;

            while to_rank >= 0 && to_rank < 8 && to_file >= 0 && to_file < 8 {
                let sq = (to_rank * 8 + to_file) as usize;
                attacks |= Bitboard::from_square(sq);
                if occupancy & Bitboard::from_square(sq) != Bitboard(0) {
                    break; // Path is blocked
                }
                to_rank += delta_rank;
                to_file += delta_file;
            }
        }
        attacks
    }

    // Generates all attacks for a specific square and piece (given by deltas)
    fn attacks_for_square(square: usize, deltas: &[(i8, i8)]) -> Vec<Bitboard> {
        let mask = Self::relevant_occupancy_mask(square, deltas);
        let occupancies = Self::enumerate_occupancies(mask);
        occupancies.iter().map(|occ| Self::sliding_attack(square, deltas, *occ)).collect()
    }

    // Generates all possible rook attacks for all squares and occupancies.
    // Used in magic number generation to populate the flat attack tables.
    fn generate_all_rook_attacks(&self) -> Vec<Vec<Bitboard>> {
        (0..64).map(|sq| Self::attacks_for_square(sq, &ROOK_DELTAS)).collect()
    }

    // Generates all possible bishop attacks for all squares and occupancies.
    // Used in magic number generation to populate the flat attack tables.
    fn generate_all_bishop_attacks(&self) -> Vec<Vec<Bitboard>> {
        (0..64).map(|sq| Self::attacks_for_square(sq, &BISHOP_DELTAS)).collect()
    }
}

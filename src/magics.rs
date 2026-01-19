//! Magic bitboard generation for sliding pieces.
//!
//! This module generates and stores all data required to compute rook and bishop
//! attacks in O(1) time using magic bitboards. At startup, it:
//!
//! - Computes relevant occupancy masks for each square
//! - Enumerates all possible blocker configurations
//! - Searches for collision-free magic multipliers
//! - Builds flat attack tables indexed via magic hashing
//!
//! Once initialized, attack lookup requires only:
//!     - masking the occupancy
//!     - a 64-bit multiplication
//!     - a shift
//!     - a table lookup
//!
//! The generated tables are read-only after initialization and contain no runtime branches, locks, or collision checks.

use rand::RngCore;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::bitboard::Bitboard;

const ROOK_DELTAS: [(i8, i8); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
const BISHOP_DELTAS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

/// Total number of rook magic attack entries.
///
/// This is the sum over all squares of:
///     2 ^ (number of relevant rook occupancy bits on that square)
///
/// Central squares have up to 12 relevant bits, edge squares fewer.
/// The exact total is 102,400.
const ROOK_MAP_SIZE: usize = 102400; // Σ over sq=0..63 (2 ^ rook_relevant_bits[sq])
/// Total number of bishop magic attack entries.
///
/// This is the sum over all squares of:
///     2 ^ (number of relevant bishop occupancy bits on that square)
///
/// Central squares have up to 9 relevant bits.
/// The exact total is 5,248.
const BISHOP_MAP_SIZE: usize = 5248; // Σ over sq=0..63 (2 ^ bishop_relevant_bits[sq])

/// Precomputed magic bitboard data for sliding piece attack generation.
///
/// This structure stores all information required to compute rook and bishop
/// attacks in O(1) time using magic bitboards. All tables are generated once
/// at startup and are read-only thereafter.
pub struct MagicTables {
    pub rook_masks: [Bitboard; 64],   // Relevant occupancy masks for rooks
    pub bishop_masks: [Bitboard; 64], // Relevant occupancy masks for bishops

    pub rook_magics: [u64; 64],   // Magic multiplier for rooks
    pub bishop_magics: [u64; 64], // Magic multiplier for bishops

    pub rook_attacks: [Bitboard; ROOK_MAP_SIZE],     // Flat rook attack table, indexed by offsets[sq] + magic_index
    pub bishop_attacks: [Bitboard; BISHOP_MAP_SIZE], // Flat bishop attack table, indexed by offsets[sq] + magic_index
    pub rook_offsets: [usize; 64],                   // Starting index in `rook_attacks` for each square
    pub bishop_offsets: [usize; 64],                 // Starting index in `bishop_attacks` for each square
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
            rook_attacks: [Bitboard(0); ROOK_MAP_SIZE],
            bishop_attacks: [Bitboard(0); BISHOP_MAP_SIZE],
            rook_offsets: [0; 64],
            bishop_offsets: [0; 64],
        }
    }

    /// Generates magic numbers and populates flat attack tables for rooks and bishops.
    ///
    /// For each square:
    /// - Enumerates all relevant occupancies
    /// - Searches for a collision-free magic number
    /// - Stores the magic, offset, and corresponding attack table entries
    ///
    /// This function is intended to be called once at startup. The generated magic numbers guarantee O(1) sliding attack lookup
    /// with no branches and no runtime collisions.
    ///
    /// Note:
    /// The specific magic values chosen do not affect runtime performance. Any collision-free magic produces identical lookup speed.
    pub fn generate_magics(&mut self) {
        self.init_relevant_occupancy_masks();

        let rook_attacks = self.generate_all_rook_attacks();
        let bishop_attacks = self.generate_all_bishop_attacks();

        let mut rook_offset = 0usize;
        let mut bishop_offset = 0usize;

        let mut rng = SmallRng::seed_from_u64(0xD10FAu64);

        // --- ROOKS ---
        for sq in 0..64 {
            let mask = self.rook_masks[sq];
            let relevant_bits = mask.0.count_ones() as usize;
            let table_size = 1 << relevant_bits;

            let occupancies = Self::enumerate_occupancies(mask);
            let attacks = &rook_attacks[sq];

            let shift = 64 - relevant_bits;

            let mut attempts: usize = 0;
            'search: for _attempt in 0..10_000_000 {
                let magic = Self::sparse_random(&mut rng);

                // Quick entropy rejection (thanks Stockfish). If high bits are mostly zero -> more collisions.
                if (mask.0.wrapping_mul(magic) & 0xFF00_0000_0000_0000).count_ones() < 6 {
                    continue;
                }

                let mut used = vec![None; table_size];

                for i in 0..occupancies.len() {
                    let occ = occupancies[i].0;
                    let index = ((occ & mask.0).wrapping_mul(magic) >> shift) as usize;

                    match used[index] {
                        None => used[index] = Some(attacks[i]),
                        Some(existing) if existing == attacks[i] => {}
                        _ => continue 'search, // Collision
                    }
                }

                // Found valid magic
                self.rook_magics[sq] = magic;
                self.rook_offsets[sq] = rook_offset;

                for i in 0..table_size {
                    self.rook_attacks[rook_offset + i] = used[i].unwrap_or(Bitboard(0));
                }

                rook_offset += table_size;
                println!("Square {}: magic found = 0x{:016X}", sq, magic);
                break;
            }
        }

        // --- BISHOPS ---
        for sq in 0..64 {
            let mask = self.bishop_masks[sq];
            let relevant_bits = mask.0.count_ones() as usize;
            let table_size = 1 << relevant_bits;

            let occupancies = Self::enumerate_occupancies(mask);
            let attacks = &bishop_attacks[sq];

            let shift = 64 - relevant_bits;

            'search: for _attempt in 0..10_000_000 {
                let magic = Self::sparse_random(&mut rng);

                if (mask.0.wrapping_mul(magic) & 0xFF00_0000_0000_0000).count_ones() < 6 {
                    continue;
                }

                let mut used = vec![None; table_size];

                for i in 0..occupancies.len() {
                    let occ = occupancies[i].0;
                    let index = ((occ & mask.0).wrapping_mul(magic) >> shift) as usize;

                    match used[index] {
                        None => used[index] = Some(attacks[i]),
                        Some(existing) if existing == attacks[i] => {}
                        _ => continue 'search,
                    }
                }

                self.bishop_magics[sq] = magic;
                self.bishop_offsets[sq] = bishop_offset;

                for i in 0..table_size {
                    self.bishop_attacks[bishop_offset + i] = used[i].unwrap_or(Bitboard(0));
                }

                bishop_offset += table_size;
                println!("Square {}: magic found = 0x{:016X}", sq, magic);
                break;
            }
        }

        debug_assert_eq!(rook_offset, ROOK_MAP_SIZE);
        debug_assert_eq!(bishop_offset, BISHOP_MAP_SIZE);
    }

    /// Generates a candidate magic number with sparse bits set,
    /// inspired by Stockfish's sparse_rand.
    ///
    /// Sparse numbers reduce collisions in magic bitboards.
    /// Deterministic if the same seed is used.
    #[inline(always)]
    fn sparse_random(seed: &mut SmallRng) -> u64 {
        let mut r = 0u64;
        for _ in 0..3 {
            // Each OR adds a few 1-bits sparsely
            r |= seed.next_u64() & seed.next_u64() & seed.next_u64();
        }
        r
    }

    /// Initializes relevant occupancy masks in the MagicTables struct.
    pub fn init_relevant_occupancy_masks(&mut self) {
        for sq in 0..64 {
            self.rook_masks[sq] = Self::relevant_occupancy_mask(sq, &ROOK_DELTAS);
            self.bishop_masks[sq] = Self::relevant_occupancy_mask(sq, &BISHOP_DELTAS);
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

    // Generates all attacks for a specific square and piece (given by deltas)
    fn attacks_for_square(square: usize, deltas: &[(i8, i8)]) -> Vec<Bitboard> {
        let mask = Self::relevant_occupancy_mask(square, deltas);
        let occupancies = Self::enumerate_occupancies(mask);
        occupancies.iter().map(|occ| Self::sliding_attack(square, deltas, *occ)).collect()
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

    #[cfg(debug_assertions)]
    fn print(&self) {
        println!("=== Rook Magics ===");
        for sq in 0..64 {
            let mask = self.rook_masks[sq];
            let magic = self.rook_magics[sq];
            let offset = self.rook_offsets[sq];
            let num_bits = mask.0.count_ones();
            println!("Square {:2}: Magic = 0x{:016X}, Offset = {:5}, Mask bits = {:2}, Mask = 0x{:016X}", sq, magic, offset, num_bits, mask.0);
        }

        println!("\n=== Bishop Magics ===");
        for sq in 0..64 {
            let mask = self.bishop_masks[sq];
            let magic = self.bishop_magics[sq];
            let offset = self.bishop_offsets[sq];
            let num_bits = mask.0.count_ones();
            println!("Square {:2}: Magic = 0x{:016X}, Offset = {:5}, Mask bits = {:2}, Mask = 0x{:016X}", sq, magic, offset, num_bits, mask.0);
        }
    }
}

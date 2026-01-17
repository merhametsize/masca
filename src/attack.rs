use crate::types::Bitboard;

use std::sync::OnceLock;

pub struct AttackTables {
    pub knight: [Bitboard; 64],
    pub king: [Bitboard; 64],
    pub pawn_attack: [[Bitboard; 64]; 2],
    pub pawn_push: [[Bitboard; 64]; 2],
    pub pawn_double_push: [[Bitboard; 64]; 2],
}

const KNIGHT_DELTAS: [(i8, i8); 8] = [(2, 1), (2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];

/// Global struct containing the attack maps for pieces.
/// Rust discourages the use of static global variables because of the lack of thread safety. Rust enforces the use of
/// unsafe{} in order to access such variables. The idiomatic way to define static global variables in Rust is to use a OnceLock.
/// A OnceLock object is initialized exactly once, is thread safe, immutable, accessible anywhere. It does 1 atomic check on
/// first access. After that, only pointer dereference. It has 0 performance cost in practice. After initialization,
/// ATTACKS.get().unwrap() compiles to a single load, with no locks nor branches.
pub static ATTACKS: OnceLock<AttackTables> = OnceLock::new();

impl AttackTables {
    pub fn new() -> Self {
        let mut knight = [0u64; 64];
        let mut king = [0u64; 64];
        let mut pawn_attack = [[0u64; 64]; 2];
        let mut pawn_push = [[0u64; 64]; 2];
        let mut pawn_double_push = [[0u64; 64]; 2];

        for sq in 0..64 {
            let (rank, file) = (sq / 8, sq % 8);
        }

        Self { knight, king, pawn_attack, pawn_push, pawn_double_push }
    }
}

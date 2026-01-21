//! Move generation for all piece types, except castling.
//!
//! This module provides:
//! - Generic move generation for sliding and leaper pieces using the `Attacker` trait.
//! - Specialized high-performance pawn move generation, including captures, promotions, double pushes, and en passant.
//!
//! All inner loops are optimized for branchless execution, bitboard manipulation, and monomorphization

use crate::attack::{self, AttackTables};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::moves::{Move, MoveType};
use crate::types::{Color, Piece, PieceType};

/// Container for moves generated for a position.
///
/// Preallocates space for up to 256 moves to avoid dynamic allocation.
/// Use `push()` to add moves in the inner loops of move generation.
pub struct MoveList {
    moves: [Move; 256],
    count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self { moves: [Move::NULL_MOVE; 256], count: 0 }
    }

    pub fn push(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

/// Trait for pieces that can generate pseudo-legal attacks.
///
/// Provides a single method `get_attacks()` returning a bitboard of target squares.
/// Generic over `P: Attacker` allows compile-time specialization and zero-cost abstraction.
/// `TYPE` is the corresponding `PieceType`.
pub trait Attacker {
    const TYPE: PieceType;
    fn get_attacks(from_square: usize, board: &Board, attack_tables: &AttackTables) -> Bitboard;
}

/// Knight move generation.
pub struct Knight;
impl Attacker for Knight {
    const TYPE: PieceType = PieceType::Knight;

    #[inline(always)]
    fn get_attacks(from_square: usize, _: &Board, attack_tables: &AttackTables) -> Bitboard {
        attack_tables.knight[from_square]
    }
}

/// King move generation.
pub struct King;
impl Attacker for King {
    const TYPE: PieceType = PieceType::King;

    #[inline(always)]
    fn get_attacks(from_square: usize, _: &Board, attack_tables: &AttackTables) -> Bitboard {
        attack_tables.king[from_square]
    }
}

/// Rook move generation using magic bitboards.
///
/// Occupancy of the board is masked to relevant squares, multiplied by the magic number, and indexed into
/// a flat attack table. `(64 - mask.popcount())` is computed on-the-fly for maximum performance.
pub struct Rook;
impl Attacker for Rook {
    const TYPE: PieceType = PieceType::Rook;

    #[inline(always)]
    fn get_attacks(from_square: usize, board: &Board, attack_tables: &AttackTables) -> Bitboard {
        let mt = &attack_tables.magic_tables;
        let mask = mt.rook_masks[from_square];
        let relevant_occupancy = board.occupied_squares() & mask;
        let magic = mt.rook_magics[from_square];

        // Calculating `(64 - mask.0.count_ones())` on the fly SHOULD be just as fast than loading it from a table
        // mov rax, [mask]                              mov rax, [mask]
        // popcnt rcx, rax                 vs           move rcx, [shift]
        // sub rcx, 64
        //
        // 1 load + 2 ALU = ~9 cycles    <---->         2 loads = ~10 cycles
        let idx = ((relevant_occupancy.0.wrapping_mul(magic)) >> (64 - mask.0.count_ones())) as usize;

        let offset = mt.rook_offsets[from_square];
        mt.rook_attacks[offset + idx]
    }
}

/// Bishop move generation using magic bitboards.
///
/// Occupancy of the board is masked to relevant squares, multiplied by the magic number, and indexed into
/// a flat attack table. `(64 - mask.popcount())` is computed on-the-fly for maximum performance.
pub struct Bishop;
impl Attacker for Bishop {
    const TYPE: PieceType = PieceType::Bishop;

    #[inline(always)]
    fn get_attacks(from_square: usize, board: &Board, attack_tables: &AttackTables) -> Bitboard {
        let mt = &attack_tables.magic_tables;
        let mask = mt.bishop_masks[from_square];
        let relevant_occupancy = board.occupied_squares() & mask;
        let magic = mt.bishop_magics[from_square];

        let idx = ((relevant_occupancy.0.wrapping_mul(magic)) >> (64 - mask.0.count_ones())) as usize;

        let offset = mt.bishop_offsets[from_square];
        mt.bishop_attacks[offset + idx]
    }
}

/// Queen move generation as a union of rook and bishop attacks.
pub struct Queen;
impl Attacker for Queen {
    const TYPE: PieceType = PieceType::Queen;

    #[inline(always)]
    fn get_attacks(from_square: usize, board: &Board, attack_tables: &AttackTables) -> Bitboard {
        Rook::get_attacks(from_square, &board, attack_tables) | Bishop::get_attacks(from_square, &board, attack_tables)
    }
}

/// Generic move generation for leaper and sliding pieces.
///
/// # Parameters
/// - `P: Attacker` — piece type to generate moves for (generic, monomorphized)
/// - `WHITE: bool` — generate moves for white (true) or black (false)
/// - `CAPTURE: bool` — if true, only generate captures; otherwise only quiet moves
///
/// # Notes
/// - Fully monomorphized: `if WHITE` and `if CAPTURE` branches are removed by the compiler
/// - Suitable for knights, kings, rooks, bishops, and queens (pawns are special)
#[inline(always)]
pub fn generate_moves<P: Attacker, const WHITE: bool, const CAPTURE: bool>(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList) {
    let us = if WHITE { board.color(Color::White) } else { board.color(Color::Black) };
    let them = if WHITE { board.color(Color::Black) } else { board.color(Color::White) };

    let mut attackers = board.piece(P::TYPE) & us;
    let target_mask = if CAPTURE { them } else { board.empty_squares() };

    while attackers != Bitboard(0) {
        let from_square = attackers.pop_lsb();
        let mut attacks = P::get_attacks(from_square, &board, attack_tables) & target_mask;

        while attacks != Bitboard(0) {
            let to_square = attacks.pop_lsb();
            if CAPTURE {
                moves.push(Move::new_special(from_square, to_square, MoveType::Capture));
            } else {
                moves.push(Move::new_normal(from_square, to_square));
            }
        }
    }
}

/// Pawn capture moves, including promotions and en passant.
///
/// Uses precomputed `pawn_capture` tables and single-bit operations. Fully branchless inside loops except for promotion/en-passant handling.
/// Generic over `const WHITE` to remove runtime color checks.
///
/// # Details
/// - Captures enemy pieces or the en passant square
/// - Generates all promotion captures automatically
/// - Branch-minimized inner loop with bitwise operations
#[inline(always)]
pub fn generate_pawn_captures<const WHITE: bool>(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList) {
    let our_color = if WHITE { Color::White } else { Color::Black };
    let mut pawns = board.piece(PieceType::Pawn) & board.color(our_color);

    let them = if WHITE { board.color(Color::Black) } else { board.color(Color::White) };
    let promotion_rank = if WHITE { Bitboard(0xFF00000000000000u64) } else { Bitboard(0x00000000000000FFu64) };

    let ep_square = if let Some(ep_square) = board.en_passant_square() { Bitboard::from_square(ep_square) } else { Bitboard(0) };

    while pawns != Bitboard(0) {
        let from_square = pawns.pop_lsb();
        let mut attacks = attack_tables.pawn_capture[our_color][from_square] & (them | ep_square);

        while attacks != Bitboard(0) {
            let to_square = attacks.lsb();
            let to_bb = Bitboard::from_square(to_square);
            attacks ^= to_bb; // pop_lsb() would re-execute lsb() internally, xoring directly is faster

            if (to_bb & ep_square) != Bitboard(0) {
                moves.push(Move::new_special(from_square, to_square, MoveType::EnPassant));
            } else if promotion_rank & to_bb != Bitboard(0) {
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionCaptureQ));
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionCaptureR));
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionCaptureB));
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionCaptureN));
            } else {
                moves.push(Move::new_special(from_square, to_square, MoveType::Capture));
            }
        }
    }
}

/// Pawn quiet moves, including single and double pushes and promotions.
///
/// Uses precomputed `pawn_push` and `pawn_double_push` tables. Fully branchless inside loops except for promotion handling.
/// Generic over `const WHITE` to remove runtime color checks.
///
/// # Details
/// - Single push only if target square empty
/// - Double push only if both squares empty
/// - Generates all promotions automatically
#[inline(always)]
pub fn generate_pawn_quiets<const WHITE: bool>(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList) {
    let our_color = if WHITE { Color::White } else { Color::Black };
    let mut pawns = board.piece(PieceType::Pawn) & board.color(our_color);

    let pawn_pushes = &attack_tables.pawn_push[our_color];
    let pawn_double = &attack_tables.pawn_double_push[our_color];

    let promotion_rank = if WHITE { Bitboard(0xFF00000000000000u64) } else { Bitboard(0x00000000000000FFu64) };
    let empty_bb = board.empty_squares();

    while pawns != Bitboard(0) {
        let from_square = pawns.pop_lsb();
        let mut attacks = pawn_pushes[from_square] & empty_bb;

        while attacks != Bitboard(0) {
            let to_square = attacks.lsb();
            let to_bb = Bitboard::from_square(to_square);
            attacks ^= to_bb; // pop_lsb() would re-execute lsb() internally, xoring directly is faster

            if promotion_rank & to_bb != Bitboard(0) {
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionQ));
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionR));
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionB));
                moves.push(Move::new_special(from_square, to_square, MoveType::PromotionN));
            } else {
                moves.push(Move::new_normal(from_square, to_square));

                let mut double_pushes = pawn_double[from_square] & empty_bb;
                if double_pushes != Bitboard(0) {
                    let to_square = double_pushes.lsb();
                    moves.push(Move::new_special(from_square, to_square, MoveType::DoublePush));
                }
            }
        }
    }
}

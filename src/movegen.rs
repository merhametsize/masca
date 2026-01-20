use crate::attack::{self, AttackTables};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::moves::{Move, MoveType};
use crate::types::{Color, PieceType};

/// Includes the list of moves generated for each position. It was found that certain position
/// can reach up to ~200 legal moves, hence the rounding to 256.
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
}

#[inline(always)]
pub fn generate_knight_captures(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList, color: Color) {
    let mut knights = board.piece(PieceType::Knight) & board.color(color);
    let enemy = board.color(!color);

    while knights != Bitboard(0) {
        let from_square = knights.pop_lsb();
        let mut attacks = attack_tables.knight[from_square] & enemy;

        while attacks != Bitboard(0) {
            let to_square = attacks.pop_lsb();
            moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::Capture));
        }
    }
}

#[inline(always)]
pub fn generate_knight_quiets(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList, color: Color) {
    let mut knights = board.piece(PieceType::Knight) & board.color(color);

    while knights != Bitboard(0) {
        let from_square = knights.pop_lsb();
        let mut attacks = attack_tables.knight[from_square] & board.empty_squares();

        while attacks != Bitboard(0) {
            let to_square = attacks.pop_lsb();
            moves.push(Move::new_normal(from_square as u8, to_square as u8));
        }
    }
}

#[inline(always)]
pub fn generate_king_captures(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList, color: Color) {
    let mut king = board.piece(PieceType::King) & board.color(color);
    let enemy = board.color(!color);

    let from_square = king.pop_lsb();
    let mut attacks = attack_tables.king[from_square] & enemy;

    while attacks != Bitboard(0) {
        let to_square = attacks.pop_lsb();
        moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::Capture));
    }
}

#[inline(always)]
pub fn generate_king_quiets(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList, color: Color) {
    let mut king = board.piece(PieceType::King) & board.color(color);

    let from_square = king.pop_lsb();
    let mut attacks = attack_tables.king[from_square] & board.empty_squares();

    while attacks != Bitboard(0) {
        let to_square = attacks.pop_lsb();
        moves.push(Move::new_normal(from_square as u8, to_square as u8));
    }
}

#[inline(always)]
pub fn generate_pawn_captures(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList, color: Color) {
    let mut pawns = board.piece(PieceType::Pawn) & board.color(color);
    let enemy = board.color(!color);

    let pawn_attacks = &attack_tables.pawn_capture[color as usize];

    while pawns != Bitboard(0) {
        let from_square = pawns.pop_lsb();
        let mut attacks = pawn_attacks[from_square] & enemy;

        while attacks != Bitboard(0) {
            let to_square = attacks.pop_lsb();

            if is_promotion(to_square, color) {
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionCaptureQ));
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionCaptureR));
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionCaptureB));
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionCaptureN));
            } else {
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::Capture));
            }
        }
    }
}

#[inline(always)]
pub fn generate_pawn_quiets(board: &Board, attack_tables: &AttackTables, moves: &mut MoveList, color: Color) {
    let mut pawns = board.piece(PieceType::Pawn) & board.color(color);
    let pawn_pushes = &attack_tables.pawn_push[color as usize];
    let pawn_double = &attack_tables.pawn_double_push[color as usize];

    while pawns != Bitboard(0) {
        let from_square = pawns.pop_lsb();
        let mut attacks = pawn_pushes[from_square] & board.empty_squares();

        while attacks != Bitboard(0) {
            let to_square = attacks.pop_lsb();

            if is_promotion(to_square, color) {
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionQ));
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionR));
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionB));
                moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::PromotionN));
            } else {
                moves.push(Move::new_normal(from_square as u8, to_square as u8));
                // double push
                let mut double_attacks = pawn_double[from_square] & board.empty_squares();
                while double_attacks != Bitboard(0) {
                    let double_to = double_attacks.pop_lsb();
                    moves.push(Move::new_special(from_square as u8, to_square as u8, MoveType::DoublePush));
                }
            }
        }
    }
}

#[inline(always)]
fn is_promotion(to_square: usize, color: Color) -> bool {
    const RANK_1_MASK: u64 = 0x00000000000000FF;
    const RANK_8_MASK: u64 = 0xFF00000000000000;

    let bb = 1u64 << to_square;

    (color == Color::White && bb & RANK_8_MASK != 0) || (color == Color::Black && bb & RANK_1_MASK != 0)
}

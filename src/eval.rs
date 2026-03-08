use crate::board::Board;
use crate::types::{Color, Piece, PieceType, Square};

/// Tapered PST evaluation values.
#[rustfmt::skip]
const PAWN_OPENING: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 10, 10, -20, -20, 10, 10, 5,
    5, -5, -10, 0, 0, -10, -5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, 5, 10, 25, 25, 10, 5, 5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
    0, 0, 0, 0, 0, 0, 0, 0,
];

#[rustfmt::skip]
const PAWN_ENDGAME: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 10, 10, -20, -20, 10, 10, 5,
    5, -5, -10, 0, 0, -10, -5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, 5, 10, 25, 25, 10, 5, 5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const PST_OPENING: [[i32; 64]; 5] = [PAWN_OPENING, PAWN_OPENING, PAWN_OPENING, PAWN_OPENING, PAWN_OPENING];
const PST_ENDGAME: [[i32; 64]; 5] = [PAWN_ENDGAME, PAWN_ENDGAME, PAWN_ENDGAME, PAWN_ENDGAME, PAWN_ENDGAME];

/// Given a piece type, returns the weight for game phase change.
#[inline(always)]
pub fn phase_weight(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 1,
        PieceType::Rook => 2,
        PieceType::Queen => 4,
        PieceType::King => 0,
    }
}

const MAX_PHASE: i32 = 24; // Sum of all piece phase weights

/// Computes game phase based on material.
/// Phase 0 = endgame, phase MAX_PHASE = opening
pub fn compute_game_phase(board: &Board) -> i32 {
    let mut phase = MAX_PHASE;

    for piece_type in PieceType::ALL {
        let count = board.piece(piece_type).popcnt();
        phase -= count as i32 * phase_weight(piece_type);
    }

    phase.min(MAX_PHASE)
}

#[inline(always)]
fn pst_score(piece_type: PieceType, sq: Square, phase: i32, eg_phase: i32) -> i32 {
    let opening = PST_OPENING[piece_type][sq];
    let endgame = PST_ENDGAME[piece_type][sq];
    opening.saturating_mul(phase) + endgame.saturating_mul(eg_phase)
}

#[inline(always)]
pub fn pst_delta_quiet(piece: Piece, from: Square, to: Square, phase: i32) -> i32 {
    let eg_phase = MAX_PHASE - phase;
    let sign = 1 - ((piece.get_color() as i32) << 1); // Branchless
    let (piece_type, color) = (piece.get_type(), piece.get_color());

    let from = if color == Color::Black { from.mirror_rank() } else { from };
    let to = if color == Color::Black { to.mirror_rank() } else { to };

    let delta = pst_score(piece_type, to, phase, eg_phase) - pst_score(piece_type, from, phase, eg_phase);

    (delta * sign) / MAX_PHASE
}

#[inline(always)]
pub fn pst_delta_capture(piece: Piece, sq: Square, phase: i32) -> i32 {
    let eg_phase = MAX_PHASE - phase;
    let sign = 1 - ((piece.get_color() as i32) << 1); // Branchless
    let (piece_type, color) = (piece.get_type(), piece.get_color());

    let sq = if color == Color::Black { sq.mirror_rank() } else { sq };

    let delta = -pst_score(piece_type, sq, phase, eg_phase);

    (delta * sign) / MAX_PHASE
}

#[inline(always)]
pub fn pst_delta_promotion(piece: Piece, sq: Square, phase: i32) -> i32 {
    let eg_phase = MAX_PHASE - phase;
    let sign = 1 - ((piece.get_color() as i32) << 1); // Branchless
    let (piece_type, color) = (piece.get_type(), piece.get_color());

    let sq = if color == Color::Black { sq.mirror_rank() } else { sq };

    let delta = pst_score(piece_type, sq, phase, eg_phase);

    (delta * sign) / MAX_PHASE
}

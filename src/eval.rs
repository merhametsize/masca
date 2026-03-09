use crate::board::Board;
use crate::types::{Color, Piece, PieceKind, Square};

/// ================== PIECE-SQUARE TABLES - WHITE ==================
// PAWN midgame
#[rustfmt::skip]
pub const PAWN_MIDGAME: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10,-20,-20, 10, 10,  5,
     5, -5,-10,  0,  0,-10, -5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0,  0,  0,  0,  0,  0,  0,
];

// PAWN endgame - equal
#[rustfmt::skip]
pub const PAWN_ENDGAME: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10,-20,-20, 10, 10,  5,
     5, -5,-10,  0,  0,-10, -5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0,  0,  0,  0,  0,  0,  0,
];

/// KNIGHT midgame
#[rustfmt::skip]
pub const KNIGHT_MIDGAME: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

// KNIGHT endgame - more centralization
#[rustfmt::skip]
pub const KNIGHT_ENDGAME: [i32; 64] = [
    -40,-30,-20,-20,-20,-20,-30,-40,
    -30,-10,  0,  0,  0,  0,-10,-30,
    -20,  0, 10, 15, 15, 10,  0,-20,
    -20,  0, 15, 20, 20, 15,  0,-20,
    -20,  0, 15, 20, 20, 15,  0,-20,
    -20,  0, 10, 15, 15, 10,  0,-20,
    -30,-10,  0,  0,  0,  0,-10,-30,
    -40,-30,-20,-20,-20,-20,-30,-40,
];

/// BISHOP midgame
#[rustfmt::skip]
pub const BISHOP_MIDGAME: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

// BISHOP endgame - favor long diagonals more
#[rustfmt::skip]
pub const BISHOP_ENDGAME: [i32; 64] = [
    -10,-10,-10,-10,-10,-10,-10,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  5, 10, 10, 10, 10,  5,-10,
    -10,  5, 10, 15, 15, 10,  5,-10,
    -10,  5, 10, 15, 15, 10,  5,-10,
    -10,  5, 10, 10, 10, 10,  5,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,-10,-10,-10,-10,-10,-10,-10,
];

/// ROOK midgame 
#[rustfmt::skip]
pub const ROOK_MIDGAME: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     5, 10, 10, 10, 10, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

// ROOK midgame - favor 7th rank more
#[rustfmt::skip]
pub const ROOK_ENDGAME: [i32; 64] = [
     0,  0,  5,  5,  5,  5,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

/// QUEEN midgame
#[rustfmt::skip]
pub const QUEEN_MIDGAME: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

// QUEEN endgame - favor centralization
#[rustfmt::skip]
pub const QUEEN_ENDGAME: [i32; 64] = [
    -10,-10,-5, -5, -5,-5,-10,-10,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5, 10, 10,  5,  0, -5,
     -5,  0,  5, 10, 10,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
    -10,-10,-5, -5, -5,-5,-10,-10,
];

/// KING midgame
#[rustfmt::skip]
pub const KING_MIDGAME: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

// KING endgame - favor centralization and active king
#[rustfmt::skip]
pub const KING_ENDGAME: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];

pub struct PieceSquareTables {
    pub table: [[[i32; 64]; 6]; 2], // Square, piece, color
}

impl PieceSquareTables {
    pub fn new() -> Self {
        let mut table = [[[0; 64]; 6]; 2];

        let pst_midgame = [PAWN_MIDGAME, KNIGHT_MIDGAME, BISHOP_MIDGAME, ROOK_MIDGAME, QUEEN_MIDGAME, KING_MIDGAME];
        let pst_endgame = [PAWN_ENDGAME, KNIGHT_ENDGAME, BISHOP_ENDGAME, ROOK_ENDGAME, QUEEN_ENDGAME, KING_ENDGAME];

        for piece in PieceKind::ALL {
            for sq in Square::ALL {
                let score_midgame = piece_value_midgame(piece) + pst_midgame[piece][sq];
                let score_endgame = piece_value_midgame(piece) + pst_endgame[piece][sq];

                table[Color::White][piece][sq] = pack(score_midgame, score_endgame);

                let mirrored = sq.mirror_rank();
                let score_midgame = -(piece_value_endgame(piece) + pst_midgame[piece][mirrored]);
                let score_endgame = -(piece_value_endgame(piece) + pst_endgame[piece][mirrored]);

                table[Color::Black][piece][sq] = pack(score_midgame, score_endgame);
            }
        }

        Self { table }
    }

    #[inline(always)]
    pub fn probe(&self, color: Color, piece_kind: PieceKind, sq: Square, phase: i32) -> i32 {
        let packed = self.table[color][piece_kind][sq];

        // Unpack midgame and endgame
        let mg = packed >> 16; // High 16 bits = midgame
        let eg = packed & 0xFFFF; // Low 16 bits = endgame

        // Handle negative numbers correctly (i16 sign)
        let mg = mg as i16 as i32;
        let eg = eg as i16 as i32;

        // Linear interpolation: score = (mg * phase + eg * (MAX_PHASE - phase)) / MAX_PHASE
        ((mg * phase + eg * (MAX_PHASE - phase)) / MAX_PHASE)
    }
}

/// Packs two i16 integers in an i32.
#[inline(always)]
fn pack(a: i32, b: i32) -> i32 {
    (a << 16) + b
}

/// Returns piece values for midgame. Should be optimized by the compiler.
#[inline(always)]
pub fn piece_value_midgame(piece_type: PieceKind) -> i32 {
    match piece_type {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0, // Dummy
    }
}

/// Returns piece values for endgame. Should be optimized by the compiler.
#[inline(always)]
pub fn piece_value_endgame(piece_type: PieceKind) -> i32 {
    match piece_type {
        PieceKind::Pawn => 100,
        PieceKind::Knight => 300,
        PieceKind::Bishop => 320,
        PieceKind::Rook => 500,
        PieceKind::Queen => 900,
        PieceKind::King => 0, // Dummy
    }
}

/// Given a piece type, returns the weight for game phase change.
#[inline(always)]
pub fn phase_weight(piece_type: PieceKind) -> i32 {
    match piece_type {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 1,
        PieceKind::Rook => 2,
        PieceKind::Queen => 4,
        PieceKind::King => 0,
    }
}

const MAX_PHASE: i32 = 24; // Sum of all piece phase weights

/// Computes game phase based on material.
/// Phase 0 = endgame, phase MAX_PHASE = opening
pub fn compute_game_phase(board: &Board) -> i32 {
    let mut phase = MAX_PHASE;

    for piece_type in PieceKind::ALL {
        let count = board.piece(piece_type).popcnt();
        phase -= count as i32 * phase_weight(piece_type);
    }

    phase.min(MAX_PHASE)
}

//! Chessboard representation
//!
//! This module contains the implementation of the Board object, representing a Bord configuration along with its
//! present state and past states, allowing for make/unmake move. The State object is memorized in a stack inside Board.

use crate::attack::AttackTables;
use crate::bitboard::Bitboard;
use crate::eval::{self, PieceSquareTables};
use crate::moves::Move;
use crate::types::{Color, Piece, PieceKind, Square};

const MAX_PLY: usize = 128;

// Castling encoding in a u8.
pub const WK: u8 = 0b0001;
pub const WQ: u8 = 0b0010;
pub const BK: u8 = 0b0100;
pub const BQ: u8 = 0b1000;

/// Chess board representation.
///
/// This structure maintains multiple redundant representations of the position to enable fast move generation and evaluation.
/// It also owns a stack of incremental states used to undo moves efficiently.
pub struct Board {
    mailbox: [Option<Piece>; 64],       // Piece-centric redundant representation
    pieces: [Bitboard; PieceKind::NUM], // p,n,b,r,q,k, color agnostic
    colors: [Bitboard; 2],              // Per-color occupancy
    side_to_move: Color,

    state_stack: [State; MAX_PLY], // Array of states for move unmake
    state_idx: usize,

    psqt: PieceSquareTables,
    pub attack_tables: AttackTables,
}

/// Incremental game state information.
///
/// This structure stores the minimal information required to unmake a move and restore the previous position.
/// It is intended to be pushed onto `state_stack` during move execution.
#[derive(Copy, Clone)]
pub struct State {
    castling: u8, // From LSB on, white-king, white-queen, black-king, black-queen side castling
    en_passant: Option<Square>,
    halfmove: usize,
    captured: Option<Piece>, // Which piece was captured in the last move

    phase: i32,
    eval_midgame: i32,
    eval_endgame: i32,

    #[allow(dead_code)]
    zobrist: Bitboard,
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes a pseudo-legal move.
    ///
    /// Does NOT check legality. Must be paired with `unmake_move`.
    pub fn make_move(&mut self, m: Move) {
        let (from, to) = (m.from(), m.to());
        let (us, them) = (self.side_to_move, !self.side_to_move);
        let mut phase_delta = 0;

        self.forward_state();

        // ------------------------------------
        // 1 -     Remove from origin
        // ------------------------------------
        debug_assert!(self.mailbox[from].is_some()); // There must be a piece in the origin square
        let moved_piece = self.piece_on_unchecked(from);
        let moved_kind = moved_piece.kind();
        self.remove_piece::<true>(moved_piece, from);

        // ------------------------------------
        // 2 - Remove captured piece, if any
        // ------------------------------------
        if m.is_enpassant() {
            let captured_sq = if us == Color::White { to.south() } else { to.north() };
            let captured_piece = Piece::new(them, PieceKind::Pawn);
            self.remove_piece::<true>(captured_piece, captured_sq);
            self.store_capture(captured_piece);
            phase_delta -= eval::phase_weight(PieceKind::Pawn);
        } else if m.is_capture() {
            debug_assert!(self.mailbox[to].is_some()); // There must be a piece in the destination square
            let captured_piece = self.piece_on_unchecked(to);
            self.remove_piece::<true>(captured_piece, to);
            self.store_capture(captured_piece);
            phase_delta -= eval::phase_weight(captured_piece.kind());
        }

        // ------------------------------------
        // 3 -    Reset halfmove count
        // ------------------------------------
        if moved_kind == PieceKind::Pawn || m.is_capture() {
            self.state_stack[self.state_idx].halfmove = 0;
        }

        // ------------------------------------
        // 4 - Place piece on destination
        // ------------------------------------
        if m.is_promotion() {
            let promoted_kind = m.promotion_piece();
            let promoted_piece = Piece::new(us, promoted_kind);
            self.add_piece::<true>(promoted_piece, to); // Yaaaaas queeeeen!
            phase_delta += eval::phase_weight(promoted_kind) - eval::phase_weight(PieceKind::Pawn);
        } else {
            self.add_piece::<true>(moved_piece, to); // Add the moved piece if quiet or capture
        }

        // ------------------------------------
        // 5 -          Castling
        // ------------------------------------
        if m.is_castling() {
            let (rook_from, rook_to) = match to {
                Square::G1 => (Square::H1, Square::F1),
                Square::C1 => (Square::A1, Square::D1),
                Square::G8 => (Square::H8, Square::F8),
                Square::C8 => (Square::A8, Square::D8),
                _ => unreachable!(),
            };

            let rook = self.piece_on_unchecked(rook_from);
            debug_assert!(rook.kind() == PieceKind::Rook);
            self.move_piece::<true>(rook, rook_from, rook_to);
        }

        // ---------------------------------------
        // 6 - Update castling rights (branchless)
        // ---------------------------------------
        let mut castling_mask: u8 = 0xFF; // Default: no change
        castling_mask &= !((from == Square::E1) as u8 * (WK | WQ)); // White king move
        castling_mask &= !((from == Square::E8) as u8 * (BK | BQ)); // Black king move
        castling_mask &= !((from == Square::H1 || to == Square::H1) as u8 * WK); // Rook moves or is captured
        castling_mask &= !((from == Square::A1 || to == Square::A1) as u8 * WQ); // Rook moves or is captured
        castling_mask &= !((from == Square::H8 || to == Square::H8) as u8 * BK); // Rook moves or is captured
        castling_mask &= !((from == Square::A8 || to == Square::A8) as u8 * BQ); // Rook moves or is captured
        self.state_stack[self.state_idx].castling &= castling_mask;

        // ---------------------------------------
        // 7 -    En passant activation
        // ---------------------------------------
        if m.is_double_push() {
            let ep_sq = if us == Color::White { to.south() } else { to.north() };
            self.state_stack[self.state_idx].en_passant = Some(ep_sq);
        }

        // ---------------------------------------
        // 8 -     Update zobrist key
        // ---------------------------------------
        //TODO

        // ---------------------------------------
        // 9 -           Update phase
        // ---------------------------------------
        // Phase is updated at the end so to avoid evaluation instability
        let state = &mut self.state_stack[self.state_idx];
        state.phase += phase_delta;

        // ---------------------------------------
        // 10 -           Flip side
        // ---------------------------------------
        self.side_to_move = !self.side_to_move;
    }

    /// Reverts the last move incrementally.
    ///
    /// After `make_move(m)` + `unmake_move(m)`: board state must be bit-identical.
    /// Phase is not updated by since the previous value is already stored in state.
    pub fn unmake_move(&mut self, m: Move) {
        let (from, to) = (m.from(), m.to());
        let mut moved_piece = self.piece_on_unchecked(to);
        let captured_piece = self.state_stack[self.state_idx].captured;

        // 1 - Flip side
        self.side_to_move = !self.side_to_move;
        let us = self.side_to_move;

        // 2 - Undo destination square
        self.remove_piece::<false>(moved_piece, to);

        // 3 - Restore captured piece
        if let Some(captured) = captured_piece {
            let captured_sq =
                if m.is_enpassant() { if us == Color::White { to.south() } else { to.north() } } else { to };
            self.add_piece::<false>(captured, captured_sq);
        }

        // 4 - Restore origin square
        if m.is_promotion() {
            moved_piece = Piece::new(us, PieceKind::Pawn); // Moved piece "becomes" a pawn
        }
        self.add_piece::<false>(moved_piece, from);

        // 5 - Undo castling
        if m.is_castling() {
            let (rook_from, rook_to) = match to {
                Square::G1 => (Square::H1, Square::F1),
                Square::C1 => (Square::A1, Square::D1),
                Square::G8 => (Square::H8, Square::F8),
                Square::C8 => (Square::A8, Square::D8),
                _ => unreachable!(),
            };
            let rook = self.piece_on_unchecked(rook_to);
            self.move_piece::<false>(rook, rook_to, rook_from);
        }

        // 6 - Pop state
        self.state_idx -= 1;
    }

    /// Adds a piece to the board at the given square.
    ///
    /// Updates bitboards and mailbox. If `UPDATE_EVAL` is true, updates the evaluation
    /// using the current phase.
    #[inline(always)]
    fn add_piece<const UPDATE_EVAL: bool>(&mut self, piece: Piece, sq: Square) {
        let color = piece.color();
        let piece_kind = piece.kind();

        self.mailbox[sq] = Some(piece);
        self.pieces[piece_kind] ^= sq.bb();
        self.colors[color] ^= sq.bb();

        if UPDATE_EVAL {
            let state = &mut self.state_stack[self.state_idx];
            let (mg, eg) = self.psqt.probe(color, piece_kind, sq);
            state.eval_midgame += mg;
            state.eval_endgame += eg;
        }
    }

    /// Removes a piece from the board at the given square.
    ///
    /// Updates bitboards and mailbox. If `UPDATE_EVAL` is true, updates the evaluation
    /// using the current phase.
    #[inline(always)]
    fn remove_piece<const UPDATE_EVAL: bool>(&mut self, piece: Piece, sq: Square) {
        let color = piece.color();
        let piece_kind = piece.kind();

        self.mailbox[sq] = None;
        self.pieces[piece_kind] ^= sq.bb();
        self.colors[color] ^= sq.bb();

        if UPDATE_EVAL {
            let state = &mut self.state_stack[self.state_idx];
            let (mg, eg) = self.psqt.probe(color, piece_kind, sq);
            state.eval_midgame -= mg;
            state.eval_endgame -= eg;
        }
    }

    /// Moves a piece from one square to another.
    ///
    /// Updates bitboards and mailbox. If `UPDATE_EVAL` is true, updates the evaluation
    /// using the current phase (adds the difference between destination and origin PSQT values).
    #[inline(always)]
    fn move_piece<const UPDATE_EVAL: bool>(&mut self, piece: Piece, from: Square, to: Square) {
        let color = piece.color();
        let piece_kind = piece.kind();

        self.mailbox[from] = None;
        self.mailbox[to] = Some(piece);
        self.pieces[piece_kind] ^= from.bb() | to.bb();
        self.colors[color] ^= from.bb() | to.bb();

        if UPDATE_EVAL {
            let state = &mut self.state_stack[self.state_idx];
            let (mg_from, eg_from) = self.psqt.probe(color, piece_kind, from);
            let (mg_to, eg_to) = self.psqt.probe(color, piece_kind, to);
            state.eval_midgame += mg_to - mg_from;
            state.eval_endgame += eg_to - eg_from;
        }
    }

    /// Stores a captured piece in the current state frame.
    ///
    /// Used to track captures for unmaking moves.
    #[inline(always)]
    fn store_capture(&mut self, captured: Piece) {
        let state = &mut self.state_stack[self.state_idx];
        state.captured = Some(captured);
    }

    /// Propagates the state forward, by copying certain fields and resetting others.
    #[inline(always)]
    fn forward_state(&mut self) {
        let old = self.state_stack[self.state_idx];
        self.state_idx += 1;
        debug_assert!(self.state_idx < MAX_PLY);
        let new = &mut self.state_stack[self.state_idx];

        *new = old;
        new.en_passant = None;
        new.halfmove += 1;
        new.captured = None;
    }

    /// Makes a null move, used for null-move pruning.
    #[inline(always)]
    pub fn make_null_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        let old_state = self.state_stack[self.state_idx];
        self.state_idx += 1;
        let new_state = &mut self.state_stack[self.state_idx];
        *new_state = old_state; // Struct assign
        new_state.en_passant = None;
    }

    /// Unmakes the null move, used for null-move pruning.
    #[inline(always)]
    pub fn unmake_null_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.state_idx -= 1;
    }

    /// Returns color-relative static evaluation of the position.
    #[inline(always)]
    pub fn evaluate_relative(&mut self) -> i32 {
        use eval::MAX_PHASE;
        let state = &self.state_stack[self.state_idx];
        let phase = state.phase;
        let sign = 1 - ((self.side_to_move as i32) << 1); // Branchless

        // Interpolate middlegame and endgame scores
        let score = (state.eval_midgame * phase + state.eval_endgame * (MAX_PHASE - phase)) / MAX_PHASE;
        score * sign
    }

    /// Returns true if `color`'s king is in check.
    ///
    /// Locates king square and calls `is_square_attacked`.
    pub fn king_in_check(&self, color: Color) -> bool {
        let king_bb = self.pieces[PieceKind::King] & self.colors[color];
        debug_assert!(king_bb != Bitboard(0));

        let king_sq = Square::new(king_bb.lsb() as u8);
        self.is_square_attacked(king_sq, !color)
    }

    /// Returns true if square `sq` is attacked by color `by`.
    ///
    /// Uses reverse attack lookup. Constant time. No iteration over all pieces.
    pub fn is_square_attacked(&self, sq: Square, by: Color) -> bool {
        let occupancy = self.occupied_squares();
        let their_pieces = self.colors[by];
        let attack_tables = &self.attack_tables;

        // Pawn attacks
        if attack_tables.pawn_capture[!by][sq] & (self.pieces[PieceKind::Pawn] & their_pieces) != Bitboard(0) {
            return true;
        }

        // Knight attacks
        if attack_tables.knight[sq] & (self.piece(PieceKind::Knight) & their_pieces) != Bitboard(0) {
            return true;
        }

        // King attacks
        if attack_tables.king[sq] & (self.piece(PieceKind::King) & their_pieces) != Bitboard(0) {
            return true;
        }

        // Bishop/Queen (diagonals)
        {
            let mt = &attack_tables.magic_tables;
            let mask = mt.bishop_masks[sq];
            let relevant = occupancy & mask;
            let magic = mt.bishop_magics[sq];
            let idx = ((relevant.0.wrapping_mul(magic)) >> (64 - mask.0.count_ones())) as usize;
            let attacks = mt.bishop_attacks[mt.bishop_offsets[sq] + idx];

            if attacks & ((self.piece(PieceKind::Bishop) | self.piece(PieceKind::Queen)) & their_pieces) != Bitboard(0)
            {
                return true;
            }
        }

        // Rook/Queen (orthogonal)
        {
            let mt = &attack_tables.magic_tables;
            let mask = mt.rook_masks[sq];
            let relevant = occupancy & mask;
            let magic = mt.rook_magics[sq];
            let idx = ((relevant.0.wrapping_mul(magic)) >> (64 - mask.0.count_ones())) as usize;
            let attacks = mt.rook_attacks[mt.rook_offsets[sq] + idx];

            if attacks & ((self.piece(PieceKind::Rook) | self.piece(PieceKind::Queen)) & their_pieces) != Bitboard(0) {
                return true;
            }
        }

        false
    }

    /// Returns a specific bitboard from `self.pieces`.
    #[inline(always)]
    pub fn piece(&self, piece_type: PieceKind) -> Bitboard {
        self.pieces[piece_type as usize]
    }

    /// Returns the piece on a specific square. Does not check if a piece is actually present.
    #[inline(always)]
    pub fn piece_on_unchecked(&self, sq: Square) -> Piece {
        debug_assert!(!self.mailbox[sq].is_none()); // There must be a piece in the square
        unsafe { self.mailbox[sq].unwrap_unchecked() }
    }

    /// Returns a specific color bitboard from `self.colors`.
    #[inline(always)]
    pub fn color(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
    }

    /// Returns black or white.
    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Returns which squares are occupied by a piece of any color.
    #[inline(always)]
    pub fn occupied_squares(&self) -> Bitboard {
        self.colors[Color::White] | self.colors[Color::Black]
    }

    /// Returns empty squares.
    #[inline(always)]
    pub fn empty_squares(&self) -> Bitboard {
        !(self.colors[Color::White] | self.colors[Color::Black])
    }

    /// Returns the en-passant capture square, if existing.
    #[inline(always)]
    pub fn en_passant_square(&self) -> Option<Square> {
        self.state_stack[self.state_idx].en_passant
    }

    /// Returns the castling rights, encoded in a u8.
    #[inline(always)]
    pub fn castling_rights(&self) -> u8 {
        self.state_stack[self.state_idx].castling
    }

    /// Sets board to the starting position.
    /// # Panics
    /// Panics if the internal FEN parser fails.
    pub fn set_startpos(&mut self) {
        self.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }

    /// Sets board state from a FEN string
    pub fn from_fen(&mut self, fen: &str) -> Result<(), &'static str> {
        let mut parts = fen.split_whitespace();
        let board_part = parts.next().ok_or("FEN missing board part")?;
        let side_part = parts.next().ok_or("FEN missing side to move")?;
        let castling_part = parts.next().unwrap_or("-");
        let en_passant_part = parts.next().unwrap_or("-");
        let halfmove_part = parts.next().unwrap_or("0");
        let _ = parts.next().unwrap_or("1"); //fullmove
        let mut eval_midgame: i32 = 0;
        let mut eval_endgame: i32 = 0;

        //Reset board
        self.mailbox.fill(Option::None);
        self.pieces.fill(Bitboard(0));
        self.colors = [Bitboard(0); 2];
        self.state_idx = 0;

        // ===== Parse board squares =====
        for (rank_idx, rank) in board_part.split('/').enumerate() {
            let rank_num = 7 - rank_idx; //FEN top rank = 7
            let mut file = 0;

            for ch in rank.chars() {
                if ch.is_digit(10) {
                    let skip = ch.to_digit(10).unwrap();
                    file += skip;
                } else {
                    let sq_idx = rank_num * 8 + file as usize;
                    let sq = Square::new(sq_idx as u8);
                    let piece = Piece::from_char(ch);
                    self.mailbox[sq] = Some(piece);

                    let color = piece.color();
                    let piece_kind = piece.kind();
                    let sq_bb = Square::new(sq as u8).bb();
                    self.pieces[piece_kind] |= sq_bb;
                    self.colors[color] |= sq_bb;

                    let (mg, eg) = self.psqt.probe(color, piece_kind, sq);
                    eval_midgame += mg;
                    eval_endgame += eg;

                    file += 1;
                }
            }
            if file != 8 {
                return Err("Invalid FEN rank length");
            }
        }

        // ===== Parse side to move =====
        self.side_to_move = match side_part {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side to move"),
        };

        // ===== Parse castling rights =====
        let mut castling = 0u8;
        for ch in castling_part.chars() {
            match ch {
                'K' => castling |= 1 << 0,
                'Q' => castling |= 1 << 1,
                'k' => castling |= 1 << 2,
                'q' => castling |= 1 << 3,
                '-' => {}
                _ => return Err("Invalid castling"),
            }
        }

        // ===== Parse en passant square =====
        let en_passant = if en_passant_part == "-" {
            None
        } else {
            let bytes = en_passant_part.as_bytes();
            let file = bytes[0].wrapping_sub(b'a');
            let rank = bytes[1].wrapping_sub(b'1');
            if file > 7 || rank > 7 {
                return Err("Invalid en passant square");
            }
            Some(Square::new((rank as u8) * 8 + (file as u8)))
        };

        // ===== Set initial state =====
        self.state_stack[0] = State {
            castling,
            en_passant,
            halfmove: halfmove_part.parse().unwrap_or_default(),
            captured: Option::None,
            eval_midgame: eval_midgame,
            eval_endgame: eval_endgame,
            phase: eval::compute_game_phase(&self),
            zobrist: Bitboard(0),
        };
        self.state_idx = 0;

        Ok(())
    }

    /// Prints the board to console terminal for debug.
    pub fn print(&self) {
        println!("Side to move: {:?}", self.side_to_move);
        println!("  +------------------------+");

        for rank in (0..8).rev() {
            // 8 = ranks, print rank 8..1
            print!("{} |", rank + 1);
            for file in 0..8 {
                let sq = rank * 8 + file;
                let ch = self.mailbox[sq].map_or('.', |p| p.to_char());
                print!(" {} ", ch);
            }
            println!("|");
        }

        println!("  +------------------------+");
        println!("    a  b  c  d  e  f  g  h");
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            mailbox: [Option::None; 64],
            pieces: [Bitboard(0); PieceKind::NUM],
            colors: [Bitboard(0); 2],
            side_to_move: Color::White,

            state_stack: [State::default(); MAX_PLY],
            state_idx: 0,

            psqt: PieceSquareTables::new(),
            attack_tables: AttackTables::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            castling: 0,
            en_passant: None,
            halfmove: 0,
            captured: Option::None,
            eval_midgame: 0,
            eval_endgame: 0,
            phase: 0,
            zobrist: Bitboard(0),
        }
    }
}

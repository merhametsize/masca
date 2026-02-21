//! Chessboard representation
//!
//! This module contains the implementation of the Board object, representing a Bord configuration along with its
//! present state and past states, allowing for make/unmake move. The State object is memorized in a stack inside Board.

use crate::attack::AttackTables;
use crate::bitboard::Bitboard;
use crate::moves::Move;
use crate::types::{Color, Piece, PieceType, Square};

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
    pieces: [Bitboard; PieceType::NUM], // p,n,b,r,q,k, color agnostic
    colors: [Bitboard; 2],              // Per-color occupancy
    side_to_move: Color,

    state_stack: [State; MAX_PLY], // Array of states for move unmake
    state_idx: usize,

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
    zobrist: Bitboard,
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes a pseudo-legal move incrementally.
    ///
    /// Does NOT check legality (king safety). Must be paired with `unmake_move`.
    pub fn make_move(&mut self, m: Move) {
        let (from, to) = (m.from(), m.to());
        let (us, them) = (self.side_to_move, !self.side_to_move);

        // 1 - Prepare state change variables
        let mut newstate_en_passant = None;
        let mut newstate_captured = None;
        let mut newstate_halfmove = 0;
        let mut newstate_castling = self.state_stack[self.state_idx].castling;

        // 2 - Remove from origin
        debug_assert!(self.mailbox[from].is_some()); // There must be a piece in the origin square
        let moved_piece = self.piece_on_unchecked(from);
        let moved_type = moved_piece.get_type();
        self.mailbox[from] = None;
        self.pieces[moved_type] ^= from.bb();
        self.colors[us] ^= from.bb();

        // 3 - Handle capture
        if m.is_enpassant() {
            let captured_sq = if us == Color::White { to.south() } else { to.north() };
            let captured_piece = self.piece_on_unchecked(captured_sq);

            self.mailbox[captured_sq] = None;
            self.pieces[PieceType::Pawn] ^= captured_sq.bb();
            self.colors[them] ^= captured_sq.bb();

            newstate_captured = Some(captured_piece);
        } else if m.is_capture() {
            debug_assert!(self.mailbox[to].is_some()); // There must be a piece in the destination square
            let captured_piece = self.piece_on_unchecked(to);
            self.pieces[captured_piece.get_type()] ^= to.bb();
            self.colors[them] ^= to.bb();

            newstate_captured = Some(captured_piece);
        }
        if moved_type == PieceType::Pawn || m.is_capture() || m.is_enpassant() {
            //m.is_enpassant() SHOULD be redundant
            newstate_halfmove = 0; // Halfmove reset
        } else {
            newstate_halfmove = self.state_stack[self.state_idx].halfmove + 1;
        }

        // 4 - Handle destination square
        if m.is_promotion() {
            let promoted_type = m.promotion_piece();
            let promoted_piece = Piece::new(us, promoted_type);

            self.mailbox[to] = Some(promoted_piece);
            self.pieces[promoted_type] ^= to.bb();
        } else {
            self.mailbox[to] = Some(moved_piece); //Normal piece move
            self.pieces[moved_type] ^= to.bb();
        }
        self.colors[us] ^= to.bb();

        // 5 - Castling
        if m.is_castling() {
            let (rook_from, rook_to) = match to {
                Square::G1 => (Square::H1, Square::F1),
                Square::C1 => (Square::A1, Square::D1),
                Square::G8 => (Square::H8, Square::F8),
                Square::C8 => (Square::A8, Square::D8),
                _ => unreachable!(),
            };

            let rook = self.piece_on_unchecked(rook_from);
            debug_assert!(rook.get_type() == PieceType::Rook);

            self.mailbox[rook_from] = None;
            self.mailbox[rook_to] = Some(rook);

            self.pieces[PieceType::Rook] ^= rook_from.bb() | rook_to.bb();
            self.colors[us] ^= rook_from.bb() | rook_to.bb();
        }

        // 6 - Update castling rights (branchless)
        let mut castling_mask: u8 = 0xFF; // Default: no change
        castling_mask &= !((from == Square::E1) as u8 * (WK | WQ)); // White king move
        castling_mask &= !((from == Square::E8) as u8 * (BK | BQ)); // Black king move
        castling_mask &= !((from == Square::H1 || to == Square::H1) as u8 * WK); // Rook move or capture
        castling_mask &= !((from == Square::A1 || to == Square::A1) as u8 * WQ);
        castling_mask &= !((from == Square::H8 || to == Square::H8) as u8 * BK);
        castling_mask &= !((from == Square::A8 || to == Square::A8) as u8 * BQ);
        newstate_castling &= castling_mask;

        // 7 - Handle double push
        if m.is_double_push() {
            let ep_sq = if us == Color::White { to.south() } else { to.north() };
            newstate_en_passant = Some(ep_sq);
        }

        // 8 - Update zobrist
        //TODO

        // 9 - Push new state
        let old_state = self.state_stack[self.state_idx];
        self.state_idx += 1;
        let new_state = &mut self.state_stack[self.state_idx];
        *new_state = old_state; // Struct assign
        new_state.en_passant = newstate_en_passant;
        new_state.captured = newstate_captured;
        new_state.castling = newstate_castling;
        new_state.halfmove = newstate_halfmove;
        debug_assert!(self.state_idx < MAX_PLY);

        // 10 - Flip side
        self.side_to_move = !self.side_to_move;
    }

    /// Reverts the last move incrementally.
    ///
    /// After `make_move(m)` + `unmake_move(m)`: board state must be bit-identical.
    pub fn unmake_move(&mut self, m: Move) {
        let (from, to) = (m.from(), m.to());
        let mut moved_piece = self.piece_on_unchecked(to);

        // 1 - Flip side
        self.side_to_move = !self.side_to_move;
        let (us, them) = (self.side_to_move, !self.side_to_move);

        // 2 - Pop state
        let state = self.state_stack[self.state_idx];
        self.state_idx -= 1;

        // 3 - Undo destination square
        self.pieces[moved_piece.get_type()] ^= to.bb();
        self.colors[us] ^= to.bb();
        self.mailbox[to] = None;

        // 4 - Restore captured piece
        if let Some(captured) = state.captured {
            let captured_sq = if m.is_enpassant() { if us == Color::White { to.south() } else { to.north() } } else { to };
            self.mailbox[captured_sq] = Some(captured);
            self.pieces[captured.get_type()] ^= captured_sq.bb();
            self.colors[them] ^= captured_sq.bb();
        }

        // 5 - Restore origin square
        if m.is_promotion() {
            moved_piece = Piece::new(!self.side_to_move, PieceType::Pawn);
        }
        self.mailbox[from] = Some(moved_piece);
        self.pieces[moved_piece.get_type()] ^= from.bb();
        self.colors[us] ^= from.bb();

        // 6 - Undo castling
        if m.is_castling() {
            let (rook_from, rook_to) = match to {
                Square::G1 => (Square::H1, Square::F1),
                Square::C1 => (Square::A1, Square::D1),
                Square::G8 => (Square::H8, Square::F8),
                Square::C8 => (Square::A8, Square::D8),
                _ => unreachable!(),
            };
            let rook = self.piece_on_unchecked(rook_to);
            self.mailbox[rook_to] = None;
            self.mailbox[rook_from] = Some(rook);
            self.pieces[PieceType::Rook] ^= rook_from.bb() | rook_to.bb();
            self.colors[us] ^= rook_from.bb() | rook_to.bb();
        }
    }

    /// Returns true if `color`'s king is in check.
    ///
    /// Locates king square and calls `is_square_attacked`.
    pub fn king_in_check(&self, color: Color) -> bool {
        let king_bb = self.pieces[PieceType::King] & self.colors[color];
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
        if attack_tables.pawn_capture[!by][sq] & (self.pieces[PieceType::Pawn] & their_pieces) != Bitboard(0) {
            return true;
        }

        // Knight attacks
        if attack_tables.knight[sq] & (self.piece(PieceType::Knight) & their_pieces) != Bitboard(0) {
            return true;
        }

        // King attacks
        if attack_tables.king[sq] & (self.piece(PieceType::King) & their_pieces) != Bitboard(0) {
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

            if attacks & ((self.piece(PieceType::Bishop) | self.piece(PieceType::Queen)) & their_pieces) != Bitboard(0) {
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

            if attacks & ((self.piece(PieceType::Rook) | self.piece(PieceType::Queen)) & their_pieces) != Bitboard(0) {
                return true;
            }
        }

        false
    }

    /// Returns a specific bitboard from `self.pieces`.
    #[inline(always)]
    pub fn piece(&self, piece_type: PieceType) -> Bitboard {
        self.pieces[piece_type as usize]
    }

    /// Returns the piece on a specific square. Does not check if a piece is actually present.
    #[inline(always)]
    pub fn piece_on_unchecked(&self, sq: Square) -> Piece {
        debug_assert!(!self.mailbox[sq].is_none()); // There must be a piece in the square
        unsafe { self.mailbox[sq].unwrap_unchecked() }
    }

    /// Returns a specific bitboard from `self.colors`.
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

        //R eset board
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
                    let sq = rank_num * 8 + file as usize;
                    let piece = Piece::from_char(ch);
                    self.mailbox[sq] = Some(piece);

                    let color = piece.get_color();
                    let ptype = piece.get_type();
                    let sq_bb = Square::new(sq as u8).bb();
                    self.pieces[ptype] |= sq_bb;
                    self.colors[color] |= sq_bb;

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
            pieces: [Bitboard(0); PieceType::NUM],
            colors: [Bitboard(0); 2],
            side_to_move: Color::White,

            state_stack: [State::default(); MAX_PLY],
            state_idx: 0,

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
            zobrist: Bitboard(0),
        }
    }
}

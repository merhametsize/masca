//! Chessboard representation
//!
//! This module contains the implementation of the Board object, representing a Bord configuration along with its
//! present state and past states, allowing for make/unmake move. The State object is memorized in a stack inside Board.

use crate::bitboard::Bitboard;
use crate::moves::Move;
use crate::types::{Color, Piece, PieceType, Square};

const MAX_PLY: usize = 128;

/// Chess board representation.
///
/// This structure maintains multiple redundant representations of the position to enable fast move generation and evaluation.
/// It also owns a stack of incremental states used to undo moves efficiently.
pub struct Board {
    mailbox: [Option<Piece>; 64],       //Piece-centric redundant representation
    pieces: [Bitboard; PieceType::NUM], //p,n,b,r,q,k, color agnostic
    colors: [Bitboard; 2],              //Per-color occupancy
    side_to_move: Color,

    state_stack: [State; MAX_PLY], //Array of states for move unmake
    state_idx: usize,
}

/// Incremental game state information.
///
/// This structure stores the minimal information required to unmake a move and restore the previous position.
/// It is intended to be pushed onto `state_stack` during move execution.
#[derive(Copy, Clone)]
pub struct State {
    castling: u8,
    en_passant: Option<Square>,
    halfmove: usize,
    captured: Option<Piece>, //Which piece was captured
    zobrist: Bitboard,
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn make_move(&mut self, m: Move) {
        let from = m.from();
        let to = m.to();

        debug_assert!(self.mailbox[from].is_some());
        let piece = unsafe { self.mailbox[from].unwrap_unchecked() };
        let captured = self.mailbox[to];

        self.mailbox[to] = Some(piece);
        self.mailbox[from] = None;
        self.pieces[piece.get_type()] ^= from.bb() | to.bb();
        self.colors[self.side_to_move] ^= from.bb() | to.bb();

        let old_state = self.state_stack[self.state_idx];
        let new_state = self.state_stack[self.state_idx + 1];
        self.state_idx += 1;
    }

    /// Returns a specific bitboard from `self.pieces`.
    #[inline(always)]
    pub fn piece(&self, piece_type: PieceType) -> Bitboard {
        self.pieces[piece_type as usize]
    }

    /// Returns a specific bitboard from `self.colors`.
    #[inline(always)]
    pub fn color(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
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
        self.state_idx = 1;

        Ok(())
    }

    /// Prints the board to console terminal for debug.
    #[cfg(debug_assertions)]
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

use std::mem::MaybeUninit;

use crate::types::{Color, NULL_SQUARE, PIECE_2_CHAR, Piece, PieceType};

const MAX_PLY: usize = 128;

pub struct Board {
    mailbox: [Piece; 64],
    pieces: [u64; PieceType::NUM], //p,n,b,r,q,k, color agnostic
    colors: [u64; 2],              //Per-color occupancy
    side_to_move: Color,

    state_stack: [MaybeUninit<State>; MAX_PLY],
    state_idx: usize,
}

pub struct State {
    castling: u8,
    en_passant: u8,
    halfmove: usize,
    captured: Piece, //Which piece was captured
    zobrist: u64,
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    ///Sets board to the starting position
    pub fn set_startpos(&mut self) {
        self.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    }

    ///Sets board state from a FEN string
    pub fn from_fen(&mut self, fen: &str) -> Result<(), &'static str> {
        let mut parts = fen.split_whitespace();
        let board_part = parts.next().ok_or("FEN missing board part")?;
        let side_part = parts.next().ok_or("FEN missing side to move")?;
        let castling_part = parts.next().unwrap_or("-");
        let en_passant_part = parts.next().unwrap_or("-");
        let halfmove_part = parts.next().unwrap_or("0");
        let _ = parts.next().unwrap_or("1"); //fullmove

        //Reset board
        self.mailbox.fill(Piece::None);
        self.pieces.fill(0);
        self.colors = [0; 2];
        self.state_idx = 0;

        // ===== Parse board squares =====
        let mut sq_idx = 0;
        for rank in board_part.split('/') {
            let mut file = 0;
            for ch in rank.chars() {
                if ch.is_digit(10) {
                    let incr = ch.to_digit(10).unwrap() as usize;
                    file += incr;
                    sq_idx += incr;
                } else {
                    if sq_idx >= 64 {
                        return Err("Too many squares in FEN");
                    }
                    let piece = PIECE_2_CHAR.iter().find(|(_, c)| *c == ch).map(|(p, _)| *p).ok_or("Invalid piece char in FEN")?;
                    self.mailbox[sq_idx] = piece;

                    let color = if ch.is_lowercase() { Color::White } else { Color::Black };
                    let ptype = match piece {
                        Piece::WhitePawn | Piece::BlackPawn => PieceType::Pawn,
                        Piece::WhiteKnight | Piece::BlackKnight => PieceType::Knight,
                        Piece::WhiteBishop | Piece::BlackBishop => PieceType::Bishop,
                        Piece::WhiteRook | Piece::BlackRook => PieceType::Rook,
                        Piece::WhiteQueen | Piece::BlackQueen => PieceType::Queen,
                        Piece::WhiteKing | Piece::BlackKing => PieceType::King,
                        Piece::None => continue,
                    };
                    self.pieces[ptype as usize] |= 1 << sq_idx;
                    self.colors[color as usize] |= 1 << sq_idx;

                    file += 1;
                    sq_idx += 1;
                }
            }
            if file != 8 {
                return Err("Invalid FEN rank length");
            }
        }
        if sq_idx != 64 {
            return Err("Invalid FEN: not enough squares");
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
            NULL_SQUARE
        } else {
            let bytes = en_passant_part.as_bytes();
            let file = bytes[0].wrapping_sub(b'a');
            let rank = bytes[1].wrapping_sub(b'1');
            if file > 7 || rank > 7 {
                return Err("Invalid en passant square");
            }
            (rank as u8) * 8 + (file as u8)
        };

        // ===== Set initial state =====
        self.state_stack[0] = MaybeUninit::new(State {
            castling,
            en_passant,
            halfmove: halfmove_part.parse().unwrap_or(0),
            captured: Piece::None,
            zobrist: 0,
        });
        self.state_idx = 1;

        Ok(())
    }

    pub fn print(&self) {
        println!("Side to move: {:?}", self.side_to_move);
        println!("  +------------------------+");

        for rank in (0..8).rev() {
            // 8 = ranks, print rank 8..1
            print!("{} |", rank + 1);
            for file in 0..8 {
                let sq = rank * 8 + file;
                let piece = self.mailbox[sq];
                let ch = PIECE_2_CHAR.iter().find(|(p, _)| *p == piece).map(|(_, c)| *c).unwrap_or('.');
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
            mailbox: [Piece::None; 64],
            pieces: [0; PieceType::NUM],
            colors: [0; 2],
            side_to_move: Color::White,

            state_stack: unsafe { MaybeUninit::uninit().assume_init() },
            state_idx: 0,
        }
    }
}

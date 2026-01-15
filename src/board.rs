use crate::types::{Color, NULL_SQUARE, Piece, PieceType};

const MAX_PLY: usize = 128;

pub struct Board {
    mailbox: [Option<Piece>; 64],
    pieces: [u64; PieceType::NUM], //p,n,b,r,q,k, color agnostic
    colors: [u64; 2],              //Per-color occupancy
    side_to_move: Color,

    state_stack: [State; MAX_PLY],
    state_idx: usize,
}
#[derive(Copy, Clone)]
pub struct State {
    castling: u8,
    en_passant: u8,
    halfmove: usize,
    captured: Option<Piece>, //Which piece was captured
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
        self.mailbox.fill(Option::None);
        self.pieces.fill(0);
        self.colors = [0; 2];
        self.state_idx = 0;

        // ===== Parse board squares =====
        for (rank_idx, rank) in board_part.split('/').enumerate() {
            let rank_num = 7 - rank_idx; //FEN top rank = 7
            let mut file = 0;

            for ch in rank.chars() {
                if ch.is_digit(10) {
                    let skip = ch.to_digit(10).unwrap() as usize;
                    file += skip;
                } else {
                    let sq = rank_num * 8 + file;
                    let piece = Piece::from_char(ch);
                    self.mailbox[sq] = Some(piece);

                    let color = piece.get_color();
                    let ptype = piece.get_type();
                    self.pieces[ptype as usize] |= 1 << sq;
                    self.colors[color as usize] |= 1 << sq;

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
        self.state_stack[0] = State {
            castling,
            en_passant,
            halfmove: halfmove_part.parse().unwrap_or_default(),
            captured: Option::None,
            zobrist: 0,
        };
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
            pieces: [0; PieceType::NUM],
            colors: [0; 2],
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
            en_passant: 0,
            halfmove: 0,
            captured: Option::None,
            zobrist: 0,
        }
    }
}

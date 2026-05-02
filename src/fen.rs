use crate::board::Board;
use crate::types::castling::CastlingRights;
use crate::types::{Color, Piece, Square};

impl Board {
    pub fn build_from_fen(&mut self, fen: &str) -> Result<(), &'static str> {
        let mut parts = fen.split_whitespace();

        let board_part = parts.next().ok_or("FEN missing board")?;
        let side_part = parts.next().ok_or("FEN missing side")?;
        let castling_part = parts.next().unwrap_or("-");
        let ep_part = parts.next().unwrap_or("-");
        let halfmove_part = parts.next().unwrap_or("0");
        let _ = parts.next().unwrap_or("1");

        self.clear();

        // ===== board =====
        for (rank_idx, rank) in board_part.split('/').enumerate() {
            let rank_num = 7 - rank_idx;
            let mut file = 0;

            for ch in rank.chars() {
                if ch.is_ascii_digit() {
                    file += ch.to_digit(10).unwrap();
                } else {
                    let sq_idx = rank_num * 8 + file as usize;
                    let sq = Square::new(sq_idx as u8);

                    let piece = Piece::from_char(ch);
                    self.add_piece::<true>(piece, sq);

                    file += 1;
                }
            }

            if file != 8 {
                return Err("Invalid FEN rank length");
            }
        }

        // ===== zobrist =====
        self.compute_zobrist();

        // ===== side =====
        let side_to_move = match side_part {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side"),
        };
        self.set_side_to_move(side_to_move);

        // ===== castling =====
        let mut castling = CastlingRights::new();
        castling.zero();

        for ch in castling_part.chars() {
            match ch {
                'K' => castling.add_white_oo(),
                'Q' => castling.add_white_ooo(),
                'k' => castling.add_black_oo(),
                'q' => castling.add_black_ooo(),
                '-' => {}
                _ => return Err("Invalid castling"),
            }
        }

        self.set_castling_rights(castling);

        // ===== en passant =====
        let ep = if ep_part == "-" {
            None
        } else {
            let bytes = ep_part.as_bytes();
            let file = bytes[0].wrapping_sub(b'a');
            let rank = bytes[1].wrapping_sub(b'1');

            if file > 7 || rank > 7 {
                return Err("Invalid en passant square");
            }

            Some(Square::new(rank * 8 + file))
        };

        self.set_en_passant_square(ep);

        // ===== halfmove =====
        let halfmove = halfmove_part.parse().unwrap_or(0);
        self.set_halfmove_clock(halfmove);

        // ===== phase =====
        self.set_game_phase();

        Ok(())
    }
}

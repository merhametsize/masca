use crate::board::Board;
use crate::types::{Color, PieceType};

const PAWN: i32 = 100;
const KNIGHT: i32 = 320;
const BISHOP: i32 = 330;
const ROOK: i32 = 500;
const QUEEN: i32 = 900;

pub fn eval_position(board: &Board) -> i32 {
    let material = eval_material(board);

    material
}

pub fn eval_material(board: &Board) -> i32 {
    let mut score = 0;

    for piece in [PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
        let white = (board.piece(piece) & board.color(Color::White)).popcnt() as i32;
        let black = (board.piece(piece) & board.color(Color::Black)).popcnt() as i32;

        //score += (white - black) * piece_value(piece);
    }

    0
    //sc
}

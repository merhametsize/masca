mod attack;
mod bitboard;
mod board;
mod movegen;
mod moves;
mod types;

use board::Board;

use crate::attack::ATTACK_TABLES;

fn main() {
    let mut board = Board::new();
    board.set_startpos();
    board.print();

    let atk = attack::init_attack_tables();
}

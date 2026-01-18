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

    attack::init_attack_tables();
    let mut f = std::fs::File::create("attack_tables.debug").unwrap();
    ATTACK_TABLES.get().unwrap().write(&mut f).unwrap();
}

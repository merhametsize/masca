mod board;
mod moves;
mod types;

use board::Board;

fn main() {
    let mut board = Board::new();
    board.set_startpos();
    board.print();
}

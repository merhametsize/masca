use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::generate_all_moves;

pub fn perft(board: &mut Board, depth: u64) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut list = MoveList::new();
    generate_all_moves(board, &mut list);

    let mut nodes = 0;

    for m in list.iter() {
        //println!("from: {:?}, to: {:?}", m.from(), m.to());
        board.make_move(*m);

        if !board.king_in_check(!board.side_to_move()) {
            nodes += perft(board, depth - 1);
        }

        board.unmake_move(*m);
    }

    nodes
}

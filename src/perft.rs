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

pub fn perft_n(depth: u64) {
    let mut board = Board::new();
    board.set_startpos();

    board.print();

    for depth in 1..=depth {
        let nodes = perft(&mut board, depth);
        println!("perft({}): {}", depth, nodes);
    }
}

pub fn kiwipete(depth: u64) {
    let mut board = Board::new();
    let kiwipete_pos = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    board.from_fen(kiwipete_pos).unwrap();

    board.print();

    for depth in 1..=depth {
        let nodes = perft(&mut board, depth);
        println!("perft({}): {}", depth, nodes);
    }
}

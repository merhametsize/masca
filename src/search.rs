use crate::board::Board;
use crate::movegen::{MoveList, generate_all_moves};
use crate::moves::Move;

pub struct Searcher {
    best_move: Move,
    nodes: u64,
    pv_table: [Move; 256],
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            best_move: Move::NULL_MOVE,
            nodes: 0,
            pv_table: [Move::NULL_MOVE; 256],
        }
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, max_depth: usize) {
        self.nodes = 0;

        for depth in 1..=max_depth {
            let score = self.pvs(board, depth, -30000, 30000);

            println!("info depth {} score cp {} nodes {} pv {}", depth, score, self.nodes, self.best_move.to_string());

            // TODO: early exit
        }
    }

    fn pvs(&mut self, board: &mut Board, depth: usize, mut alpha: i32, beta: i32) -> i32 {
        0
    }
}

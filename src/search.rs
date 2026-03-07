use crate::board::Board;
use crate::movegen::{MoveList, generate_all_captures, generate_all_moves};
use crate::moves::Move;
use crate::types::{PieceType, piece_value};

const SCORE_INF: i32 = 32_000;
const SCORE_MATE: i32 = 29_000;

pub struct Searcher<'a> {
    board: &'a mut Board,

    best_move: Move,
    nodes: u64,

    pv_table: [[Move; 64]; 64],
    pv_length: [usize; 64],

    killers: [[Move; 2]; 64],
}

impl<'a> Searcher<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Self {
            board: board,
            best_move: Move::NULL_MOVE,
            nodes: 0,

            pv_table: [[Move::NULL_MOVE; 64]; 64],
            pv_length: [0; 64],

            killers: [[Move::NULL_MOVE; 2]; 64], // Most beta cutoffs are caused by at most 2 moves per ply
        }
    }

    /// Performs iterative deepening search using Principal Variation Search (PVS).
    ///
    /// The search starts from depth 1 and progressively increases up to `max_depth`. For each depth, the best score is
    /// computed and search statistics are printed in a format compatible with typical chess engine UCI-style logging.
    pub fn iterative_deepening(&mut self, max_depth: usize) {
        self.nodes = 0;
        self.best_move = Move::NULL_MOVE;
        self.pv_table.iter_mut().for_each(|t| t.fill(Move::NULL_MOVE));
        self.pv_length.fill(0);
        self.killers = [[Move::NULL_MOVE; 2]; 64];

        for depth in 1..=max_depth {
            let score = self.search::<true>(depth, 0, -SCORE_INF, SCORE_INF);

            print!("info depth {} score cp {} nodes {} pv", depth, score, self.nodes);
            for i in 0..self.pv_length[0] {
                print!(" {}", self.pv_table[0][i]);
            }
            println!("");

            // TODO: early exit
        }
    }

    /// Principal variation search (PVS).
    fn search<const IS_PV: bool>(&mut self, depth: usize, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        // 1 - Target depth reached, quiescence search.
        if depth == 0 {
            return self.quiescence(ply, alpha, beta);
        }

        // 2 - Generate all moves and sort them.
        let mut moves = MoveList::new();
        generate_all_moves(self.board, &mut moves);
        self.order_moves::<false>(&mut moves, ply);

        // 3 - Iterate over possible moves.
        for (move_idx, m) in moves.iter().enumerate() {
            // 4 - Make move, undo and continue if illegal.
            self.board.make_move(m);
            if self.board.king_in_check(!self.board.side_to_move()) {
                self.board.unmake_move(m);
                continue;
            }

            // 5 - Principal Variation Search (PVS): only search the first/best move with full window.
            let mut score: i32;
            if move_idx == 0 {
                score = -self.search::<IS_PV>(depth - 1, ply + 1, -beta, -alpha);
            } else {
                score = -self.search::<false>(depth - 1, ply + 1, -alpha - 1, -alpha); // Null window search

                if IS_PV && score > alpha && score < beta {
                    score = -self.search::<true>(depth - 1, ply + 1, -beta, -alpha); // Fail high --> research
                }
            }

            // 6 - Unmake move
            self.board.unmake_move(m);

            // 7 - Update alpha, beta, and principal variation
            if score >= beta {
                if !m.is_capture() {
                    self.killers[ply][1] = self.killers[ply][0];
                    self.killers[ply][0] = m;
                }
                return beta; // Fail-high, beta cutoff
            }
            if score > alpha {
                alpha = score;
                self.pv_table[ply][ply] = m; // Update the PV for the current ply

                // Copy the PV from the next ply into this ply's row
                let next_ply = ply + 1;
                assert!(ply + 1 < 64);
                let child_len = self.pv_length[next_ply];
                for i in 0..child_len {
                    self.pv_table[ply][ply + 1 + i] = self.pv_table[next_ply][next_ply + i];
                }
                self.pv_length[ply] = child_len + 1;

                if ply == 0 {
                    self.best_move = m;
                }
            }
        }

        // Checkmate & stalemate detection
        if moves.count() == 0 {
            return if self.board.king_in_check(self.board.side_to_move()) {
                -SCORE_MATE + (ply as i32) // Checkmate in N
            } else {
                0 // Stalemate
            };
        }

        alpha
    }

    fn quiescence(&mut self, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;

        let in_check = self.board.king_in_check(self.board.side_to_move());
        if !in_check {
            let eval = self.board.evaluate_relative();
            if eval >= beta {
                return beta;
            }
            if alpha < eval {
                alpha = eval;
            }
        }

        let mut moves = MoveList::new();
        generate_all_captures(self.board, &mut moves);
        self.order_moves::<true>(&mut moves, ply);

        for m in moves.iter() {
            self.board.make_move(m);
            if self.board.king_in_check(!self.board.side_to_move()) {
                self.board.unmake_move(m);
                continue;
            }

            let score = -self.quiescence(ply + 1, -beta, -alpha);
            self.board.unmake_move(m);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn score_move<const QUIESCENCE: bool>(&self, m: Move, ply: usize) -> i32 {
        // 1 - PV Move gets highest priority
        if !QUIESCENCE && m == self.pv_table[ply][ply] {
            return 20000;
        }

        // 2 - Captures (most valuable victim - least valuable attacker)
        if m.is_capture() || m.is_enpassant() {
            let attacker = self.board.piece_on_unchecked(m.from()).get_type();
            let victim = if m.is_enpassant() { PieceType::Pawn } else { self.board.piece_on_unchecked(m.to()).get_type() };

            // Formula: (Victim * 100) - Attacker.
            // A Pawn (1) taking a Queen (5) = 900 - 1 = 899 (High priority)
            // A Queen (9) taking a Pawn (1) = 100 - 9 = 91 (Lower priority)
            return (piece_value(victim) * 100) - piece_value(attacker);
        }

        // 3 - Killer moves
        if !QUIESCENCE && m == self.killers[ply][0] {
            return 9000;
        } else if !QUIESCENCE && m == self.killers[ply][1] {
            return 8000;
        }

        0 // Quiet moves
    }

    /// Move ordering logic
    #[inline(always)]
    fn order_moves<const QUIESCENCE: bool>(&self, moves: &mut MoveList, ply: usize) {
        // We don't need to sort the whole list at once. Just calculate scores and let the search pick the best one.
        let mut scores = [0; 256];
        for i in 0..moves.count() {
            scores[i] = self.score_move::<QUIESCENCE>(moves.get(i), ply);
        }

        // Simple selection sort to move best scores to the front
        for i in 0..moves.count() {
            for j in i + 1..moves.count() {
                if scores[j] > scores[i] {
                    scores.swap(i, j);
                    moves.swap(i, j);
                }
            }
        }
    }
}

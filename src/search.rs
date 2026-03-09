use crate::board::Board;
use crate::movegen::{MoveList, generate_all_captures, generate_all_moves};
use crate::moves::Move;
use crate::types::{Color, PieceKind, Square};

const SCORE_INF: i32 = 32_000;
const SCORE_MATE: i32 = 29_000;

pub struct Searcher<'a> {
    board: &'a mut Board,

    best_move: Move,
    nodes: u64,

    pv_table: [[Move; 64]; 64],
    pv_length: [usize; 64],

    killers: [[Move; 2]; 64],

    lmr_table: [[usize; 64]; 64],  // Late Move Reductions (LMR) table
    history: [[[i32; 64]; 64]; 2], // History heuristics, [side][from][to]
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

            lmr_table: Self::init_lmr_table(),
            history: [[[0; 64]; 64]; 2],
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

        let side = self.board.side_to_move();
        let eval = self.board.evaluate_relative();
        let in_check = self.board.king_in_check(side);

        // 1 - Target depth reached, quiescence search.
        if depth == 0 {
            return self.quiescence(ply, alpha, beta);
        }

        // 2 - Futility pruning startup
        let mut prune_quiet_moves = false;
        if !IS_PV && depth <= 3 && !in_check && eval + 80 * depth as i32 <= alpha {
            prune_quiet_moves = true;
        }

        // 3 - Generate all moves and score them.
        let mut moves = MoveList::new();
        let mut scores = [0i32; 256];
        generate_all_moves(self.board, &mut moves);
        self.score_moves::<false>(&moves, ply, &mut scores);

        // 4 - Null move pruning
        if !IS_PV && depth >= 3 && !in_check {
            self.board.make_null_move();
            let reduction = if depth > 6 { 3 } else { 2 };
            let score = -self.search::<false>(depth - 1 - reduction, ply + 1, -beta, -beta + 1);
            self.board.unmake_null_move();

            if score >= beta {
                return beta;
            }
        }

        // 5 - Iterate over possible moves.
        let mut legal_move_count = 0; // Flag used for mate and stalemate detection
        for move_idx in 0..moves.count() {
            self.pick_best_move(&mut moves, &mut scores, move_idx);
            let m = moves.get(move_idx);

            let (is_capture, is_promotion) = (m.is_capture(), m.is_promotion());

            // 6 - Futility pruning
            if !IS_PV && prune_quiet_moves && move_idx > 3 && !is_capture && !is_promotion {
                continue;
            }

            // 7 - Make move, undo and continue if illegal.
            self.board.make_move(m);
            let in_check = self.board.king_in_check(side);
            if in_check {
                self.board.unmake_move(m);
                continue;
            }
            legal_move_count += 1;

            // 8 - Late Move Reductions
            let mut reduction = 0usize;
            let gives_check = self.board.king_in_check(!side);
            if !IS_PV
                && depth >= 3
                && move_idx >= 3
                && !is_capture
                && !is_promotion
                && !gives_check
                && m != self.killers[ply][0]
                && m != self.killers[ply][1]
            {
                reduction = self.lmr_table[depth.min(63)][move_idx.min(63)];
            }
            let reduced_depth = depth.saturating_sub(reduction + 1);

            // 9 - Principal Variation Search (PVS): only search the first/best move with full window.
            let mut score: i32;
            if IS_PV {
                if move_idx == 0 {
                    score = -self.search::<IS_PV>(depth - 1, ply + 1, -beta, -alpha); // Full window
                } else {
                    score = -self.search::<false>(reduced_depth, ply + 1, -alpha - 1, -alpha); // Null window

                    if score > alpha && beta - alpha > 1 {
                        score = -self.search::<false>(depth - 1, ply + 1, -beta, -alpha); // Fail high -> research
                    }
                }
            } else {
                // Non-PV node --> always null-window, no branch
                score = -self.search::<false>(reduced_depth, ply + 1, -alpha - 1, -alpha);
                if score > alpha && beta - alpha > 1 {
                    score = -self.search::<false>(depth - 1, ply + 1, -beta, -alpha);
                }
            }

            // 10 - Unmake move
            self.board.unmake_move(m);

            // 11 - Update alpha, beta, PV-table and history
            if score >= beta {
                if !is_capture && !is_promotion {
                    self.killers[ply][1] = self.killers[ply][0];
                    self.killers[ply][0] = m; // At this depth, this move often refutes the position

                    self.update_history(side, m.from(), m.to(), depth); // Globally, this move caused cutoffs
                }
                return beta; // Fail-high, beta cutoff
            }
            if score > alpha {
                alpha = score;
                self.pv_table[ply][ply] = m; // Update the PV for the current ply

                // Copy the PV from the next ply into this ply's row
                let next_ply = ply + 1;
                let child_len = self.pv_length[next_ply];
                for i in 0..child_len {
                    self.pv_table[ply][ply + 1 + i] = self.pv_table[next_ply][next_ply + i];
                }
                self.pv_length[ply] = child_len + 1;

                if ply == 0 {
                    self.best_move = m;
                }
            } else if !is_capture {
                self.history[side][m.from()][m.to()] -= (depth * depth) as i32; // Failed to improve alpha --> history malus
            }
        }

        // 12 - Checkmate & stalemate detection
        if legal_move_count == 0 {
            return if self.board.king_in_check(side) {
                -SCORE_MATE + (ply as i32) // Checkmate in N
            } else {
                0 // Stalemate
            };
        }

        alpha
    }

    /// Performs quiescence search.
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
        let mut scores = [0i32; 256];
        generate_all_captures(self.board, &mut moves);
        self.score_moves::<false>(&moves, ply, &mut scores);

        for move_idx in 0..moves.count() {
            self.pick_best_move(&mut moves, &mut scores, move_idx);
            let m = moves.get(move_idx);

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

    /// Assigns each move a score depending on how promising it is.
    #[inline(always)]
    fn score_moves<const QUIESCENCE: bool>(&self, moves: &MoveList, ply: usize, scores: &mut [i32; 256]) {
        let n = moves.count();

        for i in 0..n {
            scores[i] = self.score_move::<QUIESCENCE>(moves.get(i), ply);
        }
    }

    /// Assigns a score to a specific move. Uses PV-table, MVV-LVA and killer move heuristics.
    #[inline(always)]
    fn score_move<const QUIESCENCE: bool>(&self, m: Move, ply: usize) -> i32 {
        // 1 - PV Move gets highest priority
        if !QUIESCENCE && m == self.pv_table[ply][ply] {
            return 20000;
        }

        // 2 - Captures, MVV-LVA (most valuable victim - least valuable attacker)
        if m.is_capture() || m.is_enpassant() {
            let attacker = self.board.piece_on_unchecked(m.from()).kind();
            let victim = if m.is_enpassant() { PieceKind::Pawn } else { self.board.piece_on_unchecked(m.to()).kind() };

            // Formula: 10_000 + (Victim * 100) - Attacker.
            // A Pawn (1) taking a Queen (5) = 900 - 1 = 899 (High priority)
            // A Queen (9) taking a Pawn (1) = 100 - 9 = 91 (Lower priority)
            // 10k is added so that captures scores better than a killer move (which is 9000).
            return 10_000 + (Self::piece_value(victim) * 100) - Self::piece_value(attacker);
        }

        if !QUIESCENCE {
            // 3 - Killer moves
            if m == self.killers[ply][0] {
                return 9000;
            } else if m == self.killers[ply][1] {
                return 8000;
            }
            // 4 - History heuristics
            else {
                return self.history[self.board.side_to_move()][m.from()][m.to()] >> 1; // Divided by 2 so that (0-16384) --> (0-8192)
            }
        }

        0
    }

    /// Returns the value of a piece for move ordering. Should be optimized by the compiler.
    #[inline(always)]
    fn piece_value(piece_type: PieceKind) -> i32 {
        match piece_type {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 0, // Dummy
        }
    }

    /// Picks the best move among the remaining ones (start_idx..last_idx) and places it at start_idx.
    #[inline(always)]
    fn pick_best_move(&self, moves: &mut MoveList, scores: &mut [i32; 256], start_idx: usize) {
        let mut best_idx = start_idx;
        for i in (start_idx + 1)..moves.count() {
            if scores[i] > scores[best_idx] {
                best_idx = i;
            }
        }
        moves.swap(start_idx, best_idx);
        scores.swap(start_idx, best_idx);
    }

    /// Updates the history table when a move causes a beta cutoff. The used formula prevents overflow and slowly saturates to `HISTORY_MAX`.
    #[inline(always)]
    fn update_history(&mut self, side: Color, from: Square, to: Square, depth: usize) {
        const HISTORY_MAX: i32 = 1 << 14; // Fast division by power of 2
        let bonus = (depth * depth) as i32;
        let entry = &mut self.history[side][from][to];
        *entry += bonus - (*entry * bonus / HISTORY_MAX);
    }

    /// Initializes the Late Move Reduction (LMR) table.
    ///
    /// The table stores the number of plies by which the search depth should be reduced for late moves in the move ordering.
    /// It is indexed by `[depth][move_number]`. Reductions grow logarithmically with both the current search depth and the move index,
    /// meaning deeper nodes and later moves receive stronger reductions. This allows the engine to search promising moves fully
    /// while saving time on less likely candidates.
    ///
    /// The table is computed once at startup and reused during search.
    fn init_lmr_table() -> [[usize; 64]; 64] {
        let mut table = [[0usize; 64]; 64];

        for depth in 1..64 {
            for move_number in 1..64 {
                let d = depth as f64;
                let m = move_number as f64;

                let reduction = (d.ln() * m.ln() / 2.0) as usize;

                table[depth][move_number] = reduction;
            }
        }

        table
    }
}

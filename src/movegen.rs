use crate::moves::Move;

/// Includes the list of moves generated for each position. It was found that certain position
/// can reach up to ~200 legal moves, hence the rounding to 256.
pub struct MoveList {
    moves: [Move; 256],
    count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self { moves: [Move::NULL_MOVE; 256], count: 0 }
    }

    pub fn push(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    }
}

//! Masca - Rust chess engine in the making
//!
//! # Overview
//! This crate implements a complete chess engine written in (mostly) safe Rust.
//!
//! # Architecture
//! - `bitboard.rs`: low-level bitboard definition
//! - `board.rs`: chessboard representation
//! - `movegen.rs`: move generation
//! - `attack.rs`: attack tables generation on startup
//! - `magics.rs`: sliding piece attack generation on startup
//! - `moves.rs`: low-level move representation

mod attack;
mod bitboard;
mod board;
mod magics;
mod movegen;
mod moves;
mod perft;
mod search;
mod types;

use crate::board::Board;
use crate::search::Searcher;

fn main() {
    //benchmark_perft(6);
    let mut board = Board::new();
    board.from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1P/PPPB1PP1/R3K2R w KQkq - 0 1").unwrap();
    //  board.set_startpos();
    board.print();

    let mut searcher = Searcher::new(&mut board);
    searcher.iterative_deepening(11);
}

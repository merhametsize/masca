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
mod eval;
mod magics;
mod movegen;
mod moves;
mod perft;
mod search;
mod types;

use crate::board::Board;
use crate::search::Searcher;

#[allow(dead_code)]
const WAC: &str = "r1bqk2r/pppp1ppp/2n2n2/1B2p3/3PP3/2N2N2/PPP2PPP/R1BQK2R w KQkq - 0 1";
#[allow(dead_code)]
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1P/PPPB1PP1/R3K2R w KQkq - 0 1";
#[allow(dead_code)]
const QUIET_SACRIFICE: &str = "r2q1rk1/pp3ppp/2nb1n2/3pp3/3P4/2PBPN2/PP3PPP/RNBQ1RK1 w - - 0 1";
#[allow(dead_code)]
const ZUGZWANG: &str = "8/8/8/3k4/3P4/3K4/8/8 w - - 0 1";
#[allow(dead_code)]
const TACTICAL_MATE: &str = "r1b1k2r/pppp1ppp/2n2n2/1B2p3/4P3/2N5/PPPP1PPP/R1BQK2R w kq - 0 1";
#[allow(dead_code)]
const BOH: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";

fn main() {
    //benchmark_perft(6);
    let mut board = Board::new();
    //board.from_fen(BOH).unwrap();
    board.set_startpos();
    board.print();

    let mut searcher = Searcher::new(&mut board);
    searcher.iterative_deepening(15);
}

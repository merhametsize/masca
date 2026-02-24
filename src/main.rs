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
mod types;

use perft::{kiwipete, perft_n};

use crate::perft::benchmark_perft;

fn main() {
    benchmark_perft(6);
}

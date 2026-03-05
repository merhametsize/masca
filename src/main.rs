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

use crate::perft::{benchmark_perft, perft_n};

fn main() {
    //benchmark_perft(6);
    perft_n(6);
}

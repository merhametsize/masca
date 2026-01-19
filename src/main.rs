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
//! - `moves.rs`: low-level move representation

mod attack;
mod bitboard;
mod board;
mod magics;
mod movegen;
mod moves;
mod types;

use board::Board;

fn main() {
    let mut board = Board::new();
    board.set_startpos();
    board.print();

    let atk = attack::init_attack_tables();
}

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
mod types;

use board::Board;

use crate::magics::MagicTables;

fn main() {
    let attack_tables = attack::AttackTables::new();
    attack_tables.print();
}

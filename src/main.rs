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

use crate::movegen::{Alfè, Caval, MoveList, generate_moves, generate_pawn_captures, generate_pawn_quiets};

fn main() {
    let mut board = Board::new();
    board.set_startpos();

    let attack_tables = attack::AttackTables::new();

    let mut moves;

    moves = MoveList::new();
    generate_moves::<Caval, true, false>(&board, &attack_tables, &mut moves);
    println!("White knight quiet moves: {}", moves.count());

    moves = MoveList::new();
    generate_moves::<Caval, true, true>(&board, &attack_tables, &mut moves);
    println!("White knight captures: {}", moves.count());

    moves = MoveList::new();
    generate_moves::<Alfè, true, false>(&board, &attack_tables, &mut moves);
    println!("White bishop quiet moves: {}", moves.count());

    moves = MoveList::new();
    generate_pawn_quiets::<true>(&board, &attack_tables, &mut moves);
    println!("White pawn quiet moves: {}", moves.count());

    moves = MoveList::new();
    generate_pawn_captures::<true>(&board, &attack_tables, &mut moves);
    println!("White pawn capture: {}", moves.count());
}

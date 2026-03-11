//! Masca - Rust chess engine in the making
//!
//! # Overview
//! This crate implements a complete chess engine written in (mostly) safe Rust.
//!
//! # Architecture
//!
//! The engine is organized into several core modules and subfolders:
//!
//! - **bitboard.rs** — Low-level bitboard structures and operations.
//! - **board.rs** — Chessboard state representation and board-level utilities.
//! - **movegen.rs** — Legal move generation for all pieces.
//! - **attack.rs** — Precomputed attack tables for fast move and threat evaluation.
//! - **magics.rs** — Sliding piece attack generation using magic bitboards at startup.
//! - **moves.rs** — Low-level move encoding, decoding, and manipulation.
//! - **types/** — Core type definitions and enums used across the engine.

mod attack;
mod board;
mod eval;
mod fen;
mod magics;
mod movegen;
mod perft;
mod search;
mod types;
mod uci;
mod zobrist;

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
    use crate::uci::Uci;
    let mut uci = Uci::new();
    uci.run();
}

//! Perft testing module.
//!
//! This module implements the **perft (performance test)** routine, which is used to
//! validate move generation correctness by counting the number of leaf nodes reachable
//! from a given position at a specified search depth.
//!
//! Perft works by recursively generating pseudo-legal moves, filtering illegal moves
//! by checking king safety after each move, and summing the number of reachable nodes.

use std::time::Instant;

use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::generate_all_moves;

pub fn benchmark_perft(depth: u64) {
    let mut board = Board::new();
    board.set_startpos();

    const RUNS: usize = 5;

    let mut nodes_vec: Vec<u64> = Vec::new();
    let mut time_vec: Vec<f64> = Vec::new();

    // Warmup run
    let _ = perft(&mut board, depth);

    for _ in 0..RUNS {
        let mut board_clone = Board::new();
        board_clone.set_startpos();

        let start = Instant::now();
        let nodes = perft(&mut board_clone, depth);
        let elapsed = start.elapsed().as_secs_f64();

        nodes_vec.push(nodes);
        time_vec.push(elapsed);
    }

    // Sort times to remove fastest and slowest run
    time_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Remove fastest and slowest
    let trimmed_times = &time_vec[1..RUNS - 1];

    if trimmed_times.is_empty() {
        panic!("Not enough runs for stable benchmark");
    }

    let avg_time: f64 = trimmed_times.iter().sum::<f64>() / trimmed_times.len() as f64;
    assert!(nodes_vec.iter().all(|&n| n == nodes_vec[0]));
    let nodes = nodes_vec[0];

    let nps = if avg_time > 0.0 { nodes as f64 / avg_time } else { 0.0 };

    println!("Depth: {}", depth);
    println!("Nodes: {:.2}", nodes);
    println!("Avg Time: {:.6} s", avg_time);
    println!("Avg NPS: {:.2} nodes/sec", nps);
}

pub fn perft(board: &mut Board, depth: u64) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut list = MoveList::new();
    generate_all_moves(board, &mut list);

    let mut nodes = 0;

    for m in list.iter() {
        board.make_move(*m);

        if !board.king_in_check(!board.side_to_move()) {
            nodes += perft(board, depth - 1);
        }

        board.unmake_move(*m);
    }

    nodes
}

pub fn perft_n(depth: u64) -> u64 {
    let mut board = Board::new();
    board.set_startpos();

    board.print();

    let mut nodes: u64 = 0;
    for depth in 1..=depth {
        nodes += perft(&mut board, depth);
        println!("perft({}): {}", depth, nodes);
    }

    nodes
}

pub fn kiwipete(depth: u64) {
    let mut board = Board::new();
    let kiwipete_pos = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    board.from_fen(kiwipete_pos).unwrap();

    board.print();

    for depth in 1..=depth {
        let nodes = perft(&mut board, depth);
        println!("perft({}): {}", depth, nodes);
    }
}

//! UCI (Universal Chess Interface) communication layer.
//!
//! This module implements the protocol used by chess GUIs to communicate with engines.
//! It parses commands from stdin and triggers board updates and search accordingly.

use std::io::{self, BufRead};

use crate::board::Board;
use crate::movegen::{MoveList, generate_all_moves};
use crate::moves::Move;
use crate::search::Searcher;

/// UCI controller.
///
/// Owns the board and manages the command loop handling communication with the GUI.
pub struct Uci {
    board: Board,
}

impl Uci {
    /// Creates a new UCI controller with an empty board.
    pub fn new() -> Self {
        Self { board: Board::new() }
    }

    /// Runs the blocking UCI command loop.
    ///
    /// Reads commands from stdin and dispatches them to the appropriate handlers.
    pub fn run(&mut self) {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let mut parts = line.split_whitespace();

            let cmd = match parts.next() {
                Some(c) => c,
                None => continue,
            };

            match cmd {
                "uci" => self.cmd_uci(),
                "isready" => self.cmd_isready(),
                "ucinewgame" => self.cmd_ucinewgame(),
                "position" => self.cmd_position(parts.collect()),
                "go" => self.cmd_go(parts.collect()),
                "quit" => break,
                "stop" => {}               // TODO
                "d" => self.board.print(), // Debug command
                _ => {}
            }
        }
    }

    /// Handles the `uci` command.
    ///
    /// Sends engine identification and signals UCI readiness.
    fn cmd_uci(&self) {
        println!("id name Masca");
        println!("id author Gabriele Cassetta @merhametsize");
        println!("uciok");
    }

    /// Handles the `isready` command.
    ///
    /// Used by the GUI to ensure the engine finished initialization.
    fn cmd_isready(&self) {
        println!("readyok");
    }

    /// Handles the `ucinewgame` command.
    ///
    /// Resets internal state for a new game.
    fn cmd_ucinewgame(&mut self) {
        self.board.set_startpos();
    }

    /// Handles the `position` command.
    ///
    /// Sets the board from `startpos` or `fen`, then optionally applies a move list.
    fn cmd_position(&mut self, args: Vec<&str>) {
        let mut i = 0;

        if args.is_empty() {
            return;
        }

        // Handle "startpos"
        if args[i] == "startpos" {
            self.board.set_startpos();
            i += 1;
        }
        // Handle "fen"
        else if args[i] == "fen" {
            if args.len() < 7 {
                return;
            }

            let fen = args[i + 1..i + 7].join(" ");
            self.board.from_fen(&fen).unwrap();
            i += 7;
        }

        // Apply moves if present
        if i < args.len() && args[i] == "moves" {
            i += 1;

            while i < args.len() {
                let m = Self::parse_move(&self.board, args[i]);
                self.board.make_move(m);
                i += 1;
            }
        }
    }

    /// Handles the `go` command.
    ///
    /// Starts a search. Currently supports only `go depth N`.
    fn cmd_go(&mut self, args: Vec<&str>) {
        let mut depth = 10;

        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "depth" => {
                    if i + 1 < args.len() {
                        depth = args[i + 1].parse().unwrap_or(depth);
                    }
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }

        let mut searcher = Searcher::new(&mut self.board);

        searcher.iterative_deepening(depth);

        println!("bestmove {}", searcher.best_move);
    }

    /// Parses a UCI move string by matching it against generated legal moves.
    ///
    /// This guarantees the correct internal move encoding (captures, promotions, castling, etc.).
    fn parse_move(board: &Board, move_str: &str) -> Move {
        let mut moves = MoveList::new();

        generate_all_moves(board, &mut moves);

        for i in 0..moves.count() {
            let m = unsafe { moves.get_unchecked(i) };

            if m.to_string() == move_str {
                return m;
            }
        }

        panic!("Illegal move received from UCI: {}", move_str);
    }
}

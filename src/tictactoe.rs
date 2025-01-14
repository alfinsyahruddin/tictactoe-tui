use std::{cmp::max, cmp::min, collections::HashMap};

use rand::seq::IteratorRandom;

use crate::{
    constants::BOARD_SIZE,
    entities::{Board, GameResult, Player},
};

#[derive(Debug)]
pub struct TicTacToe {}

impl TicTacToe {
    pub fn get_best_move(
        board: &Board,
        player: Player,
        is_maximizing: bool,
        depth: i32,
        nodes_map: &mut HashMap<i32, Vec<i32>>,
    ) -> i32 {
        let max_depth = -1;

        // If the board state is a terminal one, return the heuristic value
        let result = TicTacToe::get_game_result(&board);
        if result != GameResult::Playing || depth == max_depth {
            if result == GameResult::Win(player.clone()) {
                return 100 - depth;
            } else if result == GameResult::Win(player.get_opponent()) {
                return -100 + depth;
            } else {
                return 0;
            }
        }

        if is_maximizing {
            // Initialize best to the lowest possible value
            let mut best = -100;

            // Loop through all empty cells
            let available_moves = TicTacToe::get_available_moves(&board);
            for index in available_moves {
                let mut board_2 = board.clone();
                board_2[index] = player.clone();

                let node_value =
                    TicTacToe::get_best_move(&board_2, player.clone(), false, depth + 1, nodes_map);

                best = max(best, node_value);

                // If it's the main function call, not a recursive one, map each heuristic value with it's moves indices
                if depth == 0 {
                    //Comma separated indices if multiple moves have the same heuristic value
                    let moves: Vec<i32> = if let Some(moves) = nodes_map.get(&node_value).clone() {
                        let mut moves = moves.clone();
                        moves.push(index as i32);
                        moves
                    } else {
                        vec![index as i32]
                    };
                    nodes_map.insert(node_value, moves);
                }
            }

            // If it's the main call, return the index of the best move or a random index if multiple indices have the same value
            if depth == 0 {
                let moves = nodes_map.get(&best).unwrap().clone();
                let return_value: i32;

                if moves.len() > 1 {
                    return_value = moves
                        .iter()
                        .choose(&mut rand::thread_rng())
                        .unwrap_or(&0i32)
                        .clone();
                } else {
                    return_value = moves[0] as i32;
                }

                return return_value;
            }

            // If not main call (recursive) return the heuristic value for next calculation
            return best;
        }

        if !is_maximizing {
            // Initialize best to the lowest possible value
            let mut best = 100;

            // Loop through all empty cells
            let available_moves = TicTacToe::get_available_moves(&board);
            for index in available_moves {
                let mut board_2 = board.clone();
                board_2[index] = player.get_opponent().clone();

                let node_value =
                    TicTacToe::get_best_move(&board_2, player.clone(), true, depth + 1, nodes_map);

                best = min(best, node_value);

                // If it's the main function call, not a recursive one, map each heuristic value with it's moves indices
                if depth == 0 {
                    //Comma separated indices if multiple moves have the same heuristic value
                    let moves: Vec<i32> = if let Some(moves) = nodes_map.get(&node_value).clone() {
                        let mut moves = moves.clone();
                        moves.push(index as i32);
                        moves
                    } else {
                        vec![index as i32]
                    };
                    nodes_map.insert(node_value, moves);
                }
            }

            // If it's the main call, return the index of the best move or a random index if multiple indices have the same value
            if depth == 0 {
                let moves = nodes_map.get(&best).unwrap().clone();
                let return_value: i32;

                if moves.len() > 1 {
                    return_value = moves
                        .iter()
                        .choose(&mut rand::thread_rng())
                        .unwrap_or(&0i32)
                        .clone();
                } else {
                    return_value = moves[0] as i32;
                }

                return return_value;
            }

            // If not main call (recursive) return the heuristic value for next calculation
            return best;
        }

        return 0;
    }

    pub fn get_available_moves(board: &Board) -> Vec<usize> {
        board
            .iter()
            .enumerate()
            .filter(|x| x.1 == &Player::None)
            .map(|x| x.0)
            .collect::<Vec<usize>>()
    }

    pub fn get_game_result(board: &Board) -> GameResult {
        if TicTacToe::is_empty(&board) {
            return GameResult::Playing;
        }

        // 0 1 2
        // 3 4 5
        // 6 7 8

        // Check Horizontal Wins
        if &board[0] != &Player::None && &board[0] == &board[1] && &board[0] == &board[2] {
            return GameResult::Win(board[0].clone());
        }

        if &board[3] != &Player::None && &board[3] == &board[4] && &board[3] == &board[5] {
            return GameResult::Win(board[3].clone());
        }

        if &board[6] != &Player::None && &board[6] == &board[7] && &board[6] == &board[8] {
            return GameResult::Win(board[6].clone());
        }

        // Check Vertical Wins
        if &board[0] != &Player::None && &board[0] == &board[3] && &board[0] == &board[6] {
            return GameResult::Win(board[0].clone());
        }

        if &board[1] != &Player::None && &board[1] == &board[4] && &board[1] == &board[7] {
            return GameResult::Win(board[1].clone());
        }

        if &board[2] != &Player::None && &board[2] == &board[5] && &board[2] == &board[8] {
            return GameResult::Win(board[2].clone());
        }

        // Check Diagonal Wins
        if &board[0] != &Player::None && &board[0] == &board[4] && &board[0] == &board[8] {
            return GameResult::Win(board[0].clone());
        }

        if &board[2] != &Player::None && &board[2] == &board[4] && &board[2] == &board[6] {
            return GameResult::Win(board[2].clone());
        }

        // Draw
        if TicTacToe::is_full(&board) {
            return GameResult::Draw;
        }

        GameResult::Playing
    }

    pub fn get_empty_board() -> Board {
        let size = BOARD_SIZE * BOARD_SIZE;
        (0..size).into_iter().map(|_| Player::None).collect()
    }

    pub fn is_empty(board: &Board) -> bool {
        let count: usize = board
            .iter()
            .map(|x| x != &Player::None)
            .filter(|x| x.clone())
            .collect::<Vec<bool>>()
            .len();
        count == 0
    }

    pub fn is_full(board: &Board) -> bool {
        let count: usize = board
            .iter()
            .map(|x| x == &Player::None)
            .filter(|x| x.clone())
            .collect::<Vec<bool>>()
            .len();
        count == 0
    }
}

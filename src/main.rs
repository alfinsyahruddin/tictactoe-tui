use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::seq::IteratorRandom;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Padding, Widget,
    },
    DefaultTerminal, Frame,
};
use std::{
    cmp::{max, min},
    collections::HashMap,
    io,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

const BOARD_SIZE: u16 = 3;

#[derive(Debug, PartialEq, Clone)]
pub enum Player {
    O,
    X,
    None,
}

type Board = Vec<Player>;

#[derive(Debug, PartialEq)]
pub enum GameResult {
    Playing,
    Win(Player),
    Draw,
}

impl Player {
    fn get_text(&self) -> String {
        match self {
            Player::X => "X".to_string(),
            Player::O => "O".to_string(),
            Player::None => " ".to_string(),
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Player::X => Color::Red,
            Player::O => Color::Green,
            Player::None => Color::DarkGray,
        }
    }

    fn get_opponent(&self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
            Player::None => Player::None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    SelectPlayer,
    Playing,
    GameOver(GameResult),
}

#[derive(Debug)]
pub struct App {
    player: Player,
    game_state: GameState,
    selected_index: u16,
    board: Board,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            player: Player::O,
            game_state: GameState::SelectPlayer,
            selected_index: 0,
            board: App::get_empty_board(),
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_ui(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_ui(&self, frame: &mut Frame) {
        // Container + Title
        self.render_container_ui(frame);

        // Select Player UI
        match self.game_state {
            GameState::SelectPlayer => {
                self.render_select_player_ui(frame);
            }
            GameState::Playing | GameState::GameOver(_) => {
                self.render_playing_ui(frame);
            }
        }
    }

    fn render_container_ui(&self, frame: &mut Frame) {
        let area = frame.area();

        let title = Title::from(" .:: TIC-TAC-TOE ::. ".yellow().bold());
        let instructions: Title = match self.game_state {
            GameState::SelectPlayer => Title::from(Line::from(vec![
                " Press ".into(),
                "<q>".yellow().bold(),
                " to Quit ".into(),
            ])),
            GameState::Playing | GameState::GameOver(_) => Title::from(Line::from(vec![
                " Press ".into(),
                "<q>".yellow().bold(),
                " to Quit | ".into(),
                "<r>".yellow().bold(),
                " to Restart | ".into(),
                "<s>".yellow().bold(),
                " to Select Player ".into(),
            ])),
        };

        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);
        frame.render_widget(block, area);
    }

    fn render_select_player_ui(&self, frame: &mut Frame) {
        let area = frame.area();
        let cell_width = CellWidget::size();
        let cell_height = CellWidget::size() / 2;

        // Select Player
        let title = Text::raw("Select Player:");
        let title_area = Rect::new(
            (area.width / 2) - ((title.width() as u16) / 2),
            (area.height / 2) - 4,
            title.width() as u16,
            title.height() as u16,
        );

        frame.render_widget(title, title_area);

        let o_player = CellWidget {
            player: Player::O,
            is_selected: self.player == Player::O,
            is_winner: false,
        };

        let o_area = Rect::new(
            (area.width / 2) - (cell_width) - 1,
            title_area.y + 2,
            cell_width,
            cell_height,
        );
        frame.render_widget(o_player, o_area);

        let x_player = CellWidget {
            player: Player::X,
            is_selected: self.player == Player::X,
            is_winner: false,
        };
        let x_area = Rect::new(
            (area.width / 2) + 1,
            title_area.y + 2,
            cell_width,
            cell_height,
        );
        frame.render_widget(x_player, x_area);
    }

    fn render_playing_ui(&self, frame: &mut Frame) {
        let area = frame.area();
        let cell_width = CellWidget::size();
        let cell_height = CellWidget::size() / 2;

        // Select Player
        let computer = if self.player == Player::O {
            Player::X
        } else {
            Player::O
        };
        let title: Text = match &self.game_state {
            GameState::GameOver(result) => Text::from(Line::from(vec![match result {
                GameResult::Win(player) => {
                    if player == &self.player {
                        "You Won 🏆".into()
                    } else {
                        "You Lose 😋".into()
                    }
                }
                _ => "Draw 🤝".into(),
            }])),
            _ => Text::from(Line::from(vec![
                "You: ".into(),
                self.player.get_text().fg(self.player.get_color()).bold(),
                " | ".fg(Color::DarkGray),
                "Computer: ".into(),
                computer.get_text().fg(computer.get_color()).bold(),
            ])),
        };

        let title_area = Rect::new(
            (area.width / 2) - ((title.width() as u16) / 2),
            7,
            title.width() as u16,
            title.height() as u16,
        );

        frame.render_widget(title, title_area);

        // Cells
        let total_width = cell_width * BOARD_SIZE;
        let margin_left = (area.width / 2) - (total_width / 2);
        let margin_top = title_area.y + 2;

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let index: u16 = (row * BOARD_SIZE) + col;
                let cell = CellWidget {
                    player: self.board[index as usize].clone(),
                    is_selected: index == self.selected_index,
                    is_winner: false,
                };

                let cell_area = Rect::new(
                    margin_left + (col * cell_width),
                    margin_top + (row * cell_height),
                    cell_width,
                    cell_height,
                );
                frame.render_widget(cell, cell_area);
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Char('q') {
            self.exit = true;
            return;
        }

        match self.game_state {
            GameState::Playing | GameState::GameOver(_) => match key_event.code {
                KeyCode::Char('s') => {
                    self.game_state = GameState::SelectPlayer;
                    self.selected_index = 0;
                    self.board = App::get_empty_board();
                }
                KeyCode::Char('r') => {
                    self.game_state = GameState::Playing;
                    self.selected_index = 0;
                    self.board = App::get_empty_board();
                }
                _ => {}
            },
            _ => {}
        }

        match self.game_state {
            GameState::SelectPlayer => match key_event.code {
                KeyCode::Left => self.player = Player::O,
                KeyCode::Right => self.player = Player::X,
                KeyCode::Enter => {
                    self.game_state = GameState::Playing;
                }
                _ => {}
            },
            GameState::Playing => match key_event.code {
                // 0 1 2
                // 3 4 5
                // 6 7 8
                KeyCode::Left => {
                    self.selected_index = match self.selected_index {
                        0..3 => max(0, self.selected_index.saturating_sub(1)),
                        3..6 => max(3, self.selected_index.saturating_sub(1)),
                        6..9 => max(6, self.selected_index.saturating_sub(1)),
                        _ => self.selected_index,
                    }
                }
                KeyCode::Right => {
                    self.selected_index = match self.selected_index {
                        0..3 => min(2, self.selected_index + 1),
                        3..6 => min(5, self.selected_index + 1),
                        6..9 => min(8, self.selected_index + 1),
                        _ => self.selected_index,
                    }
                }
                KeyCode::Up => {
                    self.selected_index = match self.selected_index % BOARD_SIZE {
                        0 => max(0, self.selected_index.saturating_sub(BOARD_SIZE)),
                        1 => max(1, self.selected_index.saturating_sub(BOARD_SIZE)),
                        2 => max(2, self.selected_index.saturating_sub(BOARD_SIZE)),
                        _ => self.selected_index,
                    }
                }
                KeyCode::Down => {
                    self.selected_index = match self.selected_index % BOARD_SIZE {
                        0 => min(6, self.selected_index + BOARD_SIZE),
                        1 => min(7, self.selected_index + BOARD_SIZE),
                        2 => min(8, self.selected_index + BOARD_SIZE),
                        _ => self.selected_index,
                    }
                }

                KeyCode::Enter => {
                    if self.board[self.selected_index as usize] == Player::None {
                        self.board[self.selected_index as usize] = self.player.clone();

                        self.play_as_computer();
                        self.check_game_state();
                    }
                }

                _ => {}
            },
            GameState::GameOver(_) => {}
        }
    }

    fn play_as_computer(&mut self) {
        if TicTacToe::is_full(&self.board) {
            return;
        }
        let mut nodes_map: HashMap<i32, Vec<i32>> = HashMap::new();
        let index = TicTacToe::get_best_move(
            &self.board,
            self.player.get_opponent(),
            true,
            0,
            &mut nodes_map,
        ) as usize;

        if index <= self.board.len() {
            self.board[index] = self.player.get_opponent();
        }
    }

    fn check_game_state(&mut self) {
        let result = TicTacToe::get_game_result(&self.board);
        if result != GameResult::Playing {
            self.game_state = GameState::GameOver(result);
        }
    }

    fn get_empty_board() -> Board {
        let size = BOARD_SIZE * BOARD_SIZE;
        (0..size).into_iter().map(|_| Player::None).collect()
    }
}

pub struct CellWidget {
    pub player: Player,
    pub is_selected: bool,
    pub is_winner: bool,
}

impl CellWidget {
    fn size() -> u16 {
        10
    }
}

impl Widget for CellWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Container
        let block = Block::bordered()
            .border_style(Style::default().fg(if self.is_selected {
                Color::LightYellow
            } else {
                match (&self.player, self.is_winner) {
                    (Player::X, true) | (Player::O, true) => self.player.get_color(),
                    _ => Color::DarkGray,
                }
            }))
            .padding(Padding::ZERO);

        block.render(area, buf);

        if self.player != Player::None {
            let text = Text::raw(self.player.get_text()).fg(self.player.get_color());
            let center_area = center(area, Constraint::Length(1), Constraint::Length(1));
            text.render(center_area, buf);
        }
    }
}

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

#[derive(Debug)]
pub struct TicTacToe {}

impl TicTacToe {
    fn get_best_move(
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

    fn get_available_moves(board: &Board) -> Vec<usize> {
        board
            .iter()
            .enumerate()
            .filter(|x| x.1 == &Player::None)
            .map(|x| x.0)
            .collect::<Vec<usize>>()
    }

    fn get_game_result(board: &Board) -> GameResult {
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

    fn is_empty(board: &Board) -> bool {
        let count: usize = board
            .iter()
            .map(|x| x != &Player::None)
            .filter(|x| x.clone())
            .collect::<Vec<bool>>()
            .len();
        count == 0
    }

    fn is_full(board: &Board) -> bool {
        let count: usize = board
            .iter()
            .map(|x| x == &Player::None)
            .filter(|x| x.clone())
            .collect::<Vec<bool>>()
            .len();
        count == 0
    }
}

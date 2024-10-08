use constants::{BOARD_SIZE, CELL_SIZE};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use entities::{Board, GameResult, GameState, Player};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block,
    },
    DefaultTerminal, Frame,
};
use std::{
    cmp::{max, min},
    collections::HashMap,
    io,
};
use tictactoe::TicTacToe;
use widgets::cell_widget::CellWidget;

mod constants;
mod entities;
mod helpers;
mod tictactoe;
mod widgets;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app = App::new().run(&mut terminal);
    ratatui::restore();
    app
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
            board: TicTacToe::get_empty_board(),
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
        let cell_width = CELL_SIZE;
        let cell_height = CELL_SIZE / 2;

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
        let cell_width = CELL_SIZE;
        let cell_height = CELL_SIZE / 2;

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
            (area.height / 2) - 8,
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
                    self.board = TicTacToe::get_empty_board();
                }
                KeyCode::Char('r') => {
                    self.game_state = GameState::Playing;
                    self.selected_index = 0;
                    self.board = TicTacToe::get_empty_board();
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
}

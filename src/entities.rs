use ratatui::style::Color;

#[derive(Debug, PartialEq, Clone)]
pub enum Player {
    O,
    X,
    None,
}

pub type Board = Vec<Player>;

#[derive(Debug, PartialEq)]
pub enum GameResult {
    Playing,
    Win(Player),
    Draw,
}

impl Player {
    pub fn get_text(&self) -> String {
        match self {
            Player::X => "X".to_string(),
            Player::O => "O".to_string(),
            Player::None => " ".to_string(),
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Player::X => Color::Red,
            Player::O => Color::Green,
            Player::None => Color::DarkGray,
        }
    }

    pub fn get_opponent(&self) -> Player {
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

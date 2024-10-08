use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, Padding, Widget},
};

use crate::{entities::Player, helpers::center};

pub struct CellWidget {
    pub player: Player,
    pub is_selected: bool,
    pub is_winner: bool,
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

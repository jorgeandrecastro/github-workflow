use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub struct Theme;

impl Theme {
    // Clean, minimalist, and refined palette
    pub const PRIMARY: Color = Color::Rgb(240, 240, 245);    // Soft white / light gray
    pub const ACCENT: Color = Color::Rgb(180, 185, 195);     // Platinum / silver
    pub const SUBTLE: Color = Color::Rgb(90, 95, 105);       // Clean medium gray
    
    #[allow(dead_code)]
    pub const SUCCESS: Color = Color::Rgb(52, 199, 89);      // Muted green
    
    #[allow(dead_code)]
    pub const ERROR: Color = Color::Rgb(255, 59, 48);        // Muted red
    
    pub const WARNING: Color = Color::Rgb(255, 149, 0);      // Muted orange
    pub const SELECTION_BG: Color = Color::Rgb(38, 38, 42);  // Deep anthracite

    /// Creates a block with rounded and elegant borders
    pub fn block(title: &str) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Self::SUBTLE))
            .title(format!(" {} ", title))
            .title_style(Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD))
    }

    /// Elegant style for the selected item
    pub fn selected_item() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .bg(Self::SELECTION_BG)
            .add_modifier(Modifier::BOLD)
    }
}
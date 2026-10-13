use prism::event::OnEvent;
use prism::Context;
use prism::drawable::Component;
use prism::layout::{Row, Offset, Size, Padding};
use prism::canvas::{Image, Align};
use ptsd::theme::TextSize;
use crate::components::text::{Text, TextStyle};
use crate::components::Icon;

use crate::theme::{Theme, Icons};

#[derive(Debug, Component, Clone)]
pub struct Alert(Row, Image, Text);
impl OnEvent for Alert {}
impl Alert {
    pub fn new(theme: &Theme, message: &str) -> Self {
        let color = theme.colors().get(ptsd::Status::Warning);
        let message = Text::new(theme, message, TextSize::Md, TextStyle::Primary, Align::Left, None);
        let icon = Icon::new(theme, Icons::Warning, Some(color), 24.0);
        Alert(Row::new(4.0, Offset::Center, Size::Fit, Padding::new(12.0)), icon, message)
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "Careful, there's a lot you can do.")
    }
}

use std::time::Duration;
use prism::{Context, drawable::{Component, Shape}, drawable::text::Align};
use prism::events::{Timer, OnChange, on_change};
use ptsd::{display::Bin, layout::{Column, Row, Stack, Offset, Size, Padding}};
use crate::{theme::Theme, components::{Circle, Rectangle, text::{Text, TextSize, TextStyle}}};

/// Received-style typing bubble. Pass a name only for group conversations.
#[derive(Clone, Debug, Component)]
pub struct TypingIndicator(Column, TypingBubble, Option<Text>);

impl TypingIndicator {
    pub fn new(theme: &Theme, name: Option<&str>) -> Self {
        Self(Column::start(6.0), TypingBubble::new(theme), name.map(|name| {
            Text::new(theme, &format!("{name} is typing..."), TextSize::Sm,
                TextStyle::Secondary, Align::Left, None).fit()
        }))
    }
}

#[derive(Clone, Debug, Component)]
struct TypingBubble {
    layout: Stack,
    background: Rectangle,
    dots: Bin<Stack, Bin<Row, Vec<Bin<Stack, Shape>>>>,
    #[skip] timer: Timer,
    #[skip] phase: usize,
    #[skip] theme: Theme,
}
impl TypingBubble {
    fn new(theme: &Theme) -> Self {
        let mut bubble = Self {
            layout: Stack::new(Offset::Center, Offset::Center, Size::Static(54.0), Size::Static(32.0), Padding::default()),
            background: Rectangle::new(theme.colors().get(ptsd::Background::Secondary), 32.0/2.0, None),
            dots: Bin(Stack::center(), Bin(Row::center(4.0), vec![])),
            timer: Timer::new(Duration::from_millis(150)), phase: 0, theme: theme.clone(),
        };
        bubble.refresh();
        bubble
    }
    fn refresh(&mut self) {
        self.dots.1.1 = (0..3).map(|index| {
            let color = if index == self.phase {self.theme.colors().get(ptsd::Text::Primary)}
                else {self.theme.colors().get(ptsd::Text::Secondary)};
            Circle::new(6.0, color, false)
        }).collect();
    }
}
#[on_change(self.timer.next())]
impl OnChange<()> for TypingBubble {
    fn on_change(&mut self, _: &mut Context, _: ()) {
        self.phase = (self.phase + 1) % 3;
        self.refresh();
        self.timer.reset(Duration::from_millis(150));
    }
}

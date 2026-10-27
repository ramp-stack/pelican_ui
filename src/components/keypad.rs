use prism::events::{KeyboardState, KeyboardEvent, Key, Modifiers};
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding};
use ptsd::display::Bin;
use prism::drawable::{Drawable, Component};
use prism::drawable::text::Align;
use prism::Context;
use ptsd::interactions;

use crate::theme::{self, Theme, Icons};
use crate::components::Icon;
use crate::components::text::{Text, TextStyle};
use crate::components::button::{ButtonWidth, ButtonSize, Button};

#[derive(Debug, Component, Clone)]
pub struct Keypad(Column, Vec<GhostButtonRow>);

impl Keypad {
    pub fn new(theme: &Theme) -> Self {
        Keypad(Column::center(0.0), vec![
            GhostButtonRow::new(theme, Padding(0.0, 0.0, 0.0, 8.0), vec![
                KeypadButton::char('1'), KeypadButton::char('2'), KeypadButton::char('3'),
            ]),
            GhostButtonRow::new(theme, Padding(0.0, 8.0, 0.0, 8.0), vec![
                KeypadButton::char('4'), KeypadButton::char('5'), KeypadButton::char('6'),
            ]),
            GhostButtonRow::new(theme, Padding(0.0, 8.0, 0.0, 8.0), vec![
                KeypadButton::char('7'), KeypadButton::char('8'), KeypadButton::char('9'),
            ]),
            GhostButtonRow::new(theme, Padding(0.0, 8.0, 0.0, 0.0), vec![
                KeypadButton::char('.'), KeypadButton::char('0'), KeypadButton::delete(),
            ]),
        ])
    }
}

#[derive(Debug, Component, Clone)]
pub struct GhostButtonRow(Row, Vec<GhostButton>);

impl GhostButtonRow {
    pub fn new(theme: &Theme, padding: Padding, data: Vec<(Option<char>, Option<Icons>, Key)>) -> Self {
        GhostButtonRow(Row::center(16.0), data.into_iter().map(|(c, i, key)| {
            let label = c.map(|character| character.to_string());
            GhostButton::new(theme, label.as_deref(), i, padding, move |ctx: &mut Context, _: &Theme| {
                ctx.emit(KeyboardEvent {
                    state: KeyboardState::Pressed, key: key.clone(), modifiers: Modifiers::default(),
                });
            })
        }).collect::<Vec<GhostButton>>())
    }
}

struct KeypadButton;

impl KeypadButton {
    fn char(c: char) -> (Option<char>, Option<Icons>, Key) {
        (Some(c), None, Key::Character(c.to_string()))
    }

    fn delete() -> (Option<char>, Option<Icons>, Key) {(None, Some(Icons::Back), Key::Delete)}
}

#[derive(Debug, Component, Clone)]
struct GhostButton(Stack, interactions::Button);
impl GhostButton {
    fn new(
        theme: &Theme, label: Option<&str>, icon: Option<Icons>, padding: Padding,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
    ) -> Self {
        let colors = theme::Button::get(theme.colors(), theme::Variant::Ghost);
        let default = {
            let font_size = ButtonSize::Large.font();
            let icon_size = ButtonSize::Large.icon();
            let mut drawables: Vec<Box<dyn Drawable>> = Vec::new();
            if let Some(label) = label {
                drawables.push(Box::new(Text::new(
                    theme, label, font_size, TextStyle::Label(colors.default.label), Align::Left, None,
                ).fit()));
            }
            if let Some(icon) = icon {
                drawables.push(Box::new(Icon::new(theme, icon, Some(colors.default.label), icon_size)));
            }
            Button::new(
                drawables, ButtonSize::Large, ButtonWidth::Fill, Offset::Center,
                colors.default.background, colors.default.outline,
            )
        };

        let default = Bin(Stack::new(
            Offset::Center, Offset::Center, Size::Fit, Size::Fit, padding,
        ), default);
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        GhostButton(Stack::default(), interactions::Button::new(
            default, None::<Bin<Stack, Button>>, None::<Bin<Stack, Button>>,
            None::<Bin<Stack, Button>>, None::<Bin<Stack, Button>>, callback, false,
        ))
    }
}

use prism::event::{OnEvent, KeyboardState, KeyboardEvent, NamedKey, Key};
use prism::layout::{Stack, Column, Row, Offset};
use prism::drawable::{Drawable, Component};
use prism::canvas::Align;
use prism::{Context, Request};

use ptsd::interactions;

use crate::theme::{self, Theme, Icons};
use crate::components::Icon;
use crate::components::text::{Text, TextStyle};
use crate::components::button::{ButtonWidth, ButtonSize, Button};

#[derive(Debug, Component, Clone)]
pub struct Keypad(Column, Vec<GhostButtonRow>);
impl OnEvent for Keypad {}

impl Keypad {
    pub fn new(theme: &Theme, special: char) -> Self {
        Keypad(Column::center(16.0), vec![
            GhostButtonRow::new(theme, vec![KeypadButton::char('1'), KeypadButton::char('2'), KeypadButton::char('3')]),
            GhostButtonRow::new(theme, vec![KeypadButton::char('4'), KeypadButton::char('5'), KeypadButton::char('6')]),
            GhostButtonRow::new(theme, vec![KeypadButton::char('7'), KeypadButton::char('8'), KeypadButton::char('9')]),
            GhostButtonRow::new(theme, vec![KeypadButton::char(special), KeypadButton::char('0'), KeypadButton::delete()]),
        ])
    }
}


struct KeypadButton;

impl KeypadButton {
    pub fn char(c: char) -> (Option<char>, Option<Icons>, Key) {
        (Some(c), None, Key::Character(c.to_string().as_str().into()))
    }

    pub fn delete() -> (Option<char>, Option<Icons>, Key) {
        (None, Some(Icons::Back), Key::Named(NamedKey::Delete))
    }
}

#[derive(Debug, Component, Clone)]
pub struct GhostButtonRow(Row, Vec<GhostButton>);
impl OnEvent for GhostButtonRow {}

impl GhostButtonRow {
    pub fn new(theme: &Theme, data: Vec<(Option<char>, Option<Icons>, Key)>) -> Self {
        GhostButtonRow(Row::center(16.0), data.into_iter().map(|(c, i, key)| {
            let label = c.map(|character| character.to_string());
            GhostButton::new(theme, label.as_deref(), i, move |ctx: &mut Context, _: &Theme| {
                ctx.send(Request::Event(Box::new(KeyboardEvent{state: KeyboardState::Pressed, key: key.clone()})));
            })
        }).collect::<Vec<GhostButton>>())
    }
}

#[derive(Debug, Component, Clone)]
struct GhostButton(Stack, pub interactions::Button);
impl OnEvent for GhostButton {}
impl GhostButton {
    fn new(theme: &Theme, label: Option<&str>, icon: Option<Icons>, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), theme::Variant::Ghost);
        let default =  {
            let font_size = ButtonSize::Large.font();
            let icon_size = ButtonSize::Large.icon();
            let mut drawables: Vec<Box<dyn Drawable>> = Vec::new();
            if let Some(l) = label { drawables.push(Box::new(Text::new(theme, l, font_size, TextStyle::Label(colors.default.label), Align::Left, None))); }
            if let Some(i) = icon { drawables.push(Box::new(Icon::new(theme, i, Some(colors.default.label), icon_size))); }
            Button::new(drawables, ButtonSize::Large, ButtonWidth::Fill, Offset::Center, colors.default.background, colors.default.outline)
        };
        
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        GhostButton(Stack::default(), interactions::Button::new(default, None::<Button>, None::<Button>, None::<Button>, callback, false))
    }
}

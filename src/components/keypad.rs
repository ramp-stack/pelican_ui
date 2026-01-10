use prism::event::{OnEvent, KeyboardState, KeyboardEvent, NamedKey, Key};
use prism::layout::{Stack, Column, Row, Offset};
use prism::drawable::{Drawable, Component};
use prism::canvas::Align;
use prism::{Context, Request};

use crate::components::Icon;
use crate::components::text::{Text, TextStyle};
use crate::components::button::{ButtonWidth, ButtonSize, Button};
use crate::{interactions, Theme};

#[derive(Debug, Component)]
pub struct Keypad(Column, Vec<GhostButtonRow>);
impl OnEvent for Keypad {}

impl Keypad {
    pub fn new(ctx: &mut Context, special: char) -> Self {
        Keypad(Column::center(16.0), vec![
            GhostButtonRow::new(ctx, vec![KeypadButton::char('1'), KeypadButton::char('2'), KeypadButton::char('3')]),
            GhostButtonRow::new(ctx, vec![KeypadButton::char('4'), KeypadButton::char('5'), KeypadButton::char('6')]),
            GhostButtonRow::new(ctx, vec![KeypadButton::char('7'), KeypadButton::char('8'), KeypadButton::char('9')]),
            GhostButtonRow::new(ctx, vec![KeypadButton::char(special), KeypadButton::char('0'), KeypadButton::delete()]),
        ])
    }
}


struct KeypadButton;

impl KeypadButton {
    pub fn char(c: char) -> (Option<char>, Option<String>, Key) {
        (Some(c), None, Key::Character(c.to_string().as_str().into()))
    }

    pub fn delete() -> (Option<char>, Option<String>, Key) {
        (None, Some("back".to_string()), Key::Named(NamedKey::Delete))
    }
}

#[derive(Debug, Component)]
pub struct GhostButtonRow(Row, Vec<GhostButton>);
impl OnEvent for GhostButtonRow {}

impl GhostButtonRow {
    pub fn new(ctx: &mut Context, data: Vec<(Option<char>, Option<String>, Key)>) -> Self {
        
        let buttons = data.into_iter().map(|(c, i, key)| {
            let label = c.map(|character| character.to_string());
            GhostButton::new(ctx, label.as_deref(), i.as_deref(), move |ctx: &mut Context| {
                ctx.send(Request::Event(Box::new(KeyboardEvent{state: KeyboardState::Pressed, key: key.clone()})));
            }, false)
        }).collect::<Vec<GhostButton>>();

        GhostButtonRow(Row::center(16.0), buttons)
    }
}

#[derive(Debug, Component)]
struct GhostButton(Stack, pub interactions::Button);
impl OnEvent for GhostButton {}
impl GhostButton {
    fn new(ctx: &mut Context, label: Option<&str>, icon: Option<&str>, on_click: impl FnMut(&mut Context) + 'static, is_disabled: bool) -> Self {
        let colors = ctx.state.get_or_default::<Theme>().colors.button.ghost;
        let default =  {
            let font_size = ButtonSize::Large.font();
            let icon_size = ButtonSize::Large.icon();
            let mut drawables: Vec<Box<dyn Drawable>> = Vec::new();
            if let Some(l) = label { drawables.push(Box::new(Text::new(ctx, l, font_size, TextStyle::Label(colors.default.label), Align::Left, None))); }
            if let Some(i) = icon { drawables.push(Box::new(Icon::new(ctx, i, Some(colors.default.label), icon_size))); }
            Button::new(drawables, ButtonSize::Large, ButtonWidth::Fill, Offset::Center, colors.default.background, colors.default.outline)
        };
        
        GhostButton(Stack::default(), interactions::Button::new(default, None::<Button>, None::<Button>, None::<Button>, is_disabled, Box::new(on_click), None))
    }
}

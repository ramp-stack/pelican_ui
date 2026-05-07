use prism::event::{self, KeyboardState, KeyboardEvent, OnEvent, Event, NamedKey, Modifiers};
use prism::canvas::{Align, Image};
use prism::{emitters, Context, Request};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Stack, Column, Row, Offset, Size, Padding, Area};
use prism::display::{Bin, Enum};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon};
use crate::components::button::GhostIconButton;

#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, KeyboardContent);
impl OnEvent for MobileKeyboard {}

impl MobileKeyboard {
    pub fn new(theme: &Theme) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| heights[1]);
        let color = theme.colors().get(ptsd::Background::Secondary);
        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()), 
            Rectangle::new(color, 0.0, None),
            KeyboardContent::new(theme)
        )
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, KeyboardHeader, KeyboardRow, KeyboardRow, KeyboardRow, KeyboardRow, #[skip] Theme);

impl KeyboardContent {
    fn new(theme: &Theme) -> Self {
        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme),
            KeyboardRow::top(theme, 0, false),
            KeyboardRow::middle(theme, 0, false),
            KeyboardRow::bottom(theme, 0, false),
            KeyboardRow::modifier(theme, false),
            theme.clone()
        )
    }
}

impl OnEvent for KeyboardContent {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::Paginator(page) => {
                    let theme = self.6.clone();
                    let caps = self.4.capslock().as_ref().unwrap().status();
                    self.2 = KeyboardRow::top(&theme, *page, caps);
                    self.3 = KeyboardRow::middle(&theme, *page, caps);
                    self.4 = KeyboardRow::bottom(&theme, *page, caps);
                },
                MobileKeyboardEvent::Capslock(caps) => {
                    let theme = self.6.clone();
                    let page = self.5.paginator().as_ref().unwrap().status();
                    self.2 = KeyboardRow::top(&theme, page, *caps);
                    self.3 = KeyboardRow::middle(&theme, page, *caps);
                    self.4 = KeyboardRow::bottom(&theme, page, *caps);
                    self.5 = KeyboardRow::modifier(&theme, *caps)
                },
            }
        }

        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
struct KeyRow(Row, Vec<Key>);
impl OnEvent for KeyRow {}

impl KeyRow {
    fn new(theme: &Theme, keys: Vec<&str>, caps_on: bool) -> Self {
        let keys = keys.iter().map(|k| {
            Key::character(theme, &match caps_on {
                true => k.to_uppercase(),
                false => k.to_lowercase(),
            })
        }).collect();
        KeyRow(Row::center(0.0), keys)
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardRow(Row, Option<Capslock>, Option<Paginator>, Option<KeyRow>, Option<Key>, Option<Key>);
// Capslock, Paginator, Character Row, Spacebar, Return
impl OnEvent for KeyboardRow {}

impl KeyboardRow {
    fn top(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let key_row = KeyRow::new(theme, top_keys(&num), caps_on);
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn middle(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let key_row = KeyRow::new(theme, mid_keys(&num), caps_on);
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn bottom(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let capslock = Capslock::new(theme, caps_on);
        let backspace = Key::backspace(theme);
        let key_row = KeyRow::new(theme, bot_keys(&num), caps_on);
        KeyboardRow(Row::center(6.0), Some(capslock), None, Some(key_row), None, Some(backspace))
    }

    fn modifier(theme: &Theme, caps_on: bool) -> Self {
        let paginator = Paginator::new(theme);
        let spacebar = Key::spacebar(theme, caps_on);
        let newline = Key::newline(theme, caps_on);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }

    fn capslock(&mut self) -> &mut Option<Capslock> {&mut self.1}
    fn paginator(&mut self) -> &mut Option<Paginator> {&mut self.2}
}


#[derive(Component, Debug, Clone)]
struct KeyboardHeader(Column, KeyboardIcons, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardHeader {}

impl KeyboardHeader {
    fn new(theme: &Theme) -> Self {
        let layout = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0,0.0,0.0,2.0));
        KeyboardHeader(Column::start(0.0),
            KeyboardIcons::new(theme),
            Bin(layout, Rectangle::new(theme.colors().get(ptsd::Outline::Secondary), 0.0, None))
        )
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardIcons(Row, Bin<Stack, Rectangle>, GhostIconButton);
impl OnEvent for KeyboardIcons {}
impl KeyboardIcons {
    fn new(theme: &Theme) -> Self {
        KeyboardIcons(
            Row::new(16.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)), 
            // icons.then(|| KeyboardActions(Stack::default(), actions)),
            Bin (
                Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(1.0),  Padding::default()), 
                Rectangle::new(Color::TRANSPARENT, 0.0, None)
            ),
            GhostIconButton::new(theme, Icons::DownArrow, |ctx: &mut Context, _: &Theme| {
                ctx.emit(ShowKeyboard(false));
                ctx.emit(event::TextInput::Focused(false));
            }),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Key(Stack, interactions::Button);
impl OnEvent for Key {}
impl Key {
    fn character(theme: &Theme, character: &str) -> Self {
        let default = _Key::character(theme, character, ButtonState::Default);
        let pressed = _Key::character(theme, character, ButtonState::Pressed);
        let character = character.to_string();
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Character(character.to_string()), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit character
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, callback, false))
    }

    fn spacebar(theme: &Theme, caps_on: bool) -> Self {
        let default = _Key::spacebar(theme, caps_on, ButtonState::Default);
        let pressed = _Key::spacebar(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Named(NamedKey::Space), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit space
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, callback, false))
    }

    fn newline(theme: &Theme, caps_on: bool) -> Self {
        let default = _Key::newline(theme, caps_on, ButtonState::Default);
        let pressed = _Key::newline(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Named(NamedKey::Enter), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit newline
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, callback, false))
    }

    fn backspace(theme: &Theme) -> Self {
        let default = _Key::backspace(theme, ButtonState::Default);
        let pressed = _Key::backspace(theme, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Named(NamedKey::Delete), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit delete
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, callback, false))
    }

    fn capslock(theme: &Theme, state: ButtonState) -> Self {
        let default = _Key::capslock(theme, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::Capslock(match state {
            ButtonState::Pressed => false,
            ButtonState::Default => true,
        })));

        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, None::<_Key>, None::<_Key>, callback, false))
    }
}

#[derive(Debug, Component, Clone)]
struct Capslock(Stack, interactions::Selectable);
impl OnEvent for Capslock {}
impl Capslock {
    fn new(theme: &Theme, is_on: bool) -> Self {
        let selected = Key::capslock(theme, ButtonState::Pressed);
        let default = Key::capslock(theme, ButtonState::Default);

        let selectable = interactions::Selectable::new(default, selected, is_on, true, Box::new(|_: &mut Context| {}), uuid::Uuid::new_v4());

        Capslock(Stack::default(), selectable)
    }

    fn status(&self) -> bool {self.1.is_selected()}
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ButtonState {Default, Pressed}

#[derive(Component, Debug, Clone)]
enum _Key {
    Character {layout: Stack, background: Rectangle, text: Bin<Stack, Text>},
    Spacebar {layout: Stack, background: Rectangle, text: Text},
    Capslock {layout: Stack, background: Rectangle, icon: Image},
    Backspace {layout: Stack, background: Rectangle, icon: Image},
    Paginator {layout: Stack, background: Rectangle, content: Box<PaginatorContent>},
    Newline {layout: Stack, background: Rectangle, text: Text,}
}

impl OnEvent for _Key {}

impl _Key {
    fn character(theme: &Theme, character: &str, state: ButtonState) -> Self {
        _Key::Character {
            layout: Stack(Offset::Center, Offset::End, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 33.0)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 130)
            }, 4.0, None),
            text: Bin(
                Stack(Offset::default(), Offset::default(), Size::default(), Size::default(), Padding(0.0, 0.0, 0.0, 10.0)),
                Text::new(theme, character, TextSize::Xl, TextStyle::Keyboard, Align::Left, None)
            )
        }
    }

    fn spacebar(theme: &Theme, caps_on: bool, state: ButtonState) -> Self {
        _Key::Spacebar {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, f32::MAX)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 130)
            }, 4.0, None),
            text: Text::new(theme, match caps_on {
                true => "SPACE",
                false => "space",
            }, TextSize::Md, TextStyle::Keyboard, Align::Left, None)
        }
    }

    fn newline(theme: &Theme, caps_on: bool, state: ButtonState) -> Self {
        _Key::Newline {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 92.0)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 130)
            }, 4.0, None),
            text: Text::new(theme, match caps_on {
                true => "RETURN",
                false => "return",
            }, TextSize::Md, TextStyle::Keyboard, Align::Left, None)
        }
    }

    fn capslock(theme: &Theme, state: ButtonState) -> Self {
        let icon = match state {
            ButtonState::Default => Icons::Capslock,
            ButtonState::Pressed => Icons::CapslockOn
        };

        _Key::Capslock {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 42.0)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(Color::from_hex("ffffff", 110), 4.0, None),
            icon: Icon::new(theme, icon, Some(Color::WHITE), 36.0),
        }
    }

    fn backspace(theme: &Theme, state: ButtonState) -> Self {
        _Key::Backspace {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 42.0)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 130)
            }, 4.0, None),
            icon: Icon::new(theme, Icons::Backspace, Some(Color::WHITE), 36.0),
        }
    }

    fn paginator(theme: &Theme, page: usize) -> Self {
        _Key::Paginator {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 92.0)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(Color::from_hex("ffffff", 110), 4.0, None),
            content: Box::new(PaginatorContent::new(theme, page))
        }
    }
}

#[derive(Debug, Component, Clone)]
struct PaginatorContent(Row, Text, Text, Text);
impl OnEvent for PaginatorContent {}
impl PaginatorContent {
    fn new(theme: &Theme, page: usize) -> Self {
        let (highlight, dim) = (TextStyle::Keyboard, TextStyle::Secondary);

        let styles = match page {
            0 => (highlight, dim, dim),
            1 => (dim, highlight, dim),
            _ => (dim, dim, highlight),
        };

        PaginatorContent(
            Row::center(1.0),
            Text::new(theme, "•", TextSize::H2, styles.0, Align::Left, None),
            Text::new(theme, "•", TextSize::H2, styles.1, Align::Left, None),
            Text::new(theme, "•", TextSize::H2, styles.2, Align::Left, None),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct Paginator(Stack, emitters::Selectable<_Paginator>);
impl OnEvent for Paginator {}
impl Paginator {
    fn new(theme: &Theme) -> Self {
        let first = _Key::paginator(theme, 0);
        let second = _Key::paginator(theme, 1);
        let third = _Key::paginator(theme, 2);

        let selectable = _Paginator::new(first, second, third);
        Self(Stack::default(), emitters::Selectable::new(selectable, uuid::Uuid::new_v4()))
    }

    fn status(&self) -> usize {self.1.1.current()}
}

impl std::ops::Deref for Paginator {
    type Target = _Paginator;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Paginator {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component, Clone)]
struct _Paginator(Stack, Enum<Box<dyn Drawable>>, #[skip] usize);

impl _Paginator {
    fn new(
        first: impl Drawable + 'static,
        second: impl Drawable + 'static,
        third: impl Drawable + 'static,
    ) -> Self {
        _Paginator(Stack::default(), Enum::new(vec![
            ("first".to_string(), Box::new(first)),
            ("second".to_string(), Box::new(second)),
            ("third".to_string(), Box::new(third))
        ], "first".to_string()), 0)
    }

    fn current(&self) -> usize {self.2}
}

impl OnEvent for _Paginator {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event::Selectable::Selected(true)) = event.downcast_ref::<event::Selectable>() {
            if &self.1.current() == "first" {
                self.1.display("second");
                self.2 = 1;
            } else if &self.1.current() == "second" {
                self.1.display("third");
                self.2 = 2;
            } else {
                self.1.display("first");
                self.2 = 0;
            }

            ctx.trigger_haptic();
            ctx.emit(MobileKeyboardEvent::Paginator(self.2));
        }
        vec![event]
    }
}

impl std::fmt::Debug for _Paginator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Paginator")
    }
}

fn top_keys(page: &usize) -> Vec<&str> {
    match page {
        0 => vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
        1 => vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
        _ => vec!["[", "]", "{", "}", "(", ")", "<", ">", "+", "="]
    }
}

fn mid_keys(page: &usize) -> Vec<&str> {
    match page {
        0 => vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"],
        1 => vec!["/", "\\", "\"", "'", "~", ".", ",", "?", "!"],
        _ => vec!["-", ":", ";", "#", "%", "$", "&", "^", "*",]
    }  
}

fn bot_keys(page: &usize) -> Vec<&str> {
    match page {
        0 => vec!["z", "x", "c", "v", "b", "n", "m"],
        1 => vec!["@", "|", "`", "˚", "€", "£", "¥"],
        _ => vec!["™", "©", "•", "¶", "€", "£", "¥"]
    }  
}

#[derive(Debug, Clone)]
enum MobileKeyboardEvent {
    Capslock(bool),
    Paginator(usize),
}

impl Event for MobileKeyboardEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}
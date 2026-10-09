use prism::event::{self, KeyboardState, KeyboardEvent, OnEvent, Event, Modifiers};
use prism::canvas::{Align, Image};
use prism::{emitters, Context};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Layout, Wrap, Stack, Column, Row, Offset, Size, Padding, Area};
use prism::display::{Bin, Enum};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon};
use crate::components::button::GhostIconButton;

lazy_static::lazy_static! {
    pub(crate) static ref EMOJIS: Vec<String> = include_str!("../../emoji.txt").lines().map(str::to_owned).collect();
}

#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, Enum<KeyboardContent>, #[skip] bool, #[skip] usize);

impl OnEvent for MobileKeyboard {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<KeyboardEvent>().is_some() {
            let page = self.4;
            match page {
                0 => self.2.display("default"),
                1 => self.2.display("page_one_caps_off"),
                2 | _ => self.2.display("page_two_caps_off"),
            };
        }
        
        if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::Paginator(page) => {
                    println!("Page {}", page);
                    self.4 = *page;
                    let caps = self.3;
                    match page {
                        0 if caps => self.2.display("page_zero_caps_on"),
                        0 => self.2.display("default"),
                        1 if caps => self.2.display("page_one_caps_on"),
                        1 => self.2.display("page_one_caps_off"),
                        2 if caps => self.2.display("page_two_caps_on"),
                        2 | _ => self.2.display("page_two_caps_off"),
                    };
                },
                MobileKeyboardEvent::Capslock(caps) => {
                    self.3 = *caps;
                    let page = self.4;
                    match page {
                        0 if *caps => self.2.display("page_zero_caps_on"),
                        0 => self.2.display("default"),
                        1 if *caps => self.2.display("page_one_caps_on"),
                        1 => self.2.display("page_one_caps_off"),
                        2 if *caps => self.2.display("page_two_caps_on"),
                        2 | _ => self.2.display("page_two_caps_off"),
                    };
                },
                MobileKeyboardEvent::Emoji(emoji) => {
                    match emoji {
                        false => self.2.display("emoji_on"),
                        true => self.2.display("default"),
                    };
                },
            }
        }

        vec![event]
    }
}

impl MobileKeyboard {
    pub fn new(theme: &Theme) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| heights[1]);
        let color = theme.colors().get(ptsd::Background::Secondary);
        let e = Enum::<KeyboardContent>::new(vec![
            ("default".to_string(), KeyboardContent::new(theme, 0, false)),
            ("page_zero_caps_on".to_string(), KeyboardContent::new(theme, 0, true)),
            ("page_one_caps_on".to_string(), KeyboardContent::new(theme, 1, true)),
            ("page_one_caps_off".to_string(), KeyboardContent::new(theme, 1, false)),
            ("page_two_caps_on".to_string(), KeyboardContent::new(theme, 2, true)),
            ("page_two_caps_off".to_string(), KeyboardContent::new(theme, 2, false)),
            ("emoji_on".to_string(), KeyboardContent::emoji(theme)),
        ], "default".to_string());

        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()), 
            Rectangle::new(color, 0.0, None),
            e, false, 0
        )
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, KeyboardHeader, Option<KeyboardRow>, Option<KeyboardRow>, Option<KeyboardRow>, Option<KeyboardRow>, #[skip] Theme);
impl OnEvent for KeyboardContent {}
impl KeyboardContent {
    pub fn new(theme: &Theme, page: usize, caps: bool) -> Self {
        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme),
            Some(KeyboardRow::top(theme, page, caps)),
            Some(KeyboardRow::middle(theme, page, caps)),
            Some(KeyboardRow::bottom(theme, page, caps)),
            Some(KeyboardRow::modifier(theme, page, caps)),
            theme.clone()
        )
    }

    pub fn emoji(theme: &Theme) -> Self {
        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme),
            Some(KeyboardRow::emoji(theme)),
            None,
            None,
            None,
            theme.clone()
        )
    }
}


#[derive(Component, Debug, Clone)]
struct KeyRow(Box<dyn Layout>, Vec<Key>);
impl OnEvent for KeyRow {}

impl KeyRow {
    fn new(theme: &Theme, keys: Vec<&str>, caps_on: bool) -> Self {
        let keys = keys.iter().map(|k| {
            Key::character(theme, match caps_on {
                true => k.to_uppercase(),
                false => k.to_lowercase(),
            }.chars().next().unwrap_or_default())
        }).collect();
        KeyRow(Box::new(Row::center(0.0)), keys)
    }

    fn emoji(theme: &Theme) -> Self {
       let emojis: Vec<String> = EMOJIS
        .iter()
        .filter(|line| line.contains("; fully-qualified"))
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (_, after_hash) = line.split_once('#')?;
            Some(after_hash.split_whitespace().next()?.to_string())
        })
        .filter(|emoji| {
            !emoji.contains('\u{1F3FB}')
                && !emoji.contains('\u{1F3FC}')
                && !emoji.contains('\u{1F3FD}')
                && !emoji.contains('\u{1F3FE}')
                && !emoji.contains('\u{1F3FF}')
        })
        // .take(700)
        .collect();

        let keys = emojis.iter().map(|k| {
            Key::emoji_character(theme, k.to_string())
        }).collect();
        KeyRow(Box::new(Wrap::start(0.0, 0.0)), keys)
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

    fn modifier(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let paginator = Paginator::new(theme, num);
        let spacebar = Key::spacebar(theme, caps_on);
        let newline = Key::newline(theme, caps_on);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }

    fn emoji(theme: &Theme) -> Self {
        let key_row = KeyRow::emoji(theme);
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
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
struct KeyboardIcons(Row, Emoji, Bin<Stack, Rectangle>, GhostIconButton);
impl OnEvent for KeyboardIcons {}
impl KeyboardIcons {
    fn new(theme: &Theme) -> Self {
        KeyboardIcons(
            Row::new(16.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)), 
            // icons.then(|| KeyboardActions(Stack::default(), actions)),
            Emoji::new(theme, false),
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
    fn character(theme: &Theme, character: char) -> Self {
        let default = _Key::character(theme, &character.to_string(), ButtonState::Default, true);
        let pressed = _Key::character(theme, &character.to_string(), ButtonState::Pressed, true);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Character(character.to_string()), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit character
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false))
    }

    fn emoji_character(theme: &Theme, emoji: String) -> Self {
        let default = _Key::character(theme, &emoji, ButtonState::Default, false);
        let pressed = _Key::character(theme, &emoji, ButtonState::Pressed, false);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Character(emoji.to_string()), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit character
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false))
    }

    fn spacebar(theme: &Theme, caps_on: bool) -> Self {
        let default = _Key::spacebar(theme, caps_on, ButtonState::Default);
        let pressed = _Key::spacebar(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Space, state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit space
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false))
    }

    fn newline(theme: &Theme, caps_on: bool) -> Self {
        let default = _Key::newline(theme, caps_on, ButtonState::Default);
        let pressed = _Key::newline(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Enter, state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit newline
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false))
    }

    fn backspace(theme: &Theme) -> Self {
        let default = _Key::backspace(theme, ButtonState::Default);
        let pressed = _Key::backspace(theme, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Delete, state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit delete
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false))
    }

    fn capslock(theme: &Theme, state: ButtonState) -> Self {
        let default = _Key::capslock(theme, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::Capslock(match state {
            ButtonState::Pressed => false,
            ButtonState::Default => true,
        })));

        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, None::<_Key>, None::<_Key>, None::<_Key>, callback, false))
    }

    fn emoji(theme: &Theme, state: ButtonState) -> Self {
        let default = _Key::emoji(theme, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::Emoji(match state {
            ButtonState::Pressed => false,
            ButtonState::Default => true,
        })));

        Key(Stack::default(), interactions::Button::new_triggers_on_release(default, None::<_Key>, None::<_Key>, None::<_Key>, None::<_Key>, callback, false))
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

#[derive(Debug, Component, Clone)]
struct Emoji(Stack, interactions::Selectable);
impl OnEvent for Emoji {}
impl Emoji {
    fn new(theme: &Theme, is_on: bool) -> Self {
        let selected = Key::emoji(theme, ButtonState::Pressed);
        let default = Key::emoji(theme, ButtonState::Default);

        let selectable = interactions::Selectable::new(default, selected, is_on, true, Box::new(|_: &mut Context| {}), uuid::Uuid::new_v4());

        Emoji(Stack::default(), selectable)
    }

    fn status(&self) -> bool {self.1.is_selected()}
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ButtonState {Default, Pressed}

#[derive(Component, Debug, Clone)]
enum _Key {
    Character {layout: Stack, background: Option<Rectangle>, text: Bin<Stack, Text>},
    Spacebar {layout: Stack, background: Rectangle, text: Text},
    Capslock {layout: Stack, background: Rectangle, icon: Image},
    Backspace {layout: Stack, background: Rectangle, icon: Image},
    Paginator {layout: Stack, background: Rectangle, content: Box<PaginatorContent>},
    Newline {layout: Stack, background: Rectangle, text: Text},
    Emoji {layout: Stack, icon: Image}
}

impl OnEvent for _Key {}

impl _Key {
    fn character(theme: &Theme, character: &str, state: ButtonState, background: bool) -> Self {
        _Key::Character {
            layout: Stack(Offset::Center, Offset::End, Size::Static(30.0), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: background.then_some(Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 130)
            }, 4.0, None)),
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

    fn emoji(theme: &Theme, state: ButtonState) -> Self {
        let color = match state {
            ButtonState::Default => Color::WHITE,
            ButtonState::Pressed => theme.colors().get(ptsd::colors::Text::Secondary),
        };

        _Key::Emoji {
            layout: Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(3.0, 6.0, 3.0, 6.0)),
            icon: Icon::new(theme, Icons::Emoji, Some(color), 36.0),
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
    fn new(theme: &Theme, page: usize) -> Self {
        let first = _Key::paginator(theme, 0);
        let second = _Key::paginator(theme, 1);
        let third = _Key::paginator(theme, 2);

        let selectable = _Paginator::new(page, first, second, third);

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
    fn new(page: usize,
        first: impl Drawable + 'static,
        second: impl Drawable + 'static,
        third: impl Drawable + 'static,
    ) -> Self {
        let start = match page {
            0 => "first",
            1 => "second",
            2 | _ => "third"
        };

        _Paginator(Stack::default(), Enum::new(vec![
            ("first".to_string(), Box::new(first)),
            ("second".to_string(), Box::new(second)),
            ("third".to_string(), Box::new(third))
        ], start.to_string()), 0)
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
    Emoji(bool),
}

impl Event for MobileKeyboardEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

fn parse_emoji_test(contents: &str) -> Vec<String> {
    let mut emojis = Vec::new();

    for line in contents.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((codes, rest)) = line.split_once(';') else {
            continue;
        };

        if !rest.contains("fully-qualified") {
            continue;
        }

        let mut emoji = String::new();

        for hex in codes.split_whitespace() {
            let cp = u32::from_str_radix(hex, 16).unwrap();
            emoji.push(char::from_u32(cp).unwrap());
        }

        emojis.push(emoji);
    }

    emojis
}
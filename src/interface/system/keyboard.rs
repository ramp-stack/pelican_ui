use prism::event::{self, MouseEvent, TickEvent, KeyboardState, KeyboardEvent, OnEvent, Event, Modifiers, MouseState, MouseButton};
use prism::canvas::{Align, Image};
use prism::{emitters, Context, IS_MOBILE};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Layout, Stack, Column, Row, Offset, Size, Padding};
use prism::display::{Bin, Enum, Opt};
use std::time::{Duration, Instant};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon, SendEvent};

use crate::components::button::GhostIconButton;

use super::emoji::EmojiPanel;
use super::sticker::StickerPanel;
use super::gif::{GifPanel, GifSearch};
use crate::interface::message_bar::MessageBarEvent;

#[derive(Debug, Clone, Default)]
struct HoldRepeat { started: Option<Instant>, last: Option<Instant>, target: KeyTarget }

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyTarget {
    #[default]
    Input,
    GifSearch,
    Bar
}

impl KeyTarget {
    pub fn press(self, ctx: &mut Context, key: event::Key) {
        match (self, key) {
            (KeyTarget::GifSearch, event::Key::Enter) => ctx.emit(MobileKeyboardEvent::SubmitGifSearch),
            (KeyTarget::Bar, key) => ctx.emit(MessageBarEvent::Key(key)),
            (_, key) => ctx.emit(KeyboardEvent{key, state: KeyboardState::Pressed, modifiers: Modifiers::default()}),
        }
    }
}

pub(crate) const TEXT_FIELD_GROUP: &str = "123e4567-e89b-12d3-a456-426614174000";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyboardPage { Keys, Emoji, Stickers, Gifs }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardPages { pub emoji: bool, pub stickers: bool, pub gifs: bool, pub toolbar: bool }

impl KeyboardPages {
    pub const ALL: Self = KeyboardPages { emoji: true, stickers: true, gifs: true, toolbar: false };
    pub const TOOLBAR: Self = KeyboardPages { emoji: true, stickers: true, gifs: true, toolbar: true };
    pub const EMOJI: Self = KeyboardPages { emoji: true, stickers: false, gifs: false, toolbar: false };
    pub const NONE: Self = KeyboardPages { emoji: false, stickers: false, gifs: false, toolbar: false };
}

#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard {
    layout: Stack,
    background: Rectangle,
    content: KeyboardContent,
    #[skip] caps: bool,
    #[skip] keys_page: usize,
    #[skip] page: KeyboardPage,
    #[skip] theme: Theme,
    #[skip] gif_search: GifSearch,
    #[skip] pages: KeyboardPages,
    #[skip] announced: Option<Option<KeyboardPage>>,
}

impl OnEvent for MobileKeyboard {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        let mut page = None;
        let mut search_keys = None;
        let mut refresh = false;

        if let Some(KeyboardEvent{key: event::Key::Character(_), state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.caps && (self.page == KeyboardPage::Keys || self.gif_search.open) {
            self.caps = false;
            refresh = true;
        } else if let Some(ShowKeyboard(true)) = event.downcast_ref::<ShowKeyboard>() {
            refresh = true;
        } else if self.gif_search.open && (matches!(event.downcast_ref::<SendEvent>(), Some(SendEvent::Gif(_))) || matches!(event.downcast_ref::<MobileKeyboardEvent>(), Some(MobileKeyboardEvent::CancelGifSearch))) {
            self.gif_search.open = false;
            page = Some(KeyboardPage::Gifs);
            ctx.emit(MobileKeyboardEvent::FullScreen(false));
            if let Some((id, group)) = self.gif_search.return_to.take() { ctx.emit(event::Selectable::Pressed(id, group)); }
        } else if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::Pages(_) if !self.gif_search.open => page = Some(self.page),
                MobileKeyboardEvent::Paginator(n) => {
                    self.keys_page = *n;
                    refresh = true;
                },
                MobileKeyboardEvent::Capslock(caps) => {
                    self.caps = *caps;
                    refresh = true;
                },
                MobileKeyboardEvent::Toggle(p) => page = Some(if self.page == *p { KeyboardPage::Keys } else { *p }),
                MobileKeyboardEvent::Show(p) if !self.gif_search.open => {
                    self.pages = KeyboardPages::TOOLBAR;
                    page = Some(*p);
                },
                MobileKeyboardEvent::StartGifSearch if !self.gif_search.open && self.page == KeyboardPage::Gifs => {
                    self.gif_search.open = true;
                    self.gif_search.return_to = self.gif_search.focused.clone();
                    self.content.panels.inner().gifs.inner().set_full(true);
                    self.content.header = None;
                    search_keys = Some(true);
                    ctx.emit(MobileKeyboardEvent::FullScreen(true));
                },
                MobileKeyboardEvent::SubmitGifSearch if self.gif_search.open => search_keys = Some(false),
                _ => {},
            }
        }

        if refresh {
            match self.gif_search.open {
                true => search_keys = Some(true),
                false => page = Some(self.page),
            }
        }

        if let Some(page) = page {
            let page = match page {
                KeyboardPage::Emoji if !self.pages.emoji => KeyboardPage::Keys,
                KeyboardPage::Stickers if !self.pages.stickers => KeyboardPage::Keys,
                KeyboardPage::Gifs if !self.pages.gifs => KeyboardPage::Keys,
                page => page,
            };
            self.page = page;
            let panels = self.content.panels.inner();
            panels.gifs.inner().close_search();
            panels.gifs.inner().set_full(false);
            if page != KeyboardPage::Keys { panels.show(page); }
            self.content.panels.display(page != KeyboardPage::Keys);
            self.content.keys = (page == KeyboardPage::Keys).then(|| KeyRows::new(&self.theme, self.keys_page, self.caps, KeyTarget::Input));
            self.content.header = (!self.pages.toolbar).then(|| KeyboardHeader(
                Column::start(0.0),
                KeyboardIcons::new(&self.theme, page, self.pages),
                Bin(
                    Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0, 0.0, 0.0, 2.0)),
                    Rectangle::new(self.theme.colors().get(ptsd::Outline::Secondary), 0.0, None),
                ),
            ));
            self.content.layout.padding().1 = if self.content.header.is_none() && page == KeyboardPage::Keys { 22.0 } else { 8.0 };
        }

        if let Some(show) = search_keys {
            self.content.keys = show.then(|| KeyRows::new(&self.theme, self.keys_page, self.caps, KeyTarget::GifSearch));
        }

        vec![event]
    }
}

impl MobileKeyboard {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        let (bottom, _, _, _) = ctx.get_safe_area();
        MobileKeyboard {
            layout: Stack(Offset::Start, Offset::Start, Size::Fill, Size::custom(|heights: Vec<(f32, f32)>| heights[1]), Padding::default()),
            background: Rectangle::new(color, 0.0, None),
            content: KeyboardContent {
                layout: Column::new(0.0, Offset::Center, Size::Fit, Padding::new(8.0), None),
                header: None,
                panels: Opt::new(Panels::new(theme), false),
                keys: Some(KeyRows::new(theme, 0, false, KeyTarget::Input)),
                safe_area: Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(bottom), Padding::default()), Rectangle::new(color, 0.0, None)),
            },
            caps: false,
            keys_page: 0,
            page: KeyboardPage::Keys,
            theme: theme.clone(),
            gif_search: GifSearch::default(),
            pages: KeyboardPages::default(),
            announced: None,
        }
    }

    pub fn on_interface_event(&mut self, ctx: &mut Context, event: &dyn Event, showing: bool) -> bool {
        if let Some(event::Selectable::Pressed(id, group)) = event.downcast_ref::<event::Selectable>() && group == TEXT_FIELD_GROUP {
            self.gif_search.focused = Some((id.clone(), group.clone()));
        } else if let Some(MobileKeyboardEvent::Pages(pages)) = event.downcast_ref::<MobileKeyboardEvent>() && !self.gif_search.open {
            self.pages = *pages;
        } else if let Some(ShowKeyboard(false)) = event.downcast_ref::<ShowKeyboard>() && self.gif_search.open {
            return false;
        } else if event.downcast_ref::<TickEvent>().is_some() {
            if !showing && std::mem::take(&mut self.gif_search.open) { ctx.emit(MobileKeyboardEvent::FullScreen(false)); }
            let now = showing.then_some(self.page);
            if self.announced != Some(now) {
                self.announced = Some(now);
                ctx.emit(MobileKeyboardEvent::Showing(now));
            }
        }
        true
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardContent {
    layout: Column,
    header: Option<KeyboardHeader>,
    panels: Opt<Panels>,
    keys: Option<KeyRows>,
    safe_area: Bin<Stack, Rectangle>,
}
impl OnEvent for KeyboardContent {}

#[derive(Component, Debug, Clone)]
pub struct Panels {
    layout: Stack,
    background: Option<Rectangle>,
    emoji: Opt<Bin<Stack, EmojiPanel>>,
    stickers: Opt<Bin<Stack, StickerPanel>>,
    pub(crate) gifs: Opt<Bin<Stack, GifPanel>>,
}
impl OnEvent for Panels {}

impl Panels {
    pub(crate) fn new(theme: &Theme) -> Self {
        let target = if IS_MOBILE { KeyTarget::Input } else { KeyTarget::Bar };
        let room = Padding::new(if IS_MOBILE { 0.0 } else { 8.0 });
        Panels {
            layout: Stack(Offset::Start, Offset::Start, Size::Fill, Size::custom(|heights: Vec<(f32, f32)>| {
                heights.iter().rev().take(3).fold((0.0_f32, 0.0_f32), |tallest, h| (tallest.0.max(h.0), tallest.1.max(h.1)))
            }), Padding::default()),
            background: (!IS_MOBILE).then(|| Rectangle::new(theme.colors().get(ptsd::Background::Secondary), 12.0, None)),
            emoji: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, room), EmojiPanel::new(theme, target)), true),
            stickers: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, room), StickerPanel::new(theme)), false),
            gifs: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, room), GifPanel::new(theme)), false),
        }
    }

    pub(crate) fn show(&mut self, page: KeyboardPage) {
        self.emoji.display(page == KeyboardPage::Emoji);
        self.stickers.display(page == KeyboardPage::Stickers);
        self.gifs.display(page == KeyboardPage::Gifs);
        match page == KeyboardPage::Gifs {
            true => self.gifs.inner().retry(),
            false => self.gifs.inner().close_search(),
        }
    }
}

#[derive(Component, Debug, Clone)]
struct KeyRow(Box<dyn Layout>, Vec<Key>);
impl OnEvent for KeyRow {}

impl KeyRow {
    fn new(theme: &Theme, keys: Vec<&str>, caps_on: bool, target: KeyTarget) -> Self {
        let keys = keys.iter().map(|k| {
            Key::character(theme, match caps_on {
                true => k.to_uppercase(),
                false => k.to_lowercase(),
            }.chars().next().unwrap_or_default(), target)
        }).collect();
        KeyRow(Box::new(Row::center(0.0)), keys)
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardRow(Row, Option<Capslock>, Option<Paginator>, Option<KeyRow>, Option<Key>, Option<Key>);
impl OnEvent for KeyboardRow {}

impl KeyboardRow {
    fn top(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let keys = match num {
            0 => vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
            1 => vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
            _ => vec!["[", "]", "{", "}", "(", ")", "<", ">", "+", "="]
        };
        KeyboardRow(Row::center(0.0), None, None, Some(KeyRow::new(theme, keys, caps_on, target)), None, None)
    }

    fn middle(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let keys = match num {
            0 => vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"],
            1 => vec!["/", "\\", "\"", "'", "~", ".", ",", "?", "!"],
            _ => vec!["-", ":", ";", "#", "%", "$", "&", "^", "*",]
        };
        KeyboardRow(Row::center(0.0), None, None, Some(KeyRow::new(theme, keys, caps_on, target)), None, None)
    }

    fn bottom(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let keys = match num {
            0 => vec!["z", "x", "c", "v", "b", "n", "m"],
            1 => vec!["@", "|", "`", "˚", "€", "£", "¥"],
            _ => vec!["™", "©", "•", "¶", "€", "£", "¥"]
        };
        let capslock = Capslock::new(theme, caps_on);
        let backspace = Key::backspace_sized(theme, 48.0, target);
        KeyboardRow(Row::center(6.0), Some(capslock), None, Some(KeyRow::new(theme, keys, caps_on, target)), None, Some(backspace))
    }

    fn modifier(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let paginator = Paginator::new(theme, num);
        let spacebar = Key::spacebar(theme, caps_on, target);
        let newline = Key::newline(theme, caps_on, target);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }
}

#[derive(Component, Debug, Clone)]
struct KeyRows(Column, KeyboardRow, KeyboardRow, KeyboardRow, KeyboardRow);
impl OnEvent for KeyRows {}

impl KeyRows {
    fn new(theme: &Theme, page: usize, caps: bool, target: KeyTarget) -> Self {
        KeyRows(
            Column::new(0.0, Offset::Center, Size::Fit, Padding::default(), None),
            KeyboardRow::top(theme, page, caps, target),
            KeyboardRow::middle(theme, page, caps, target),
            KeyboardRow::bottom(theme, page, caps, target),
            KeyboardRow::modifier(theme, page, caps, target),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardHeader(Column, KeyboardIcons, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardHeader {}

#[derive(Component, Debug, Clone)]
struct KeyboardIcons(Row, Option<Key>, Option<Key>, Option<Key>, Bin<Stack, HeaderLabel>, GhostIconButton);
impl OnEvent for KeyboardIcons {}
impl KeyboardIcons {
    fn new(theme: &Theme, page: KeyboardPage, pages: KeyboardPages) -> Self {
        let button = |icon: Icons, of: KeyboardPage, offered: bool| offered.then(|| {
            let state = if page == of {ButtonState::Pressed} else {ButtonState::Default};
            Key::page_button(theme, icon, state, MobileKeyboardEvent::Toggle(of))
        });
        KeyboardIcons(
            Row::new(12.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)),
            button(Icons::Emoji, KeyboardPage::Emoji, pages.emoji),
            button(Icons::Sticker, KeyboardPage::Stickers, pages.stickers),
            button(Icons::Gif, KeyboardPage::Gifs, pages.gifs),
            Bin(
                Stack(Offset::Start, Offset::Center, Size::custom(|_: Vec<(f32, f32)>| (0.0, f32::MAX)), Size::Static(48.0), Padding::default()),
                HeaderLabel::new(theme)
            ),
            GhostIconButton::new(theme, Icons::DownArrow, |ctx: &mut Context, _: &Theme| {
                ctx.emit(ShowKeyboard(false));
                ctx.emit(event::TextInput::Focused(false));
            }),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct HeaderLabel(Row, Text);
impl OnEvent for HeaderLabel {}
impl HeaderLabel {
    fn new(theme: &Theme) -> Self {
        HeaderLabel(
            Row::new(8.0, Offset::Center, Size::Fit, Padding::default()),
            Text::new(theme, "", TextSize::Md, TextStyle::Keyboard, Align::Left, None),
        )
    }
}

#[derive(Debug, Component, Clone)]
pub(super) struct Key(Stack, interactions::Button, #[skip] Option<HoldRepeat>);

impl OnEvent for Key {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(hold) = &mut self.2 {
            if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
                match mouse.state {
                    MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                        *hold = HoldRepeat { started: Some(Instant::now()), last: None, target: hold.target };
                    }
                    MouseState::Released(_) => *hold = HoldRepeat { target: hold.target, ..HoldRepeat::default() },
                    MouseState::Moved if mouse.position.is_none() => *hold = HoldRepeat { target: hold.target, ..HoldRepeat::default() },
                    _ => {}
                }
            } else if event.downcast_ref::<TickEvent>().is_some()
                && let Some(started) = hold.started
            {
                let held = started.elapsed();
                let interval = if held >= Duration::from_millis(2000) { Duration::from_millis(35) } else { Duration::from_millis(80) };
                if held >= Duration::from_millis(500) && hold.last.is_none_or(|t| t.elapsed() >= interval) {
                    hold.last = Some(Instant::now());
                    hold.target.press(ctx, event::Key::Delete);
                }
            }
        }
        vec![event]
    }
}
impl Key {
    fn character(theme: &Theme, character: char, target: KeyTarget) -> Self {
        let default = _Key::character(theme, &character.to_string(), ButtonState::Default, true);
        let pressed = _Key::character(theme, &character.to_string(), ButtonState::Pressed, true);
        let callback = Box::new(move |ctx: &mut Context| target.press(ctx, event::Key::Character(character.to_string())));
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn spacebar(theme: &Theme, caps_on: bool, target: KeyTarget) -> Self {
        let default = _Key::spacebar(theme, caps_on, ButtonState::Default);
        let pressed = _Key::spacebar(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| target.press(ctx, event::Key::Space));
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn newline(theme: &Theme, caps_on: bool, target: KeyTarget) -> Self {
        let label = match target { KeyTarget::GifSearch => "search", _ => "return" };
        let default = _Key::newline(theme, caps_on, ButtonState::Default, label);
        let pressed = _Key::newline(theme, caps_on, ButtonState::Pressed, label);
        let callback = Box::new(move |ctx: &mut Context| target.press(ctx, event::Key::Enter));
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    pub(super) fn backspace_sized(theme: &Theme, height: f32, target: KeyTarget) -> Self {
        let default = _Key::backspace(theme, ButtonState::Default, height);
        let pressed = _Key::backspace(theme, ButtonState::Pressed, height);
        let callback = Box::new(move |ctx: &mut Context| target.press(ctx, event::Key::Delete));
        let hold = HoldRepeat { target, ..HoldRepeat::default() };
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), Some(hold))
    }

    fn capslock(theme: &Theme, state: ButtonState) -> Self {
        let default = _Key::capslock(theme, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::Capslock(match state {
            ButtonState::Pressed => false,
            ButtonState::Default => true,
        })));

        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, None::<_Key>, None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn page_button(theme: &Theme, icon: Icons, state: ButtonState, event: MobileKeyboardEvent) -> Self {
        let default = _Key::page_button(theme, icon, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(event.clone()));

        Key(Stack::default(), interactions::Button::new_triggers_on_release(default, None::<_Key>, None::<_Key>, None::<_Key>, None::<_Key>, callback, false), None)
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
    PageButton {layout: Stack, icon: Image}
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

    fn newline(theme: &Theme, caps_on: bool, state: ButtonState, label: &str) -> Self {
        _Key::Newline {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 92.0)), Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 130)
            }, 4.0, None),
            text: Text::new(theme, &match caps_on {
                true => label.to_uppercase(),
                false => label.to_string(),
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

    fn page_button(theme: &Theme, icon: Icons, state: ButtonState) -> Self {
        let color = match state {
            ButtonState::Default => Color::WHITE,
            ButtonState::Pressed => theme.colors().get(ptsd::colors::Text::Secondary),
        };

        _Key::PageButton {
            layout: Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(3.0, 6.0, 3.0, 6.0)),
            icon: Icon::new(theme, icon, Some(color), 36.0),
        }
    }

    fn backspace(theme: &Theme, state: ButtonState, height: f32) -> Self {
        _Key::Backspace {
            layout: Stack(Offset::Center, Offset::Center, Size::custom(move |widths: Vec<(f32, f32)>|(widths[1].0, 42.0)), Size::Static(height), Padding(3.0, 6.0, 3.0, 6.0)),
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
            _ => "third"
        };

        _Paginator(Stack::default(), Enum::new(vec![
            ("first".to_string(), Box::new(first)),
            ("second".to_string(), Box::new(second)),
            ("third".to_string(), Box::new(third))
        ], start.to_string()), 0)
    }
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

#[derive(Debug, Clone)]
pub(crate) enum MobileKeyboardEvent {
    Pages(KeyboardPages),
    Showing(Option<KeyboardPage>),
    FullScreen(bool),
    Capslock(bool),
    Paginator(usize),
    Toggle(KeyboardPage),
    Show(KeyboardPage),
    EmojiCategory(usize),
    StartGifSearch,
    CancelGifSearch,
    SubmitGifSearch,
}

prism::impl_event_all_children!(MobileKeyboardEvent);

use prism::event::{self, MouseEvent, TickEvent, KeyboardState, KeyboardEvent, OnEvent, Event, Modifiers, MouseState, MouseButton};
use prism::canvas::{Align, Image};
use prism::{emitters, Context};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Layout, Stack, Column, Row, Offset, Size, Padding, Area};
use prism::display::{Bin, Enum, Opt};
use std::time::{Duration, Instant};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon};
use crate::components::button::GhostIconButton;

const REPEAT_DELAY: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL: Duration = Duration::from_millis(80);
const FAST_AFTER: Duration = Duration::from_millis(2000);
const FAST_INTERVAL: Duration = Duration::from_millis(35);

#[derive(Debug, Clone, Default)]
struct HoldRepeat { started: Option<Instant>, last: Option<Instant> }

#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, Enum<KeyboardContent>, #[skip] bool, #[skip] usize, #[skip] bool, #[skip] Theme);

impl OnEvent for MobileKeyboard {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // Caps works as a one-shot shift: it turns off after a character is typed.
        // Typing on the emoji panel doesn't leave it.
        if let Some(KeyboardEvent{key: event::Key::Character(_), state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.3 && !self.5 {
            self.3 = false;
            self.show_keys(ctx);
        }

        if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::Paginator(page) => {
                    self.4 = *page;
                    self.show_keys(ctx);
                },
                MobileKeyboardEvent::Capslock(caps) => {
                    self.3 = *caps;
                    self.show_keys(ctx);
                },
                MobileKeyboardEvent::ToggleEmoji => {
                    self.5 = !self.5;
                    if self.5 { self.2.display("emoji"); } else { self.show_keys(ctx); }
                },
                // Handled by the emoji panel.
                MobileKeyboardEvent::EmojiCategory(_) => {},
            }
        }

        vec![event]
    }
}

impl MobileKeyboard {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| heights[1]);
        let color = theme.colors().get(ptsd::Background::Secondary);
        let e = Enum::<KeyboardContent>::new(vec![
            ("keys".to_string(), KeyboardContent::new(ctx, theme, 0, false)),
            ("emoji".to_string(), KeyboardContent::emoji(ctx, theme)),
        ], "keys".to_string());

        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()),
            Rectangle::new(color, 0.0, None),
            e, false, 0, false, theme.clone()
        )
    }

    /// Shows the key layout for the current page and caps state.
    fn show_keys(&mut self, ctx: &mut Context) {
        self.5 = false;
        self.2.display("keys");
        *self.2.drawable().inner() = KeyboardContent::new(ctx, &self.6, self.4, self.3);
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, KeyboardHeader, Option<KeyboardRow>, Option<KeyboardRow>, Option<KeyboardRow>, Option<KeyboardRow>, Option<EmojiPanel>, Bin<Stack, Rectangle>, #[skip] Theme);

impl OnEvent for KeyboardContent {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() && let Some(panel) = &self.6 {
            let (category, page) = panel.label();
            self.1.set_label(&category, &page);
        }
        vec![event]
    }
}

impl KeyboardContent {
    pub fn new(ctx: &mut Context, theme: &Theme, page: usize, caps: bool) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        let (b, _, _, _) = ctx.get_safe_area();
        let layout = Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(b), Padding::default());

        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme, false),
            Some(KeyboardRow::top(theme, page, caps)),
            Some(KeyboardRow::middle(theme, page, caps)),
            Some(KeyboardRow::bottom(theme, page, caps)),
            Some(KeyboardRow::modifier(theme, page, caps)),
            None,
            Bin(layout, Rectangle::new(color, 0.0, None)),
            theme.clone()
        )
    }

    pub fn emoji(ctx: &mut Context, theme: &Theme) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        let (b, _, _, _) = ctx.get_safe_area();
        let layout = Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(b), Padding::default());

        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme, true),
            None,
            None,
            None,
            None,
            Some(EmojiPanel::new(theme)),
            Bin(layout, Rectangle::new(color, 0.0, None)),
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

    fn capslock(&mut self) -> &mut Option<Capslock> {&mut self.1}
    fn paginator(&mut self) -> &mut Option<Paginator> {&mut self.2}
}


#[derive(Component, Debug, Clone)]
struct KeyboardHeader(Column, KeyboardIcons, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardHeader {}

impl KeyboardHeader {
    fn new(theme: &Theme, emoji_on: bool) -> Self {
        let layout = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0,0.0,0.0,2.0));
        KeyboardHeader(Column::start(0.0),
            KeyboardIcons::new(theme, emoji_on),
            Bin(layout, Rectangle::new(theme.colors().get(ptsd::Outline::Secondary), 0.0, None))
        )
    }

    fn set_label(&mut self, title: &str, detail: &str) { self.1.set_label(title, detail); }
}

#[derive(Component, Debug, Clone)]
struct KeyboardIcons(Row, Key, Bin<Stack, HeaderLabel>, GhostIconButton);
impl OnEvent for KeyboardIcons {}
impl KeyboardIcons {
    fn new(theme: &Theme, emoji_on: bool) -> Self {
        let emoji_state = if emoji_on {ButtonState::Pressed} else {ButtonState::Default};
        KeyboardIcons(
            Row::new(16.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)),
            // icons.then(|| KeyboardActions(Stack::default(), actions)),
            Key::emoji(theme, emoji_state),
            // Fills the space between the keys; shows the emoji category and page.
            Bin(
                Stack(Offset::Start, Offset::Center, Size::Fill, Size::Static(48.0), Padding::default()),
                HeaderLabel::new(theme)
            ),
            GhostIconButton::new(theme, Icons::DownArrow, |ctx: &mut Context, _: &Theme| {
                ctx.emit(ShowKeyboard(false));
                ctx.emit(event::TextInput::Focused(false));
            }),
        )
    }

    fn set_label(&mut self, title: &str, _detail: &str) {
        let label = &mut self.2.1;
        if label.1.spans[0] != title { label.1.spans[0] = title.to_string(); }
        // if label.2.spans[0] != detail { label.2.spans[0] = detail.to_string(); }
    }
}

/// Title and detail text in the keyboard header (empty on the letter keys).
#[derive(Component, Debug, Clone)]
struct HeaderLabel(Row, Text);
impl OnEvent for HeaderLabel {}
impl HeaderLabel {
    fn new(theme: &Theme) -> Self {
        HeaderLabel(
            Row::new(8.0, Offset::Center, Size::Fit, Padding::default()),
            Text::new(theme, "", TextSize::Md, TextStyle::Keyboard, Align::Left, None),
            // Text::new(theme, "", TextSize::Md, TextStyle::Secondary, Align::Left, None),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Key(Stack, interactions::Button, #[skip] Option<HoldRepeat>);

impl OnEvent for Key {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(hold) = &mut self.2 {
            if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
                match mouse.state {
                    MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                        *hold = HoldRepeat { started: Some(Instant::now()), last: None };
                    }
                    // Stop on release anywhere, or when the finger slides off the key.
                    MouseState::Released(_) => *hold = HoldRepeat::default(),
                    MouseState::Moved if mouse.position.is_none() => *hold = HoldRepeat::default(),
                    _ => {}
                }
            } else if event.downcast_ref::<TickEvent>().is_some()
                && let Some(started) = hold.started
            {
                let held = started.elapsed();
                let interval = if held >= FAST_AFTER { FAST_INTERVAL } else { REPEAT_INTERVAL };
                if held >= REPEAT_DELAY && hold.last.is_none_or(|t| t.elapsed() >= interval) {
                    hold.last = Some(Instant::now());
                    ctx.emit(KeyboardEvent{key: event::Key::Delete, state: KeyboardState::Pressed, modifiers: Modifiers::default()});
                }
            }
        }
        vec![event]
    }
}
impl Key {
    fn character(theme: &Theme, character: char) -> Self {
        let default = _Key::character(theme, &character.to_string(), ButtonState::Default, true);
        let pressed = _Key::character(theme, &character.to_string(), ButtonState::Pressed, true);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Character(character.to_string()), state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit character
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn spacebar(theme: &Theme, caps_on: bool) -> Self {
        let default = _Key::spacebar(theme, caps_on, ButtonState::Default);
        let pressed = _Key::spacebar(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Space, state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit space
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn newline(theme: &Theme, caps_on: bool) -> Self {
        let default = _Key::newline(theme, caps_on, ButtonState::Default);
        let pressed = _Key::newline(theme, caps_on, ButtonState::Pressed);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Enter, state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit newline
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn backspace(theme: &Theme) -> Self { Self::backspace_sized(theme, 48.0) }

    /// Backspace for the emoji panel's 48px tab row.
    fn backspace_compact(theme: &Theme) -> Self { Self::backspace_sized(theme, 48.0 - 12.0) }

    fn backspace_sized(theme: &Theme, height: f32) -> Self {
        let default = _Key::backspace(theme, ButtonState::Default, height);
        let pressed = _Key::backspace(theme, ButtonState::Pressed, height);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(KeyboardEvent{key: event::Key::Delete, state: KeyboardState::Pressed, modifiers: Modifiers::default()})); // emmit delete
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), Some(HoldRepeat::default()))
    }

    fn capslock(theme: &Theme, state: ButtonState) -> Self {
        let default = _Key::capslock(theme, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::Capslock(match state {
            ButtonState::Pressed => false,
            ButtonState::Default => true,
        })));

        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, None::<_Key>, None::<_Key>, None::<_Key>, callback, false), None)
    }

    /// `state` only sets the look (highlighted while the emoji panel is open).
    fn emoji(theme: &Theme, state: ButtonState) -> Self {
        let default = _Key::emoji(theme, state);
        let callback = Box::new(move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::ToggleEmoji));

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
    ToggleEmoji,
    /// Show the first page of this emoji category.
    EmojiCategory(usize),
}

impl Event for MobileKeyboardEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

struct EmojiCategory { name: String, icon: &'static str, emojis: Vec<String> }

lazy_static::lazy_static! {
    static ref EMOJI_CATEGORIES: Vec<EmojiCategory> = parse_emoji_categories(include_str!("../../emoji.txt"));

    static ref EMOJI_PAGES: Vec<(usize, usize)> = EMOJI_CATEGORIES.iter().enumerate()
        .flat_map(|(c, category)| (0..category.emojis.len()).step_by(32).map(move |start| (c, start)))
        .collect();
}

fn category_icon(group: &str) -> Option<&'static str> {
    Some(match group {
        "Smileys & Emotion" => "😀",
        "People & Body" => "👋",
        "Animals & Nature" => "🐻",
        "Food & Drink" => "🍔",
        "Travel & Places" => "🚗",
        "Activities" => "⚽",
        "Objects" => "💡",
        "Symbols" => "🔣",
        "Flags" => "🏁",
        _ => return None,
    })
}

fn parse_emoji_categories(contents: &str) -> Vec<EmojiCategory> {
    let mut categories: Vec<EmojiCategory> = Vec::new();
    let mut in_category = false;

    for line in contents.lines() {
        if let Some(group) = line.strip_prefix("# group:") {
            let name = group.trim();
            in_category = category_icon(name).map(|icon| {
                categories.push(EmojiCategory { name: name.to_string(), icon, emojis: Vec::new() });
            }).is_some();
            continue;
        }
        if !in_category || line.starts_with('#') { continue; }

        let Some((codes, status)) = line.split_once(';') else { continue };
        if !status.trim_start().starts_with("fully-qualified") { continue; }

        let emoji: Option<String> = codes.split_whitespace()
            .map(|hex| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32))
            .collect();
        let Some(emoji) = emoji else { continue };
        if emoji.chars().any(|c| ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)) { continue; }

        if let Some(category) = categories.last_mut() { category.emojis.push(emoji); }
    }

    categories
}

#[derive(Component, Debug, Clone)]
struct EmojiPanel(Column, EmojiGrid, EmojiTabs, #[skip] usize, #[skip] Option<EmojiTouch>, #[skip] Theme);

#[derive(Debug, Clone, Copy)]
struct EmojiTouch { cell: Option<(usize, usize)>, last: (f32, f32), travel: (f32, f32) }

impl EmojiPanel {
    fn new(theme: &Theme) -> Self {
        EmojiPanel(
            Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None),
            EmojiGrid::new(theme, 0),
            EmojiTabs::new(theme, 0),
            0, None, theme.clone(),
        )
    }

    fn category(&self) -> usize { EMOJI_PAGES[self.3].0 }

    fn show_page(&mut self, page: usize) {
        if page == self.3 || page >= EMOJI_PAGES.len() { return; }
        let category = self.category();
        self.3 = page;
        self.4 = None;
        self.1 = EmojiGrid::new(&self.5, page);
        if self.category() != category { self.2 = EmojiTabs::new(&self.5, self.category()); }
    }

    fn label(&self) -> (String, String) {
        let (c, start) = EMOJI_PAGES[self.3];
        let category = &EMOJI_CATEGORIES[c];
        let pages = category.emojis.len().div_ceil(32);
        (category.name.clone(), format!("{}/{}", start / 32 + 1, pages))
    }

    fn cell_at(sized: &SizedTree, pos: (f32, f32)) -> Option<(usize, usize)> {
        let contains = |offset: (f32, f32), size: (f32, f32), p: (f32, f32)|
            p.0 >= offset.0 && p.0 < offset.0 + size.0 && p.1 >= offset.1 && p.1 < offset.1 + size.1;

        let (grid_offset, grid) = sized.1.first()?;
        let p = (pos.0 - grid_offset.0, pos.1 - grid_offset.1);
        grid.1.iter().enumerate().find_map(|(r, (row_offset, row))| {
            let p = (p.0 - row_offset.0, p.1 - row_offset.1);
            row.1.iter().position(|(cell_offset, cell)| contains(*cell_offset, cell.0, p)).map(|c| (r, c))
        })
    }

    fn highlight(&mut self, cell: Option<(usize, usize)>, on: bool) {
        if let Some(cell) = cell.and_then(|c| self.1.cell(c)) { cell.1.display(on); }
    }

    fn on_touch(&mut self, ctx: &mut Context, sized: &SizedTree, mouse: &MouseEvent) {
        match mouse.state {
            MouseState::Pressed(MouseButton::Left) => {
                let on_grid = |p: (f32, f32)| sized.1.first().is_some_and(|(o, grid)| p.1 >= o.1 && p.1 < o.1 + grid.0.1);
                self.4 = mouse.position.filter(|p| on_grid(*p)).map(|p| {
                    let cell = Self::cell_at(sized, p).filter(|c| self.1.cell(*c).is_some_and(|cell| cell.3.is_some()));
                    EmojiTouch { cell, last: p, travel: (0.0, 0.0) }
                });
                if let Some(touch) = self.4 { self.highlight(touch.cell, true); }
            }
            MouseState::Scroll(dx, dy) => if let Some(touch) = &mut self.4 {
                touch.travel.0 -= dx;
                touch.travel.1 -= dy;
            },
            MouseState::Moved => if let (Some(touch), Some(p)) = (&mut self.4, mouse.position) {
                touch.travel.0 += p.0 - touch.last.0;
                touch.travel.1 += p.1 - touch.last.1;
                touch.last = p;
            },
            MouseState::Released(_) => if let Some(touch) = self.4.take() {
                self.highlight(touch.cell, false);
                let (dx, dy) = touch.travel;

                if dx.abs() >= 40.0 && dx.abs() > dy.abs() {
                    let page = if dx < 0.0 { self.3 + 1 } else { self.3.saturating_sub(1) };
                    // if page != self.3 && page < EMOJI_PAGES.len() { ctx.trigger_haptic(); }
                    self.show_page(page);
                } else if dx.hypot(dy) < 12.0
                    && let Some(emoji) = touch.cell.and_then(|c| self.1.cell(c)).and_then(|cell| cell.3.clone())
                {
                    ctx.trigger_haptic();
                    ctx.emit(KeyboardEvent{key: event::Key::Character(emoji), state: KeyboardState::Pressed, modifiers: Modifiers::default()});
                }
            },
            _ => {}
        }

        // Once a press has moved too far to be a tap, stop highlighting its emoji.
        if let Some(touch) = self.4 && touch.travel.0.hypot(touch.travel.1) >= 12.0 {
            self.highlight(touch.cell, false);
        }
    }
}

impl OnEvent for EmojiPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(MobileKeyboardEvent::EmojiCategory(category)) = event.downcast_ref::<MobileKeyboardEvent>() {
            if let Some(page) = EMOJI_PAGES.iter().position(|(c, _)| c == category) { self.show_page(page); }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            self.on_touch(ctx, sized, mouse);
        }
        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiGrid(Column, Vec<EmojiGridRow>);
impl OnEvent for EmojiGrid {}

impl EmojiGrid {
    fn new(theme: &Theme, page: usize) -> Self {
        let (c, start) = EMOJI_PAGES[page];
        let emojis = &EMOJI_CATEGORIES[c].emojis;
        let rows = (0..4).map(|r| {
            let cells = (0..8).map(|col| {
                EmojiCell::new(theme, emojis.get(start + r * 8 + col).map(String::as_str))
            }).collect();
            EmojiGridRow(Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), cells)
        }).collect();
        EmojiGrid(Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None), rows)
    }

    fn cell(&mut self, (row, col): (usize, usize)) -> Option<&mut EmojiCell> {
        self.1.get_mut(row)?.1.get_mut(col)
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiGridRow(Row, Vec<EmojiCell>);
impl OnEvent for EmojiGridRow {}

/// One emoji (or a blank), with a highlight shown while it's pressed.
#[derive(Component, Debug, Clone)]
struct EmojiCell(Stack, Opt<Rectangle>, Text, #[skip] Option<String>);
impl OnEvent for EmojiCell {}

impl EmojiCell {
    fn new(theme: &Theme, emoji: Option<&str>) -> Self {
        // Every cell asks for the same width, so all 8 columns get equal space.
        let width = Size::custom(|_: Vec<(f32, f32)>| (32.0, f32::MAX));
        EmojiCell(
            Stack(Offset::Center, Offset::Center, width, Size::Static(48.0 - 4.0), Padding(2.0, 2.0, 2.0, 2.0)),
            Opt::new(Rectangle::new(Color::from_hex("ffffff", 110), 8.0, None), false),
            Text::new(theme, emoji.unwrap_or(""), TextSize::H2, TextStyle::Keyboard, Align::Left, None),
            emoji.map(str::to_string),
        )
    }
}

/// Category tabs, with the active one highlighted, and backspace.
#[derive(Component, Debug, Clone)]
struct EmojiTabs(Row, Vec<EmojiTab>, Key);
impl OnEvent for EmojiTabs {}

impl EmojiTabs {
    fn new(theme: &Theme, active: usize) -> Self {
        let tabs = EMOJI_CATEGORIES.iter().enumerate()
            .map(|(i, category)| EmojiTab::new(theme, i, category.icon, i == active))
            .collect();
        EmojiTabs(Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), tabs, Key::backspace_compact(theme))
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTab(Stack, interactions::Button);
impl OnEvent for EmojiTab {}

impl EmojiTab {
    fn new(theme: &Theme, index: usize, icon: &str, active: bool) -> Self {
        let face = EmojiTabFace::new(theme, icon, active);
        let callback = move |ctx: &mut Context| ctx.emit(MobileKeyboardEvent::EmojiCategory(index));
        EmojiTab(Stack::default(), interactions::Button::new(face, None::<EmojiTabFace>, None::<EmojiTabFace>, None::<EmojiTabFace>, None::<EmojiTabFace>, callback, false))
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTabFace(Stack, Option<Rectangle>, Text);
impl OnEvent for EmojiTabFace {}

impl EmojiTabFace {
    fn new(theme: &Theme, icon: &str, active: bool) -> Self {
        let width = Size::custom(|_: Vec<(f32, f32)>| (28.0, f32::MAX));
        EmojiTabFace(
            Stack(Offset::Center, Offset::Center, width, Size::Static(48.0 - 8.0), Padding(2.0, 4.0, 2.0, 4.0)),
            active.then(|| Rectangle::new(Color::from_hex("ffffff", 110), 8.0, None)),
            Text::new(theme, icon, TextSize::H4, TextStyle::Keyboard, Align::Left, None),
        )
    }
}
use prism::event::{self, MouseEvent, TickEvent, KeyboardState, KeyboardEvent, OnEvent, Event, Modifiers, MouseState, MouseButton, PickedPhoto};
use prism::canvas::{Align, Image, ShapeType};
use prism::{emitters, Context};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Layout, Stack, Column, Row, Offset, Size, Padding, Area};
use prism::display::{Bin, Enum, Opt};
use std::time::{Duration, Instant};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use maverick_os::air::Instance;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon, Sticker, SendStickerEvent, StickerPack, StickerPackAction};
use crate::components::button::GhostIconButton;

const REPEAT_DELAY: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL: Duration = Duration::from_millis(80);
const FAST_AFTER: Duration = Duration::from_millis(2000);
const FAST_INTERVAL: Duration = Duration::from_millis(35);

#[derive(Debug, Clone, Default)]
struct HoldRepeat { started: Option<Instant>, last: Option<Instant> }

/// Fields after the Enum: caps on, current page of letter keys, which
/// keyboard page is showing (letters, emoji or stickers), theme.
///
/// The key layout is rebuilt whenever caps or the page changes, so the caps-lock
/// and page keys always match the real state. The emoji and sticker pages are
/// built once and keep their place while the letter keys are showing.
#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, Enum<KeyboardContent>, #[skip] bool, #[skip] usize, #[skip] KeyboardPage, #[skip] Theme);

/// The keyboard's pages: letter keys, emoji, or stickers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardPage { Keys, Emoji, Stickers }

impl OnEvent for MobileKeyboard {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // Caps works as a one-shot shift: it turns off after a character is typed.
        // Typing on the emoji page doesn't leave it.
        if let Some(KeyboardEvent{key: event::Key::Character(_), state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.3 && self.5 == KeyboardPage::Keys {
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
                MobileKeyboardEvent::ToggleEmoji => self.toggle(ctx, KeyboardPage::Emoji),
                MobileKeyboardEvent::ToggleStickers => self.toggle(ctx, KeyboardPage::Stickers),
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
            ("stickers".to_string(), KeyboardContent::stickers(ctx, theme)),
        ], "keys".to_string());

        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()),
            Rectangle::new(color, 0.0, None),
            e, false, 0, KeyboardPage::Keys, theme.clone()
        )
    }

    /// Opens the emoji or sticker page, or goes back to the letter keys if
    /// it's already open.
    fn toggle(&mut self, ctx: &mut Context, page: KeyboardPage) {
        if self.5 == page { return self.show_keys(ctx); }
        self.5 = page;
        self.2.display(match page {
            KeyboardPage::Keys => "keys",
            KeyboardPage::Emoji => "emoji",
            KeyboardPage::Stickers => "stickers",
        });
    }

    /// Shows the key layout for the current page and caps state.
    fn show_keys(&mut self, ctx: &mut Context) {
        self.5 = KeyboardPage::Keys;
        self.2.display("keys");
        *self.2.drawable().inner() = KeyboardContent::new(ctx, &self.6, self.4, self.3);
    }
}

/// Header, then the 4 rows of letter keys, the emoji page or the sticker
/// page, then the safe area below.
#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, KeyboardHeader, Option<KeyboardRow>, Option<KeyboardRow>, Option<KeyboardRow>, Option<KeyboardRow>, Option<EmojiPanel>, Option<StickerPanel>, Bin<Stack, Rectangle>, #[skip] Theme);

impl OnEvent for KeyboardContent {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // Show the emoji category or "Stickers", and the page, in the header.
        if event.downcast_ref::<TickEvent>().is_some() {
            let label = match (&self.6, &self.7) {
                (Some(panel), _) => Some(panel.label()),
                (_, Some(panel)) => Some(panel.label()),
                _ => None,
            };
            if let Some((title, page)) = label { self.1.set_label(&title, &page); }
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
            KeyboardHeader::new(theme, KeyboardPage::Keys),
            Some(KeyboardRow::top(theme, page, caps)),
            Some(KeyboardRow::middle(theme, page, caps)),
            Some(KeyboardRow::bottom(theme, page, caps)),
            Some(KeyboardRow::modifier(theme, page, caps)),
            None,
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
            KeyboardHeader::new(theme, KeyboardPage::Emoji),
            None,
            None,
            None,
            None,
            Some(EmojiPanel::new(theme)),
            None,
            Bin(layout, Rectangle::new(color, 0.0, None)),
            theme.clone()
        )
    }

    pub fn stickers(ctx: &mut Context, theme: &Theme) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        let (b, _, _, _) = ctx.get_safe_area();
        let layout = Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(b), Padding::default());

        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme, KeyboardPage::Stickers),
            None,
            None,
            None,
            None,
            None,
            Some(StickerPanel::new(theme)),
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
    fn new(theme: &Theme, page: KeyboardPage) -> Self {
        let layout = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0,0.0,0.0,2.0));
        KeyboardHeader(Column::start(0.0),
            KeyboardIcons::new(theme, page),
            Bin(layout, Rectangle::new(theme.colors().get(ptsd::Outline::Secondary), 0.0, None))
        )
    }

    fn set_label(&mut self, title: &str, detail: &str) { self.1.set_label(title, detail); }
}

/// The emoji and sticker buttons, the label, and the button that hides the keyboard.
#[derive(Component, Debug, Clone)]
struct KeyboardIcons(Row, Key, Key, Bin<Stack, HeaderLabel>, GhostIconButton);
impl OnEvent for KeyboardIcons {}
impl KeyboardIcons {
    fn new(theme: &Theme, page: KeyboardPage) -> Self {
        let state = |on: bool| if on {ButtonState::Pressed} else {ButtonState::Default};
        KeyboardIcons(
            Row::new(12.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)),
            // icons.then(|| KeyboardActions(Stack::default(), actions)),
            Key::page_button(theme, Icons::Emoji, state(page == KeyboardPage::Emoji), MobileKeyboardEvent::ToggleEmoji),
            Key::page_button(theme, Icons::Sticker, state(page == KeyboardPage::Stickers), MobileKeyboardEvent::ToggleStickers),
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
        let label = &mut self.3.1;
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
    fn backspace_compact(theme: &Theme) -> Self { Self::backspace_sized(theme, EMOJI_ROW_HEIGHT - 12.0) }

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

    /// A header button that opens a keyboard page (emoji or stickers).
    /// `state` only sets the look (highlighted while its page is open).
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
    ToggleStickers,
    /// Show the first page of this emoji category.
    EmojiCategory(usize),
}

impl Event for MobileKeyboardEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

// ---------------------------------------------------------------------------
// Emoji and sticker pages
//
// Each shows one page of cells at a time, in a fixed grid, and turns pages
// with a sideways swipe. Only the visible page is built, so a page costs the
// same however many emoji or stickers there are, and both are exactly as tall
// as the letter keys.
// ---------------------------------------------------------------------------

/// The emoji grid is 4 rows of 48px plus the 48px tab row: the same 240px as
/// the 4 rows of letter keys, so the keyboard doesn't change height.
const EMOJI_COLUMNS: usize = 8;
const EMOJI_ROWS: usize = 4;
const EMOJI_ROW_HEIGHT: f32 = 48.0;
const EMOJIS_PER_PAGE: usize = EMOJI_COLUMNS * EMOJI_ROWS;

/// The sticker grid is 3 rows of 80px: the same 240px again.
const STICKER_COLUMNS: usize = 4;
const STICKER_ROWS: usize = 3;
const STICKER_ROW_HEIGHT: f32 = 80.0;
const STICKERS_PER_PAGE: usize = STICKER_COLUMNS * STICKER_ROWS;
/// Sticker size on the page. 4 of these fit across the narrowest phones.
const STICKER_IMAGE: f32 = 72.0;
const STICKER_TITLE: &str = "Stickers";
const STICKER_OUTLINE: f32 = 3.0;

/// A press that moves less than this is a tap.
const TAP_SLOP: f32 = 12.0;
/// Sideways travel that turns the page.
const SWIPE_DISTANCE: f32 = 40.0;

struct EmojiCategory { name: String, icon: &'static str, emojis: Vec<String> }

lazy_static::lazy_static! {
    static ref EMOJI_CATEGORIES: Vec<EmojiCategory> = parse_emoji_categories(include_str!("../../emoji.txt"));

    /// Every page as (category, index of its first emoji). Pages never span
    /// categories, so each category starts on a fresh page.
    static ref EMOJI_PAGES: Vec<(usize, usize)> = EMOJI_CATEGORIES.iter().enumerate()
        .flat_map(|(c, category)| (0..category.emojis.len()).step_by(EMOJIS_PER_PAGE).map(move |start| (c, start)))
        .collect();
}

/// The tab icon for each emoji-test.txt group. `None` for "Component"
/// (skin tones and hair styles on their own), which isn't shown.
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

/// Reads Unicode's emoji-test.txt into its groups, in its (CLDR) order,
/// keeping fully-qualified emoji and leaving out skin-tone variants.
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

/// The emoji page: a page of emoji and a row of category tabs.
/// Swipe the grid sideways to turn pages; tap a tab to jump to its category.
#[derive(Component, Debug, Clone)]
struct EmojiPanel {
    layout: Column,
    grid: PanelGrid,
    tabs: EmojiTabs,
    #[skip] page: usize,
    #[skip] touch: Option<PanelTouch>,
    #[skip] theme: Theme,
}

impl EmojiPanel {
    fn new(theme: &Theme) -> Self {
        EmojiPanel {
            layout: Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None),
            grid: PanelGrid::emoji(theme, 0),
            tabs: EmojiTabs::new(theme, 0),
            page: 0,
            touch: None,
            theme: theme.clone(),
        }
    }

    fn category(&self) -> usize { EMOJI_PAGES[self.page].0 }

    fn show_page(&mut self, page: usize) {
        if page == self.page || page >= EMOJI_PAGES.len() { return; }
        let category = self.category();
        self.page = page;
        self.touch = None;
        self.grid = PanelGrid::emoji(&self.theme, page);
        if self.category() != category { self.tabs = EmojiTabs::new(&self.theme, self.category()); }
    }

    /// The category's name, and this page's number out of the category's pages.
    fn label(&self) -> (String, String) {
        let (c, start) = EMOJI_PAGES[self.page];
        let category = &EMOJI_CATEGORIES[c];
        let pages = category.emojis.len().div_ceil(EMOJIS_PER_PAGE);
        (category.name.clone(), format!("{}/{}", start / EMOJIS_PER_PAGE + 1, pages))
    }
}

impl OnEvent for EmojiPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(MobileKeyboardEvent::EmojiCategory(category)) = event.downcast_ref::<MobileKeyboardEvent>() {
            if let Some(page) = EMOJI_PAGES.iter().position(|(c, _)| c == category) { self.show_page(page); }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match track_touch(&mut self.touch, &mut self.grid, sized, mouse) {
                Some(TouchEnd::Turn(next)) => self.show_page(if next { self.page + 1 } else { self.page.saturating_sub(1) }),
                Some(TouchEnd::Tap(CellAction::Emoji(emoji))) => {
                    ctx.trigger_haptic();
                    ctx.emit(KeyboardEvent{key: event::Key::Character(emoji), state: KeyboardState::Pressed, modifiers: Modifiers::default()});
                }
                _ => {}
            }
        }
        vec![event]
    }
}

/// The sticker page: your stickers, a page at a time, starting with a tile
/// that makes a new one from a photo. Swipe sideways to turn pages; tap a
/// sticker to send it (with a [`StickerEvent::Send`] for the chat).
///
/// The stickers are your [`StickerPack`]. The page finds it (or makes it) the
/// first time it's shown, adds the stickers you make to it, and keeps up with
/// it while it's showing, so stickers added on another device appear too.
#[derive(Component, Debug, Clone)]
struct StickerPanel {
    layout: Stack,
    grid: PanelGrid,
    #[skip] page: usize,
    #[skip] touch: Option<PanelTouch>,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] waiting_for_photo: bool,
    #[skip] theme: Theme,
}

impl StickerPanel {
    fn new(theme: &Theme) -> Self {
        StickerPanel {
            layout: Stack::default(),
            grid: PanelGrid::stickers(theme, &[], 0),
            page: 0,
            touch: None,
            pack: None,
            stickers: Vec::new(),
            waiting_for_photo: false,
            theme: theme.clone(),
        }
    }

    /// The new-sticker tile takes the first cell of the first page.
    fn page_count(&self) -> usize { (self.stickers.len() + 1).div_ceil(STICKERS_PER_PAGE) }

    fn show_page(&mut self, page: usize) {
        if page == self.page || page >= self.page_count() { return; }
        self.page = page;
        self.touch = None;
        self.grid = PanelGrid::stickers(&self.theme, &self.stickers, page);
    }

    /// Shows any changes to the pack, staying on this page if it still exists.
    fn sync(&mut self, ctx: &mut Context) {
        let pack = self.pack.get_or_insert_with(|| StickerPack::mine(ctx));
        if pack.pending().stickers == self.stickers { return; }
        self.stickers = pack.pending().stickers.clone();
        self.page = self.page.min(self.page_count() - 1);
        self.touch = None;
        self.grid = PanelGrid::stickers(&self.theme, &self.stickers, self.page);
    }

    fn label(&self) -> (String, String) {
        (STICKER_TITLE.to_string(), format!("{}/{}", self.page + 1, self.page_count()))
    }
}

impl OnEvent for StickerPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            self.sync(ctx);
        } else if let Some(PickedPhoto(photo)) = event.downcast_ref::<PickedPhoto>() && self.waiting_for_photo {
            // The photo picked with the new-sticker tile.
            self.waiting_for_photo = false;
            if let Some(sticker) = Sticker::from_photo(photo) && let Some(pack) = &mut self.pack {
                pack.send(StickerPackAction::Add(sticker));
                self.sync(ctx);
            }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match track_touch(&mut self.touch, &mut self.grid, sized, mouse) {
                Some(TouchEnd::Turn(next)) => self.show_page(if next { self.page + 1 } else { self.page.saturating_sub(1) }),
                Some(TouchEnd::Tap(CellAction::Sticker(sticker))) => {
                    ctx.trigger_haptic();
                    ctx.emit(SendStickerEvent(sticker));
                }
                Some(TouchEnd::Tap(CellAction::NewSticker)) => {
                    self.waiting_for_photo = true;
                    ctx.pick_photo();
                }
                _ => {}
            }
        }
        vec![event]
    }
}

/// A press on a page's grid: the cell it started on, and how far it has moved.
#[derive(Debug, Clone, Copy)]
struct PanelTouch { cell: Option<(usize, usize)>, last: (f32, f32), travel: (f32, f32) }

/// How a press on a page's grid ended.
enum TouchEnd {
    /// A tap on a cell.
    Tap(CellAction),
    /// A sideways swipe: `true` for the next page, `false` for the previous.
    Turn(bool),
}

/// Follows a press on a page whose first child is its grid, highlighting the
/// cell under it. Taps and swipes are handled here rather than by buttons on
/// each cell: a tap tolerates a little finger movement, and a swipe never taps.
fn track_touch(touch: &mut Option<PanelTouch>, grid: &mut PanelGrid, sized: &SizedTree, mouse: &MouseEvent) -> Option<TouchEnd> {
    let mut end = None;
    match mouse.state {
        MouseState::Pressed(MouseButton::Left) => {
            let on_grid = |p: (f32, f32)| sized.1.first().is_some_and(|(o, g)| p.1 >= o.1 && p.1 < o.1 + g.0.1);
            *touch = mouse.position.filter(|p| on_grid(*p)).map(|p| {
                let cell = PanelGrid::cell_at(sized, p).filter(|c| grid.cell(*c).is_some_and(|cell| cell.3.is_some()));
                PanelTouch { cell, last: p, travel: (0.0, 0.0) }
            });
            if let Some(t) = *touch { grid.highlight(t.cell, true); }
        }
        // Touch drags arrive as scroll deltas (the negated finger movement),
        // and keep arriving after the finger leaves the page.
        MouseState::Scroll(dx, dy) => if let Some(t) = touch {
            t.travel.0 -= dx;
            t.travel.1 -= dy;
        },
        // A mouse drags with positions instead.
        MouseState::Moved => if let (Some(t), Some(p)) = (touch.as_mut(), mouse.position) {
            t.travel.0 += p.0 - t.last.0;
            t.travel.1 += p.1 - t.last.1;
            t.last = p;
        },
        MouseState::Released(_) => if let Some(t) = touch.take() {
            grid.highlight(t.cell, false);
            let (dx, dy) = t.travel;
            if dx.abs() >= SWIPE_DISTANCE && dx.abs() > dy.abs() {
                // Finger moving left shows the next page, right the previous one.
                end = Some(TouchEnd::Turn(dx < 0.0));
            } else if dx.hypot(dy) < TAP_SLOP
                && let Some(action) = t.cell.and_then(|c| grid.cell(c)).and_then(|cell| cell.3.clone())
            {
                end = Some(TouchEnd::Tap(action));
            }
        },
        _ => {}
    }

    // Once a press has moved too far to be a tap, stop highlighting its cell.
    if let Some(t) = *touch && t.travel.0.hypot(t.travel.1) >= TAP_SLOP {
        grid.highlight(t.cell, false);
    }
    end
}

/// One page of cells: emoji in 4 rows of 8, or stickers in 3 rows of 4.
/// Always full rows (blank cells past the end), so the columns line up and
/// the height never changes.
#[derive(Component, Debug, Clone)]
struct PanelGrid(Column, Vec<PanelRow>);
impl OnEvent for PanelGrid {}

impl PanelGrid {
    fn emoji(theme: &Theme, page: usize) -> Self {
        let (c, start) = EMOJI_PAGES[page];
        let emojis = &EMOJI_CATEGORIES[c].emojis;
        Self::new(EMOJI_ROWS, EMOJI_COLUMNS, PanelCell::emoji_layout(), 6.0, |i| {
            emojis.get(start + i).map(|emoji| PanelCell::emoji(theme, emoji))
        })
    }

    /// Sticker page `page`. The first page starts with the new-sticker tile.
    fn stickers(theme: &Theme, stickers: &[Sticker], page: usize) -> Self {
        let start = page * STICKERS_PER_PAGE;
        Self::new(STICKER_ROWS, STICKER_COLUMNS, PanelCell::sticker_layout(), 12.0, |i| match start + i {
            0 => Some(PanelCell::new_sticker(theme)),
            n => stickers.get(n - 1).map(PanelCell::sticker),
        })
    }

    /// `cell(i)` gives the i-th cell on the page; `None` leaves it blank.
    fn new(rows: usize, columns: usize, layout: Stack, spacing: f32, mut cell: impl FnMut(usize) -> Option<PanelCell>) -> Self {
        let rows = (0..rows).map(|r| {
            let cells = (0..columns).map(|c| {
                cell(r * columns + c).unwrap_or_else(|| PanelCell::blank(layout.clone()))
            }).collect();
            PanelRow(Row::new(spacing, Offset::Start, Size::Fit, Padding::new(0.0)), cells)
        }).collect();
        PanelGrid(Column::new(spacing, Offset::Start, Size::Fill, Padding::new(12.0), None), rows)
    }

    fn cell(&mut self, (row, col): (usize, usize)) -> Option<&mut PanelCell> {
        self.1.get_mut(row)?.1.get_mut(col)
    }

    /// The cell under `pos`, given the sized tree of the page the grid is the
    /// first child of, and `pos` in that page's coordinates.
    fn cell_at(page: &SizedTree, pos: (f32, f32)) -> Option<(usize, usize)> {
        let contains = |offset: (f32, f32), size: (f32, f32), p: (f32, f32)|
            p.0 >= offset.0 && p.0 < offset.0 + size.0 && p.1 >= offset.1 && p.1 < offset.1 + size.1;

        let (grid_offset, grid) = page.1.first()?;
        let p = (pos.0 - grid_offset.0, pos.1 - grid_offset.1);
        grid.1.iter().enumerate().find_map(|(r, (row_offset, row))| {
            let p = (p.0 - row_offset.0, p.1 - row_offset.1);
            row.1.iter().position(|(cell_offset, cell)| contains(*cell_offset, cell.0, p)).map(|c| (r, c))
        })
    }

    fn highlight(&mut self, cell: Option<(usize, usize)>, on: bool) {
        if let Some(cell) = cell.and_then(|c| self.cell(c)) { cell.press(on); }
    }
}

#[derive(Component, Debug, Clone)]
struct PanelRow(Row, Vec<PanelCell>);
impl OnEvent for PanelRow {}

/// What tapping a cell does.
#[derive(Debug, Clone)]
enum CellAction {
    /// Types the emoji.
    Emoji(String),
    /// Sends the sticker.
    Sticker(Sticker),
    /// Picks a photo to make a new sticker from.
    NewSticker,
}

/// One cell: a highlight shown while it's pressed, what's in it, and what
/// tapping it does (blank cells have neither).

/// One cell: a highlight shown behind an emoji while it's pressed, what's in
/// it, and what tapping it does (blank cells have neither).
#[derive(Component, Debug, Clone)]
struct PanelCell(Stack, Opt<Rectangle>, Option<CellFace>, #[skip] Option<CellAction>);
impl OnEvent for PanelCell {}

#[derive(Component, Debug, Clone)]
enum CellFace {
    Emoji {layout: Stack, text: Text},
    /// The outline is drawn over the sticker while it's pressed.
    Sticker {layout: Stack, image: Image, outline: Opt<Rectangle>},
    NewSticker {layout: Stack, background: Rectangle, icon: Image, outline: Opt<Rectangle>},
}
impl OnEvent for CellFace {}

impl PanelCell {
    /// Every cell in a grid asks for the same width, so the columns get equal space.
    fn layout(width: f32, height: f32) -> Stack {
        let width = Size::custom(move |_: Vec<(f32, f32)>| (width, f32::MAX));
        Stack(Offset::Center, Offset::Center, width, Size::Static(height), Padding(2.0, 2.0, 2.0, 2.0))
    }

    fn emoji_layout() -> Stack { Self::layout(32.0, EMOJI_ROW_HEIGHT) }

    /// Fixed at sticker size, so blank cells keep their place on the last page.
    fn sticker_layout() -> Stack { Self::sticker_box() }

    /// A sticker-sized box. The outline fills it exactly, so it never grows.
    fn sticker_box() -> Stack {
        Stack(Offset::Center, Offset::Center, Size::Static(STICKER_IMAGE), Size::Static(STICKER_IMAGE), Padding::default())
    }

    /// Shown behind an emoji while it's pressed.
    fn highlight(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::from_hex("ffffff", 110), radius, None), false)
    }

    /// Drawn over a sticker while it's pressed (inside its edge).
    fn outline() -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::TRANSPARENT, 12.0, Some((STICKER_OUTLINE, Color::WHITE))), false)
    }

    /// Shows the cell as pressed: an outline over a sticker, or a highlight
    /// behind an emoji.
    fn press(&mut self, on: bool) {
        match &mut self.2 {
            Some(CellFace::Sticker {outline, ..}) | Some(CellFace::NewSticker {outline, ..}) => outline.display(on),
            _ => self.1.display(on),
        }
    }

    fn emoji(theme: &Theme, emoji: &str) -> Self {
        let text = Text::new(theme, emoji, TextSize::H2, TextStyle::Keyboard, Align::Left, None);
        let face = CellFace::Emoji {layout: Stack::default(), text};
        PanelCell(Self::emoji_layout(), Self::highlight(8.0), Some(face), Some(CellAction::Emoji(emoji.to_string())))
    }

    fn sticker(sticker: &Sticker) -> Self {
        let shape = ShapeType::RoundedRectangle(0.0, (STICKER_IMAGE, STICKER_IMAGE), 0.0, 12.0);
        let face = CellFace::Sticker {
            layout: Self::sticker_box(),
            image: Image {shape, image: sticker.image(), color: None},
            outline: Self::outline(),
        };
        PanelCell(Self::sticker_layout(), Self::highlight(12.0), Some(face), Some(CellAction::Sticker(sticker.clone())))
    }

    fn new_sticker(theme: &Theme) -> Self {
        let face = CellFace::NewSticker {
            layout: Self::sticker_box(),
            background: Rectangle::new(Color::from_hex("ffffff", 110), 12.0, None),
            icon: Icon::new(theme, Icons::Add, Some(Color::WHITE), 36.0),
            outline: Self::outline(),
        };
        PanelCell(Self::sticker_layout(), Self::highlight(12.0), Some(face), Some(CellAction::NewSticker))
    }

    fn blank(layout: Stack) -> Self {
        PanelCell(layout, Self::highlight(0.0), None, None)
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
            Stack(Offset::Center, Offset::Center, width, Size::Static(EMOJI_ROW_HEIGHT - 8.0), Padding(2.0, 4.0, 2.0, 4.0)),
            active.then(|| Rectangle::new(Color::from_hex("ffffff", 110), 8.0, None)),
            Text::new(theme, icon, TextSize::H4, TextStyle::Keyboard, Align::Left, None),
        )
    }
}
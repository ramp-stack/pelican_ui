use prism::event::{self, MouseEvent, TickEvent, KeyboardState, KeyboardEvent, OnEvent, Event, Modifiers, MouseState, MouseButton, PickedPhoto};
use prism::canvas::{Align, Image, ShapeType, Instruction};
use prism::{emitters, Context};
use prism::drawable::{Drawable, Component, SizedTree, RequestTree, Rect};
use prism::layout::{Layout, Stack, Column, Row, Offset, Size, Padding, Area, ScrollAnchor};
use prism::display::{Bin, Enum, Opt};
use std::time::{Duration, Instant};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use maverick_os::air::Instance;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon, Sticker, SendStickerEvent, StickerPack, StickerPackAction};
use crate::components::{Gif, GifPlayer, GifBatch, GifError, SendGifEvent, Task, find_gifs};
use crate::components::TextInput;

use unicode_segmentation::UnicodeSegmentation;
use crate::components::button::GhostIconButton;

const REPEAT_DELAY: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL: Duration = Duration::from_millis(80);
const FAST_AFTER: Duration = Duration::from_millis(2000);
const FAST_INTERVAL: Duration = Duration::from_millis(35);

#[derive(Debug, Clone, Default)]
struct HoldRepeat { started: Option<Instant>, last: Option<Instant>, target: KeyTarget }

/// Where the keys type: into the focused text field, or into the GIF search
/// field, where return runs the search instead of starting a new line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum KeyTarget { #[default] Input, GifSearch }

impl KeyTarget {
    fn press(self, ctx: &mut Context, key: event::Key) {
        match (self, key) {
            (KeyTarget::GifSearch, event::Key::Enter) => ctx.emit(MobileKeyboardEvent::SubmitGifSearch),
            (_, key) => ctx.emit(KeyboardEvent{key, state: KeyboardState::Pressed, modifiers: Modifiers::default()}),
        }
    }
}

/// The group prism puts every text field in (`emitters::TEXT_INPUT_UUID`),
/// to tell a text field getting the focus from other selections.
const TEXT_FIELD_GROUP: &str = "123e4567-e89b-12d3-a456-426614174000";

/// What the GIF search field says while it's empty.
const GIF_SEARCH_PLACEHOLDER: &str = "Search GIPHY";

/// The pages a text field's keyboard offers besides the letter keys. A
/// text field says which when it gets the focus (see
/// [`TextInput::keyboard_pages`]): every field has emoji, but only the chat's
/// message field has stickers and GIFs, which it sends as messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardPages { pub emoji: bool, pub stickers: bool, pub gifs: bool }

impl KeyboardPages {
    /// Emoji, stickers and GIFs: for a chat's message field.
    pub const ALL: Self = KeyboardPages { emoji: true, stickers: true, gifs: true };
    /// Just emoji: for searching for a username, say. Text fields have these
    /// unless they say otherwise.
    pub const EMOJI: Self = KeyboardPages { emoji: true, stickers: false, gifs: false };
    /// Only the letter keys.
    pub const NONE: Self = KeyboardPages { emoji: false, stickers: false, gifs: false };

    fn allows(&self, page: KeyboardPage) -> bool {
        match page {
            KeyboardPage::Keys => true,
            KeyboardPage::Emoji => self.emoji,
            KeyboardPage::Stickers => self.stickers,
            KeyboardPage::Gifs => self.gifs,
        }
    }
}

impl Default for KeyboardPages {
    fn default() -> Self { Self::EMOJI }
}

impl Event for KeyboardPages {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// The keyboard filling the screen (`true`, searching GIFs) or back to its
/// usual size. The pages behind it give it their room meanwhile.
#[derive(Debug, Clone, Copy)]
pub struct KeyboardFullScreen(pub bool);

impl Event for KeyboardFullScreen {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Fields after the Enum: caps on, current page of letter keys, which
/// keyboard page is showing (letters, emoji, stickers or GIFs), theme, the
/// GIF search, and the pages the focused text field offers.
///
/// The key layout is rebuilt whenever caps or the page changes, so the caps-lock
/// and page keys always match the real state. The emoji, sticker and GIF pages
/// are built once and keep their place while the letter keys are showing.
#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, Enum<KeyboardContent>, #[skip] bool, #[skip] usize, #[skip] KeyboardPage, #[skip] Theme, #[skip] GifSearch, #[skip] KeyboardPages);

/// The keyboard's pages: letter keys, emoji, stickers, or GIFs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardPage { Keys, Emoji, Stickers, Gifs }

/// Searching GIFs, the keyboard fills the screen: the search field at the
/// top, the GIFs, and the letter keys while you type. The search field is a
/// real text field, so it takes the focus from the one the keyboard was
/// typing into, and the search gives it back when it ends.
#[derive(Debug, Clone, Default)]
struct GifSearch {
    open: bool,
    /// The text field that last got the focus (prism's id and group for it).
    focused: Option<(String, String)>,
    /// The one that had it when the search opened.
    return_to: Option<(String, String)>,
}

impl OnEvent for MobileKeyboard {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // Caps works as a one-shot shift: it turns off after a character is typed.
        // Typing on the emoji page doesn't leave it.
        if let Some(KeyboardEvent{key: event::Key::Character(_), state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.3 && (self.5 == KeyboardPage::Keys || self.7.open) {
            self.3 = false;
            self.refresh_keys(ctx);
        }

        if self.7.open {
            // Sending a GIF ends the search, back to the chat.
            if event.downcast_ref::<SendGifEvent>().is_some() {
                self.close_gif_search(ctx, true);
            // The search field tapped again, after searching hid the keys.
            } else if let Some(ShowKeyboard(true)) = event.downcast_ref::<ShowKeyboard>()
                && self.2.drawable().inner().5.is_none() {
                self.set_search_keys(true);
            }
        }

        if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::Paginator(page) => {
                    self.4 = *page;
                    self.refresh_keys(ctx);
                },
                MobileKeyboardEvent::Capslock(caps) => {
                    self.3 = *caps;
                    self.refresh_keys(ctx);
                },
                MobileKeyboardEvent::ToggleEmoji => self.toggle(ctx, KeyboardPage::Emoji),
                MobileKeyboardEvent::ToggleStickers => self.toggle(ctx, KeyboardPage::Stickers),
                MobileKeyboardEvent::ToggleGifs => self.toggle(ctx, KeyboardPage::Gifs),
                MobileKeyboardEvent::StartGifSearch => self.open_gif_search(ctx),
                MobileKeyboardEvent::CancelGifSearch => self.close_gif_search(ctx, true),
                MobileKeyboardEvent::SubmitGifSearch => self.submit_gif_search(),
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
            ("keys".to_string(), KeyboardContent::new(ctx, theme, 0, false, KeyboardPages::default())),
            ("emoji".to_string(), KeyboardContent::emoji(ctx, theme)),
            ("stickers".to_string(), KeyboardContent::stickers(ctx, theme)),
            ("gifs".to_string(), KeyboardContent::gifs(ctx, theme)),
        ], "keys".to_string());

        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()),
            Rectangle::new(color, 0.0, None),
            e, false, 0, KeyboardPage::Keys, theme.clone(), GifSearch::default(), KeyboardPages::default()
        )
    }

    /// Opens the emoji, sticker or GIF page, or goes back to the letter keys
    /// if it's already open.
    fn toggle(&mut self, ctx: &mut Context, page: KeyboardPage) {
        if self.5 == page { return self.show_keys(ctx); }
        self.open(page);
    }

    fn open(&mut self, page: KeyboardPage) {
        self.5 = page;
        self.2.display(match page {
            KeyboardPage::Keys => "keys",
            KeyboardPage::Emoji => "emoji",
            KeyboardPage::Stickers => "stickers",
            KeyboardPage::Gifs => "gifs",
        });
        // Its page buttons are the ones the focused field offers.
        let header = KeyboardHeader::new(&self.6, page, self.8);
        let content = self.2.drawable().inner();
        content.1 = header;
        if let Some(panel) = content.gif_panel() { panel.retry(); }
    }

    /// Shows the key layout for the current page and caps state.
    fn show_keys(&mut self, ctx: &mut Context) {
        self.5 = KeyboardPage::Keys;
        self.2.display("keys");
        *self.2.drawable().inner() = KeyboardContent::new(ctx, &self.6, self.4, self.3, self.8);
    }

    /// The pages the focused text field offers. The interface passes them on
    /// whenever a field gets the focus. A page it doesn't offer goes back to
    /// the letter keys.
    pub fn set_pages(&mut self, ctx: &mut Context, pages: KeyboardPages) {
        // The GIF search's own field doesn't change what the chat's offers.
        if self.7.open || pages == self.8 { return; }
        self.8 = pages;
        match pages.allows(self.5) {
            true => self.open(self.5),
            false => self.show_keys(ctx),
        }
    }

    /// After caps or the page of keys changes: rebuilds the letter keys, or
    /// the keys under the GIF search.
    fn refresh_keys(&mut self, ctx: &mut Context) {
        match self.7.open {
            true => self.set_search_keys(true),
            false => self.show_keys(ctx),
        }
    }

    /// Shows (building them for the caps state and page of keys) or hides the
    /// keys under the GIF search.
    fn set_search_keys(&mut self, show: bool) {
        let keys = show.then(|| KeyRows::new(&self.6, self.4, self.3, KeyTarget::GifSearch));
        self.2.drawable().inner().5 = keys;
    }

    /// Opens the GIF search (from the GIF page), filling the screen.
    fn open_gif_search(&mut self, ctx: &mut Context) {
        if self.7.open || self.5 != KeyboardPage::Gifs { return; }
        self.7.open = true;
        self.7.return_to = self.7.focused.clone();
        let theme = self.6.clone();
        let content = self.2.drawable().inner();
        let query = content.gif_panel().and_then(|panel| panel.query.clone()).unwrap_or_default();
        content.1 = KeyboardHeader::search(&theme, &query);
        if let Some(panel) = content.gif_panel() { panel.set_full(true); }
        self.set_search_keys(true);
        ctx.emit(KeyboardFullScreen(true));
    }

    /// Searches for what's in the search field (trending GIFs if it's
    /// empty), and hides the keys so the GIFs have the room.
    fn submit_gif_search(&mut self) {
        if !self.7.open { return; }
        self.set_search_keys(false);
        let content = self.2.drawable().inner();
        let query = content.1.search_query().unwrap_or_default().trim().to_string();
        if let Some(panel) = content.gif_panel() { panel.search((!query.is_empty()).then_some(query)); }
    }

    /// Back to the GIF page at its usual size. The text field the keyboard
    /// was typing into gets the focus back, unless the keyboard is going away.
    fn close_gif_search(&mut self, ctx: &mut Context, give_back_focus: bool) {
        if !std::mem::take(&mut self.7.open) { return; }
        self.set_search_keys(false);
        let theme = self.6.clone();
        let content = self.2.drawable().inner();
        content.1 = KeyboardHeader::new(&theme, KeyboardPage::Gifs, self.8);
        if let Some(panel) = content.gif_panel() { panel.set_full(false); }
        ctx.emit(KeyboardFullScreen(false));
        if let Some((id, group)) = self.7.return_to.take() && give_back_focus {
            ctx.emit(event::Selectable::Pressed(id, group));
        }
    }

    /// Whether the GIF search is open (the keyboard filling the screen).
    pub fn searching_gifs(&self) -> bool { self.7.open }

    /// The interface calls this while the keyboard isn't showing (hidden,
    /// or the page changed). That ends a GIF search.
    pub fn hidden(&mut self, ctx: &mut Context) {
        self.close_gif_search(ctx, false);
    }

    /// Something was selected. The interface passes on every selection, so
    /// the keyboard knows which text field has the focus (to give it back
    /// after a GIF search).
    pub fn selected(&mut self, id: &str, group: &str) {
        if group == TEXT_FIELD_GROUP { self.7.focused = Some((id.to_string(), group.to_string())); }
    }
}

/// Header, then the emoji, sticker or GIF page and/or the 4 rows of letter
/// keys (searching GIFs, the keys go under the GIFs), then the safe area below.
#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, KeyboardHeader, Option<EmojiPanel>, Option<StickerPanel>, Option<GifPanel>, Option<KeyRows>, Bin<Stack, Rectangle>);

impl OnEvent for KeyboardContent {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // Show the emoji category, "Stickers" or what the GIFs are in the header.
        if event.downcast_ref::<TickEvent>().is_some() {
            let label = match (&self.2, &self.3, &self.4) {
                (Some(panel), _, _) => Some(panel.label()),
                (_, Some(panel), _) => Some(panel.label()),
                (_, _, Some(panel)) => Some(panel.label()),
                _ => None,
            };
            if let Some((title, page)) = label { self.1.set_label(&title, &page); }
        }
        vec![event]
    }
}

impl KeyboardContent {
    /// The letter keys.
    pub fn new(ctx: &mut Context, theme: &Theme, page: usize, caps: bool, pages: KeyboardPages) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::new(theme, KeyboardPage::Keys, pages));
        content.5 = Some(KeyRows::new(theme, page, caps, KeyTarget::Input));
        content
    }

    pub fn emoji(ctx: &mut Context, theme: &Theme) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::new(theme, KeyboardPage::Emoji, KeyboardPages::ALL));
        content.2 = Some(EmojiPanel::new(theme));
        content
    }

    pub fn stickers(ctx: &mut Context, theme: &Theme) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::new(theme, KeyboardPage::Stickers, KeyboardPages::ALL));
        content.3 = Some(StickerPanel::new(theme));
        content
    }

    pub fn gifs(ctx: &mut Context, theme: &Theme) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::new(theme, KeyboardPage::Gifs, KeyboardPages::ALL));
        content.4 = Some(GifPanel::new(theme));
        content
    }

    /// Just the header (and the safe area below). The emoji, sticker and GIF
    /// pages get their header's buttons when they're opened.
    fn with_header(ctx: &mut Context, theme: &Theme, header: KeyboardHeader) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        let (b, _, _, _) = ctx.get_safe_area();
        let layout = Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(b), Padding::default());

        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            header,
            None,
            None,
            None,
            None,
            Bin(layout, Rectangle::new(color, 0.0, None)),
        )
    }

    fn gif_panel(&mut self) -> Option<&mut GifPanel> { self.4.as_mut() }
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
// Capslock, Paginator, Character Row, Spacebar, Return
impl OnEvent for KeyboardRow {}

impl KeyboardRow {
    fn top(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let key_row = KeyRow::new(theme, top_keys(&num), caps_on, target);
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn middle(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let key_row = KeyRow::new(theme, mid_keys(&num), caps_on, target);
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn bottom(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let capslock = Capslock::new(theme, caps_on);
        let backspace = Key::backspace(theme, target);
        let key_row = KeyRow::new(theme, bot_keys(&num), caps_on, target);
        KeyboardRow(Row::center(6.0), Some(capslock), None, Some(key_row), None, Some(backspace))
    }

    fn modifier(theme: &Theme, num: usize, caps_on: bool, target: KeyTarget) -> Self {
        let paginator = Paginator::new(theme, num);
        let spacebar = Key::spacebar(theme, caps_on, target);
        let newline = Key::newline(theme, caps_on, target);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }

    fn capslock(&mut self) -> &mut Option<Capslock> {&mut self.1}
    fn paginator(&mut self) -> &mut Option<Paginator> {&mut self.2}
}

/// The 4 rows of letter keys.
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


/// The page buttons and label, or (searching GIFs) the search field.
#[derive(Component, Debug, Clone)]
struct KeyboardHeader(Column, Option<KeyboardIcons>, Option<GifSearchHeader>, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardHeader {}

impl KeyboardHeader {
    fn new(theme: &Theme, page: KeyboardPage, pages: KeyboardPages) -> Self {
        KeyboardHeader(Column::start(0.0), Some(KeyboardIcons::new(theme, page, pages)), None, Self::divider(theme))
    }

    fn search(theme: &Theme, query: &str) -> Self {
        KeyboardHeader(Column::start(0.0), None, Some(GifSearchHeader::new(theme, query)), Self::divider(theme))
    }

    fn divider(theme: &Theme) -> Bin<Stack, Rectangle> {
        let layout = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0,0.0,0.0,2.0));
        Bin(layout, Rectangle::new(theme.colors().get(ptsd::Outline::Secondary), 0.0, None))
    }

    fn set_label(&mut self, title: &str, detail: &str) {
        if let Some(icons) = &mut self.1 { icons.set_label(title, detail); }
    }

    /// What's in the GIF search field.
    fn search_query(&self) -> Option<String> {
        self.2.as_ref().map(|search| search.2.value())
    }
}

/// The emoji, sticker and GIF buttons (those the focused field offers), the
/// label, and the button that hides the keyboard.
#[derive(Component, Debug, Clone)]
struct KeyboardIcons(Row, Option<Key>, Option<Key>, Option<Key>, Bin<Stack, HeaderLabel>, GhostIconButton);
impl OnEvent for KeyboardIcons {}
impl KeyboardIcons {
    fn new(theme: &Theme, page: KeyboardPage, pages: KeyboardPages) -> Self {
        let button = |icon: Icons, of: KeyboardPage, event: MobileKeyboardEvent| pages.allows(of).then(|| {
            let state = if page == of {ButtonState::Pressed} else {ButtonState::Default};
            Key::page_button(theme, icon, state, event)
        });
        KeyboardIcons(
            Row::new(12.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)),
            button(Icons::Emoji, KeyboardPage::Emoji, MobileKeyboardEvent::ToggleEmoji),
            button(Icons::Sticker, KeyboardPage::Stickers, MobileKeyboardEvent::ToggleStickers),
            button(Icons::Gif, KeyboardPage::Gifs, MobileKeyboardEvent::ToggleGifs),
            // Fills the space between the keys; shows the emoji category and page.
            // It never asks for more room than there is, so a long label can't
            // widen the keyboard.
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

    fn set_label(&mut self, title: &str, _detail: &str) {
        let label = &mut self.4.1;
        if label.1.spans[0] != title { label.1.spans[0] = title.to_string(); }
        // if label.2.spans[0] != detail { label.2.spans[0] = detail.to_string(); }
    }
}

/// Searching GIFs: a back button (to the GIF page) and the search field.
///
/// The field gets the focus once it's showing, as a tap on it would give it.
/// The last field counts down the frames until then (the field has to have
/// been laid out, to be tapped).
#[derive(Component, Debug, Clone)]
struct GifSearchHeader(Row, GhostIconButton, TextInput, #[skip] Option<u8>);

impl GifSearchHeader {
    fn new(theme: &Theme, query: &str) -> Self {
        let mut back = GhostIconButton::new(theme, Icons::Left, |ctx: &mut Context, _: &Theme| ctx.emit(MobileKeyboardEvent::CancelGifSearch));
        back.1.set_trigger_on_release();
        let field = TextInput::new(theme, (!query.is_empty()).then_some(query), None, Some(GIF_SEARCH_PLACEHOLDER), None, None, KeyboardPages::NONE);
        GifSearchHeader(Row::new(12.0, Offset::Center, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)), back, field, Some(2))
    }
}

impl OnEvent for GifSearchHeader {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() && let Some(frames) = &mut self.3 {
            *frames = frames.saturating_sub(1);
            if *frames == 0 && let Some((_, field)) = sized.1.get(1) {
                self.3 = None;
                // A tap at the field's right end, which also puts the cursor
                // after any text in it.
                let at = Some(((field.0.0 - 16.0).max(0.0), field.0.1 / 2.0));
                for state in [MouseState::Pressed(MouseButton::Left), MouseState::Released(MouseButton::Left)] {
                    self.2.event(ctx, field, Box::new(MouseEvent { position: at, state }));
                }
            }
        }
        vec![event]
    }
}

/// `text` cut to `max` characters with "…", keeping its end or its start.
fn shorten(text: &str, max: usize, keep_end: bool) -> String {
    let chars: Vec<&str> = text.graphemes(true).collect();
    if chars.len() <= max { return text.to_string(); }
    match keep_end {
        true => format!("\u{2026}{}", chars[chars.len() + 1 - max..].concat()),
        false => format!("{}\u{2026}", chars[..max - 1].concat()),
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
                        *hold = HoldRepeat { started: Some(Instant::now()), last: None, target: hold.target };
                    }
                    // Stop on release anywhere, or when the finger slides off the key.
                    MouseState::Released(_) => *hold = HoldRepeat { target: hold.target, ..HoldRepeat::default() },
                    MouseState::Moved if mouse.position.is_none() => *hold = HoldRepeat { target: hold.target, ..HoldRepeat::default() },
                    _ => {}
                }
            } else if event.downcast_ref::<TickEvent>().is_some()
                && let Some(started) = hold.started
            {
                let held = started.elapsed();
                let interval = if held >= FAST_AFTER { FAST_INTERVAL } else { REPEAT_INTERVAL };
                if held >= REPEAT_DELAY && hold.last.is_none_or(|t| t.elapsed() >= interval) {
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

    /// Return, or (typing a GIF search) the key that runs the search.
    fn newline(theme: &Theme, caps_on: bool, target: KeyTarget) -> Self {
        let label = match target { KeyTarget::Input => "return", KeyTarget::GifSearch => "search" };
        let default = _Key::newline(theme, caps_on, ButtonState::Default, label);
        let pressed = _Key::newline(theme, caps_on, ButtonState::Pressed, label);
        let callback = Box::new(move |ctx: &mut Context| target.press(ctx, event::Key::Enter));
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn backspace(theme: &Theme, target: KeyTarget) -> Self { Self::backspace_sized(theme, 48.0, target) }

    /// Backspace for the emoji panel's 48px tab row.
    fn backspace_compact(theme: &Theme) -> Self { Self::backspace_sized(theme, EMOJI_ROW_HEIGHT - 12.0, KeyTarget::Input) }

    fn backspace_sized(theme: &Theme, height: f32, target: KeyTarget) -> Self {
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

    /// A header button that opens a keyboard page (emoji, stickers or GIFs).
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
    ToggleGifs,
    /// Show the first page of this emoji category.
    EmojiCategory(usize),
    /// Open the GIF search (the search button on the GIF page was tapped).
    StartGifSearch,
    /// Back to the GIF page from the GIF search.
    CancelGifSearch,
    /// Search for what's in the GIF search field (its return key was tapped).
    SubmitGifSearch,
}

impl Event for MobileKeyboardEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

// ---------------------------------------------------------------------------
// Emoji, sticker and GIF pages
//
// Emoji show one page of cells at a time, in a fixed grid, and turn pages
// with a sideways swipe. Stickers and GIFs are in columns that scroll up and
// down, built a few rows at a time as you near the bottom. Either way only
// what you've reached is built, so the pages cost about the same however
// many emoji, stickers or GIFs there are.
// ---------------------------------------------------------------------------

/// The emoji grid is 4 rows of 48px plus the 48px tab row: the same 240px as
/// the 4 rows of letter keys, so the keyboard doesn't change height.
const EMOJI_COLUMNS: usize = 8;
const EMOJI_ROWS: usize = 4;
const EMOJI_ROW_HEIGHT: f32 = 48.0;
const EMOJIS_PER_PAGE: usize = EMOJI_COLUMNS * EMOJI_ROWS;

/// The sticker and GIF pages: square cells in rows that scroll, 4 stickers
/// or 3 GIFs to a row, as big as fills the width (see [`cell_side`]). Both
/// pages are as tall as 2 rows of GIFs and the GIF bar (see [`tray_height`]),
/// so the keyboard doesn't change height between them.
const STICKER_COLUMNS: usize = 4;
const STICKER_TITLE: &str = "Stickers";
/// Width of the outline drawn over a sticker or GIF while it's pressed.
const STICKER_OUTLINE: f32 = 3.0;

const GIF_COLUMNS: usize = 3;
/// The bar above the GIFs (search button and attribution).
const GIF_BAR_HEIGHT: f32 = 48.0;
/// GIPHY asks apps to show this wherever its GIFs are.
const GIPHY_ATTRIBUTION: &str = "Powered by GIPHY";
/// A page lays out for this width until it knows how wide it is (its first
/// frame showing).
const ASSUMED_WIDTH: f32 = 374.0;

/// Space between stickers or GIFs, and around them.
const GRID_SPACING: f32 = 12.0;
const GRID_PADDING: f32 = 12.0;
/// Sticker and GIF rows are built this many at a time, once the bottom of
/// the ones built is this close to showing.
const LOAD_ROWS: usize = 4;
/// GIFs play this many rows above and below the ones showing. Further away
/// they stop and let go of their frames.
const LIVE_MARGIN_ROWS: usize = 2;

/// The side of a square cell, `columns` to a row filling `width`: the width
/// less the grid's padding at the sides and spacing between, shared out.
fn cell_side(width: f32, columns: usize) -> f32 {
    let gaps = 2.0 * GRID_PADDING + (columns - 1) as f32 * GRID_SPACING;
    ((width - gaps) / columns as f32).floor().max(1.0)
}

/// The height of the sticker and GIF pages, `width` wide.
fn tray_height(width: f32) -> f32 {
    GRID_PADDING + 2.0 * (cell_side(width, GIF_COLUMNS) + GRID_SPACING) + GIF_BAR_HEIGHT
}

/// A page's width from its sized tree, once it's been laid out.
fn laid_out_width(sized: &SizedTree) -> Option<f32> {
    // Less than a row of cells means it hasn't been yet.
    (sized.0.0 > 100.0).then_some(sized.0.0)
}

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
            match track_touch(&mut self.touch, &mut self.grid, GridPlace::find(sized, &[0]), mouse) {
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

/// The sticker page: your stickers in a column of rows that scrolls up and
/// down, starting with a tile that makes a new one from a photo. Tap a sticker
/// to send it (with a [`SendStickerEvent`] for the chat).
///
/// Rows are built a few at a time as you scroll near the bottom, so a big
/// pack doesn't slow the keyboard down.
///
/// The stickers are your [`StickerPack`]. The page finds it (or makes it) the
/// first time it's shown, adds the stickers you make to it, and keeps up with
/// it while it's showing, so stickers added on another device appear too.
#[derive(Component, Debug, Clone)]
struct StickerPanel {
    layout: Stack,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] waiting_for_photo: bool,
    /// The side of a sticker: none are built until it's known.
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl StickerPanel {
    fn new(theme: &Theme) -> Self {
        StickerPanel {
            layout: Stack::default(),
            grid: GrowingGrid::new(tray_height(ASSUMED_WIDTH), STICKER_COLUMNS),
            touch: None,
            pack: None,
            stickers: Vec::new(),
            waiting_for_photo: false,
            side: None,
            theme: theme.clone(),
        }
    }

    /// Sizes the stickers to fit `width`, rebuilding the ones built if
    /// that changes their size.
    fn fit(&mut self, width: f32) {
        let side = cell_side(width, STICKER_COLUMNS);
        if self.side.is_some_and(|s| (s - side).abs() < 0.5) { return; }
        self.side = Some(side);
        self.grid.set_cells(side);
        self.grid.set_height(Size::Static(tray_height(width)));
        let built = self.grid.built();
        self.touch = None;
        self.grid.empty();
        self.build(built);
    }

    /// Picks up changes to the pack (new stickers go first), rebuilding the
    /// rows built so far so the page stays where it was.
    fn sync(&mut self, ctx: &mut Context) {
        let pack = self.pack.get_or_insert_with(|| StickerPack::mine(ctx));
        if pack.pending().stickers == self.stickers { return; }
        self.stickers = pack.pending().stickers.clone();
        let built = self.grid.built();
        self.touch = None;
        self.grid.empty();
        self.build(built);
    }

    /// Adds rows while you're near the bottom of the ones built.
    fn grow(&mut self, sized: &SizedTree) {
        if self.grid.near_end(sized) { self.build(self.grid.built() + STICKER_COLUMNS * LOAD_ROWS); }
    }

    /// Builds cells up to `count` (or the end). The first is the new-sticker tile.
    fn build(&mut self, count: usize) {
        let Some(side) = self.side else { return };
        let (built, end) = (self.grid.built(), count.min(self.stickers.len() + 1));
        if built >= end { return; }
        let cells = (built..end).map(|i| match i {
            0 => PanelCell::new_sticker(&self.theme, side),
            n => PanelCell::sticker(&self.stickers[n - 1], side),
        }).collect();
        self.grid.append(cells);
    }

    fn label(&self) -> (String, String) {
        (STICKER_TITLE.to_string(), String::new())
    }
}

impl OnEvent for StickerPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(width) = laid_out_width(sized) { self.fit(width); }
            self.sync(ctx);
            if let Some((_, grid)) = sized.1.first() { self.grow(grid); }
        } else if let Some(PickedPhoto(photo)) = event.downcast_ref::<PickedPhoto>() && self.waiting_for_photo {
            // The photo picked with the new-sticker tile.
            self.waiting_for_photo = false;
            if let Some(sticker) = Sticker::from_photo(photo) && let Some(pack) = &mut self.pack {
                pack.send(StickerPackAction::Add(sticker));
                self.sync(ctx);
            }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match track_touch(&mut self.touch, self.grid.grid(), GridPlace::find(sized, &[0, 0, 0]), mouse) {
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

/// The GIF page: trending GIFs from GIPHY, or the results of a search, in a
/// column that scrolls up and down, under a bar with a search button and
/// GIPHY's attribution. Tap a GIF to send it (with a [`SendGifEvent`] for the
/// chat). The search button opens the GIF search, which fills the screen
/// (see [`GifSearch`]); the page shows its results after.
///
/// GIFs are fetched from GIPHY in batches as you scroll near the bottom, so
/// the column seems to go on for ever. Only GIFs near the part showing play
/// and keep their frames in memory.
#[derive(Component, Debug, Clone)]
struct GifPanel {
    layout: Column,
    bar: GifBar,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    /// The search shown, or `None` for trending GIFs.
    #[skip] query: Option<String>,
    #[skip] gifs: Vec<Gif>,
    /// Whether GIPHY has more GIFs after these.
    #[skip] more: bool,
    #[skip] fetch: Option<Task<Result<GifBatch, GifError>>>,
    #[skip] error: Option<GifError>,
    /// Whether it's in the GIF search, filling the screen.
    #[skip] full: bool,
    /// How wide it is, and the side of a GIF: none are built until it's known.
    #[skip] width: f32,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl GifPanel {
    fn new(theme: &Theme) -> Self {
        GifPanel {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            bar: GifBar::new(theme, false),
            grid: GrowingGrid::new(tray_height(ASSUMED_WIDTH) - GIF_BAR_HEIGHT, GIF_COLUMNS),
            touch: None,
            query: None,
            gifs: Vec::new(),
            more: true,
            fetch: None,
            error: None,
            full: false,
            width: ASSUMED_WIDTH,
            side: None,
            theme: theme.clone(),
        }
    }

    /// Sizes the GIFs to fit `width`, rebuilding the ones built if that
    /// changes their size.
    fn fit(&mut self, width: f32) {
        let side = cell_side(width, GIF_COLUMNS);
        if self.side.is_some_and(|s| (s - side).abs() < 0.5) { return; }
        self.width = width;
        self.side = Some(side);
        self.grid.set_cells(side);
        self.grid.set_height(self.grid_height());
        let built = self.grid.built();
        self.touch = None;
        self.grid.empty();
        self.grid.append(self.gifs[..built].iter().map(|gif| PanelCell::gif(gif, side)).collect());
    }

    /// The page's height less the bar; in the GIF search, all the room there is.
    fn grid_height(&self) -> Size {
        let height = tray_height(self.width) - GIF_BAR_HEIGHT;
        match self.full {
            true => Size::custom(move |_: Vec<(f32, f32)>| (height, f32::MAX)),
            false => Size::Static(height),
        }
    }

    /// In the GIF search (`true`), the GIFs take all the room there is, and
    /// the bar says what they are in place of the search button.
    fn set_full(&mut self, full: bool) {
        if full == self.full { return; }
        self.full = full;
        self.bar = GifBar::new(&self.theme, full);
        self.grid.set_height(self.grid_height());
    }

    /// Shows trending GIFs (`None`) or the results of a search, from the top.
    fn search(&mut self, query: Option<String>) {
        self.query = query;
        self.gifs.clear();
        self.touch = None;
        self.more = true;
        self.error = None;
        self.grid.clear();
        self.fetch = Some(find_gifs(self.query.clone(), 0));
    }

    /// After an error, tries again (when the page is opened again).
    fn retry(&mut self) {
        if self.error.is_some() { self.error = None; }
    }

    /// Adds GIPHY's next batch to the list.
    fn received(&mut self, result: Result<GifBatch, GifError>) {
        self.fetch = None;
        match result {
            Ok(batch) => {
                // GIPHY sometimes repeats a GIF across batches.
                for gif in batch.gifs {
                    if !self.gifs.iter().any(|g| g.id == gif.id) { self.gifs.push(gif); }
                }
                self.more = batch.more;
            },
            Err(error) => self.error = Some(error),
        }
    }

    /// Near the bottom: adds rows for GIFs already fetched, or fetches more
    /// (this is also how the first batch is fetched).
    fn grow(&mut self, sized: &SizedTree) {
        let Some(side) = self.side else { return };
        if !self.grid.near_end(sized) { return; }
        let built = self.grid.built();
        if built < self.gifs.len() {
            let end = (built + GIF_COLUMNS * LOAD_ROWS).min(self.gifs.len());
            self.grid.append(self.gifs[built..end].iter().map(|gif| PanelCell::gif(gif, side)).collect());
        } else if self.more && self.fetch.is_none() && self.error.is_none() {
            self.fetch = Some(find_gifs(self.query.clone(), self.gifs.len()));
        }
    }

    /// Only GIFs in or near the part showing play; the rest stop and let go
    /// of their frames.
    fn update_live(&mut self, sized: &SizedTree) {
        let showing = self.grid.showing_rows(sized);
        let live = showing.start.saturating_sub(LIVE_MARGIN_ROWS)..showing.end + LIVE_MARGIN_ROWS;
        self.grid.grid().set_live(live);
    }

    fn label(&self) -> (String, String) {
        let title = match (self.error, self.gifs.is_empty()) {
            (Some(GifError::NoApiKey), _) => "No GIPHY API key".to_string(),
            (Some(GifError::Unavailable), true) => "Can't reach GIPHY".to_string(),
            (None, true) if self.fetch.is_some() => "Loading GIFs\u{2026}".to_string(),
            (None, true) if !self.more => "No GIFs found".to_string(),
            _ => match &self.query {
                Some(query) => format!("\u{201c}{}\u{201d}", shorten(query, 14, false)),
                None => "Trending GIFs".to_string(),
            },
        };
        (title, String::new())
    }
}

impl OnEvent for GifPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(result) = self.fetch.as_ref().and_then(|fetch| fetch.take()) { self.received(result); }
            if let Some(width) = laid_out_width(sized) { self.fit(width); }
            if self.full { self.bar.set_status(&self.label().0); }
            if let Some((_, grid)) = sized.1.get(1) {
                self.grow(grid);
                self.update_live(grid);
            }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            if let Some(TouchEnd::Tap(CellAction::Gif(gif))) = track_touch(&mut self.touch, self.grid.grid(), GridPlace::find(sized, &[1, 0, 0]), mouse) {
                ctx.trigger_haptic();
                ctx.emit(SendGifEvent(gif));
            }
        }
        vec![event]
    }
}

/// Above the GIFs: the search button (or, in the GIF search, what the GIFs
/// are) and GIPHY's attribution.
#[derive(Component, Debug, Clone)]
struct GifBar(Row, Option<GhostIconButton>, Option<Text>, Bin<Stack, Text>);
impl OnEvent for GifBar {}

impl GifBar {
    fn new(theme: &Theme, searching: bool) -> Self {
        let search = (!searching).then(|| {
            let mut button = GhostIconButton::new(theme, Icons::Search, |ctx: &mut Context, _: &Theme| ctx.emit(MobileKeyboardEvent::StartGifSearch));
            button.1.set_trigger_on_release();
            button
        });
        let status = searching.then(|| Text::new(theme, "", TextSize::Md, TextStyle::Keyboard, Align::Left, None));
        let attribution = Text::new(theme, GIPHY_ATTRIBUTION, TextSize::Sm, TextStyle::Secondary, Align::Left, None);
        GifBar(
            // Lined up with the GIFs below (the grid has 12px padding).
            Row::new(12.0, Offset::Center, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)),
            search,
            status,
            // The rest of the row, with the attribution at its end.
            Bin(Stack(Offset::End, Offset::Center, Size::Fill, Size::Static(GIF_BAR_HEIGHT - 12.0), Padding::default()), attribution),
        )
    }

    fn set_status(&mut self, status: &str) {
        if let Some(text) = &mut self.2 && text.spans[0] != status { text.spans[0] = status.to_string(); }
    }
}

/// A grid in a column that scrolls up and down, built a few rows at a time:
/// more are added as you scroll near the bottom, so it seems to go on for
/// ever but only holds what's been loaded so far.
///
/// Fields after the scrolling grid (cut off at its edges): the cells built,
/// the columns, the side of a (square) cell, and, while a finger is down,
/// whether it went down on the grid.
#[derive(Component, Debug, Clone)]
struct GrowingGrid(Stack, Clip<GridScroll>, #[skip] usize, #[skip] usize, #[skip] f32, #[skip] Option<bool>);

impl GrowingGrid {
    /// `height` is how much of it shows at once.
    fn new(height: f32, columns: usize) -> Self {
        GrowingGrid(
            Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(height), Padding::default()),
            Clip(GridScroll::new()),
            0, columns, 0.0, None,
        )
    }

    /// The side of the cells it's given (for the blank ones it adds, and to
    /// know how tall a row is).
    fn set_cells(&mut self, side: f32) { self.4 = side; }

    fn grid(&mut self) -> &mut PanelGrid { &mut self.1.0.1 }

    fn built(&self) -> usize { self.2 }

    /// How much of it shows at once.
    fn set_height(&mut self, height: Size) { self.0.3 = height; }

    /// Empties it and scrolls back to the top.
    fn clear(&mut self) {
        self.1 = Clip(GridScroll::new());
        self.2 = 0;
    }

    /// Empties it, keeping how far it's scrolled for the cells built next.
    fn empty(&mut self) {
        self.1.0.1 = PanelGrid::empty(GRID_SPACING);
        self.2 = 0;
    }

    /// Adds cells after the ones built, filling out the last row first.
    fn append(&mut self, mut cells: Vec<PanelCell>) {
        let columns = self.3;
        let partial = self.2 % columns;
        if partial != 0 && let Some(row) = self.1.0.1.1.pop() {
            let mut kept: Vec<PanelCell> = row.1.into_iter().take(partial).collect();
            kept.append(&mut cells);
            cells = kept;
            self.2 -= partial;
        }
        self.2 += cells.len();
        for chunk in cells.chunks(columns) {
            let mut row = chunk.to_vec();
            while row.len() < columns { row.push(PanelCell::blank(PanelCell::square(self.4))); }
            self.1.0.1.push_row(row, GRID_SPACING);
        }
    }

    /// From its sized tree: how far it's scrolled, how much shows, and how
    /// tall the grid is.
    fn measure(sized: &SizedTree) -> Option<(f32, f32, f32)> {
        let (_, scroll) = sized.1.first()?;
        let (grid_offset, grid) = scroll.1.first()?;
        Some((-grid_offset.1, scroll.0.1, grid.0.1))
    }

    /// Whether the bottom of what's built is within a few rows of showing.
    fn near_end(&self, sized: &SizedTree) -> bool {
        Self::measure(sized).is_none_or(|(scrolled, showing, height)| {
            scrolled + showing + LOAD_ROWS as f32 * (self.4 + GRID_SPACING) >= height
        })
    }

    /// The rows that are (at least partly) showing.
    fn showing_rows(&self, sized: &SizedTree) -> std::ops::Range<usize> {
        let Some((scrolled, showing, _)) = Self::measure(sized) else { return 0..0 };
        let row = self.4 + GRID_SPACING;
        let first = ((scrolled - GRID_PADDING - self.4) / row).floor() + 1.0;
        let last = ((scrolled + showing - GRID_PADDING) / row).ceil();
        first.max(0.0) as usize..last.max(0.0) as usize
    }
}

impl OnEvent for GrowingGrid {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) => self.5 = Some(mouse.position.is_some()),
                MouseState::Released(_) => self.5 = None,
                // A drag scrolls it if it started on it, even once the finger
                // moves off it. A mouse wheel scrolls it while over it.
                MouseState::Scroll(_, dy) if self.5.unwrap_or(mouse.position.is_some()) => self.1.0.0.adjust_scroll(dy),
                _ => {}
            }
        }
        vec![event]
    }
}

/// Draws what's in it cut off at its own edges, top, bottom and sides.
///
/// prism cuts a child off where its parent starts but not where it ends
/// (it gives the child the parent's height from the child's own top), so
/// without this the rows of a scrolled grid show below it.
#[derive(Debug, Clone)]
struct Clip<D: Drawable + Clone>(D);

impl<D: Drawable + Clone + 'static> Drawable for Clip<D> {
    fn request_size(&self) -> RequestTree { self.0.request_size() }

    fn build(&self, size: (f32, f32), request: &RequestTree) -> SizedTree { self.0.build(size, request) }

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let Some(edges) = overlap(bound, (offset.0, offset.1, sized.0.0, sized.0.1)) else { return Vec::new() };
        // Anything entirely outside isn't drawn at all.
        self.0.draw(sized, offset, edges).into_iter().filter_map(|Instruction(mut area, item)| {
            area.bounds = Some(overlap(area.bounds.unwrap_or(edges), edges)?);
            Some(Instruction(area, item))
        }).collect()
    }

    fn name(&self) -> String { self.0.name() }

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) { self.0.event(ctx, sized, event) }
}

/// Where two rectangles (x, y, width, height) overlap, if they do.
fn overlap(a: Rect, b: Rect) -> Option<Rect> {
    let (x0, y0) = (a.0.max(b.0), a.1.max(b.1));
    let (x1, y1) = ((a.0 + a.2).min(b.0 + b.2), (a.1 + a.3).min(b.1 + b.3));
    (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
}

/// The grid, in a column that scrolls (the [`GrowingGrid`] scrolls it).
#[derive(Component, Debug, Clone)]
struct GridScroll(Column, PanelGrid);
impl OnEvent for GridScroll {}

impl GridScroll {
    fn new() -> Self {
        GridScroll(Column::new(0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start)), PanelGrid::empty(GRID_SPACING))
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

/// Where a page's grid is: the area presses count in (the part showing), and
/// the grid's own position (which moves as it scrolls) and sized tree.
struct GridPlace<'a> {
    view: ((f32, f32), (f32, f32)),
    offset: (f32, f32),
    grid: &'a SizedTree,
}

impl<'a> GridPlace<'a> {
    /// `path` leads from the page to its grid, the child to take at each
    /// step: `[0]` when the grid is the page's first child, or on through a
    /// [`GrowingGrid`] and its scrolling column (`[.., 0, 0]`). Presses count
    /// in the first step's area.
    fn find(page: &'a SizedTree, path: &[usize]) -> Option<Self> {
        let (&first, rest) = path.split_first()?;
        let (view_offset, view) = page.1.get(first)?;
        let mut offset = *view_offset;
        let mut grid = view;
        for &i in rest {
            let (o, child) = grid.1.get(i)?;
            offset = (offset.0 + o.0, offset.1 + o.1);
            grid = child;
        }
        Some(GridPlace { view: (*view_offset, view.0), offset, grid })
    }

    fn contains(&self, p: (f32, f32)) -> bool {
        let ((x, y), (w, h)) = self.view;
        p.0 >= x && p.0 < x + w && p.1 >= y && p.1 < y + h
    }

    /// The cell under `p`, in the page's coordinates.
    fn cell_at(&self, p: (f32, f32)) -> Option<(usize, usize)> {
        let contains = |offset: (f32, f32), size: (f32, f32), p: (f32, f32)|
            p.0 >= offset.0 && p.0 < offset.0 + size.0 && p.1 >= offset.1 && p.1 < offset.1 + size.1;
        let p = (p.0 - self.offset.0, p.1 - self.offset.1);
        self.grid.1.iter().enumerate().find_map(|(r, (row_offset, row))| {
            let p = (p.0 - row_offset.0, p.1 - row_offset.1);
            row.1.iter().position(|(cell_offset, cell)| contains(*cell_offset, cell.0, p)).map(|c| (r, c))
        })
    }
}

/// Follows a press on a page's grid, highlighting the cell under it. Taps
/// and swipes are handled here rather than by buttons on each cell: a tap
/// tolerates a little finger movement, and a drag (to scroll, or turn the
/// page) never taps.
fn track_touch(touch: &mut Option<PanelTouch>, grid: &mut PanelGrid, place: Option<GridPlace>, mouse: &MouseEvent) -> Option<TouchEnd> {
    let mut end = None;
    match mouse.state {
        MouseState::Pressed(MouseButton::Left) => {
            *touch = mouse.position.zip(place).filter(|(p, place)| place.contains(*p)).map(|(p, place)| {
                let cell = place.cell_at(p).filter(|c| grid.cell(*c).is_some_and(|cell| cell.3.is_some()));
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

/// Rows of cells: a page of emoji in 4 rows of 8, or the stickers or GIFs
/// built so far, 4 or 3 to a row. Always full rows (blank cells past the
/// end), so the columns line up.
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

    /// `cell(i)` gives the i-th cell on the page; `None` leaves it blank.
    fn new(rows: usize, columns: usize, layout: Stack, spacing: f32, mut cell: impl FnMut(usize) -> Option<PanelCell>) -> Self {
        let mut grid = Self::empty(spacing);
        for r in 0..rows {
            let cells = (0..columns).map(|c| {
                cell(r * columns + c).unwrap_or_else(|| PanelCell::blank(layout.clone()))
            }).collect();
            grid.push_row(cells, spacing);
        }
        grid
    }

    fn empty(spacing: f32) -> Self {
        PanelGrid(Column::new(spacing, Offset::Start, Size::Fill, Padding::new(GRID_PADDING), None), Vec::new())
    }

    fn push_row(&mut self, cells: Vec<PanelCell>, spacing: f32) {
        self.1.push(PanelRow(Row::new(spacing, Offset::Start, Size::Fit, Padding::new(0.0)), cells));
    }

    fn cell(&mut self, (row, col): (usize, usize)) -> Option<&mut PanelCell> {
        self.1.get_mut(row)?.1.get_mut(col)
    }

    fn highlight(&mut self, cell: Option<(usize, usize)>, on: bool) {
        if let Some(cell) = cell.and_then(|c| self.cell(c)) { cell.press(on); }
    }

    /// Lets the GIFs in `rows` play, and the rest stop and drop their frames.
    fn set_live(&mut self, rows: std::ops::Range<usize>) {
        for (r, row) in self.1.iter_mut().enumerate() {
            for cell in &mut row.1 {
                if let Some(CellFace::Gif {player, ..}) = &mut cell.2 { player.set_live(rows.contains(&r)); }
            }
        }
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
    /// Sends the GIF.
    Gif(Gif),
}

/// One cell: a highlight shown behind an emoji while it's pressed, what's in
/// it, and what tapping it does (blank cells have neither).
#[derive(Component, Debug, Clone)]
struct PanelCell(Stack, Opt<Rectangle>, Option<CellFace>, #[skip] Option<CellAction>);
impl OnEvent for PanelCell {}

#[derive(Component, Debug, Clone)]
enum CellFace {
    Emoji {layout: Stack, text: Text},
    /// The outline is drawn over the sticker or GIF while it's pressed.
    Sticker {layout: Stack, image: Image, outline: Opt<Rectangle>},
    NewSticker {layout: Stack, background: Rectangle, icon: Image, outline: Opt<Rectangle>},
    Gif {layout: Stack, player: GifPlayer, outline: Opt<Rectangle>},
}
impl OnEvent for CellFace {}

impl PanelCell {
    /// Every cell in a grid asks for the same width, so the columns get equal space.
    fn layout(width: f32, height: f32) -> Stack {
        let width = Size::custom(move |_: Vec<(f32, f32)>| (width, f32::MAX));
        Stack(Offset::Center, Offset::Center, width, Size::Static(height), Padding(2.0, 2.0, 2.0, 2.0))
    }

    fn emoji_layout() -> Stack { Self::layout(32.0, EMOJI_ROW_HEIGHT) }

    /// A sticker or GIF cell: a square box. The outline fills it exactly, so
    /// it never grows.
    fn square(side: f32) -> Stack {
        Stack(Offset::Center, Offset::Center, Size::Static(side), Size::Static(side), Padding::default())
    }

    /// Shown behind an emoji while it's pressed.
    fn highlight(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::from_hex("ffffff", 110), radius, None), false)
    }

    /// Drawn over a sticker or GIF while it's pressed (inside its edge).
    fn outline(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::TRANSPARENT, radius, Some((STICKER_OUTLINE, Color::WHITE))), false)
    }

    /// Shows the cell as pressed: an outline over a sticker or GIF, or a
    /// highlight behind an emoji.
    fn press(&mut self, on: bool) {
        match &mut self.2 {
            Some(CellFace::Sticker {outline, ..}) |
            Some(CellFace::NewSticker {outline, ..}) |
            Some(CellFace::Gif {outline, ..}) => outline.display(on),
            _ => self.1.display(on),
        }
    }

    fn emoji(theme: &Theme, emoji: &str) -> Self {
        let text = Text::new(theme, emoji, TextSize::H2, TextStyle::Keyboard, Align::Left, None);
        let face = CellFace::Emoji {layout: Stack::default(), text};
        PanelCell(Self::emoji_layout(), Self::highlight(8.0), Some(face), Some(CellAction::Emoji(emoji.to_string())))
    }

    fn sticker(sticker: &Sticker, side: f32) -> Self {
        let shape = ShapeType::RoundedRectangle(0.0, (side, side), 0.0, 12.0);
        let face = CellFace::Sticker {
            layout: Self::square(side),
            image: Image {shape, image: sticker.image(), color: None},
            outline: Self::outline(12.0),
        };
        PanelCell(Self::square(side), Self::highlight(12.0), Some(face), Some(CellAction::Sticker(sticker.clone())))
    }

    fn new_sticker(theme: &Theme, side: f32) -> Self {
        let face = CellFace::NewSticker {
            layout: Self::square(side),
            background: Rectangle::new(Color::from_hex("ffffff", 110), 12.0, None),
            icon: Icon::new(theme, Icons::Add, Some(Color::WHITE), 36.0),
            outline: Self::outline(12.0),
        };
        PanelCell(Self::square(side), Self::highlight(12.0), Some(face), Some(CellAction::NewSticker))
    }

    fn gif(gif: &Gif, side: f32) -> Self {
        let face = CellFace::Gif {
            layout: Self::square(side),
            player: GifPlayer::thumbnail(gif, (side, side)),
            outline: Self::outline(8.0),
        };
        PanelCell(Self::square(side), Self::highlight(8.0), Some(face), Some(CellAction::Gif(gif.clone())))
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
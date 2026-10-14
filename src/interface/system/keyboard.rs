//! The on-screen keyboard: letter keys, the header with the page buttons,
//! and the emoji, sticker and GIF pages it shows in place of the keys. On a
//! computer, those pages under the message bar's toolbar instead.

use prism::event::{self, MouseEvent, TickEvent, KeyboardState, KeyboardEvent, OnEvent, Event, Modifiers, MouseState, MouseButton};
use prism::canvas::{Align, Image};
use prism::{emitters, Context};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Layout, Stack, Column, Row, Offset, Size, Padding, Area};
use prism::display::{Bin, Enum};
use std::time::{Duration, Instant};

use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use crate::theme::{Theme, Color, Icons};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon, SendGifEvent};

use crate::components::button::GhostIconButton;

use super::emoji::EmojiPanel;
use super::sticker::StickerPanel;
use super::gif::{GifPanel, GifSearch, GifSearchHeader};

const REPEAT_DELAY: Duration = Duration::from_millis(500);
const REPEAT_INTERVAL: Duration = Duration::from_millis(80);
const FAST_AFTER: Duration = Duration::from_millis(2000);
const FAST_INTERVAL: Duration = Duration::from_millis(35);

#[derive(Debug, Clone, Default)]
struct HoldRepeat { started: Option<Instant>, last: Option<Instant>, target: KeyTarget }

/// Where the keys type: into the focused text field, into the GIF search
/// field, where return runs the search instead of starting a new line, or
/// (the emoji page on a computer) straight into the field of the message bar
/// with this number, whether it has the focus or not (see [`BarKey`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum KeyTarget { #[default] Input, GifSearch, Bar(u64) }

impl KeyTarget {
    pub(super) fn press(self, ctx: &mut Context, key: event::Key) {
        match (self, key) {
            (KeyTarget::GifSearch, event::Key::Enter) => ctx.emit(MobileKeyboardEvent::SubmitGifSearch),
            (KeyTarget::Bar(bar), key) => ctx.emit(BarKey(bar, key)),
            (_, key) => ctx.emit(KeyboardEvent{key, state: KeyboardState::Pressed, modifiers: Modifiers::default()}),
        }
    }
}

/// A key on the emoji page under the toolbar of the message bar with this
/// number (on a computer). The bar puts it straight into its field.
#[derive(Debug, Clone)]
pub(crate) struct BarKey(pub u64, pub event::Key);

impl Event for BarKey {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Room around the keyboard's contents.
const KEYBOARD_PADDING: f32 = 8.0;
/// Extra room above the letter keys when the keyboard has no header (the
/// message bar's toolbar is above it instead): twice as much in all.
const KEYS_TOP_SPACE: f32 = 14.0;

/// The group prism puts every text field in (`emitters::TEXT_INPUT_UUID`),
/// to tell a text field getting the focus from other selections.
pub(crate) const TEXT_FIELD_GROUP: &str = "123e4567-e89b-12d3-a456-426614174000";

/// The pages a text field's keyboard offers besides the letter keys. A
/// text field says which when it gets the focus (see
/// [`TextInput::keyboard_pages`](crate::components::TextInput::keyboard_pages)): every field has emoji, but only the chat's
/// message field has stickers and GIFs, which it sends as messages.
///
/// The keyboard has a row of buttons for the pages above its keys, unless
/// `toolbar` is set: then the field has its own buttons for them (the
/// message bar's toolbar, see [`MessageBar`](crate::interface::message_bar::MessageBar)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardPages { pub emoji: bool, pub stickers: bool, pub gifs: bool, pub toolbar: bool }

impl KeyboardPages {
    /// Emoji, stickers and GIFs, from buttons on the keyboard.
    pub const ALL: Self = KeyboardPages { emoji: true, stickers: true, gifs: true, toolbar: false };
    /// Emoji, stickers and GIFs, from the message bar's toolbar: for a chat's
    /// message field.
    pub const TOOLBAR: Self = KeyboardPages { emoji: true, stickers: true, gifs: true, toolbar: true };
    /// Just emoji: for searching for a username, say. Text fields have these
    /// unless they say otherwise.
    pub const EMOJI: Self = KeyboardPages { emoji: true, stickers: false, gifs: false, toolbar: false };
    /// Only the letter keys.
    pub const NONE: Self = KeyboardPages { emoji: false, stickers: false, gifs: false, toolbar: false };

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

/// What the keyboard is showing, or `None` while it's hidden. The interface
/// sends this whenever it changes, for the message bar's toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeyboardShowing(pub Option<KeyboardPage>);

impl Event for KeyboardShowing {
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
/// GIF search, the pages the focused text field offers, and what it last
/// said it was showing (see [`KeyboardShowing`]).
///
/// The key layout is rebuilt whenever caps or the page changes, so the caps-lock
/// and page keys always match the real state. The emoji, sticker and GIF pages
/// are built once and keep their place while the letter keys are showing.
#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, Enum<KeyboardContent>, #[skip] bool, #[skip] usize, #[skip] KeyboardPage, #[skip] Theme, #[skip] GifSearch, #[skip] KeyboardPages, #[skip] Option<Option<KeyboardPage>>);

/// The keyboard's pages: letter keys, emoji, stickers, or GIFs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyboardPage { Keys, Emoji, Stickers, Gifs }

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
                MobileKeyboardEvent::Toggle(page) => self.toggle(ctx, *page),
                // From the message bar's toolbar: its field has its pages there
                // (even if the field's own word on that hasn't arrived yet).
                MobileKeyboardEvent::Show(page) if !self.7.open => {
                    self.8 = KeyboardPages::TOOLBAR;
                    match page {
                        KeyboardPage::Keys => self.show_keys(ctx),
                        page => self.open(*page),
                    }
                },
                MobileKeyboardEvent::Show(_) => {},
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
            e, false, 0, KeyboardPage::Keys, theme.clone(), GifSearch::default(), KeyboardPages::default(), None,
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
        let header = KeyboardHeader::for_page(&self.6, page, self.8);
        let content = self.2.drawable().inner();
        content.set_header(header);
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
        let query = content.gif_panel().and_then(|panel| panel.query()).unwrap_or_default();
        content.set_header(Some(KeyboardHeader::search(&theme, &query)));
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
        let query = content.1.as_ref().and_then(|header| header.search_query()).unwrap_or_default().trim().to_string();
        if let Some(panel) = content.gif_panel() { panel.search((!query.is_empty()).then_some(query)); }
    }

    /// Back to the GIF page at its usual size. The text field the keyboard
    /// was typing into gets the focus back, unless the keyboard is going away.
    fn close_gif_search(&mut self, ctx: &mut Context, give_back_focus: bool) {
        if !std::mem::take(&mut self.7.open) { return; }
        self.set_search_keys(false);
        let theme = self.6.clone();
        let content = self.2.drawable().inner();
        content.set_header(KeyboardHeader::for_page(&theme, KeyboardPage::Gifs, self.8));
        if let Some(panel) = content.gif_panel() { panel.set_full(false); }
        ctx.emit(KeyboardFullScreen(false));
        if let Some((id, group)) = self.7.return_to.take() && give_back_focus {
            ctx.emit(event::Selectable::Pressed(id, group));
        }
    }

    /// Whether the GIF search is open (the keyboard filling the screen).
    pub fn searching_gifs(&self) -> bool { self.7.open }

    /// The interface calls this every frame with whether the keyboard is
    /// showing. It says when what it's showing changes (see [`KeyboardShowing`]).
    pub fn announce(&mut self, ctx: &mut Context, showing: bool) {
        let now = showing.then_some(self.5);
        if self.9 != Some(now) {
            self.9 = Some(now);
            ctx.emit(KeyboardShowing(now));
        }
    }

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

/// Header (unless the text field has a toolbar for the pages), then the
/// emoji, sticker or GIF page and/or the 4 rows of letter keys (searching
/// GIFs, the keys go under the GIFs), then the safe area below.
#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, Option<KeyboardHeader>, Option<EmojiPanel>, Option<StickerPanel>, Option<GifPanel>, Option<KeyRows>, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardContent {}

impl KeyboardContent {
    /// The letter keys.
    pub fn new(ctx: &mut Context, theme: &Theme, page: usize, caps: bool, pages: KeyboardPages) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::for_page(theme, KeyboardPage::Keys, pages));
        content.5 = Some(KeyRows::new(theme, page, caps, KeyTarget::Input));
        let header = content.1.take();
        content.set_header(header);
        content
    }

    pub fn emoji(ctx: &mut Context, theme: &Theme) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::for_page(theme, KeyboardPage::Emoji, KeyboardPages::ALL));
        content.2 = Some(EmojiPanel::new(theme, KeyTarget::Input));
        content
    }

    pub fn stickers(ctx: &mut Context, theme: &Theme) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::for_page(theme, KeyboardPage::Stickers, KeyboardPages::ALL));
        content.3 = Some(StickerPanel::new(theme));
        content
    }

    pub fn gifs(ctx: &mut Context, theme: &Theme) -> Self {
        let mut content = Self::with_header(ctx, theme, KeyboardHeader::for_page(theme, KeyboardPage::Gifs, KeyboardPages::ALL));
        content.4 = Some(GifPanel::new(theme));
        content
    }

    /// Just the header (and the safe area below). The emoji, sticker and GIF
    /// pages get their header's buttons when they're opened.
    fn with_header(ctx: &mut Context, theme: &Theme, header: Option<KeyboardHeader>) -> Self {
        let (b, _, _, _) = ctx.get_safe_area();
        Self::with_bottom(theme, header, b)
    }

    /// An emoji, sticker or GIF page on its own, with no header or keys: on
    /// a computer, under the toolbar of the message bar with this number.
    fn panel(theme: &Theme, page: KeyboardPage, bar: u64) -> Self {
        let mut content = Self::with_bottom(theme, None, 0.0);
        match page {
            KeyboardPage::Emoji => content.2 = Some(EmojiPanel::new(theme, KeyTarget::Bar(bar))),
            KeyboardPage::Stickers => content.3 = Some(StickerPanel::new(theme)),
            KeyboardPage::Gifs => content.4 = Some(GifPanel::new(theme)),
            KeyboardPage::Keys => {},
        }
        content
    }

    /// The header, and `bottom` of space below for the safe area.
    fn with_bottom(theme: &Theme, header: Option<KeyboardHeader>, bottom: f32) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        let layout = Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(bottom), Padding::default());

        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding::new(KEYBOARD_PADDING), None),
            header,
            None,
            None,
            None,
            None,
            Bin(layout, Rectangle::new(color, 0.0, None)),
        )
    }

    fn gif_panel(&mut self) -> Option<&mut GifPanel> { self.4.as_mut() }

    /// Puts in a header (or none). The letter keys get more room above
    /// them without one (under the message bar's toolbar).
    fn set_header(&mut self, header: Option<KeyboardHeader>) {
        self.1 = header;
        let extra = match self.1.is_none() && self.5.is_some() { true => KEYS_TOP_SPACE, false => 0.0 };
        self.0.padding().1 = KEYBOARD_PADDING + extra;
    }
}

/// The emoji, sticker and GIF pages on a computer, which has no on-screen
/// keyboard to show them in: the message bar's toolbar opens one under
/// itself. Searching GIFs, the search field goes above the GIFs, and return
/// runs the search.
///
/// Fields after the pages: the one showing, whether the GIF search is open,
/// and the theme. Emoji go straight into the field of the message bar it
/// was made for.
#[derive(Component, Debug, Clone)]
pub(crate) struct DesktopPanels(Stack, Rectangle, Enum<KeyboardContent>, #[skip] KeyboardPage, #[skip] bool, #[skip] Theme);

impl DesktopPanels {
    pub(crate) fn new(theme: &Theme, bar: u64) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| heights[1]);
        let pages = Enum::new(vec![
            ("emoji".to_string(), KeyboardContent::panel(theme, KeyboardPage::Emoji, bar)),
            ("stickers".to_string(), KeyboardContent::panel(theme, KeyboardPage::Stickers, bar)),
            ("gifs".to_string(), KeyboardContent::panel(theme, KeyboardPage::Gifs, bar)),
        ], "emoji".to_string());
        DesktopPanels(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()),
            Rectangle::new(theme.colors().get(ptsd::Background::Secondary), 12.0, None),
            pages, KeyboardPage::Emoji, false, theme.clone(),
        )
    }

    /// The page showing (emoji, stickers or GIFs).
    pub(crate) fn page(&self) -> KeyboardPage { self.3 }

    /// Shows the emoji, sticker or GIF page. Leaving the GIFs ends a search.
    pub(crate) fn show(&mut self, page: KeyboardPage) {
        let name = match page {
            KeyboardPage::Emoji => "emoji",
            KeyboardPage::Stickers => "stickers",
            KeyboardPage::Gifs => "gifs",
            KeyboardPage::Keys => return,
        };
        if page != self.3 { self.close_search(); }
        self.3 = page;
        self.2.display(name);
        if let Some(panel) = self.2.drawable().inner().gif_panel() { panel.retry(); }
    }

    /// Whether the GIF search is open.
    pub(crate) fn searching(&self) -> bool { self.4 }

    /// Puts the search field above the GIFs (it takes the focus once it's
    /// showing).
    fn open_search(&mut self) {
        if self.4 || self.3 != KeyboardPage::Gifs { return; }
        self.4 = true;
        let theme = self.5.clone();
        let content = self.2.drawable().inner();
        let query = content.gif_panel().and_then(|panel| panel.query()).unwrap_or_default();
        content.set_header(Some(KeyboardHeader::search(&theme, &query)));
        if let Some(panel) = content.gif_panel() { panel.set_search_button(false); }
    }

    /// Searches for what's in the search field (trending GIFs if it's empty).
    fn submit_search(&mut self) {
        if !self.4 { return; }
        let content = self.2.drawable().inner();
        let query = content.1.as_ref().and_then(|header| header.search_query()).unwrap_or_default().trim().to_string();
        if let Some(panel) = content.gif_panel() { panel.search((!query.is_empty()).then_some(query)); }
    }

    /// Takes the search field away. The GIFs found stay.
    pub(crate) fn close_search(&mut self) {
        if !std::mem::take(&mut self.4) { return; }
        if self.3 != KeyboardPage::Gifs { return; }
        let content = self.2.drawable().inner();
        content.set_header(None);
        if let Some(panel) = content.gif_panel() { panel.set_search_button(true); }
    }

    /// Whether the search field has the focus.
    fn search_focused(&mut self) -> bool {
        self.4 && self.2.drawable().inner().1.as_ref().and_then(|header| header.2.as_ref()).is_some_and(|search| search.has_focus())
    }
}

impl OnEvent for DesktopPanels {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::StartGifSearch => self.open_search(),
                MobileKeyboardEvent::CancelGifSearch => self.close_search(),
                MobileKeyboardEvent::SubmitGifSearch => self.submit_search(),
                _ => {},
            }
        } else if let Some(KeyboardEvent{key: event::Key::Enter, state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.search_focused()
        {
            // Return in the search field searches (rather than going in it).
            self.submit_search();
            return vec![];
        } else if event.downcast_ref::<SendGifEvent>().is_some() {
            // Sending a GIF ends the search.
            self.close_search();
        }
        vec![event]
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
    /// The page buttons the field offers, the label, and the button that
    /// hides the keyboard. None if the field has a toolbar for all that.
    fn for_page(theme: &Theme, page: KeyboardPage, pages: KeyboardPages) -> Option<Self> {
        (!pages.toolbar).then(|| KeyboardHeader(Column::start(0.0), Some(KeyboardIcons::new(theme, page, pages)), None, Self::divider(theme)))
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
        self.2.as_ref().map(|search| search.query())
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
            button(Icons::Emoji, KeyboardPage::Emoji, MobileKeyboardEvent::Toggle(KeyboardPage::Emoji)),
            button(Icons::Sticker, KeyboardPage::Stickers, MobileKeyboardEvent::Toggle(KeyboardPage::Stickers)),
            button(Icons::Gif, KeyboardPage::Gifs, MobileKeyboardEvent::Toggle(KeyboardPage::Gifs)),
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
pub(super) struct Key(Stack, interactions::Button, #[skip] Option<HoldRepeat>);

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
        let label = match target { KeyTarget::GifSearch => "search", _ => "return" };
        let default = _Key::newline(theme, caps_on, ButtonState::Default, label);
        let pressed = _Key::newline(theme, caps_on, ButtonState::Pressed, label);
        let callback = Box::new(move |ctx: &mut Context| target.press(ctx, event::Key::Enter));
        Key(Stack::default(), interactions::Button::new(default, None::<_Key>, Some(pressed), None::<_Key>, None::<_Key>, callback, false), None)
    }

    fn backspace(theme: &Theme, target: KeyTarget) -> Self { Self::backspace_sized(theme, 48.0, target) }

    /// Backspace `height` tall (the emoji page's is shorter, to fit its tab row).
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
pub(crate) enum MobileKeyboardEvent {
    Capslock(bool),
    Paginator(usize),
    /// Opens the page, or goes back to the letter keys if it's open (the
    /// keyboard's own page buttons).
    Toggle(KeyboardPage),
    /// Shows the page (the message bar's toolbar, whose field has its pages
    /// there rather than on the keyboard).
    Show(KeyboardPage),
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

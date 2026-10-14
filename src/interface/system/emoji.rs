//! The emoji page.

use prism::event::{self, MouseEvent, OnEvent, Event, MouseState, MouseButton};
use prism::canvas::Align;
use prism::{Context, IS_MOBILE};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Stack, Column, Row, Offset, Size, Padding};

use ptsd::interactions;

use crate::theme::{Theme, Color};
use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::Rectangle;

use super::keyboard::{Key, KeyTarget, MobileKeyboardEvent};
use super::{PanelGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace, track_touch};

/// The emoji grid is 4 rows of 48px plus the 48px tab row: the same 240px as
/// the 4 rows of letter keys, so the keyboard doesn't change height.
const EMOJI_COLUMNS: usize = 8;
const EMOJI_ROWS: usize = 4;
const EMOJI_ROW_HEIGHT: f32 = 48.0;
const EMOJIS_PER_PAGE: usize = EMOJI_COLUMNS * EMOJI_ROWS;

struct EmojiCategory { name: String, icon: &'static str, emojis: Vec<String> }

lazy_static::lazy_static! {
    static ref EMOJI_CATEGORIES: Vec<EmojiCategory> = parse_emoji_categories(include_str!("../../../emoji.txt"));

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
/// The emoji (and backspace) type where `target` says.
#[derive(Component, Debug, Clone)]
pub(super) struct EmojiPanel {
    layout: Column,
    grid: PanelGrid,
    tabs: EmojiTabs,
    #[skip] page: usize,
    #[skip] touch: Option<PanelTouch>,
    #[skip] target: KeyTarget,
    #[skip] theme: Theme,
}

impl EmojiPanel {
    pub(super) fn new(theme: &Theme, target: KeyTarget) -> Self {
        EmojiPanel {
            layout: Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None),
            grid: PanelGrid::emoji(theme, 0),
            tabs: EmojiTabs::new(theme, 0, target),
            page: 0,
            touch: None,
            target,
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
        if self.category() != category { self.tabs = EmojiTabs::new(&self.theme, self.category(), self.target); }
    }

    /// The category's name, and this page's number out of the category's pages.
    pub(super) fn label(&self) -> (String, String) {
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
                // On a phone an emoji types as the finger lifts (a drag
                // turns the page instead).
                Some(TouchEnd::Tap(CellAction::Emoji(emoji))) if IS_MOBILE => {
                    ctx.trigger_haptic();
                    self.target.press(ctx, event::Key::Character(emoji));
                }
                _ => {}
            }
            // With a mouse it types as the button goes down, like a key. (A
            // trackpad tap can bring the press and release in the same frame,
            // and the release can be handled first.)
            if !IS_MOBILE && let MouseState::Pressed(MouseButton::Left) = mouse.state
                && let Some(CellAction::Emoji(emoji)) = self.touch.and_then(|t| t.cell).and_then(|c| self.grid.cell(c)).and_then(|cell| cell.3.clone())
            {
                self.target.press(ctx, event::Key::Character(emoji));
            }
        }
        vec![event]
    }
}

impl PanelGrid {
    fn emoji(theme: &Theme, page: usize) -> Self {
        let (c, start) = EMOJI_PAGES[page];
        let emojis = &EMOJI_CATEGORIES[c].emojis;
        Self::new(EMOJI_ROWS, EMOJI_COLUMNS, PanelCell::emoji_layout(), 6.0, |i| {
            emojis.get(start + i).map(|emoji| PanelCell::emoji(theme, emoji))
        })
    }
}

impl PanelCell {
    fn emoji_layout() -> Stack { Self::layout(32.0, EMOJI_ROW_HEIGHT) }

    fn emoji(theme: &Theme, emoji: &str) -> Self {
        let text = Text::new(theme, emoji, TextSize::H2, TextStyle::Keyboard, Align::Left, None);
        let face = CellFace::Emoji {layout: Stack::default(), text};
        PanelCell(Self::emoji_layout(), Self::highlight(8.0), Some(face), Some(CellAction::Emoji(emoji.to_string())))
    }
}

/// Category tabs, with the active one highlighted, and backspace.
#[derive(Component, Debug, Clone)]
struct EmojiTabs(Row, Vec<EmojiTab>, Key);
impl OnEvent for EmojiTabs {}

impl EmojiTabs {
    fn new(theme: &Theme, active: usize, target: KeyTarget) -> Self {
        let tabs = EMOJI_CATEGORIES.iter().enumerate()
            .map(|(i, category)| EmojiTab::new(theme, i, category.icon, i == active))
            .collect();
        EmojiTabs(Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), tabs, Key::backspace_compact(theme, target))
    }
}

impl Key {
    /// Backspace for the emoji panel's 48px tab row.
    fn backspace_compact(theme: &Theme, target: KeyTarget) -> Self { Self::backspace_sized(theme, EMOJI_ROW_HEIGHT - 12.0, target) }
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

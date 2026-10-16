use prism::event::{self, MouseEvent, OnEvent, Event, MouseState, MouseButton};
use prism::canvas::Align;
use prism::{Context, IS_MOBILE};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Stack, Column, Row, Offset, Size, Padding};

use std::time::{Duration, Instant};

use ptsd::interactions;

use crate::theme::{Theme, Color};
use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::Rectangle;

use super::keyboard::{Key, KeyTarget, MobileKeyboardEvent};
use super::{PanelGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace, track_touch, SWIPE_DISTANCE};

struct EmojiCategory { icon: &'static str, emojis: Vec<String> }

#[derive(Component, Debug, Clone)]
pub struct EmojiPanel {
    layout: Column,
    grid: PanelGrid,
    tabs: EmojiTabs,
    #[skip] page: usize,
    #[skip] touch: Option<PanelTouch>,
    #[skip] scroll: Option<ScrollGesture>,
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
            scroll: None,
            target,
            theme: theme.clone(),
        }
    }
}

impl OnEvent for EmojiPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        let mut page = None;
        let mut turn = None;

        if let Some(MobileKeyboardEvent::EmojiCategory(category)) = event.downcast_ref::<MobileKeyboardEvent>() {
            page = EMOJI_PAGES.iter().position(|(c, _)| c == category);
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match track_touch(&mut self.touch, &mut self.grid, GridPlace::find(sized, &[0]), mouse) {
                Some(TouchEnd::Turn(next)) => turn = Some(next),
                Some(TouchEnd::Tap(CellAction::Emoji(emoji))) if IS_MOBILE => {
                    ctx.trigger_haptic();
                    self.target.press(ctx, event::Key::Character(emoji));
                }
                _ => {}
            }

            if !IS_MOBILE && let MouseState::Scroll(dx, dy) = mouse.state && self.touch.is_none()
                && mouse.position.zip(GridPlace::find(sized, &[0])).is_some_and(|(p, grid)| grid.contains(p))
            {
                let now = Instant::now();
                let start = (dx, dy) == (0.0, 0.0);
                let mut gesture = self.scroll.filter(|g| !start && now.duration_since(g.last) < Duration::from_millis(200))
                    .unwrap_or(ScrollGesture { last: now, travel: (0.0, 0.0), needed: if start { SWIPE_DISTANCE } else { 0.0 }, turned: false });
                gesture.last = now;
                gesture.travel = (gesture.travel.0 + dx, gesture.travel.1 + dy);
                let (x, y) = gesture.travel;
                if !gesture.turned && x.abs().max(y.abs()) > gesture.needed {
                    gesture.turned = true;
                    turn = Some(if x.abs() >= y.abs() { x > 0.0 } else { y > 0.0 });
                }
                self.scroll = Some(gesture);
            }

            if !IS_MOBILE && let MouseState::Pressed(MouseButton::Left) = mouse.state
                && let Some(CellAction::Emoji(emoji)) = self.touch.and_then(|t| t.cell).and_then(|c| self.grid.cell(c)).and_then(|cell| cell.3.clone())
            {
                self.target.press(ctx, event::Key::Character(emoji));
            }
        }

        let page = page.or(turn.map(|next| if next { self.page + 1 } else { self.page.saturating_sub(1) }));
        if let Some(page) = page && page != self.page && page < EMOJI_PAGES.len() {
            self.touch = None;
            self.grid = PanelGrid::emoji(&self.theme, page);
            if EMOJI_PAGES[page].0 != EMOJI_PAGES[self.page].0 { self.tabs = EmojiTabs::new(&self.theme, EMOJI_PAGES[page].0, self.target); }
            self.page = page;
        }

        vec![event]
    }
}

#[derive(Debug, Clone, Copy)]
struct ScrollGesture { last: Instant, travel: (f32, f32), needed: f32, turned: bool }

impl PanelGrid {
    fn emoji(theme: &Theme, page: usize) -> Self {
        let (c, start) = EMOJI_PAGES[page];
        Self::new(4, 8, PanelCell::layout(32.0, 48.0), 6.0, |i| EMOJI_CATEGORIES[c].emojis.get(start + i).map(|emoji| {
            let face = CellFace::Emoji {layout: Stack::default(), text: Text::new(theme, emoji, TextSize::H3, TextStyle::Keyboard, Align::Left, None)};
            PanelCell(PanelCell::layout(32.0, 48.0), PanelCell::highlight(8.0), Some(face), Some(CellAction::Emoji(emoji.to_string())))
        }))
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTabs(Row, Vec<EmojiTab>, Key);
impl OnEvent for EmojiTabs {}

impl EmojiTabs {
    fn new(theme: &Theme, active: usize, target: KeyTarget) -> Self {
        let tabs = EMOJI_CATEGORIES.iter().enumerate()
            .map(|(i, category)| EmojiTab::new(theme, i, category.icon, i == active))
            .collect();
        EmojiTabs(Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), tabs, Key::backspace_sized(theme, 36.0, target))
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
        EmojiTabFace(
            Stack(Offset::Center, Offset::Center, Size::custom(|_: Vec<(f32, f32)>| (28.0, f32::MAX)), Size::Static(40.0), Padding(2.0, 4.0, 2.0, 4.0)),
            active.then(|| Rectangle::new(Color::from_hex("ffffff", 110), 8.0, None)),
            Text::new(theme, icon, TextSize::H4, TextStyle::Keyboard, Align::Left, None),
        )
    }
}

lazy_static::lazy_static! {
    static ref EMOJI_CATEGORIES: Vec<EmojiCategory> = {
        let mut groups: Vec<Vec<String>> = Vec::new();
        for line in include_str!("../../../emoji.txt").lines() {
            if line.starts_with("# group:") { groups.push(Vec::new()); continue; }
            if line.starts_with('#') { continue; }

            let Some((codes, status)) = line.split_once(';') else { continue };
            if !status.trim_start().starts_with("fully-qualified") { continue; }

            let emoji: Option<String> = codes.split_whitespace()
                .map(|hex| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32))
                .collect();
            let Some(emoji) = emoji else { continue };
            if emoji.chars().any(|c| ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)) { continue; }

            if let Some(group) = groups.last_mut() { group.push(emoji); }
        }
        
        ["😀", "👋", "🐻", "🍔", "🚗", "⚽", "💡", "🔣", "🏁"].into_iter()
            .zip(groups.into_iter().filter(|group| !group.is_empty()))
            .map(|(icon, emojis)| EmojiCategory { icon, emojis })
            .collect()
    };

    static ref EMOJI_PAGES: Vec<(usize, usize)> = EMOJI_CATEGORIES.iter().enumerate()
        .flat_map(|(c, category)| (0..category.emojis.len()).step_by(32).map(move |start| (c, start)))
        .collect();
}

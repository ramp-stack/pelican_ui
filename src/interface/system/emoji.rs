use prism::drawable::{Drawable, RangeTree, Rect, Instruction, Component, LayoutTree};
use prism::drawable::text::Align;
use prism::events::{DynEvent, MouseEvent, MouseState, MouseButton};
use prism::Context;
use ptsd::interactions;
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding};

use std::time::{Duration, Instant};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::Rectangle;
use crate::theme::{Theme, Color};
use super::key::InputKey;
use super::{PanelGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace};

#[derive(Debug, Clone)]
pub struct EmojiPanel(EmojiPanelContent);

impl EmojiPanel {
    pub fn new(theme: &Theme) -> Self { Self(EmojiPanelContent::new(theme)) }
}

impl Drawable for EmojiPanel {
    fn changed(&mut self) -> prism::events::ChangeFuture<'_> {
        Box::pin(async move {
            Drawable::changed(&mut self.0).await.map::<Self>(|this| &mut this.0)
        })
    }
    fn size_range(&self) -> RangeTree { self.0.size_range() }
    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        Drawable::layout(&mut self.0, size, range)
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.0.draw(layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        for event in self.0.on_event(ctx, layout, event) { self.0.event(ctx, layout, event); }
    }
}

#[derive(Debug, Clone)]
struct EmojiCategoryEvent(usize);
impl prism::events::Event for EmojiCategoryEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

#[derive(Debug, Clone)]
struct EmojiCategory {
    icon: &'static str,
    emojis: Vec<String>,
}

impl EmojiCategory {
    fn load() -> Vec<Self> {
        let mut groups: Vec<Vec<String>> = Vec::new();
        for line in include_str!("../../../emoji.txt").lines() {
            if line.starts_with("# group:") {
                groups.push(Vec::new());
                continue;
            }
            if line.starts_with('#') { continue; }

            let Some((codes, status)) = line.split_once(';') else { continue };
            if !status.trim_start().starts_with("fully-qualified") { continue; }

            let emoji: Option<String> = codes.split_whitespace()
                .map(|hex| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)).collect();
            let Some(emoji) = emoji else { continue };
            if emoji.chars().any(|c| ('\u{1F3FB}'..='\u{1F3FF}').contains(&c)) { continue; }

            if let Some(group) = groups.last_mut() { group.push(emoji); }
        }

        ["😀", "👋", "🐻", "🍔", "🚗", "⚽", "💡", "🔣", "🏁"].into_iter()
            .zip(groups.into_iter().filter(|group| !group.is_empty()))
            .map(|(icon, emojis)| EmojiCategory { icon, emojis }).collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct ScrollGesture {
    last: Instant,
    travel: (f32, f32),
    needed: f32,
    turned: bool,
}

#[derive(Component, Debug, Clone)]
struct EmojiPanelContent {
    layout: Column,
    grid: PanelGrid,
    tabs: EmojiTabs,
    #[skip]
    categories: Vec<EmojiCategory>,
    #[skip]
    pages: Vec<(usize, usize)>,
    #[skip]
    page: usize,
    #[skip]
    touch: Option<PanelTouch>,
    #[skip]
    scroll: Option<ScrollGesture>,
    #[skip]
    theme: Theme,
}

impl EmojiPanelContent {
    fn new(theme: &Theme) -> Self {
        let categories = EmojiCategory::load();
        let pages: Vec<_> = categories.iter().enumerate().flat_map(|(c, category)| {
            (0..category.emojis.len()).step_by(32).map(move |start| (c, start))
        }).collect();
        EmojiPanelContent {
            layout: Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None),
            grid: PanelGrid::emoji(theme, &categories[pages[0].0], pages[0].1),
            tabs: EmojiTabs::new(theme, &categories, 0), categories, pages, page: 0, touch: None,
            scroll: None, theme: theme.clone(),
        }
    }
}

impl EmojiPanelContent {
    fn on_event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let mut page = None;
        let mut turn = None;

        if let Some(EmojiCategoryEvent(category)) = event.downcast_ref::<EmojiCategoryEvent>() {
            page = self.pages.iter().position(|(c, _)| c == category);
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match PanelTouch::track(
                ctx, &mut self.touch, &mut self.grid, GridPlace::find(sized, &[0]), mouse,
            ) {
                Some(TouchEnd::Turn(next)) => turn = Some(next),
                Some(TouchEnd::Tap(CellAction::Emoji(emoji))) if ctx.window.touch_controls => {
                    ctx.hardware.trigger_haptic();
                    InputKey::press(ctx, prism::events::Key::Character(emoji));
                }
                _ => {}
            }

            if !ctx.window.touch_controls
                && let MouseState::Scroll(dx, dy) = mouse.state
                && self.touch.is_none()
                && mouse.position.zip(GridPlace::find(sized, &[0]))
                    .is_some_and(|(p, grid)| grid.contains(p))
            {
                let now = Instant::now();
                let start = (dx, dy) == (0.0, 0.0);
                let mut gesture = self.scroll
                    .filter(|g| !start && now.duration_since(g.last) < Duration::from_millis(200))
                    .unwrap_or(ScrollGesture {
                        last: now, travel: (0.0, 0.0),
                        needed: if start { 40.0 } else { 0.0 }, turned: false,
                    });
                gesture.last = now;
                gesture.travel = (gesture.travel.0 + dx, gesture.travel.1 + dy);
                let (x, y) = gesture.travel;
                if !gesture.turned && x.abs().max(y.abs()) > gesture.needed {
                    gesture.turned = true;
                    turn = Some(if x.abs() >= y.abs() { x > 0.0 } else { y > 0.0 });
                }
                self.scroll = Some(gesture);
            }

            if !ctx.window.touch_controls
                && let MouseState::Pressed(MouseButton::Left) = mouse.state
                && let Some(CellAction::Emoji(emoji)) = self.touch.and_then(|t| t.cell)
                    .and_then(|c| self.grid.cell(c)).and_then(|cell| cell.3.clone())
            {
                InputKey::press(ctx, prism::events::Key::Character(emoji));
            }
        }

        let page = page.or(turn.map(|next| {
            if next { self.page + 1 } else { self.page.saturating_sub(1) }
        }));
        if let Some(page) = page && page != self.page && page < self.pages.len() {
            self.touch = None;
            let (category, start) = self.pages[page];
            self.grid = PanelGrid::emoji(&self.theme, &self.categories[category], start);
            if self.pages[page].0 != self.pages[self.page].0 {
                self.tabs = EmojiTabs::new(&self.theme, &self.categories, category);
            }
            self.page = page;
        }

        vec![event]
    }
}

impl PanelGrid {
    fn emoji(theme: &Theme, category: &EmojiCategory, start: usize) -> Self {
        Self::new(4, 8, PanelCell::layout(32.0, 48.0), 6.0, |i| {
            category.emojis.get(start + i).map(|emoji| {
                let face = CellFace::Emoji {
                    layout: Stack::default(),
                    text: Text::new(
                        theme, emoji, TextSize::H3, TextStyle::Keyboard, Align::Left, None,
                    ),
                };
                PanelCell(
                    PanelCell::layout(32.0, 48.0), PanelCell::highlight(8.0), Some(face),
                    Some(CellAction::Emoji(emoji.to_string())),
                )
            })
        })
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTabs(Row, Vec<EmojiTab>, InputKey);

impl EmojiTabs {
    fn new(theme: &Theme, categories: &[EmojiCategory], active: usize) -> Self {
        let tabs = categories.iter().enumerate()
            .map(|(i, category)| EmojiTab::new(theme, i, category.icon, i == active)).collect();
        EmojiTabs(
            Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), tabs,
            InputKey::backspace(theme, 36.0),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTab(Stack, interactions::Button);

impl EmojiTab {
    fn new(theme: &Theme, index: usize, icon: &str, active: bool) -> Self {
        let face = EmojiTabFace::new(theme, icon, active);
        let callback = move |ctx: &mut Context| ctx.emit(EmojiCategoryEvent(index));
        EmojiTab(
            Stack::default(),
            interactions::Button::new(
                face, None::<EmojiTabFace>, None::<EmojiTabFace>, None::<EmojiTabFace>,
                None::<EmojiTabFace>, callback, false,
            ),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTabFace(Stack, Option<Rectangle>, Text);

impl EmojiTabFace {
    fn new(theme: &Theme, icon: &str, active: bool) -> Self {
        EmojiTabFace(
            Stack(
                Offset::Center, Offset::Center, Size::custom(|_: Vec<(f32, f32)>| (28.0, f32::MAX)),
                Size::Static(40.0), Padding(2.0, 4.0, 2.0, 4.0),
            ), active.then(|| Rectangle::new(Color::from_hex("ffffff", 40), 8.0, None)),
            Text::new(theme, icon, TextSize::H4, TextStyle::Keyboard, Align::Left, None),
        )
    }
}

use prism::drawable::{Area, Component, Drawable, Layout};
use prism::drawable::text::Align;
use futures::future::Either;
use prism::events::{DynEvent, ListenCallback, MouseEvent, MouseState, WindowEvent, OnChange, OnEvent, on_change, on_event};
use prism::Context;
use ptsd::interactions;
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding};
use ptsd::emitters::{DeferredPress, Press};
use ptsd::events::Press as PressEvent;
use std::ops::Range;
use uuid::Uuid;

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::Rectangle;
use crate::theme::{Theme, Color};
use super::key::InputKey;
use super::{Clip, PanelGrid, PanelCell, CellFace, CellAction, TouchEnd};

#[derive(Component, Debug, Clone)]
pub struct EmojiPanel {
    layout: Column,
    grid: Clip<EmojiStrip>,
    tabs: EmojiTabs,
    #[skip]
    categories: Vec<EmojiCategory>,
    #[skip]
    cells: Vec<Option<String>>,
    #[skip]
    starts: Vec<usize>,
    #[skip]
    category: usize,
    #[skip]
    owner: Uuid,
    #[skip]
    theme: Theme,
}

impl EmojiPanel {
    pub fn new(theme: &Theme) -> Self {
        let owner = Uuid::new_v4();
        let categories = EmojiCategory::load();
        let mut cells = Vec::new();
        let mut starts = Vec::new();
        for category in &categories {
            starts.push(cells.len() / 4);
            cells.extend(category.emojis.iter().cloned().map(Some));
            while cells.len() % 4 != 0 {cells.push(None);}
        }
        EmojiPanel {
            layout: Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None),
            grid: Clip(EmojiStrip::new(cells.len() / 4, owner)),
            tabs: EmojiTabs::new(theme, &categories, 0, owner), categories, cells, starts,
            category: 0,
            owner, theme: theme.clone(),
        }
    }
}

impl EmojiPanel {
    fn show_category(&mut self, category: usize) {
        if let Some(column) = self.starts.get(category) {
            self.grid.0.layout.scroll = *column as f32 * EmojiStripLayout::STRIDE;
        }
    }

    fn show(&mut self, view: Range<usize>) {
        // Keep the complete scroll extent, but only shape emoji text near the
        // viewport. Category jumps do not build thousands of offscreen glyphs.
        let live = view.start.saturating_sub(2)..view.end + 2;
        for (c, column) in self.grid.0.columns.iter_mut().enumerate() {
            for (r, cell) in column.1.iter_mut().enumerate() {
                if live.contains(&c) {
                    if cell.3.is_none() && let Some(emoji) = &self.cells[c * 4 + r] {
                        *cell = PanelCell::emoji(&self.theme, emoji, self.owner);
                    }
                } else if cell.3.is_some() {
                    *cell = PanelCell::blank(EmojiStrip::cell_layout(), self.owner);
                }
            }
        }
        let category = self.starts.iter().rposition(|column| *column <= view.start).unwrap_or(0);
        if category != self.category {
            self.category = category;
            self.tabs = EmojiTabs::new(&self.theme, &self.categories, category, self.owner);
        }
    }
}

#[on_change(self.grid.0.next())]
impl OnChange<Either<Range<usize>, ListenCallback>> for EmojiPanel {
    fn on_change(&mut self, ctx: &mut Context, update: Either<Range<usize>, ListenCallback>) {
        match update {
            Either::Left(view) => self.show(view),
            Either::Right(callback) => callback(&mut self.grid.0, ctx),
        }
    }
}

#[on_event]
impl OnEvent<EmojiCategoryEvent> for EmojiPanel {
    fn on(&mut self, _ctx: &mut Context, event: EmojiCategoryEvent) -> Vec<Box<dyn DynEvent>> {
        if event.0 == self.owner {self.show_category(event.1);}
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<TouchEnd> for EmojiPanel {
    fn on(&mut self, ctx: &mut Context, event: TouchEnd) -> Vec<Box<dyn DynEvent>> {
        match &event {
            TouchEnd::Tap(owner, CellAction::Emoji(emoji)) if *owner == self.owner => {
                if ctx.window.is_mobile {ctx.hardware.trigger_haptic();}
                InputKey::press(ctx, prism::events::Key::Character(emoji.clone()));
            }
            _ => {}
        }
        prism::events![event]
    }
}

#[derive(Debug, Clone, Default)]
struct EmojiStripLayout {
    columns: usize,
    scroll: f32,
    width: f32,
    view: Option<Range<usize>>,
    dirty: bool,
}
impl EmojiStripLayout {
    const STRIDE: f32 = 46.0;
    fn limit(&self) -> f32 {
        (24.0 + self.columns as f32 * Self::STRIDE - 6.0 - self.width).max(0.0)
    }
}
impl Layout for EmojiStripLayout {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        self.columns = children.len();
        (0.0, f32::MAX)
    }
    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        self.width = width;
        vec![40.0; self.columns]
    }
    fn request_height(&mut self, _: f32, _: Vec<(f32, f32)>) -> (f32, f32) {(250.0, 250.0)}
    fn layout(&mut self, size: (f32, f32)) -> Vec<Area> {
        self.width = size.0;
        self.scroll = self.scroll.clamp(0.0, self.limit());
        let first = ((self.scroll - 12.0) / Self::STRIDE).floor().max(0.0) as usize;
        let last = ((self.scroll + size.0 - 12.0) / Self::STRIDE).ceil().max(0.0) as usize;
        let view = first.min(self.columns)..last.min(self.columns);
        if self.view.as_ref() != Some(&view) {
            self.view = Some(view);
            self.dirty = true;
        }
        (0..self.columns).map(|c| Area {
            offset: (12.0 + c as f32 * Self::STRIDE - self.scroll, 12.0),
            size: (40.0, 226.0),
        }).collect()
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiColumn(Column, Vec<PanelCell>);

#[derive(Component, Debug, Clone)]
struct EmojiStrip {
    layout: EmojiStripLayout,
    columns: Vec<EmojiColumn>,
    #[skip] press: DeferredPress,
}
impl EmojiStrip {
    fn cell_layout() -> Stack {
        Stack::new(Offset::Center, Offset::Center, Size::Static(40.0), Size::Static(52.0), Padding::default())
    }
    fn new(count: usize, owner: Uuid) -> Self {
        Self {
            layout: EmojiStripLayout {columns: count, ..Default::default()},
            columns: (0..count).map(|_| EmojiColumn(
                Column::new(6.0, Offset::Start, Size::Fit, Padding::default(), None),
                (0..4).map(|_| PanelCell::blank(Self::cell_layout(), owner)).collect(),
            )).collect(),
            press: DeferredPress::default(),
        }
    }
    async fn next(&mut self) -> Either<Range<usize>, ListenCallback> {
        if let Some(view) = &self.layout.view && std::mem::take(&mut self.layout.dirty) {
            return Either::Left(view.clone());
        }
        Either::Right(Drawable::changed(self).await)
    }
}
#[on_event]
impl OnEvent<MouseEvent> for EmojiStrip {
    fn on(&mut self, ctx: &mut Context, mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let events = PanelGrid::pointer(ctx, &mut self.press, mouse);
        for event in &events {
            if let Some(MouseEvent {state: MouseState::Scroll(dx, dy), position}) = event.downcast_ref()
                && !self.press.claimed() && (position.is_some() || self.press.travel().is_some())
            {
                // Touch and trackpad movement follows the horizontal axis.
                // A desktop wheel can also browse the strip with vertical ticks.
                let delta = if !ctx.window.is_mobile && dx.abs() < dy.abs() {*dy} else {*dx};
                self.layout.scroll = (self.layout.scroll + delta).clamp(0.0, self.layout.limit());
            }
        }
        events
    }
}
#[on_event]
impl OnEvent<PressEvent> for EmojiStrip {
    fn on(&mut self, _: &mut Context, event: PressEvent) -> Vec<Box<dyn DynEvent>> {
        self.press.press(event)
    }
}
#[on_event]
impl OnEvent<WindowEvent> for EmojiStrip {
    fn on(&mut self, _: &mut Context, event: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, WindowEvent::Focused(false)) && let Some(cancel) = self.press.cancel() {
            return prism::events![cancel, event];
        }
        prism::events![event]
    }
}
#[on_change(self.press.next())]
impl OnChange<PressEvent> for EmojiStrip {
    fn on_change(&mut self, ctx: &mut Context, event: PressEvent) {ctx.emit(event);}
}

#[derive(Debug, Clone)]
struct EmojiCategoryEvent(Uuid, usize);
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

impl PanelCell {
    fn emoji(theme: &Theme, emoji: &str, owner: Uuid) -> Self {
        let face = CellFace::Emoji {
            layout: Stack::default(),
            text: Text::new(
                theme, emoji, TextSize::H3, TextStyle::Keyboard, Align::Left, None,
            ).fit(),
        };
        PanelCell(
            EmojiStrip::cell_layout(), PanelCell::highlight(8.0), Some(face),
            Some(CellAction::Emoji(emoji.to_string())), Press::default(), owner,
        )
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTabs(Row, Vec<EmojiTab>, InputKey);

impl EmojiTabs {
    fn new(theme: &Theme, categories: &[EmojiCategory], active: usize, owner: Uuid) -> Self {
        let tabs = categories.iter().enumerate()
            .map(|(i, category)| EmojiTab::new(theme, i, category.icon, i == active, owner)).collect();
        EmojiTabs(
            Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), tabs,
            InputKey::backspace(theme, 36.0),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct EmojiTab(Stack, interactions::Button);

impl EmojiTab {
    fn new(theme: &Theme, index: usize, icon: &str, active: bool, owner: Uuid) -> Self {
        let face = EmojiTabFace::new(theme, icon, active);
        let callback = move |ctx: &mut Context| ctx.emit(EmojiCategoryEvent(owner, index));
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
            Stack::new(
                Offset::Center, Offset::Center, Size::custom(|_: Vec<(f32, f32)>| (28.0, f32::MAX)),
                Size::Static(40.0), Padding(2.0, 4.0, 2.0, 4.0),
            ), active.then(|| Rectangle::new(Color::from_hex("ffffff", 40), 8.0, None)),
            Text::new(theme, icon, TextSize::H4, TextStyle::Keyboard, Align::Left, None).fit(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(panel: &mut EmojiPanel, width: f32) {
        for _ in 0..3 {
            panel.request_width();
            panel.request_height(width);
            Drawable::layout(panel, (width, 298.0));
            panel.show(panel.grid.0.layout.view.clone().unwrap());
        }
    }

    #[test]
    fn categories_jump_within_one_strip_and_only_nearby_emoji_are_built() {
        let mut panel = EmojiPanel::new(&Theme::default());
        settle(&mut panel, 375.0);
        assert!(panel.cells.len() > 1000);
        for category in 0..panel.categories.len() {
            panel.show_category(category);
            settle(&mut panel, 375.0);
            let start = panel.starts[category];
            assert!(panel.grid.0.layout.view.as_ref().unwrap().contains(&start));
            assert!(panel.grid.0.columns[start].1[0].3.is_some());
            let built = panel.grid.0.columns.iter().flat_map(|c| &c.1)
                .filter(|cell| cell.3.is_some()).count();
            assert!(built <= 64, "offscreen emoji should not be shaped: {built}");
        }
    }

    #[test]
    fn horizontal_scroll_clamps_at_both_ends_and_after_resize() {
        let mut panel = EmojiPanel::new(&Theme::default());
        panel.grid.0.layout.scroll = -100.0;
        settle(&mut panel, 375.0);
        assert_eq!(panel.grid.0.layout.scroll, 0.0);
        panel.grid.0.layout.scroll = f32::MAX;
        settle(&mut panel, 375.0);
        assert_eq!(panel.grid.0.layout.scroll, panel.grid.0.layout.limit());
        settle(&mut panel, 768.0);
        assert_eq!(panel.grid.0.layout.scroll, panel.grid.0.layout.limit());
    }
}

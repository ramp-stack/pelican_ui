use prism::drawable::Component;
use prism::drawable::text::Align;
use prism::events::DynEvent;
use prism::events::{OnEvent, on_event};
use prism::Context;
use ptsd::interactions;
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding};
use ptsd::emitters::Press;
use uuid::Uuid;

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::Rectangle;
use crate::theme::{Theme, Color};
use super::key::InputKey;
use super::{PanelGrid, PanelCell, CellFace, CellAction, TouchEnd};

#[derive(Component, Debug, Clone)]
pub struct EmojiPanel {
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
    owner: Uuid,
    #[skip]
    theme: Theme,
}

impl EmojiPanel {
    pub fn new(theme: &Theme) -> Self {
        let owner = Uuid::new_v4();
        let categories = EmojiCategory::load();
        let pages: Vec<_> = categories.iter().enumerate().flat_map(|(c, category)| {
            (0..category.emojis.len()).step_by(32).map(move |start| (c, start))
        }).collect();
        EmojiPanel {
            layout: Column::new(0.0, Offset::Center, Size::Fill, Padding::default(), None),
            grid: PanelGrid::emoji(theme, &categories[pages[0].0], pages[0].1, owner),
            tabs: EmojiTabs::new(theme, &categories, 0, owner), categories, pages, page: 0,
            owner, theme: theme.clone(),
        }
    }
}

impl EmojiPanel {
    fn show_page(&mut self, page: usize) {
        if page != self.page && page < self.pages.len() {
            let (category, start) = self.pages[page];
            let scroll = self.grid.4;
            self.grid = PanelGrid::emoji(&self.theme, &self.categories[category], start, self.owner);
            self.grid.4 = scroll;
            if self.pages[page].0 != self.pages[self.page].0 {
                self.tabs = EmojiTabs::new(&self.theme, &self.categories, category, self.owner);
            }
            self.page = page;
        }
    }
}

#[on_event]
impl OnEvent<EmojiCategoryEvent> for EmojiPanel {
    fn on(&mut self, _ctx: &mut Context, event: EmojiCategoryEvent) -> Vec<Box<dyn DynEvent>> {
        if event.0 == self.owner
            && let Some(page) = self.pages.iter().position(|(category, _)| *category == event.1)
        {
            self.show_page(page);
        }
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
            TouchEnd::Turn(owner, next) if *owner == self.owner => {
                self.show_page(if *next {self.page + 1} else {self.page.saturating_sub(1)});
            }
            _ => {}
        }
        prism::events![event]
    }
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

impl PanelGrid {
    fn emoji(theme: &Theme, category: &EmojiCategory, start: usize, owner: Uuid) -> Self {
        Self::new(4, 8, PanelCell::layout(32.0, 48.0), 6.0, owner, |i| {
            category.emojis.get(start + i).map(|emoji| {
                let face = CellFace::Emoji {
                    layout: Stack::default(),
                    text: Text::new(
                        theme, emoji, TextSize::H3, TextStyle::Keyboard, Align::Left, None,
                    ).fit(),
                };
                PanelCell(
                    PanelCell::layout(32.0, 48.0), PanelCell::highlight(8.0), Some(face),
                    Some(CellAction::Emoji(emoji.to_string())), Press::default(), owner,
                )
            })
        })
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

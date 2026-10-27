use futures::future::Either;
use prism::events::{DynEvent, ListenCallback, OnChange, OnEvent, on_change, on_event};
use prism::events::{KeyboardEvent, KeyboardState, Key};
use prism::drawable::text::Align;
use prism::Context;
use prism::drawable::Component;
use ptsd::layout::{Column, Row, Stack, Offset, Size, Padding};
use ptsd::display::Bin;
use ptsd::interfaces::ShowKeyboard;
use ptsd::emitters::Press;
use uuid::Uuid;

use crate::theme::{Theme, Icons};
use crate::components::text::{Text, TextStyle, TextSize};
use crate::Task;
use crate::components::{Gif, GifPlayer, GifBatch, GifError, SendEvent};
use crate::components::{TextInput, TextInputEvent, Rectangle};
use crate::components::button::GhostIconButton;

use super::{GrowingGrid, PanelCell, CellFace, CellAction, TouchEnd};
use super::{GridView, PanelGrid};

#[derive(Debug, Clone, Copy)]
pub enum GifSearchEvent {Open, Submit, Close}
impl prism::events::Event for GifSearchEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Component, Debug, Clone)]
pub struct GifSearchHeader(Row, Bin<Stack, TextInput>, GhostIconButton);

impl GifSearchHeader {
    pub fn new(theme: &Theme, query: &str) -> Self {
        let field = TextInput::new(
            theme, uuid::Uuid::new_v4(), (!query.is_empty()).then_some(query), None,
            Some("Search GIPHY"), None, None,
        );
        let mut close = GhostIconButton::new(
            theme, Icons::Close,
            Stack::new(
                Offset::Center, Offset::Center, Size::Fit, Size::Fill,
                Padding(6.0, 6.0, 12.0, 16.0),
            ),
            |ctx, _| ctx.emit(GifSearchEvent::Close),
        );
        close.1.set_trigger_on_release();
        let field = Bin(
            Stack::new(
                Offset::Center, Offset::Center, Size::Fit, Size::Fit,
                Padding(12.0, 6.0, 6.0, 16.0),
            ),
            field,
        );
        let height = Size::custom(|heights| heights[0]);
        Self(Row::new(0.0, Offset::Center, height, Padding::default()), field, close)
    }

    fn value(&self) -> String {self.1.value()}
    fn has_focus(&self) -> bool {self.1.has_focus()}
}

#[derive(Component, Debug, Clone)]
pub struct GifPanel {
    layout: Column,
    search_field: Option<GifSearchHeader>,
    line: Option<Bin<Stack, Rectangle>>,
    bar: GifBar,
    grid: GrowingGrid,
    #[skip] owner: Uuid,
    #[skip] query: Option<String>,
    #[skip] gifs: Vec<Gif>,
    #[skip] more: bool,
    #[skip] offset: usize,
    #[skip] fetch: Option<Task<Result<GifBatch, GifError>>>,
    #[skip] error: Option<GifError>,
    #[skip] width: f32,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl GifPanel {
    pub fn new(theme: &Theme) -> Self {
        let owner = Uuid::new_v4();
        GifPanel {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            search_field: None, line: None, bar: GifBar::new(theme, false),
            grid: GrowingGrid::new(PanelGrid::tray_height(374.0) - 48.0, 3, owner),
            owner, query: None, gifs: Vec::new(), more: true, offset: 0,
            fetch: None, error: None, width: 374.0, side: None,
            theme: theme.clone(),
        }
    }

    pub fn searching(&self) -> bool { self.search_field.is_some() }

    pub fn close_search(&mut self) {
        if self.search_field.take().is_none() { return; }
        self.line = None;
        self.bar = GifBar::new(&self.theme, false);
        self.resize_height();
    }

    pub fn retry(&mut self) {
        self.error = None;
        self.bar = GifBar::new(&self.theme, self.searching());
        self.grid.invalidate();
    }
    fn resize_height(&mut self) {
        let height = PanelGrid::tray_height(self.width) - 48.0;
        self.grid.set_height(if self.searching() {
            Size::custom(move |_| (0.0, height))
        } else {Size::Static(height)});
    }
}

#[on_change(self.grid.next())]
impl OnChange<Either<GridView, ListenCallback>> for GifPanel {
    fn on_change(&mut self, ctx: &mut Context, update: Either<GridView, ListenCallback>) {
        match update {
            Either::Left(view) => self.grow(view),
            Either::Right(callback) => callback(&mut self.grid, ctx),
        }
    }
}

#[on_change(async {
    let Some(task) = &mut self.fetch else {return std::future::pending().await};
    match task.next().await {Some(result) => result, None => std::future::pending().await}
})]
impl OnChange<Result<GifBatch, GifError>> for GifPanel {
    fn on_change(&mut self, _ctx: &mut Context, result: Result<GifBatch, GifError>) {
        self.fetch = None;
        match result {
            Ok(batch) => {
                for gif in batch.gifs {
                    if !self.gifs.iter().any(|g| g.id == gif.id) {self.gifs.push(gif);}
                }
                self.more = batch.more;
                self.offset = batch.next_offset;
                if !batch.failures.is_empty() {
                    self.bar.2.1 = Text::new(
                        &self.theme, "Some GIFs could not be loaded", TextSize::Sm,
                        TextStyle::Error, Align::Center, None,
                    );
                }
            }
            Err(error) => {
                let message = match error {
                    GifError::NoApiKey => "GIF search is not configured",
                    GifError::Unavailable => "GIFs could not be loaded. Reopen to retry.",
                };
                self.bar.2.1 = Text::new(
                    &self.theme, message, TextSize::Sm, TextStyle::Error, Align::Center, None,
                );
                self.error = Some(error);
            }
        }
        self.grid.invalidate();
    }
}

impl GifPanel {
    fn grow(&mut self, view: GridView) {
        let side = PanelGrid::cell_side(view.width, 3);
        let resized = self.side.is_none_or(|previous| (previous - side).abs() >= 0.5);
        let mut count = self.grid.built();
        if resized {
            self.width = view.width;
            self.side = Some(side);
            self.grid.set_cells(side);
            self.resize_height();
            self.grid.empty();
        }

        if view.near_end {
            if count < self.gifs.len() {count += 12;}
            else if self.more && self.fetch.is_none() && self.error.is_none() {
                self.fetch = Some(self.theme.gifs().search(self.query.clone(), self.offset));
            }
        }

        let end = count.min(self.gifs.len());
        if self.grid.built() < end {
            let cells = self.gifs[self.grid.built()..end].iter().map(|gif| {
                let mut player = GifPlayer::thumbnail(&self.theme, gif, (side, side));
                player.set_active(false);
                let face = CellFace::Gif {
                    layout: PanelCell::square(side), player, outline: PanelCell::outline(8.0),
                };
                PanelCell(
                    PanelCell::square(side), PanelCell::highlight(8.0), Some(face),
                    Some(CellAction::Gif(gif.clone())), Press::default(), self.owner,
                )
            }).collect();
            self.grid.append(cells);
        }

        if !resized {
            let live = view.showing.start.saturating_sub(2)..view.showing.end + 2;
            for (r, row) in self.grid.grid().1.iter_mut().enumerate() {
                for cell in &mut row.1 {
                    if let Some(CellFace::Gif {player, ..}) = &mut cell.2 {player.set_active(live.contains(&r));}
                }
            }
        }
    }
}

#[on_event]
impl OnEvent<TouchEnd> for GifPanel {
    fn on(&mut self, ctx: &mut Context, event: TouchEnd) -> Vec<Box<dyn DynEvent>> {
        if let TouchEnd::Tap(owner, CellAction::Gif(gif)) = &event && *owner == self.owner {
            ctx.hardware.trigger_haptic();
            ctx.emit(SendEvent::Gif(gif.clone()));
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<GifSearchEvent> for GifPanel {
    fn on(&mut self, ctx: &mut Context, event: GifSearchEvent) -> Vec<Box<dyn DynEvent>> {
        match event {
            GifSearchEvent::Open if self.search_field.is_none() => {
                let field = self.search_field.insert(GifSearchHeader::new(
                    &self.theme, self.query.as_deref().unwrap_or_default(),
                ));
                ctx.emit(TextInputEvent::Focus {id: field.1.id()});
                let line = Stack::new(
                    Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0),
                    Padding(0.0, 0.0, 0.0, 2.0),
                );
                self.line = Some(Bin(line, Rectangle::new(
                    self.theme.colors().get(ptsd::Outline::Secondary), 0.0, None,
                )));
                self.bar = GifBar::new(&self.theme, true);
                self.resize_height();
            },
            GifSearchEvent::Submit => if let Some(field) = &self.search_field {
                let query = field.value().trim().to_string();
                self.query = (!query.is_empty()).then_some(query);
                self.gifs.clear();
                self.offset = 0;
                self.more = true;
                self.error = None;
                self.grid.clear();
                self.fetch = Some(self.theme.gifs().search(self.query.clone(), 0));
                ctx.emit(GifSearchEvent::Close);
            },
            GifSearchEvent::Close if self.searching() => {
                self.close_search();
                ctx.emit(ShowKeyboard::Hide);
            },
            _ => {},
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<SendEvent> for GifPanel {
    fn on(&mut self, ctx: &mut Context, event: SendEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, SendEvent::Gif(_)) && self.searching() {
            self.close_search();
            ctx.emit(ShowKeyboard::Hide);
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<KeyboardEvent> for GifPanel {
    fn on(&mut self, ctx: &mut Context, event: KeyboardEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(&event, KeyboardEvent {key: Key::Enter, state: KeyboardState::Pressed, ..})
            && self.search_field.as_ref().is_some_and(|field| field.has_focus())
        {
            ctx.emit(GifSearchEvent::Submit);
            return vec![];
        }
        prism::events![event]
    }
}

#[derive(Component, Debug, Clone)]
struct GifBar(Row, Option<GhostIconButton>, Bin<Stack, Text>);

impl GifBar {
    fn new(theme: &Theme, searching: bool) -> Self {
        GifBar(
            Row::center(0.0),
            (!searching).then(|| {
                let mut button = GhostIconButton::new(
                    theme, Icons::Search,
                    Stack::new(
                        Offset::Center, Offset::Center, Size::Fit, Size::Fit,
                        Padding(12.0, 6.0, 6.0, 6.0),
                    ),
                    |ctx, _| ctx.emit(GifSearchEvent::Open),
                );
                button.1.set_trigger_on_release();
                button
            }),
            Bin(
                Stack::new(
                    Offset::Center, Offset::Center, Size::Fit, Size::Fit,
                    if searching {Padding(12.0, 6.0, 12.0, 6.0)} else {Padding(6.0, 0.0, 12.0, 0.0)},
                ),
                Text::new(
                    theme, "Powered by GIPHY", TextSize::Sm, TextStyle::Secondary, Align::Right, None,
                ),
            ),
        )
    }
}

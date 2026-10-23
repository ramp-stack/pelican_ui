use prism::drawable::{RangeTree, Rect, Instruction};
use prism::events::{DynEvent, OnChange, on_change, ChangeSource};
use prism::events::{MouseEvent, KeyboardEvent, KeyboardState, Key};
use prism::drawable::text::Align;
use prism::Context;
use prism::drawable::{Drawable, Component, LayoutTree};
use ptsd::layout::{Column, Row, Stack, Offset, Size, Padding};
use ptsd::display::Bin;
use ptsd::interfaces::ShowKeyboard;

use crate::theme::{Theme, Icons};
use crate::components::text::{ExpandableText, TextStyle, TextSize};
use crate::Task;
use crate::components::{Gif, GifPlayer, GifBatch, GifError, SendEvent};
use crate::components::{TextInput, TextInputEvent, Rectangle};
use crate::components::button::GhostIconButton;

use super::{GrowingGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace};
use super::{GridView, PanelGrid};

#[derive(Debug, Clone, Copy)]
pub enum GifSearchEvent {Open, Submit, Close}
impl prism::events::Event for GifSearchEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone)]
pub struct GifPanel(GifPanelContent);

impl GifPanel {
    pub fn new(theme: &Theme) -> Self {Self(GifPanelContent::new(theme))}
    pub fn searching(&self) -> bool {self.0.searching()}
    pub fn close_search(&mut self) {self.0.close_search()}
    pub fn retry(&mut self) {self.0.retry()}
}

impl Drawable for GifPanel {
    fn caret(&self, layout: &LayoutTree) -> Option<prism::drawable::Caret> {
        Drawable::caret(&self.0, layout)
    }
    fn changed(&mut self) -> prism::events::ChangeFuture<'_> {
        Box::pin(async move {
            Drawable::changed(&mut self.0).await.map::<Self>(|this| &mut this.0)
        })
    }
    fn size_range(&self) -> RangeTree {self.0.size_range()}
    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        Drawable::layout(&mut self.0, size, range)
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.0.draw(layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        for event in self.0.on_event(ctx, layout, event) {self.0.event(ctx, layout, event);}
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
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fill, Padding(6.0, 6.0, 12.0, 16.0)),
            |ctx, _| ctx.emit(GifSearchEvent::Close),
        );
        close.1.set_trigger_on_release();
        let field = Bin(
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(12.0, 6.0, 6.0, 16.0)),
            field,
        );
        let height = Size::custom(|heights| heights[0]);
        Self(Row::new(0.0, Offset::Center, height, Padding::default()), field, close)
    }

    fn value(&self) -> String {self.1.value()}
    fn has_focus(&self) -> bool {self.1.has_focus()}
}

#[derive(Component, Debug, Clone)]
struct GifPanelContent {
    layout: Column,
    search_field: Option<GifSearchHeader>,
    line: Option<Bin<Stack, Rectangle>>,
    bar: GifBar,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
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

impl GifPanelContent {
    fn new(theme: &Theme) -> Self {
        GifPanelContent {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            search_field: None, line: None, bar: GifBar::new(theme, false),
            grid: GrowingGrid::new(PanelGrid::tray_height(374.0) - 48.0, 3),
            touch: None, query: None, gifs: Vec::new(), more: true, offset: 0,
            fetch: None, error: None, width: 374.0, side: None,
            theme: theme.clone(),
        }
    }

    fn searching(&self) -> bool { self.search_field.is_some() }

    fn close_search(&mut self) {
        if self.search_field.take().is_none() { return; }
        self.line = None;
        self.bar = GifBar::new(&self.theme, false);
        self.resize_height();
    }

    fn retry(&mut self) {
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

#[on_change(ChangeSource::changed(&mut self.fetch))]
impl OnChange<Result<GifBatch, GifError>> for GifPanelContent {
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
                    self.bar.2.1 = ExpandableText::new(
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
                self.bar.2.1 = ExpandableText::new(
                    &self.theme, message, TextSize::Sm, TextStyle::Error, Align::Center, None,
                );
                self.error = Some(error);
            }
        }
        self.grid.invalidate();
    }
}

#[on_change(ChangeSource::changed(&mut self.grid))]
impl OnChange<futures::future::Either<GridView, prism::events::Change>> for GifPanelContent {
    fn on_change(&mut self, ctx: &mut Context, update: futures::future::Either<GridView, prism::events::Change>) {
        let view = match update {
            futures::future::Either::Left(view) => view,
            futures::future::Either::Right(change) => {change.apply(&mut self.grid, ctx); return}
        };
        let side = PanelGrid::cell_side(view.width, 3);
        let resized = self.side.is_none_or(|previous| (previous - side).abs() >= 0.5);
        let mut count = self.grid.built();
        if resized {
            self.width = view.width;
            self.side = Some(side);
            self.grid.set_cells(side);
            self.resize_height();
            self.touch = None;
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
                PanelCell(PanelCell::square(side), PanelCell::highlight(8.0), Some(face), Some(CellAction::Gif(gif.clone())))
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

impl GifPanelContent {
    fn on_event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(e) = event.downcast_ref::<GifSearchEvent>() {
            match e {
                GifSearchEvent::Open if self.search_field.is_none() => {
                    let field = self.search_field.insert(GifSearchHeader::new(
                        &self.theme, self.query.as_deref().unwrap_or_default(),
                    ));
                    ctx.emit(TextInputEvent::Focus {id: field.1.id()});
                    let line = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0, 0.0, 0.0, 2.0));
                    self.line = Some(Bin(line, Rectangle::new(self.theme.colors().get(ptsd::Outline::Secondary), 0.0, None)));
                    self.bar = GifBar::new(&self.theme, true);
                    self.resize_height();
                },
                GifSearchEvent::Submit => if let Some(field) = &self.search_field {
                    let query = field.value().trim().to_string();
                    self.query = (!query.is_empty()).then_some(query);
                    self.gifs.clear();
                    self.offset = 0;
                    self.touch = None;
                    self.more = true;
                    self.error = None;
                    self.grid.clear();
                    self.fetch = Some(self.theme.gifs().search(self.query.clone(), 0));
                    ctx.emit(GifSearchEvent::Close);
                },
                GifSearchEvent::Close if self.searching() => {
                    self.close_search();
                    ctx.emit(ShowKeyboard(false));
                },
                _ => {},
            }
        } else if let Some(SendEvent::Gif(_)) = event.downcast_ref::<SendEvent>() && self.searching() {
            self.close_search();
            ctx.emit(ShowKeyboard(false));
        } else if let Some(KeyboardEvent{key: Key::Enter, state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.search_field.as_ref().is_some_and(|field| field.has_focus())
        {
            ctx.emit(GifSearchEvent::Submit);
            return vec![];
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            let grid = sized.1.len().saturating_sub(1);
            if let Some(TouchEnd::Tap(CellAction::Gif(gif))) = PanelTouch::track(ctx, &mut self.touch, self.grid.grid(), GridPlace::find(sized, &[grid, 0, 0]), mouse) {
                ctx.hardware.trigger_haptic();
                ctx.emit(SendEvent::Gif(gif));
            }
        }
        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
struct GifBar(Row, Option<GhostIconButton>, Bin<Stack, ExpandableText>);

impl GifBar {
    fn new(theme: &Theme, searching: bool) -> Self {
        GifBar(
            Row::center(0.0),
            (!searching).then(|| {
                let mut button = GhostIconButton::new(
                    theme, Icons::Search,
                    Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(12.0, 6.0, 6.0, 6.0)),
                    |ctx, _| ctx.emit(GifSearchEvent::Open),
                );
                button.1.set_trigger_on_release();
                button
            }),
            Bin(
                Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit,
                    if searching {Padding(12.0, 6.0, 12.0, 6.0)} else {Padding(6.0, 0.0, 12.0, 0.0)}),
                ExpandableText::new(theme, "Powered by GIPHY", TextSize::Sm, TextStyle::Secondary, Align::Right, None),
            ),
        )
    }
}

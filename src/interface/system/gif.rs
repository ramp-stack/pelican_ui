use prism::event::{MouseEvent, TickEvent, OnEvent, Event, MouseState, MouseButton, KeyboardEvent, KeyboardState, Key};
use prism::canvas::Align;
use prism::Context;
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Column, Row, Stack, Offset, Size, Padding};
use prism::display::Bin;

use crate::theme::{Theme, Icons};
use crate::components::text::{ExpandableText, TextStyle, TextSize};
use crate::Task;
use crate::components::{Gif, GifPlayer, GifBatch, GifError, SendEvent, find_gifs};
use crate::components::{TextInput, Rectangle};
use crate::components::button::GhostIconButton;

use super::keyboard::{KeyboardPages, MobileKeyboardEvent};
use super::{GrowingGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace, track_touch};
use super::{cell_side, tray_height, laid_out_width, ASSUMED_WIDTH};

#[derive(Component, Debug, Clone)]
pub struct GifPanel {
    layout: Column,
    search_field: Option<GifSearchHeader>,
    line: Option<Bin<Stack, Rectangle>>,
    bar: GifBar,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    #[skip] query: Option<String>,
    #[skip] gifs: Vec<Gif>,
    #[skip] more: bool,
    #[skip] fetch: Option<Task<Result<GifBatch, GifError>>>,
    #[skip] error: Option<GifError>,
    #[skip] full: bool,
    #[skip] width: f32,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl GifPanel {
    pub fn new(theme: &Theme) -> Self {
        GifPanel {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            search_field: None,
            line: None,
            bar: GifBar::new(theme, false),
            grid: GrowingGrid::new(tray_height(ASSUMED_WIDTH) - 48.0, 3),
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

    pub fn searching(&self) -> bool { self.search_field.is_some() }

    pub fn close_search(&mut self) {
        if self.search_field.take().is_none() { return; }
        self.line = None;
        self.bar = GifBar::new(&self.theme, false);
    }

    pub fn set_full(&mut self, full: bool) { self.full = full; }

    pub fn retry(&mut self) { self.error = None; }
}

impl OnEvent for GifPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(result) = self.fetch.as_ref().and_then(|fetch| fetch.take()) {
                self.fetch = None;
                match result {
                    Ok(batch) => {
                        for gif in batch.gifs {
                            if !self.gifs.iter().any(|g| g.id == gif.id) { self.gifs.push(gif); }
                        }
                        self.more = batch.more;
                    },
                    Err(error) => self.error = Some(error),
                }
            }

            let mut count = self.grid.built();

            if let Some(width) = laid_out_width(sized).filter(|w| self.side.is_none_or(|s| (s - cell_side(*w, 3)).abs() >= 0.5)) {
                let side = cell_side(width, 3);
                self.width = width;
                self.side = Some(side);
                self.grid.set_cells(side);
                self.touch = None;
                self.grid.empty();
            }

            let height = tray_height(self.width) - 48.0;
            self.grid.set_height(match self.full {
                true => Size::custom(move |_: Vec<(f32, f32)>| (height, f32::MAX)),
                false => Size::Static(height),
            });

            let grid = sized.1.last().map(|(_, grid)| grid);
            if let Some(grid) = grid && self.side.is_some() && self.grid.near_end(grid) {
                if count < self.gifs.len() {
                    count += 12;
                } else if self.more && self.fetch.is_none() && self.error.is_none() {
                    self.fetch = Some(find_gifs(self.query.clone(), self.gifs.len()));
                }
            }

            let end = count.min(self.gifs.len());
            if let Some(side) = self.side && self.grid.built() < end {
                let cells = self.gifs[self.grid.built()..end].iter().map(|gif| {
                    let face = CellFace::Gif {
                        layout: PanelCell::square(side),
                        player: GifPlayer::thumbnail(gif, (side, side)),
                        outline: PanelCell::outline(8.0),
                    };
                    PanelCell(PanelCell::square(side), PanelCell::highlight(8.0), Some(face), Some(CellAction::Gif(gif.clone())))
                }).collect();
                self.grid.append(cells);
            }

            if let Some(grid) = grid {
                let showing = self.grid.showing_rows(grid);
                let live = showing.start.saturating_sub(2)..showing.end + 2;
                for (r, row) in self.grid.grid().1.iter_mut().enumerate() {
                    for cell in &mut row.1 {
                        if let Some(CellFace::Gif {player, ..}) = &mut cell.2 { player.set_live(live.contains(&r)); }
                    }
                }
            }
        } else if let Some(e) = event.downcast_ref::<MobileKeyboardEvent>() {
            match e {
                MobileKeyboardEvent::StartGifSearch if self.search_field.is_none() => {
                    self.search_field = Some(GifSearchHeader::new(&self.theme, self.query.as_deref().unwrap_or_default()));
                    let line = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0, 0.0, 0.0, 2.0));
                    self.line = Some(Bin(line, Rectangle::new(self.theme.colors().get(ptsd::Outline::Secondary), 0.0, None)));
                    self.bar = GifBar::new(&self.theme, true);
                },
                MobileKeyboardEvent::SubmitGifSearch => if let Some(field) = &self.search_field {
                    let query = field.1.value().trim().to_string();
                    self.query = (!query.is_empty()).then_some(query);
                    self.gifs.clear();
                    self.touch = None;
                    self.more = true;
                    self.error = None;
                    self.grid.clear();
                    self.fetch = Some(find_gifs(self.query.clone(), 0));
                },
                MobileKeyboardEvent::CancelGifSearch => self.close_search(),
                _ => {},
            }
        } else if let Some(SendEvent::Gif(_)) = event.downcast_ref::<SendEvent>() {
            self.close_search();
        } else if let Some(KeyboardEvent{key: Key::Enter, state: KeyboardState::Pressed, ..}) = event.downcast_ref::<KeyboardEvent>()
            && self.search_field.as_ref().is_some_and(|field| field.1.has_focus())
        {
            ctx.emit(MobileKeyboardEvent::SubmitGifSearch);
            return vec![];
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            let grid = sized.1.len().saturating_sub(1);
            if let Some(TouchEnd::Tap(CellAction::Gif(gif))) = track_touch(&mut self.touch, self.grid.grid(), GridPlace::find(sized, &[grid, 0, 0]), mouse) {
                ctx.trigger_haptic();
                ctx.emit(SendEvent::Gif(gif));
            }
        }
        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
struct GifBar(Row, Option<GhostIconButton>, ExpandableText);
impl OnEvent for GifBar {}

impl GifBar {
    fn new(theme: &Theme, searching: bool) -> Self {
        GifBar(
            Row::new(12.0, Offset::Center, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)),
            (!searching).then(|| {
                let mut button = GhostIconButton::new(theme, Icons::Search, |ctx: &mut Context, _: &Theme| ctx.emit(MobileKeyboardEvent::StartGifSearch));
                button.1.set_trigger_on_release();
                button
            }),
            ExpandableText::new(theme, "Powered by GIPHY", TextSize::Sm, TextStyle::Secondary, Align::Right, None),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct GifSearch {
    pub open: bool,
    pub focused: Option<(String, String)>,
    pub return_to: Option<(String, String)>,
}

#[derive(Component, Debug, Clone)]
pub struct GifSearchHeader(Row, TextInput, GhostIconButton, #[skip] Option<u8>);

impl GifSearchHeader {
    pub fn new(theme: &Theme, query: &str) -> Self {
        let field = TextInput::new(theme, (!query.is_empty()).then_some(query), None, Some("Search GIPHY"), None, None, KeyboardPages::NONE);
        let mut close = GhostIconButton::new(theme, Icons::Close, |ctx: &mut Context, _: &Theme| ctx.emit(MobileKeyboardEvent::CancelGifSearch));
        close.1.set_trigger_on_release();
        GifSearchHeader(Row::new(12.0, Offset::Center, Size::Fit, Padding(12.0, 6.0, 12.0, 16.0)), field, close, Some(2))
    }
}

impl OnEvent for GifSearchHeader {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() && let Some(frames) = &mut self.3 {
            *frames = frames.saturating_sub(1);
            if *frames == 0 && let Some((_, field)) = sized.1.first() {
                self.3 = None;
                let at = Some(((field.0.0 - 16.0).max(0.0), field.0.1 / 2.0));
                for state in [MouseState::Pressed(MouseButton::Left), MouseState::Released(MouseButton::Left)] {
                    self.1.event(ctx, field, Box::new(MouseEvent { position: at, state }));
                }
            }
        }
        vec![event]
    }
}

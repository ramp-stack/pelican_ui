//! The GIF page, and the GIF search.

use prism::event::{MouseEvent, TickEvent, OnEvent, Event, MouseState, MouseButton};
use prism::canvas::Align;
use prism::Context;
use prism::drawable::{Drawable, Component, SizedTree};
use prism::layout::{Column, Row, Offset, Size, Padding};

use crate::theme::{Theme, Icons};
use crate::components::text::{Text, ExpandableText, TextStyle, TextSize};
use crate::components::{Gif, GifPlayer, GifBatch, GifError, SendGifEvent, Task, find_gifs};
use crate::components::TextInput;
use crate::components::button::GhostIconButton;

use super::keyboard::{KeyboardPages, MobileKeyboardEvent};
use super::{GrowingGrid, PanelGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace, track_touch};
use super::{cell_side, tray_height, laid_out_width, ASSUMED_WIDTH, LOAD_ROWS};

#[derive(Component, Debug, Clone)]
pub struct GifPanel {
    layout: Column,
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

    fn fit(&mut self, width: f32) {
        let side = cell_side(width, 3);
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

    fn grid_height(&self) -> Size {
        let height = tray_height(self.width) - 48.0;
        match self.full {
            true => Size::custom(move |_: Vec<(f32, f32)>| (height, f32::MAX)),
            false => Size::Static(height),
        }
    }

    pub fn query(&self) -> Option<String> { self.query.clone() }

    pub fn set_full(&mut self, full: bool) {
        if full == self.full { return; }
        self.full = full;
        self.bar = GifBar::new(&self.theme, full);
        self.grid.set_height(self.grid_height());
    }

    pub fn set_search_button(&mut self, show: bool) {
        self.bar = GifBar::new(&self.theme, !show);
    }

    pub fn search(&mut self, query: Option<String>) {
        self.query = query;
        self.gifs.clear();
        self.touch = None;
        self.more = true;
        self.error = None;
        self.grid.clear();
        self.fetch = Some(find_gifs(self.query.clone(), 0));
    }

    pub fn retry(&mut self) {
        if self.error.is_some() { self.error = None; }
    }

    fn received(&mut self, result: Result<GifBatch, GifError>) {
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

    fn grow(&mut self, sized: &SizedTree) {
        let Some(side) = self.side else { return };
        if !self.grid.near_end(sized) { return; }
        let built = self.grid.built();
        if built < self.gifs.len() {
            let end = (built + 3 * LOAD_ROWS).min(self.gifs.len());
            self.grid.append(self.gifs[built..end].iter().map(|gif| PanelCell::gif(gif, side)).collect());
        } else if self.more && self.fetch.is_none() && self.error.is_none() {
            self.fetch = Some(find_gifs(self.query.clone(), self.gifs.len()));
        }
    }

    fn update_live(&mut self, sized: &SizedTree) {
        let showing = self.grid.showing_rows(sized);
        let live = showing.start.saturating_sub(2)..showing.end + 2;
        self.grid.grid().set_live(live);
    }

    fn status(&self) -> String {
        match (self.error, self.gifs.is_empty()) {
            (Some(GifError::NoApiKey), _) => "No GIPHY API key".to_string(),
            (Some(GifError::Unavailable), true) => "Can't reach GIPHY".to_string(),
            (None, true) if self.fetch.is_some() => "Loading GIFs\u{2026}".to_string(),
            (None, true) if !self.more => "No GIFs found".to_string(),
            _ => String::new(),
        }
    }
}

impl OnEvent for GifPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(result) = self.fetch.as_ref().and_then(|fetch| fetch.take()) { self.received(result); }
            if let Some(width) = laid_out_width(sized) { self.fit(width); }
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

impl PanelGrid {
    fn set_live(&mut self, rows: std::ops::Range<usize>) {
        self.1.iter_mut().enumerate().for_each(|(r, row)| row.1.iter_mut().for_each(|cell|
            if let Some(CellFace::Gif {player, ..}) = &mut cell.2 { player.set_live(rows.contains(&r)); }
        ))
    }
}

impl PanelCell {
    fn gif(gif: &Gif, side: f32) -> Self {
        let face = CellFace::Gif {
            layout: Self::square(side),
            player: GifPlayer::thumbnail(gif, (side, side)),
            outline: Self::outline(8.0),
        };
        PanelCell(Self::square(side), Self::highlight(8.0), Some(face), Some(CellAction::Gif(gif.clone())))
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

    pub fn query(&self) -> String { self.1.value() }
    pub fn has_focus(&self) -> bool { self.1.has_focus() }
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

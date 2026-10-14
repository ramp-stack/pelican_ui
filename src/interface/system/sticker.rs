//! The sticker page.

use prism::event::{MouseEvent, TickEvent, OnEvent, Event, PickedPhoto};
use prism::canvas::{Image, ShapeType};
use prism::Context;
use prism::drawable::{Component, SizedTree};
use prism::layout::{Stack, Size};

use maverick_os::air::Instance;

use crate::theme::{Theme, Color, Icons};
use crate::components::{Rectangle, Icon, Sticker, SendStickerEvent, StickerPack, StickerPackAction};

use super::{GrowingGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace, track_touch};
use super::{cell_side, tray_height, laid_out_width, ASSUMED_WIDTH, LOAD_ROWS};

/// Stickers to a row (see [`cell_side`]).
const STICKER_COLUMNS: usize = 4;
const STICKER_TITLE: &str = "Stickers";

/// The sticker page: your stickers in a column of rows that scrolls up and
/// down, starting with a tile that makes a new one from a photo. Tap a sticker
/// to send it (with a [`SendStickerEvent`] for the chat).
///
/// Rows are built a few at a time as you scroll near the bottom, so a big
/// pack doesn't slow the keyboard down.
///
/// The stickers are your [`StickerPack`]. The page finds it (or makes it) the
/// first time it's shown, adds the stickers you make to it, and keeps up with
/// it while it's showing, so stickers added on another device appear too.
#[derive(Component, Debug, Clone)]
pub(super) struct StickerPanel {
    layout: Stack,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] waiting_for_photo: bool,
    /// The side of a sticker: none are built until it's known.
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl StickerPanel {
    pub(super) fn new(theme: &Theme) -> Self {
        StickerPanel {
            layout: Stack::default(),
            grid: GrowingGrid::new(tray_height(ASSUMED_WIDTH), STICKER_COLUMNS),
            touch: None,
            pack: None,
            stickers: Vec::new(),
            waiting_for_photo: false,
            side: None,
            theme: theme.clone(),
        }
    }

    /// Sizes the stickers to fit `width`, rebuilding the ones built if
    /// that changes their size.
    fn fit(&mut self, width: f32) {
        let side = cell_side(width, STICKER_COLUMNS);
        if self.side.is_some_and(|s| (s - side).abs() < 0.5) { return; }
        self.side = Some(side);
        self.grid.set_cells(side);
        self.grid.set_height(Size::Static(tray_height(width)));
        let built = self.grid.built();
        self.touch = None;
        self.grid.empty();
        self.build(built);
    }

    /// Picks up changes to the pack (new stickers go first), rebuilding the
    /// rows built so far so the page stays where it was.
    fn sync(&mut self, ctx: &mut Context) {
        let pack = self.pack.get_or_insert_with(|| StickerPack::mine(ctx));
        if pack.pending().stickers == self.stickers { return; }
        self.stickers = pack.pending().stickers.clone();
        let built = self.grid.built();
        self.touch = None;
        self.grid.empty();
        self.build(built);
    }

    /// Adds rows while you're near the bottom of the ones built.
    fn grow(&mut self, sized: &SizedTree) {
        if self.grid.near_end(sized) { self.build(self.grid.built() + STICKER_COLUMNS * LOAD_ROWS); }
    }

    /// Builds cells up to `count` (or the end). The first is the new-sticker tile.
    fn build(&mut self, count: usize) {
        let Some(side) = self.side else { return };
        let (built, end) = (self.grid.built(), count.min(self.stickers.len() + 1));
        if built >= end { return; }
        let cells = (built..end).map(|i| match i {
            0 => PanelCell::new_sticker(&self.theme, side),
            n => PanelCell::sticker(&self.stickers[n - 1], side),
        }).collect();
        self.grid.append(cells);
    }

    /// For the keyboard's header.
    pub(super) fn label(&self) -> (String, String) {
        (STICKER_TITLE.to_string(), String::new())
    }
}

impl OnEvent for StickerPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(width) = laid_out_width(sized) { self.fit(width); }
            self.sync(ctx);
            if let Some((_, grid)) = sized.1.first() { self.grow(grid); }
        } else if let Some(PickedPhoto(photo)) = event.downcast_ref::<PickedPhoto>() && self.waiting_for_photo {
            // The photo picked with the new-sticker tile.
            self.waiting_for_photo = false;
            if let Some(sticker) = Sticker::from_photo(photo) && let Some(pack) = &mut self.pack {
                pack.send(StickerPackAction::Add(sticker));
                self.sync(ctx);
            }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match track_touch(&mut self.touch, self.grid.grid(), GridPlace::find(sized, &[0, 0, 0]), mouse) {
                Some(TouchEnd::Tap(CellAction::Sticker(sticker))) => {
                    ctx.trigger_haptic();
                    ctx.emit(SendStickerEvent(sticker));
                }
                Some(TouchEnd::Tap(CellAction::NewSticker)) => {
                    self.waiting_for_photo = true;
                    ctx.pick_photo();
                }
                _ => {}
            }
        }
        vec![event]
    }
}

impl PanelCell {
    fn sticker(sticker: &Sticker, side: f32) -> Self {
        let shape = ShapeType::RoundedRectangle(0.0, (side, side), 0.0, 12.0);
        let face = CellFace::Sticker {
            layout: Self::square(side),
            image: Image {shape, image: sticker.image(), color: None},
            outline: Self::outline(12.0),
        };
        PanelCell(Self::square(side), Self::highlight(12.0), Some(face), Some(CellAction::Sticker(sticker.clone())))
    }

    fn new_sticker(theme: &Theme, side: f32) -> Self {
        let face = CellFace::NewSticker {
            layout: Self::square(side),
            background: Rectangle::new(Color::from_hex("ffffff", 110), 12.0, None),
            icon: Icon::new(theme, Icons::Add, Some(Color::WHITE), 36.0),
            outline: Self::outline(12.0),
        };
        PanelCell(Self::square(side), Self::highlight(12.0), Some(face), Some(CellAction::NewSticker))
    }
}

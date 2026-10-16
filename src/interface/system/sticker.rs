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
use super::{cell_side, tray_height, laid_out_width, ASSUMED_WIDTH};

#[derive(Component, Debug, Clone)]
pub struct StickerPanel {
    layout: Stack,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] waiting_for_photo: bool,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl StickerPanel {
    pub fn new(theme: &Theme) -> Self {
        StickerPanel {
            layout: Stack::default(),
            grid: GrowingGrid::new(tray_height(ASSUMED_WIDTH), 4),
            touch: None,
            pack: None,
            stickers: Vec::new(),
            waiting_for_photo: false,
            side: None,
            theme: theme.clone(),
        }
    }
}

impl OnEvent for StickerPanel {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            let pack = self.pack.get_or_insert_with(|| StickerPack::mine(ctx));
            let width = laid_out_width(sized);
            let resized = width.is_some_and(|w| self.side.is_none_or(|s| (s - cell_side(w, 4)).abs() >= 0.5));
            let mut count = self.grid.built();

            if resized || pack.pending().stickers != self.stickers {
                if let Some(width) = width.filter(|_| resized) {
                    let side = cell_side(width, 4);
                    self.side = Some(side);
                    self.grid.set_cells(side);
                    self.grid.set_height(Size::Static(tray_height(width)));
                }
                self.stickers = pack.pending().stickers.clone();
                self.touch = None;
                self.grid.empty();
            }
            if let Some((_, grid)) = sized.1.first() && self.grid.near_end(grid) { count += 16; }

            let end = count.min(self.stickers.len() + 1);
            if let Some(side) = self.side && self.grid.built() < end {
                let cells = (self.grid.built()..end).map(|i| {
                    let (face, action) = match i {
                        0 => (CellFace::NewSticker {
                            layout: PanelCell::square(side),
                            background: Rectangle::new(Color::from_hex("ffffff", 110), 12.0, None),
                            icon: Icon::new(&self.theme, Icons::Add, Some(Color::WHITE), 36.0),
                            outline: PanelCell::outline(12.0),
                        }, CellAction::NewSticker),
                        n => (CellFace::Sticker {
                            layout: PanelCell::square(side),
                            image: Image {shape: ShapeType::RoundedRectangle(0.0, (side, side), 0.0, 12.0), image: self.stickers[n - 1].image(), color: None},
                            outline: PanelCell::outline(12.0),
                        }, CellAction::Sticker(self.stickers[n - 1].clone())),
                    };
                    PanelCell(PanelCell::square(side), PanelCell::highlight(12.0), Some(face), Some(action))
                }).collect();
                self.grid.append(cells);
            }
        } else if let Some(PickedPhoto(photo)) = event.downcast_ref::<PickedPhoto>() && self.waiting_for_photo {
            self.waiting_for_photo = false;
            if let Some(sticker) = Sticker::from_photo(photo) && let Some(pack) = &mut self.pack {
                pack.send(StickerPackAction::Add(sticker));
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
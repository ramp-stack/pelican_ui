use prism::drawable::Drawable;
use prism::drawable::{RangeTree, Rect, Instruction};
use prism::events::DynEvent;
use prism::events::{MouseEvent, TickEvent};
use prism::hardware::{Image, Picked};
use prism::drawable::shape::Variant;
use std::cell::RefCell;
use std::rc::Rc;
use prism::Context;
use prism::drawable::{Component, LayoutTree};
use ptsd::layout::{Stack, Size};

use prism::air::Instance;

use crate::theme::{Theme, Color, Icons};
use crate::Task;
use crate::components::{Rectangle, Icon, MaskedImage, Sticker, SendEvent, StickerPack, StickerPackAction};

use super::{GrowingGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace, track_touch};
use super::{cell_side, tray_height, laid_out_width, ASSUMED_WIDTH};

#[derive(Debug, Clone)]
pub struct StickerPanel(StickerPanelContent);

impl StickerPanel {
    pub fn new(theme: &Theme) -> Self {Self(StickerPanelContent::new(theme))}
}

impl Drawable for StickerPanel {
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

#[derive(Component, Clone)]
struct StickerPanelContent {
    layout: Stack,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] picking: Rc<RefCell<Option<Picked<Image>>>>,
    #[skip] making: Option<Task<Option<Sticker>>>,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl StickerPanelContent {
    pub fn new(theme: &Theme) -> Self {
        StickerPanelContent {
            layout: Stack::default(),
            grid: GrowingGrid::new(tray_height(ASSUMED_WIDTH), 4),
            touch: None,
            pack: None,
            stickers: Vec::new(),
            picking: Rc::new(RefCell::new(None)),
            making: None,
            side: None,
            theme: theme.clone(),
        }
    }
}

impl StickerPanelContent {
    fn on_event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            let picked = self.picking.borrow_mut().as_mut().and_then(Picked::try_get);
            if let Some(picked) = picked {
                self.picking.borrow_mut().take();
                if let Ok(photo) = picked {
                    self.making = Some(Task::spawn(move || Sticker::from_photo(&photo)));
                }
            }
            let pack = self.pack.get_or_insert_with(|| StickerPack::mine(ctx));
            if let Some(made) = self.making.as_ref().and_then(|task| task.take()) {
                self.making = None;
                if let Some(sticker) = made { pack.send(StickerPackAction::Add(sticker)); }
            }
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
                            image: MaskedImage(self.stickers[n - 1].image(), Variant::RoundedRectangle(12.0)),
                            outline: PanelCell::outline(12.0),
                        }, CellAction::Sticker(self.stickers[n - 1].clone())),
                    };
                    PanelCell(PanelCell::square(side), PanelCell::highlight(12.0), Some(face), Some(action))
                }).collect();
                self.grid.append(cells);
            }
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match track_touch(&mut self.touch, self.grid.grid(), GridPlace::find(sized, &[0, 0, 0]), mouse) {
                Some(TouchEnd::Tap(CellAction::Sticker(sticker))) => {
                    ctx.hardware.trigger_haptic();
                    ctx.emit(SendEvent::Sticker(sticker));
                }
                Some(TouchEnd::Tap(CellAction::NewSticker)) => {
                    *self.picking.borrow_mut() = Some(ctx.hardware.pick());
                }
                _ => {}
            }
        }
        vec![event]
    }
}

impl std::fmt::Debug for StickerPanelContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StickerPanelContent").field("grid", &self.grid).finish_non_exhaustive()
    }
}

use prism::drawable::{Drawable, Component, RangeTree, LayoutTree, Rect, Instruction};
use prism::drawable::shape::Variant;
use prism::events::{DynEvent, OnChange, on_change, ChangeSource, MouseEvent};
use prism::hardware::{Image, Picked};
use prism::air::Instance;
use prism::Context;
use ptsd::layout::{Stack, Size};

use crate::theme::{Theme, Color, Icons};
use crate::Task;
use crate::components::{Rectangle, Icon, MaskedImage, Sticker, SendEvent, StickerPack, StickerPackAction};

use super::{GrowingGrid, PanelCell, CellFace, CellAction, PanelTouch, TouchEnd, GridPlace};
use super::{GridView, PanelGrid};

#[derive(Debug, Clone)]
pub struct StickerPanel(StickerPanelContent);

impl StickerPanel {
    pub fn new(theme: &Theme) -> Self {Self(StickerPanelContent::new(theme))}
}

impl Drawable for StickerPanel {
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

#[derive(Component, Clone)]
struct StickerPanelContent {
    layout: Stack,
    grid: GrowingGrid,
    #[skip] touch: Option<PanelTouch>,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] picking: Option<Picked<Image>>,
    #[skip] making: Option<Task<Option<Sticker>>>,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl StickerPanelContent {
    pub fn new(theme: &Theme) -> Self {
        StickerPanelContent {
            layout: Stack::default(),
            grid: GrowingGrid::new(PanelGrid::tray_height(374.0), 4),
            touch: None,
            pack: None,
            stickers: Vec::new(),
            picking: None,
            making: None,
            side: None,
            theme: theme.clone(),
        }
    }
}

impl StickerPanelContent {
    fn refresh(&mut self) {
        self.stickers = self.pack.as_ref().unwrap().pending().stickers.clone();
        self.touch = None;
        self.grid.empty();
    }
}
#[on_change(ChangeSource::changed(&mut self.pack))]
impl OnChange<prism::air::Update<()>> for StickerPanelContent {
    fn on_change(&mut self, _ctx: &mut Context, _: prism::air::Update<()>) {self.refresh();}
}
#[on_change(ChangeSource::changed(&mut self.picking))]
impl OnChange<prism::hardware::PickResult<Image>> for StickerPanelContent {
    fn on_change(&mut self, _ctx: &mut Context, result: prism::hardware::PickResult<Image>) {
        self.picking = None;
        if let Ok(photo) = result {
            self.making = Some(Task::spawn(move || Sticker::from_photo(&photo)));
        }
    }
}
#[on_change(ChangeSource::changed(&mut self.making))]
impl OnChange<Option<Sticker>> for StickerPanelContent {
    fn on_change(&mut self, ctx: &mut Context, made: Option<Sticker>) {
        self.making = None;
        if let Some(sticker) = made {
            self.pack.get_or_insert_with(|| StickerPack::mine(ctx)).send(StickerPackAction::Add(sticker));
            self.refresh();
        }
    }
}

#[on_change(ChangeSource::changed(&mut self.grid))]
impl OnChange<futures::future::Either<GridView, prism::events::Change>> for StickerPanelContent {
    fn on_change(&mut self, ctx: &mut Context, update: futures::future::Either<GridView, prism::events::Change>) {
        let view = match update {
            futures::future::Either::Left(view) => view,
            futures::future::Either::Right(change) => {change.apply(&mut self.grid, ctx); return}
        };
        if self.pack.is_none() {
            self.pack = Some(StickerPack::mine(ctx));
            self.refresh();
        }
        let side = PanelGrid::cell_side(view.width, 4);
        let mut count = self.grid.built();
        if self.side.is_none_or(|previous| (previous - side).abs() >= 0.5) {
            self.side = Some(side);
            self.grid.set_cells(side);
            self.grid.set_height(Size::Static(PanelGrid::tray_height(view.width)));
            self.touch = None;
            self.grid.empty();
        }
        if view.near_end {count += 16;}

        let end = count.min(self.stickers.len() + 1);
        if self.grid.built() < end {
            let cells = (self.grid.built()..end).map(|i| {
                let (face, action) = match i {
                    0 => (CellFace::NewSticker {
                        layout: PanelCell::square(side),
                        background: Rectangle::new(Color::from_hex("ffffff", 40), 12.0, None),
                        icon: Icon::new(&self.theme, Icons::Add, Some(Color::white()), 36.0),
                        outline: PanelCell::outline(12.0),
                    }, CellAction::NewSticker),
                    n => (CellFace::Sticker {
                        layout: PanelCell::square(side),
                        image: MaskedImage(
                            ctx.hardware.image_from_rgba(self.stickers[n - 1].image()),
                            Variant::RoundedRectangle(12.0),
                        ),
                        outline: PanelCell::outline(12.0),
                    }, CellAction::Sticker(self.stickers[n - 1].clone())),
                };
                PanelCell(PanelCell::square(side), PanelCell::highlight(12.0), Some(face), Some(action))
            }).collect();
            self.grid.append(cells);
        }
    }
}

impl StickerPanelContent {
    fn on_event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match PanelTouch::track(ctx, &mut self.touch, self.grid.grid(), GridPlace::find(sized, &[0, 0, 0]), mouse) {
                Some(TouchEnd::Tap(CellAction::Sticker(sticker))) => {
                    ctx.hardware.trigger_haptic();
                    ctx.emit(SendEvent::Sticker(sticker));
                }
                Some(TouchEnd::Tap(CellAction::NewSticker)) => {
                    self.picking = Some(ctx.hardware.pick());
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

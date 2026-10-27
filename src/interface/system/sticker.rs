use futures::future::Either;
use prism::drawable::Component;
use prism::drawable::shape::Variant;
use prism::events::{DynEvent, ListenCallback, OnChange, OnEvent, on_change, on_event};
use prism::hardware::{Image, Picked};
use prism::air::Instance;
use prism::Context;
use ptsd::layout::{Stack, Size};
use ptsd::emitters::Press;
use uuid::Uuid;

use crate::theme::{Theme, Color, Icons};
use crate::Task;
use crate::components::{Rectangle, Icon, MaskedImage, Sticker, SendEvent, StickerPack, StickerPackAction};

use super::{GrowingGrid, PanelCell, CellFace, CellAction, TouchEnd};
use super::{GridView, PanelGrid};

#[derive(Component, Clone)]
pub struct StickerPanel {
    layout: Stack,
    grid: GrowingGrid,
    #[skip] owner: Uuid,
    #[skip] pack: Option<Instance<StickerPack>>,
    #[skip] stickers: Vec<Sticker>,
    #[skip] picking: Option<Picked<Image>>,
    #[skip] making: Option<Task<Option<Sticker>>>,
    #[skip] side: Option<f32>,
    #[skip] theme: Theme,
}

impl StickerPanel {
    pub fn new(theme: &Theme) -> Self {
        let owner = Uuid::new_v4();
        StickerPanel {
            layout: Stack::default(),
            grid: GrowingGrid::new(PanelGrid::tray_height(374.0), 4, owner),
            owner,
            pack: None,
            stickers: Vec::new(),
            picking: None,
            making: None,
            side: None,
            theme: theme.clone(),
        }
    }
}

impl StickerPanel {
    fn refresh(&mut self) {
        self.stickers = self.pack.as_ref().unwrap().pending().stickers.clone();
        self.grid.empty();
    }
}
#[on_change(self.grid.next())]
impl OnChange<Either<GridView, ListenCallback>> for StickerPanel {
    fn on_change(&mut self, ctx: &mut Context, update: Either<GridView, ListenCallback>) {
        match update {
            Either::Left(view) => self.grow(ctx, view),
            Either::Right(callback) => callback(&mut self.grid, ctx),
        }
    }
}
#[on_change(async {
    match &mut self.pack {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<prism::air::Update<()>> for StickerPanel {
    fn on_change(&mut self, _ctx: &mut Context, _: prism::air::Update<()>) {self.refresh();}
}
#[on_change(async {
    match &mut self.picking {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<prism::hardware::PickResult<Image>> for StickerPanel {
    fn on_change(&mut self, _ctx: &mut Context, result: prism::hardware::PickResult<Image>) {
        self.picking = None;
        if let Ok(photo) = result {
            self.making = Some(Task::spawn(move || Sticker::from_photo(&photo)));
        }
    }
}
#[on_change(async {
    let Some(task) = &mut self.making else {return std::future::pending().await};
    match task.next().await {Some(result) => result, None => std::future::pending().await}
})]
impl OnChange<Option<Sticker>> for StickerPanel {
    fn on_change(&mut self, ctx: &mut Context, made: Option<Sticker>) {
        self.making = None;
        if let Some(sticker) = made {
            self.pack.get_or_insert_with(|| StickerPack::mine(ctx)).send(StickerPackAction::Add(sticker));
            self.refresh();
        }
    }
}

impl StickerPanel {
    fn grow(&mut self, ctx: &mut Context, view: GridView) {
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
                            self.stickers[n - 1].image(), Variant::RoundedRectangle(12.0),
                        ),
                        outline: PanelCell::outline(12.0),
                    }, CellAction::Sticker(self.stickers[n - 1].clone())),
                };
                PanelCell(
                    PanelCell::square(side), PanelCell::highlight(12.0), Some(face), Some(action),
                    Press::default(), self.owner,
                )
            }).collect();
            self.grid.append(cells);
        }
    }
}

#[on_event]
impl OnEvent<TouchEnd> for StickerPanel {
    fn on(&mut self, ctx: &mut Context, event: TouchEnd) -> Vec<Box<dyn DynEvent>> {
        if let TouchEnd::Tap(owner, action) = &event && *owner == self.owner {
            match action {
                CellAction::Sticker(sticker) => {
                    ctx.hardware.trigger_haptic();
                    ctx.emit(SendEvent::Sticker(sticker.clone()));
                }
                CellAction::NewSticker => self.picking = Some(ctx.hardware.pick()),
                _ => {}
            }
        }
        prism::events![event]
    }
}

impl std::fmt::Debug for StickerPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StickerPanel").field("grid", &self.grid).finish_non_exhaustive()
    }
}

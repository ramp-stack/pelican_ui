use futures::future::Either;
use prism::events::{DynEvent, ListenCallback, ListenFuture, MouseEvent, MouseState, MouseButton};
use prism::events::{ChildEvent, Event, OnEvent, OnChange, WindowEvent, on_event, on_change};
use prism::drawable::{Area, Drawable, Component, Instruction, Layout, LayoutTree, Rect};
use prism::Context;
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding, ScrollAnchor};
use ptsd::display::Opt;
use ptsd::emitters::{DeferredPress, Press};
use ptsd::events::Press as PressEvent;
use uuid::Uuid;

use std::any::Any;

use crate::theme::Color;
use crate::components::text::Text;
use crate::components::{Rectangle, Sticker, Gif, GifPlayer, Icon, MaskedImage};

mod key;
mod keyboard;
pub use keyboard::CustomKeyboard;
mod panels;
pub use panels::{Panel, Panels};
mod emoji;
pub use emoji::EmojiPanel;
mod sticker;
pub use sticker::StickerPanel;
mod gif;
pub use gif::{GifPanel, GifSearchEvent};

#[derive(Debug, Clone)]
enum CellAction { Emoji(String), Sticker(Sticker), NewSticker, Gif(Gif) }

#[derive(Debug, Clone, PartialEq)]
struct GridView {
    width: f32,
    showing: std::ops::Range<usize>,
    near_end: bool,
}

#[derive(Debug, Clone)]
enum TouchEnd {Tap(Uuid, CellAction)}
impl Event for TouchEnd {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Debug, Clone)]
struct GridLayout {
    bounds: Stack,
    scroll: Column,
    side: f32,
    view: Option<GridView>,
    dirty: bool,
}
impl Layout for GridLayout {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        let width = self.scroll.request_width(children);
        self.bounds.request_width(vec![width])
    }
    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        let width = self.bounds.assign_width(width)[0];
        self.scroll.assign_width(width)
    }
    fn request_height(&mut self, width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        let height = self.scroll.request_height(width, children);
        self.bounds.request_height(width, vec![height])
    }
    fn layout(&mut self, size: (f32, f32)) -> Vec<Area> {
        let viewport = self.bounds.layout(size)[0];
        let mut grid = self.scroll.layout(viewport.size)[0];
        if size.0 <= 100.0 {self.view = None;}
        else {
            let (scrolled, showing) = (-grid.offset.1, viewport.size.1);
            let row = self.side + 12.0;
            let first = ((scrolled - 12.0 - self.side) / row).floor() + 1.0;
            let last = ((scrolled + showing - 12.0) / row).ceil();
            let view = GridView {
                width: size.0, showing: first.max(0.0) as usize..last.max(0.0) as usize,
                near_end: scrolled + showing + 4.0 * row >= grid.size.1,
            };
            if self.view.as_ref() != Some(&view) {
                self.view = Some(view);
                self.dirty = true;
            }
        }
        grid.offset = (grid.offset.0 + viewport.offset.0, grid.offset.1 + viewport.offset.1);
        vec![grid]
    }
}

#[derive(Component, Debug, Clone)]
struct GrowingGrid {
    layout: GridLayout,
    grid: Clip<PanelGrid>,
    #[skip] built: usize,
    #[skip] columns: usize,
    #[skip] press: DeferredPress,
}
impl GrowingGrid {
    fn new(height: f32, columns: usize, owner: Uuid) -> Self {
        Self {
            layout: GridLayout {
                bounds: Stack::new(
                    Offset::Start, Offset::Start, Size::Fill, Size::Static(height), Padding::default(),
                ),
                scroll: Column::new(
                    0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start),
                ),
                side: 0.0, view: None, dirty: false,
            },
            grid: Clip(PanelGrid::empty(12.0, owner)), built: 0, columns,
            press: DeferredPress::default(),
        }
    }
    async fn next(&mut self) -> Either<GridView, ListenCallback> {
        if let Some(view) = &self.layout.view && std::mem::take(&mut self.layout.dirty) {
            return Either::Left(view.clone())
        }
        Either::Right(Drawable::changed(self).await)
    }
    fn set_cells(&mut self, side: f32) {self.layout.side = side;}
    fn grid(&mut self) -> &mut PanelGrid {&mut self.grid.0}
    fn built(&self) -> usize {self.built}
    fn set_height(&mut self, height: Size) {
        self.layout.bounds.3 = height;
        self.layout.view = None;
        self.invalidate();
    }
    fn invalidate(&mut self) {self.layout.dirty = true;}
    fn clear(&mut self) {
        self.layout.scroll = Column::new(
            0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start),
        );
        self.empty();
    }
    fn empty(&mut self) {
        self.grid.0 = PanelGrid::empty(12.0, self.grid.0.2);
        self.built = 0;
        self.press = DeferredPress::default();
        self.layout.view = None;
        self.invalidate();
    }
    fn append(&mut self, mut cells: Vec<PanelCell>) {
        let columns = self.columns;
        let partial = self.built % columns;
        if partial != 0 && let Some(row) = self.grid.0.1.pop() {
            let mut kept: Vec<PanelCell> = row.1.into_iter().take(partial).collect();
            kept.append(&mut cells);
            cells = kept;
            self.built -= partial;
        }
        self.built += cells.len();
        for chunk in cells.chunks(columns) {
            let mut row = chunk.to_vec();
            while row.len() < columns {
                row.push(PanelCell::blank(PanelCell::square(self.layout.side), self.grid.0.2));
            }
            self.grid.0.push_row(row, 12.0);
        }
        // Wait for fresh layout measurements before requesting another batch.
        self.layout.view = None;
        self.invalidate();
    }
}
#[on_event]
impl OnEvent<MouseEvent> for GrowingGrid {
    fn on(&mut self, ctx: &mut Context, mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let events = PanelGrid::pointer(ctx, &mut self.press, mouse);
        for event in &events {
            if let Some(MouseEvent {state: MouseState::Scroll(_, dy), position}) = event.downcast_ref()
                && !self.press.claimed() && (position.is_some() || self.press.travel().is_some())
            {
                self.layout.scroll.adjust_scroll(*dy);
            }
        }
        events
    }
}

#[on_event]
impl OnEvent<PressEvent> for GrowingGrid {
    fn on(&mut self, _ctx: &mut Context, event: PressEvent) -> Vec<Box<dyn DynEvent>> {
        self.press.press(event)
    }
}

#[on_event]
impl OnEvent<WindowEvent> for GrowingGrid {
    fn on(&mut self, _ctx: &mut Context, event: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, WindowEvent::Focused(false)) && let Some(cancel) = self.press.cancel() {
            return prism::events![cancel, event];
        }
        prism::events![event]
    }
}

#[on_change(self.press.next())]
impl OnChange<PressEvent> for GrowingGrid {
    fn on_change(&mut self, ctx: &mut Context, event: PressEvent) {ctx.emit(event);}
}

#[derive(Debug, Clone)]
struct Clip<D: Drawable + Clone>(D);

impl<D: Drawable + Clone> Clip<D> {
    fn overlap(a: Rect, b: Rect) -> Option<Rect> {
        let (x0, y0) = (a.0.max(b.0), a.1.max(b.1));
        let (x1, y1) = ((a.0 + a.2).min(b.0 + b.2), (a.1 + a.3).min(b.1 + b.3));
        (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
    }
}

impl<D: Drawable + Clone + 'static> Drawable for Clip<D> {
    fn name(&self) -> String { self.0.name() }
    fn request_width(&mut self) -> (f32, f32) {self.0.request_width()}
    fn request_height(&mut self, width: f32) -> (f32, f32) {self.0.request_height(width)}

    fn layout(&mut self, size: (f32, f32)) -> LayoutTree {
        self.0.layout(size)
    }

    fn draw(&self, sized: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let Some(edges) = Self::overlap(bound, (offset.0, offset.1, sized.0.0, sized.0.1))
        else { return Vec::new(); };
        self.0.draw(sized, offset, edges).into_iter().filter_map(|mut instruction| {
            instruction.bound = Self::overlap(instruction.bound, edges)?;
            Some(instruction)
        }).collect()
    }

    fn event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) {
        self.0.event(ctx, sized, event)
    }

    fn changed(&mut self) -> ListenFuture<'_> {
        Box::pin(async move {
            let callback = Drawable::changed(&mut self.0).await;
            Box::new(move |target: &mut dyn Any, ctx: &mut Context| {
                callback(&mut target.downcast_mut::<Self>().unwrap().0, ctx);
            }) as ListenCallback
        })
    }
}

#[derive(Component, Debug, Clone)]
struct PanelGrid(
    Column, Vec<PanelRow>, #[skip] Uuid,
);

impl PanelGrid {
    fn cell_side(width: f32, columns: usize) -> f32 {
        let gaps = 2.0 * 12.0 + (columns - 1) as f32 * 12.0;
        ((width - gaps) / columns as f32).floor().max(1.0)
    }

    fn tray_height(width: f32) -> f32 { 12.0 + 2.0 * (Self::cell_side(width, 3) + 12.0) + 48.0 }

    fn empty(spacing: f32, owner: Uuid) -> Self {
        PanelGrid(
            Column::new(spacing, Offset::Start, Size::Fill, Padding::new(12.0), None),
            Vec::new(), owner,
        )
    }

    fn push_row(&mut self, cells: Vec<PanelCell>, spacing: f32) {
        self.1.push(PanelRow(
            Row::new(spacing, Offset::Start, Size::Fit, Padding::new(0.0)), cells,
        ));
    }

    fn pointer(ctx: &mut Context, press: &mut DeferredPress, mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let events = if !ctx.window.is_mobile && mouse.position.is_some()
            && mouse.state == MouseState::Pressed(MouseButton::Left)
        {
            press.press(PressEvent::Pending {id: Uuid::new_v4(), position: mouse.position})
        } else {press.mouse(ctx.window.is_mobile, mouse)};
        for event in &events {
            if let Some(PressEvent::Cancel(id)) = event.downcast_ref::<PressEvent>() {
                ctx.emit(PressEvent::Cancel(*id));
            }
        }
        events
    }
}

#[derive(Component, Debug, Clone)]
struct PanelRow(Row, Vec<PanelCell>);

#[derive(Component, Debug, Clone)]
enum CellFace {
    Emoji { layout: Stack, text: Text },
    Sticker { layout: Stack, image: MaskedImage, outline: Opt<Rectangle> },
    NewSticker { layout: Stack, background: Rectangle, icon: Icon, outline: Opt<Rectangle> },
    Gif { layout: Stack, player: GifPlayer, outline: Opt<Rectangle> },
}

#[derive(Component, Debug, Clone)]
struct PanelCell(
    Stack, Opt<Rectangle>, Option<CellFace>, #[skip] Option<CellAction>, #[skip] Press, #[skip] Uuid,
);

impl PanelCell {
    fn square(side: f32) -> Stack {
        Stack::new(
            Offset::Center, Offset::Center, Size::Static(side), Size::Static(side),
            Padding::default(),
        )
    }

    fn highlight(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::from_hex("ffffff", 40), radius, None), false)
    }

    fn outline(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::transparent(), radius, Some((1.0, Color::white()))), false)
    }

    fn press(&mut self, on: bool) {
        match &mut self.2 {
            Some(CellFace::Sticker { outline, .. })
            | Some(CellFace::NewSticker { outline, .. })
            | Some(CellFace::Gif { outline, .. }) => outline.display(on), _ => self.1.display(on),
        }
    }

    fn activate(&mut self, ctx: &mut Context) {
        if let Some(action) = self.3.clone() {
            self.press(true);
            ctx.emit(TouchEnd::Tap(self.5, action));
        }
    }

    fn blank(layout: Stack, owner: Uuid) -> Self {
        PanelCell(layout, Self::highlight(0.0), None, None, Press::default(), owner)
    }
}

#[on_event]
impl OnEvent<MouseEvent> for PanelCell {
    fn on(&mut self, ctx: &mut Context, event: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        match event.state {
            MouseState::Pressed(MouseButton::Left) => {
                self.4.clear();
                if event.position.is_some() {self.activate(ctx);}
            }
            MouseState::Released(_) => {
                self.4.clear();
                self.press(false);
            }
            _ => {}
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<PressEvent> for PanelCell {
    fn on(&mut self, ctx: &mut Context, event: PressEvent) -> Vec<Box<dyn DynEvent>> {
        match event {
            PressEvent::Pending {id, position} => {
                let position = position.filter(|_| self.3.is_some());
                self.4.pending(id, position);
                self.press(position.is_some());
                if !ctx.window.is_mobile && matches!(self.3, Some(CellAction::Emoji(_)))
                    && self.4.commit(id).is_some()
                {
                    self.activate(ctx);
                }
            }
            PressEvent::Commit(id) => if self.4.commit(id).is_some() {
                self.activate(ctx);
                ctx.emit(PressEvent::Claim(id));
            },
            PressEvent::Cancel(id) => if self.4.cancel(id) {self.press(false);},
            PressEvent::Claim(_) => {}
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<WindowEvent> for PanelCell {
    fn on(&mut self, _ctx: &mut Context, event: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, WindowEvent::Focused(false)) {
            self.4.clear();
            self.press(false);
        }
        prism::events![event]
    }
}

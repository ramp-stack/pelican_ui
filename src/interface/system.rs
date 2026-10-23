use prism::events::{DynEvent, ListenFuture, MouseEvent, MouseState, MouseButton};
use prism::drawable::{Drawable, Component, Instruction, LayoutTree, RangeTree, Rect};
use prism::Context;
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding, ScrollAnchor};
use ptsd::display::{Bin, Opt};

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

enum TouchEnd { Tap(CellAction), Turn(bool) }

#[derive(Debug, Clone, PartialEq)]
struct GridView {
    width: f32,
    showing: std::ops::Range<usize>,
    near_end: bool,
}

struct GridPlace<'a> {
    view: ((f32, f32), (f32, f32)),
    offset: (f32, f32),
    grid: &'a LayoutTree,
}

impl<'a> GridPlace<'a> {
    fn overlap(a: Rect, b: Rect) -> Option<Rect> {
        let (x0, y0) = (a.0.max(b.0), a.1.max(b.1));
        let (x1, y1) = ((a.0 + a.2).min(b.0 + b.2), (a.1 + a.3).min(b.1 + b.3));
        (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
    }

    fn find(page: &'a LayoutTree, path: &[usize]) -> Option<Self> {
        let (&first, rest) = path.split_first()?;
        let (view_offset, view) = page.1.get(first)?;
        let mut offset = *view_offset;
        let mut grid = view;
        for &i in rest {
            let (o, child) = grid.1.get(i)?;
            offset = (offset.0 + o.0, offset.1 + o.1);
            grid = child;
        }
        Some(GridPlace { view: (*view_offset, view.0), offset, grid })
    }

    fn contains(&self, p: (f32, f32)) -> bool {
        let ((x, y), (w, h)) = self.view;
        p.0 >= x && p.0 < x + w && p.1 >= y && p.1 < y + h
    }
}

#[derive(Debug, Clone)]
struct GrowingGrid {
    scroll: Bin<Stack, Clip<GridScroll>>,
    built: usize,
    columns: usize,
    side: f32,
    pressed: Option<bool>,
    view: Option<GridView>,
    dirty: bool,
}

impl GrowingGrid {
    fn new(height: f32, columns: usize) -> Self {
        Self {
            scroll: Bin(Stack(
                Offset::Start, Offset::Start, Size::Fill, Size::Static(height), Padding::default(),
            ), Clip(GridScroll::new())), built: 0, columns, side: 0.0, pressed: None,
            view: None, dirty: false,
        }
    }

    async fn next(&mut self) -> (Vec<usize>, Box<dyn Any>) {
        if let Some(view) = &self.view && std::mem::take(&mut self.dirty) {
            return (Vec::new(), Box::new(view.clone()))
        }
        let (mut path, update) = Drawable::listen(&mut self.scroll).await;
        path.push(0);
        (path, update)
    }

    fn set_cells(&mut self, side: f32) { self.side = side; }

    fn grid(&mut self) -> &mut PanelGrid { &mut self.scroll.1.0.1 }

    fn built(&self) -> usize { self.built }

    fn set_height(&mut self, height: Size) {
        self.scroll.0.3 = height;
        self.view = None;
        self.invalidate();
    }

    fn invalidate(&mut self) { self.dirty = true; }

    fn clear(&mut self) {
        self.scroll.1 = Clip(GridScroll::new());
        self.built = 0;
        self.view = None;
        self.invalidate();
    }

    fn empty(&mut self) {
        self.scroll.1.0.1 = PanelGrid::empty(12.0);
        self.built = 0;
        self.view = None;
        self.invalidate();
    }

    fn append(&mut self, mut cells: Vec<PanelCell>) {
        let columns = self.columns;
        let partial = self.built % columns;
        if partial != 0 && let Some(row) = self.scroll.1.0.1.1.pop() {
            let mut kept: Vec<PanelCell> = row.1.into_iter().take(partial).collect();
            kept.append(&mut cells);
            cells = kept;
            self.built -= partial;
        }
        self.built += cells.len();
        for chunk in cells.chunks(columns) {
            let mut row = chunk.to_vec();
            while row.len() < columns { row.push(PanelCell::blank(PanelCell::square(self.side))); }
            self.scroll.1.0.1.push_row(row, 12.0);
        }
        // Wait for fresh layout measurements before requesting another batch.
        self.view = None;
        self.invalidate();
    }
}

impl Drawable for GrowingGrid {
    fn listen(&mut self) -> ListenFuture<'_> {Drawable::listen(&mut self.scroll)}
    fn update(&mut self, ctx: &mut Context, path: Vec<usize>, update: Box<dyn Any>) {
        Drawable::update(&mut self.scroll, ctx, path, update);
    }

    fn size_range(&self) -> RangeTree { self.scroll.size_range() }

    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        let sized = Drawable::layout(&mut self.scroll, size, range);
        if sized.0.0 <= 100.0 {
            self.view = None;
            return sized;
        }
        let Some((_, scroll)) = sized.1.first() else { return sized };
        let Some((offset, grid)) = scroll.1.first() else { return sized };
        let (scrolled, showing) = (-offset.1, scroll.0.1);
        let row = self.side + 12.0;
        let first = ((scrolled - 12.0 - self.side) / row).floor() + 1.0;
        let last = ((scrolled + showing - 12.0) / row).ceil();
        let view = GridView {
            width: sized.0.0, showing: first.max(0.0) as usize..last.max(0.0) as usize,
            near_end: scrolled + showing + 4.0 * row >= grid.0.1,
        };
        if self.view.as_ref() != Some(&view) {
            self.view = Some(view);
            self.invalidate();
        }
        sized
    }

    fn draw(&self, sized: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.scroll.draw(sized, offset, bound)
    }

    fn event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) {
        if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) => {
                    self.pressed = Some(mouse.position.is_some());
                }
                MouseState::Released(_) => self.pressed = None,
                MouseState::Scroll(_, dy) if self.pressed.unwrap_or(mouse.position.is_some()) => {
                    self.scroll.1.0.0.adjust_scroll(dy);
                }
                _ => {}
            }
        }
        self.scroll.event(ctx, sized, event);
    }
}

#[derive(Debug, Clone)]
struct Clip<D: Drawable + Clone>(D);

impl<D: Drawable + Clone + 'static> Drawable for Clip<D> {
    fn listen(&mut self) -> ListenFuture<'_> {Drawable::listen(&mut self.0)}
    fn update(&mut self, ctx: &mut Context, path: Vec<usize>, update: Box<dyn Any>) {
        Drawable::update(&mut self.0, ctx, path, update);
    }
    fn size_range(&self) -> RangeTree { self.0.size_range() }

    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        self.0.layout(size, range)
    }

    fn draw(&self, sized: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let Some(edges) = GridPlace::overlap(bound, (offset.0, offset.1, sized.0.0, sized.0.1))
        else { return Vec::new(); };
        self.0.draw(sized, offset, edges).into_iter().filter_map(|mut instruction| {
            instruction.bound = GridPlace::overlap(instruction.bound, edges)?;
            Some(instruction)
        }).collect()
    }

    fn name(&self) -> String { self.0.name() }

    fn event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) {
        self.0.event(ctx, sized, event)
    }
}

#[derive(Component, Debug, Clone)]
struct GridScroll(Column, PanelGrid);

impl GridScroll {
    fn new() -> Self {
        GridScroll(
            Column::new(
                0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start),
            ), PanelGrid::empty(12.0),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct PanelTouch {
    cell: Option<(usize, usize)>,
    last: (f32, f32),
    travel: (f32, f32),
}

impl PanelTouch {
    fn track(touch: &mut Option<PanelTouch>, grid: &mut PanelGrid, place: Option<GridPlace>, mouse: &MouseEvent) -> Option<TouchEnd> {
        let mut end = None;
        match mouse.state {
            MouseState::Pressed(MouseButton::Left) => {
                if let Some(old) = touch.take() { grid.highlight(old.cell, false); }
                *touch = mouse.position.zip(place).filter(|(p, place)| place.contains(*p))
                    .map(|(p, place)| {
                        let local = (p.0 - place.offset.0, p.1 - place.offset.1);
                        let cell = place.grid.1.iter().enumerate()
                            .find_map(|(r, (row_offset, row))| {
                                let p = (local.0 - row_offset.0, local.1 - row_offset.1);
                                row.1.iter().position(|(o, cell)| {
                                    p.0 >= o.0 && p.0 < o.0 + cell.0.0
                                        && p.1 >= o.1 && p.1 < o.1 + cell.0.1
                                }).map(|c| (r, c))
                            }).filter(|c| grid.cell(*c).is_some_and(|cell| cell.3.is_some()));
                        PanelTouch { cell, last: p, travel: (0.0, 0.0) }
                    });
                if let Some(t) = *touch {grid.highlight(t.cell, true);}
            }
            MouseState::Scroll(dx, dy) => {
                if let Some(t) = touch {
                    t.travel.0 -= dx;
                    t.travel.1 -= dy;
                }
            }
            MouseState::Moved => {
                if let (Some(t), Some(p)) = (touch.as_mut(), mouse.position) {
                    t.travel.0 += p.0 - t.last.0;
                    t.travel.1 += p.1 - t.last.1;
                    t.last = p;
                }
            }
            MouseState::Released(_) => {
                if let Some(t) = touch.take() {
                    grid.highlight(t.cell, false);
                    let (dx, dy) = t.travel;
                    if dx.abs() >= 40.0 && dx.abs() > dy.abs() {
                        end = Some(TouchEnd::Turn(dx < 0.0));
                    } else if dx.hypot(dy) < 12.0
                        && let Some(action) =
                            t.cell.and_then(|c| grid.cell(c)).and_then(|cell| cell.3.clone())
                    {
                        end = Some(TouchEnd::Tap(action));
                    }
                }
            }
            _ => {}
        }

        if let Some(t) = *touch && t.travel.0.hypot(t.travel.1) >= 12.0 {
            grid.highlight(t.cell, false);
        }
        end
    }
}

#[derive(Component, Debug, Clone)]
struct PanelGrid(Column, Vec<PanelRow>);

impl PanelGrid {
    fn cell_side(width: f32, columns: usize) -> f32 {
        let gaps = 2.0 * 12.0 + (columns - 1) as f32 * 12.0;
        ((width - gaps) / columns as f32).floor().max(1.0)
    }

    fn tray_height(width: f32) -> f32 { 12.0 + 2.0 * (Self::cell_side(width, 3) + 12.0) + 48.0 }

    fn new(rows: usize, columns: usize, layout: Stack, spacing: f32, mut cell: impl FnMut(usize) -> Option<PanelCell>) -> Self {
        let mut grid = Self::empty(spacing);
        for r in 0..rows {
            let cells = (0..columns)
                .map(|c| cell(r * columns + c).unwrap_or_else(|| PanelCell::blank(layout.clone())))
                .collect();
            grid.push_row(cells, spacing);
        }
        grid
    }

    fn empty(spacing: f32) -> Self {
        PanelGrid(
            Column::new(spacing, Offset::Start, Size::Fill, Padding::new(12.0), None), Vec::new(),
        )
    }

    fn push_row(&mut self, cells: Vec<PanelCell>, spacing: f32) {
        self.1.push(PanelRow(
            Row::new(spacing, Offset::Start, Size::Fit, Padding::new(0.0)), cells,
        ));
    }

    fn cell(&mut self, (row, col): (usize, usize)) -> Option<&mut PanelCell> {
        self.1.get_mut(row)?.1.get_mut(col)
    }

    fn highlight(&mut self, cell: Option<(usize, usize)>, on: bool) {
        if let Some(cell) = cell.and_then(|c| self.cell(c)) { cell.press(on); }
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
struct PanelCell(Stack, Opt<Rectangle>, Option<CellFace>, #[skip] Option<CellAction>);

impl PanelCell {
    fn layout(width: f32, height: f32) -> Stack {
        let width = Size::custom(move |_: Vec<(f32, f32)>| (width, f32::MAX));
        Stack(
            Offset::Center, Offset::Center, width, Size::Static(height),
            Padding(2.0, 2.0, 2.0, 2.0),
        )
    }

    fn square(side: f32) -> Stack {
        Stack(
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

    fn blank(layout: Stack) -> Self { PanelCell(layout, Self::highlight(0.0), None, None) }
}

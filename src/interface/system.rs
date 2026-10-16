//! The on-screen keyboard, and its emoji, sticker and GIF pages.
//!
//! - `keyboard`: the keyboard itself (letter keys, header, the pages it shows
//!   them in), and the pages on a computer under the message bar's toolbar.
//! - `emoji`, `sticker`, `gif`: the pages.
//! - Here: the grid of cells the pages share, and taps and swipes on it.

mod keyboard;
mod emoji;
mod sticker;
mod gif;

pub use keyboard::{KeyboardPages, Panels, KeyboardFullScreen, MobileKeyboard};
pub(crate) use keyboard::{TEXT_FIELD_GROUP, KeyboardShowing, KeyboardPage, MobileKeyboardEvent};

use prism::event::{MouseEvent, OnEvent, Event, MouseState, MouseButton};
use prism::canvas::{Image, Instruction};
use prism::Context;
use prism::drawable::{Drawable, Component, SizedTree, RequestTree, Rect};
use prism::layout::{Stack, Column, Row, Offset, Size, Padding, ScrollAnchor};
use prism::display::Opt;

use crate::theme::Color;
use crate::components::text::Text;
use crate::components::{Rectangle, Sticker, Gif, GifPlayer};

const ASSUMED_WIDTH: f32 = 374.0;

/// The side of a square cell, `columns` to a row filling `width`: the width
/// less the grid's padding at the sides and spacing between, shared out.
fn cell_side(width: f32, columns: usize) -> f32 {
    let gaps = 2.0 * 12.0 + (columns - 1) as f32 * 12.0;
    ((width - gaps) / columns as f32).floor().max(1.0)
}

/// The height of the sticker and GIF pages, `width` wide.
fn tray_height(width: f32) -> f32 {
    12.0 + 2.0 * (cell_side(width, 3) + 12.0) + 48.0
}

/// A page's width from its sized tree, once it's been laid out.
fn laid_out_width(sized: &SizedTree) -> Option<f32> {
    // Less than a row of cells means it hasn't been yet.
    (sized.0.0 > 100.0).then_some(sized.0.0)
}

/// Sideways travel that turns the page.
const SWIPE_DISTANCE: f32 = 40.0;

/// A grid in a column that scrolls up and down, built a few rows at a time:
/// more are added as you scroll near the bottom, so it seems to go on for
/// ever but only holds what's been loaded so far.
///
/// Fields after the scrolling grid (cut off at its edges): the cells built,
/// the columns, the side of a (square) cell, and, while a finger is down,
/// whether it went down on the grid.
#[derive(Component, Debug, Clone)]
struct GrowingGrid(Stack, Clip<GridScroll>, #[skip] usize, #[skip] usize, #[skip] f32, #[skip] Option<bool>);

impl GrowingGrid {
    /// `height` is how much of it shows at once.
    fn new(height: f32, columns: usize) -> Self {
        GrowingGrid(
            Stack(Offset::Start, Offset::Start, Size::Fill, Size::Static(height), Padding::default()),
            Clip(GridScroll::new()),
            0, columns, 0.0, None,
        )
    }

    /// The side of the cells it's given (for the blank ones it adds, and to
    /// know how tall a row is).
    fn set_cells(&mut self, side: f32) { self.4 = side; }

    fn grid(&mut self) -> &mut PanelGrid { &mut self.1.0.1 }

    fn built(&self) -> usize { self.2 }

    /// How much of it shows at once.
    fn set_height(&mut self, height: Size) { self.0.3 = height; }

    /// Empties it and scrolls back to the top.
    fn clear(&mut self) {
        self.1 = Clip(GridScroll::new());
        self.2 = 0;
    }

    /// Empties it, keeping how far it's scrolled for the cells built next.
    fn empty(&mut self) {
        self.1.0.1 = PanelGrid::empty(12.0);
        self.2 = 0;
    }

    /// Adds cells after the ones built, filling out the last row first.
    fn append(&mut self, mut cells: Vec<PanelCell>) {
        let columns = self.3;
        let partial = self.2 % columns;
        if partial != 0 && let Some(row) = self.1.0.1.1.pop() {
            let mut kept: Vec<PanelCell> = row.1.into_iter().take(partial).collect();
            kept.append(&mut cells);
            cells = kept;
            self.2 -= partial;
        }
        self.2 += cells.len();
        for chunk in cells.chunks(columns) {
            let mut row = chunk.to_vec();
            while row.len() < columns { row.push(PanelCell::blank(PanelCell::square(self.4))); }
            self.1.0.1.push_row(row, 12.0);
        }
    }

    /// From its sized tree: how far it's scrolled, how much shows, and how
    /// tall the grid is.
    fn measure(sized: &SizedTree) -> Option<(f32, f32, f32)> {
        let (_, scroll) = sized.1.first()?;
        let (grid_offset, grid) = scroll.1.first()?;
        Some((-grid_offset.1, scroll.0.1, grid.0.1))
    }

    /// Whether the bottom of what's built is within a few rows of showing.
    fn near_end(&self, sized: &SizedTree) -> bool {
        Self::measure(sized).is_none_or(|(scrolled, showing, height)| {
            scrolled + showing + 4.0 * (self.4 + 12.0) >= height
        })
    }

    /// The rows that are (at least partly) showing.
    fn showing_rows(&self, sized: &SizedTree) -> std::ops::Range<usize> {
        let Some((scrolled, showing, _)) = Self::measure(sized) else { return 0..0 };
        let row = self.4 + 12.0;
        let first = ((scrolled - 12.0 - self.4) / row).floor() + 1.0;
        let last = ((scrolled + showing - 12.0) / row).ceil();
        first.max(0.0) as usize..last.max(0.0) as usize
    }
}

impl OnEvent for GrowingGrid {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) => self.5 = Some(mouse.position.is_some()),
                MouseState::Released(_) => self.5 = None,
                // A drag scrolls it if it started on it, even once the finger
                // moves off it. A mouse wheel scrolls it while over it.
                MouseState::Scroll(_, dy) if self.5.unwrap_or(mouse.position.is_some()) => self.1.0.0.adjust_scroll(dy),
                _ => {}
            }
        }
        vec![event]
    }
}

/// Draws what's in it cut off at its own edges, top, bottom and sides.
///
/// prism cuts a child off where its parent starts but not where it ends
/// (it gives the child the parent's height from the child's own top), so
/// without this the rows of a scrolled grid show below it.
#[derive(Debug, Clone)]
struct Clip<D: Drawable + Clone>(D);

impl<D: Drawable + Clone + 'static> Drawable for Clip<D> {
    fn request_size(&self) -> RequestTree { self.0.request_size() }

    fn build(&self, size: (f32, f32), request: &RequestTree) -> SizedTree { self.0.build(size, request) }

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let Some(edges) = overlap(bound, (offset.0, offset.1, sized.0.0, sized.0.1)) else { return Vec::new() };
        // Anything entirely outside isn't drawn at all.
        self.0.draw(sized, offset, edges).into_iter().filter_map(|Instruction(mut area, item)| {
            area.bounds = Some(overlap(area.bounds.unwrap_or(edges), edges)?);
            Some(Instruction(area, item))
        }).collect()
    }

    fn name(&self) -> String { self.0.name() }

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) { self.0.event(ctx, sized, event) }
}

/// Where two rectangles (x, y, width, height) overlap, if they do.
fn overlap(a: Rect, b: Rect) -> Option<Rect> {
    let (x0, y0) = (a.0.max(b.0), a.1.max(b.1));
    let (x1, y1) = ((a.0 + a.2).min(b.0 + b.2), (a.1 + a.3).min(b.1 + b.3));
    (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
}

/// The grid, in a column that scrolls (the [`GrowingGrid`] scrolls it).
#[derive(Component, Debug, Clone)]
struct GridScroll(Column, PanelGrid);
impl OnEvent for GridScroll {}

impl GridScroll {
    fn new() -> Self {
        GridScroll(Column::new(0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start)), PanelGrid::empty(12.0))
    }
}

/// A press on a page's grid: the cell it started on, and how far it has moved.
#[derive(Debug, Clone, Copy)]
struct PanelTouch { cell: Option<(usize, usize)>, last: (f32, f32), travel: (f32, f32) }

/// How a press on a page's grid ended.
enum TouchEnd {
    /// A tap on a cell.
    Tap(CellAction),
    /// A sideways swipe: `true` for the next page, `false` for the previous.
    Turn(bool),
}

/// Where a page's grid is: the area presses count in (the part showing), and
/// the grid's own position (which moves as it scrolls) and sized tree.
struct GridPlace<'a> {
    view: ((f32, f32), (f32, f32)),
    offset: (f32, f32),
    grid: &'a SizedTree,
}

impl<'a> GridPlace<'a> {
    /// `path` leads from the page to its grid, the child to take at each
    /// step: `[0]` when the grid is the page's first child, or on through a
    /// [`GrowingGrid`] and its scrolling column (`[.., 0, 0]`). Presses count
    /// in the first step's area.
    fn find(page: &'a SizedTree, path: &[usize]) -> Option<Self> {
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

    /// The cell under `p`, in the page's coordinates.
    fn cell_at(&self, p: (f32, f32)) -> Option<(usize, usize)> {
        let contains = |offset: (f32, f32), size: (f32, f32), p: (f32, f32)|
            p.0 >= offset.0 && p.0 < offset.0 + size.0 && p.1 >= offset.1 && p.1 < offset.1 + size.1;
        let p = (p.0 - self.offset.0, p.1 - self.offset.1);
        self.grid.1.iter().enumerate().find_map(|(r, (row_offset, row))| {
            let p = (p.0 - row_offset.0, p.1 - row_offset.1);
            row.1.iter().position(|(cell_offset, cell)| contains(*cell_offset, cell.0, p)).map(|c| (r, c))
        })
    }
}

/// Follows a press on a page's grid, highlighting the cell under it. Taps
/// and swipes are handled here rather than by buttons on each cell: a tap
/// tolerates a little finger movement, and a drag (to scroll, or turn the
/// page) never taps.
fn track_touch(touch: &mut Option<PanelTouch>, grid: &mut PanelGrid, place: Option<GridPlace>, mouse: &MouseEvent) -> Option<TouchEnd> {
    let mut end = None;
    match mouse.state {
        MouseState::Pressed(MouseButton::Left) => {
            // One whose release never came (or came first) lets go.
            if let Some(old) = touch.take() { grid.highlight(old.cell, false); }
            *touch = mouse.position.zip(place).filter(|(p, place)| place.contains(*p)).map(|(p, place)| {
                let cell = place.cell_at(p).filter(|c| grid.cell(*c).is_some_and(|cell| cell.3.is_some()));
                PanelTouch { cell, last: p, travel: (0.0, 0.0) }
            });
            if let Some(t) = *touch { grid.highlight(t.cell, true); }
        }
        // Touch drags arrive as scroll deltas (the negated finger movement),
        // and keep arriving after the finger leaves the page.
        MouseState::Scroll(dx, dy) => if let Some(t) = touch {
            t.travel.0 -= dx;
            t.travel.1 -= dy;
        },
        // A mouse drags with positions instead.
        MouseState::Moved => if let (Some(t), Some(p)) = (touch.as_mut(), mouse.position) {
            t.travel.0 += p.0 - t.last.0;
            t.travel.1 += p.1 - t.last.1;
            t.last = p;
        },
        MouseState::Released(_) => if let Some(t) = touch.take() {
            grid.highlight(t.cell, false);
            let (dx, dy) = t.travel;
            if dx.abs() >= SWIPE_DISTANCE && dx.abs() > dy.abs() {
                // Finger moving left shows the next page, right the previous one.
                end = Some(TouchEnd::Turn(dx < 0.0));
            } else if dx.hypot(dy) < 12.0
                && let Some(action) = t.cell.and_then(|c| grid.cell(c)).and_then(|cell| cell.3.clone())
            {
                end = Some(TouchEnd::Tap(action));
            }
        },
        _ => {}
    }

    // Once a press has moved too far to be a tap, stop highlighting its cell.
    if let Some(t) = *touch && t.travel.0.hypot(t.travel.1) >= 12.0 {
        grid.highlight(t.cell, false);
    }
    end
}

/// Rows of cells: a page of emoji in 4 rows of 8, or the stickers or GIFs
/// built so far, 4 or 3 to a row. Always full rows (blank cells past the
/// end), so the columns line up.
#[derive(Component, Debug, Clone)]
struct PanelGrid(Column, Vec<PanelRow>);
impl OnEvent for PanelGrid {}

impl PanelGrid {
    /// `cell(i)` gives the i-th cell on the page; `None` leaves it blank.
    fn new(rows: usize, columns: usize, layout: Stack, spacing: f32, mut cell: impl FnMut(usize) -> Option<PanelCell>) -> Self {
        let mut grid = Self::empty(spacing);
        for r in 0..rows {
            let cells = (0..columns).map(|c| {
                cell(r * columns + c).unwrap_or_else(|| PanelCell::blank(layout.clone()))
            }).collect();
            grid.push_row(cells, spacing);
        }
        grid
    }

    fn empty(spacing: f32) -> Self {
        PanelGrid(Column::new(spacing, Offset::Start, Size::Fill, Padding::new(12.0), None), Vec::new())
    }

    fn push_row(&mut self, cells: Vec<PanelCell>, spacing: f32) {
        self.1.push(PanelRow(Row::new(spacing, Offset::Start, Size::Fit, Padding::new(0.0)), cells));
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
impl OnEvent for PanelRow {}

/// What tapping a cell does.
#[derive(Debug, Clone)]
enum CellAction {
    /// Types the emoji.
    Emoji(String),
    /// Sends the sticker.
    Sticker(Sticker),
    /// Picks a photo to make a new sticker from.
    NewSticker,
    /// Sends the GIF.
    Gif(Gif),
}

/// One cell: a highlight shown behind an emoji while it's pressed, what's in
/// it, and what tapping it does (blank cells have neither).
#[derive(Component, Debug, Clone)]
struct PanelCell(Stack, Opt<Rectangle>, Option<CellFace>, #[skip] Option<CellAction>);
impl OnEvent for PanelCell {}

#[derive(Component, Debug, Clone)]
enum CellFace {
    Emoji {layout: Stack, text: Text},
    /// The outline is drawn over the sticker or GIF while it's pressed.
    Sticker {layout: Stack, image: Image, outline: Opt<Rectangle>},
    NewSticker {layout: Stack, background: Rectangle, icon: Image, outline: Opt<Rectangle>},
    Gif {layout: Stack, player: GifPlayer, outline: Opt<Rectangle>},
}
impl OnEvent for CellFace {}

impl PanelCell {
    /// Every cell in a grid asks for the same width, so the columns get equal space.
    fn layout(width: f32, height: f32) -> Stack {
        let width = Size::custom(move |_: Vec<(f32, f32)>| (width, f32::MAX));
        Stack(Offset::Center, Offset::Center, width, Size::Static(height), Padding(2.0, 2.0, 2.0, 2.0))
    }

    /// A sticker or GIF cell: a square box. The outline fills it exactly, so
    /// it never grows.
    fn square(side: f32) -> Stack {
        Stack(Offset::Center, Offset::Center, Size::Static(side), Size::Static(side), Padding::default())
    }

    /// Shown behind an emoji while it's pressed.
    fn highlight(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::from_hex("ffffff", 110), radius, None), false)
    }

    /// Drawn over a sticker or GIF while it's pressed (inside its edge).
    fn outline(radius: f32) -> Opt<Rectangle> {
        Opt::new(Rectangle::new(Color::TRANSPARENT, radius, Some((1.0, Color::WHITE))), false)
    }

    /// Shows the cell as pressed: an outline over a sticker or GIF, or a
    /// highlight behind an emoji.
    fn press(&mut self, on: bool) {
        match &mut self.2 {
            Some(CellFace::Sticker {outline, ..}) |
            Some(CellFace::NewSticker {outline, ..}) |
            Some(CellFace::Gif {outline, ..}) => outline.display(on),
            _ => self.1.display(on),
        }
    }

    fn blank(layout: Stack) -> Self {
        PanelCell(layout, Self::highlight(0.0), None, None)
    }
}
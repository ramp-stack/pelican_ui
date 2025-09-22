use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Layout, Area, SizeRequest};
use pelican_ui::{Context, Component};

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum Offset {
    #[default]
    Start,
    Center,
    End,
    Static(f32)
}

impl Offset {
    pub fn get(&self, max_size: f32, item_size: f32) -> f32 {
        match self {
            Self::Start => 0.0,
            Self::Center => (max_size - item_size) / 2.0,
            Self::End => max_size - item_size,
            Self::Static(offset) => *offset,
        }
    }

    pub fn size(&self) -> Option<f32> {
        match self {
            Self::Start => Some(0.0),
            Self::Center | Self::End => None,
            Self::Static(offset) => Some(*offset),
        }
    }
}

type CustomFunc = dyn Fn(Vec<(f32, f32)>) -> (f32, f32);
type FitFunc = fn(Vec<(f32, f32)>) -> (f32, f32);

/// Enum specifying how a layout should size and resize its content.
#[derive(Default)]
pub enum Size {
    #[default]
    /// Layout automatically fits the size of its children.
    Fit,
    /// Layout fills the available space but remains between the specified minimum and maximum.
    /// 
    /// # Parameters
    /// - `min` (`f32`): Minimum size.
    /// - `max` (`f32`): Maximum size.
    Fill(f32, f32),
    /// Layout uses a fixed, static size.
    ///
    /// # Parameters
    /// - `size` (`f32`): The exact size to use.
    Static(f32),
    /// Layout size is determined by a custom function.
    ///
    /// # Parameters
    /// - `func` (`Box<CustomFunc>`): Function that returns the desired size based on available space.
    Custom(Box<CustomFunc>),
}


impl Size {
    pub fn fill() -> Self {Size::Fill(0.0, f32::MAX)}
    pub fn custom(func: impl Fn(Vec<(f32, f32)>) -> (f32, f32) + 'static) -> Self {
        Size::Custom(Box::new(func))
    }

    pub fn get(&self, items: Vec<(f32, f32)>, fit: FitFunc) -> (f32, f32) {
        match self {
            Size::Fit => fit(items),
            Size::Fill(min, max) => (*min, *max),
            Size::Static(s) => (*s, *s),
            Size::Custom(f) => f(items)
        }
    }

    pub fn max(items: Vec<(f32, f32)>) -> (f32, f32) {
        items.into_iter().reduce(|s, i| (s.0.max(i.0), s.1.max(i.1))).unwrap_or_default()
    }

    pub fn add(items: Vec<(f32, f32)>) -> (f32, f32) {
        items.into_iter().reduce(|s, i| (s.0+i.0, s.1+i.1)).unwrap_or_default()
    }
}

impl std::fmt::Debug for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Size::Fit => write!(f, "Size::Fit"),
            Size::Fill(min, max) => write!(f, "Size::Fill(min: {min}, max: {max})"),
            Size::Static(val) => write!(f, "Size::Static({val})"),
            Size::Custom(_) => write!(f, "Size::Custom(<function>)"),
        }
    }
}

/// Structure used to define top, left, bottom, and right padding of an UI element.
///```rust
/// let padding = Padding(24.0, 16.0, 24.0, 16.0);
///```
#[derive(Clone, Debug, Default)]
pub struct Padding(pub f32, pub f32, pub f32, pub f32);

impl Padding {
    pub fn new(p: f32) -> Self {Padding(p, p, p, p)}

    pub fn adjust_size(&self, size: (f32, f32)) -> (f32, f32) {
        let wp = self.0+self.2;
        let hp = self.1+self.3;
        (size.0-wp, size.1-hp)
    }

    pub fn adjust_offset(&self, offset: (f32, f32)) -> (f32, f32) {
        (offset.0+self.0, offset.1+self.1)
    }

    pub fn adjust_request(&self, request: SizeRequest) -> SizeRequest {
        let wp = self.0+self.2;
        let hp = self.1+self.3;
        request.add(wp, hp)
    }
}

pub struct UniformExpand;

impl UniformExpand {
    pub fn get(sizes: Vec<(f32, f32)>, max_size: f32, spacing: f32) -> Vec<f32> {
        // Calculate the total spacing and the minimum size required
        let spacing = (sizes.len() - 1) as f32 * spacing;
        let min_size = sizes.iter().fold(0.0, |s, i| s + i.0) + spacing;

        let mut sizes = sizes.into_iter().map(|s| (s.0, s.1)).collect::<Vec<_>>();

        let mut free_space = (max_size - min_size).max(0.0);
        while free_space > 0.0 {
            let (min_exp, count, next) = sizes.iter().fold((None, 0.0, free_space), |(mut me, mut c, mut ne), size| {
                let min = size.0;
                let max = size.1;
                if min < max { // Item can expand
                    match me {
                        Some(w) if w < min => {
                            ne = ne.min(min - w); // Next size could be the min size of the next expandable block
                        },
                        Some(w) if w == min => {
                            ne = ne.min(max - min); // Next size could be the max size of one of the smallest items
                            c += 1.0;
                        },
                        Some(w) if w > min => {
                            ne = ne.min(max - min).min(w - min); // Next size could be the max size of one of the smallest items
                            me = Some(min);
                            c = 1.0;
                        },
                        _ => {
                            ne = ne.min(max - min); // Next size could be the max size of one of the smallest items
                            me = Some(min);
                            c = 1.0;
                        }
                    }
                }
                (me, c, ne)
            });

            if min_exp.is_none() { break; }
            let min_exp = min_exp.unwrap();

            let expand = (next * count).min(free_space); // Next size could be the rest of the free space
            free_space -= expand;
            let expand = expand / count;

            sizes.iter_mut().for_each(|size| {
                if size.0 < size.1 && size.0 == min_exp {
                    size.0 += expand;
                }
            });
        }

        sizes.into_iter().map(|s| s.0).collect()
    }
}

/// Horizontal layout of items.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/row.png"
///      alt="Row Example"
///      width="250">
///
///```rust
/// let layout = Row::new(24.0, Offset::Center, Size::Fit, Padding::new(8.0));
///```
#[derive(Debug)]
pub struct Row(f32, Offset, Size, Padding);

impl Row {
    pub fn new(spacing: f32, offset: Offset, size: Size, padding: Padding) -> Self {
        Row(spacing, offset, size, padding)
    }

    pub fn center(spacing: f32) -> Self {
        Row::new(spacing, Offset::Center, Size::Fit, Padding::default())
    }

    pub fn padding(&mut self) -> &mut Padding {&mut self.3}
}

impl Layout for Row {
    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|i|
            ((i.min_width(), i.max_width()), (i.min_height(), i.max_height()))
        ).unzip();
        let spacing = self.0*(widths.len()-1) as f32;
        let width = Size::add(widths);
        let height = self.2.get(heights, Size::max);
        self.3.adjust_request(SizeRequest::new(width.0, height.0, width.1, height.1).add_width(spacing))
    }

    fn build(&self, _ctx: &mut Context, row_size: (f32, f32), children: Vec<SizeRequest>) -> Vec<Area> {
        let row_size = self.3.adjust_size(row_size);

        let widths = UniformExpand::get(children.iter().map(|i| (i.min_width(), i.max_width())).collect::<Vec<_>>(), row_size.0, self.0);

        let mut offset = 0.0;
        children.into_iter().zip(widths).map(|(i, width)| {
            let size = i.get((width, row_size.1));
            let off = self.3.adjust_offset((offset, self.1.get(row_size.1, size.1)));
            offset += size.0+self.0;
            Area{offset: off, size}
        }).collect()
    }
}

/// Vertical layout of items.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/column.png"
///      alt="Column Example"
///      width="250">
///
///```rust
/// let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::new(8.0));
///```
#[derive(Debug)]
pub struct Column(f32, Offset, Size, Padding);

impl Column {
    pub fn new(spacing: f32, offset: Offset, size: Size, padding: Padding) -> Self {
        Column(spacing, offset, size, padding)
    }

    pub fn center(spacing: f32) -> Self {
        Column(spacing, Offset::Center, Size::Fit, Padding::default())
    }

    pub fn padding(&mut self) -> &mut Padding {&mut self.3}
}

impl Layout for Column {
    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|i|
            ((i.min_width(), i.max_width()), (i.min_height(), i.max_height()))
        ).unzip();
        let spacing = self.0*(heights.len()-1) as f32;
        let width = self.2.get(widths, Size::max);
        let height = Size::add(heights);
        self.3.adjust_request(SizeRequest::new(width.0, height.0, width.1, height.1).add_height(spacing))
    }

    fn build(&self, _ctx: &mut Context, col_size: (f32, f32), children: Vec<SizeRequest>) -> Vec<Area> {
        let col_size = self.3.adjust_size(col_size);

        let heights = UniformExpand::get(children.iter().map(|i| (i.min_height(), i.max_height())).collect::<Vec<_>>(), col_size.1, self.0);

        let mut offset = 0.0;
        children.into_iter().zip(heights).map(|(i, height)| {
            let size = i.get((col_size.0, height));
            let off = self.3.adjust_offset((self.1.get(col_size.0, size.0), offset));
            offset += size.1+self.0;
            Area{offset: off, size}
        }).collect()
    }
}

/// Items stacked on top of each other
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/stack.png"
///      alt="Stack Example"
///      width="250">
///
///```rust
/// let layout = Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::new(8.0));
///```
#[derive(Debug, Default)]
pub struct Stack(pub Offset, pub Offset, pub Size, pub Size, pub Padding);

impl Stack {
    pub fn center() -> Self {
        Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default())
    }

    pub fn fill() -> Self {
        Stack(Offset::Center, Offset::Center, Size::fill(), Size::fill(), Padding::default())
    }
}

impl Layout for Stack {
    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|r|
            ((r.min_width(), r.max_width()), (r.min_height(), r.max_height()))
        ).unzip();
        let width = self.2.get(widths, Size::max);
        let height = self.3.get(heights, Size::max);
        self.4.adjust_request(SizeRequest::new(width.0, height.0, width.1, height.1))
    }

    fn build(&self, _ctx: &mut Context, stack_size: (f32, f32), children: Vec<SizeRequest>) -> Vec<Area> {
        let stack_size = self.4.adjust_size(stack_size);
        children.into_iter().map(|i| {
            let size = i.get(stack_size);
            let offset = (self.0.get(stack_size.0, size.0), self.1.get(stack_size.1, size.1));
            Area{offset: self.4.adjust_offset(offset), size}
        }).collect()
    }
}

/// Horizontal layout that automatically wraps items to the next row when the maximum width is exceeded.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/wrap.png"
///      alt="Wrap Example"
///      width="350">
///
///```rust
/// let layout = Wrap::new(8.0, 8.0);
///```
#[derive(Debug)]
pub struct Wrap(pub f32, pub f32, pub Offset, pub Offset, pub Padding, Arc<Mutex<f32>>);

impl Wrap {
    pub fn new(w_spacing: f32, h_spacing: f32) -> Self {
        Wrap(w_spacing, h_spacing, Offset::Center, Offset::Center, Padding::default(), Arc::new(Mutex::new(0.0)))
    }
}
impl Layout for Wrap {
    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        let mut lw = self.4.1;
        let mut lh = 0.0;
        let mut th = self.4.0;
        let mut max_lw: f32 = 0.0;
        for child in children {
            let (w, h) = (child.min_width(), child.min_height());
            if lw + w > *self.5.lock().unwrap() && lw > self.4.1 {
                th += lh + self.1;
                max_lw = max_lw.max(lw - self.0);
                lw = self.4.1;
                lh = 0.0;
            }
            lw += w + self.0;
            lh = lh.max(h);
        }
        if lw > self.4.1 {
            th += lh;
            max_lw = max_lw.max(lw - self.0);
        }
        SizeRequest::new(max_lw + self.4.2, th + self.4.3, f32::MAX, f32::MAX)
    }

    fn build(&self, _ctx: &mut Context, maximum_size: (f32, f32), children: Vec<SizeRequest>) -> Vec<Area> {
        *self.5.lock().unwrap() = maximum_size.0;

        let mut areas = Vec::new();
        let mut line = Vec::new();
        let mut tw = self.4.1;
        let mut ho = self.4.0;
        let mut lh = 0.0;

        let flush = |line: &[(f32, f32)], tw: f32, _: f32, ho: f32| {
            if line.is_empty() { return Vec::new(); }
            let line_w = tw - self.0 - self.4.1;
            let extra = (maximum_size.0 - line_w).max(0.0);
            let start_x = match self.2 {
                Offset::Start => self.4.1,
                Offset::End => self.4.1 + extra,
                Offset::Center => self.4.1 + extra / 2.0,
                Offset::Static(_) => 0.0,
            };
            let mut x = start_x;
            line.iter().map(|&(w, h)| {
                let a = Area { offset: (x, ho), size: (w, h) };
                x += w + self.0;
                a
            }).collect()
        };

        for child in children {
            let (w, h) = (child.min_width(), child.min_height());
            if tw + w > maximum_size.0 && tw > self.4.1 {
                areas.extend(flush(&line, tw, lh, ho));
                ho += lh + self.1;
                tw = self.4.1;
                lh = 0.0;
                line.clear();
            }
            line.push((w, h));
            tw += w + self.0;
            lh = lh.max(h);
        }
        areas.extend(flush(&line, tw, lh, ho));
        areas
    }
}


/// Defines the reference point for scrolling content.
#[derive(Debug, Clone, Copy)]
pub enum ScrollAnchor {
    Start,
    End,
}

/// Scrollable layout of items.
#[derive(Debug)]
pub enum Scroll {
    Vertical(Offset, Offset, Size, Size, Padding, Arc<Mutex<f32>>, ScrollAnchor),
    Horizontal(Offset, Offset, Size, Size, Padding, Arc<Mutex<f32>>, ScrollAnchor)
}

impl Default for Scroll {
    fn default() -> Self {
        Scroll::new(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default(), ScrollAnchor::Start)
    }
}

impl Scroll {
    pub fn new(offset_x: Offset, offset_y: Offset, size_x: Size, size_y: Size, padding: Padding, anchor: ScrollAnchor) -> Self {
        Scroll::Vertical(offset_x, offset_y, size_x, size_y, padding, Arc::new(Mutex::new(0.0)), anchor)
    }

    pub fn horizontal(offset_x: Offset, offset_y: Offset, size_x: Size, size_y: Size, padding: Padding, anchor: ScrollAnchor) -> Self {
        Scroll::Horizontal(offset_x, offset_y, size_x, size_y, padding, Arc::new(Mutex::new(0.0)), anchor)
    }

    pub fn adjust_scroll(&mut self, delta: f32) { 
        println!("ADJUSTING SCROLL");
        match self {
            Scroll::Vertical(_, _, _, _, _, m, a) => {
                let mut guard = m.lock().unwrap();
                match a {
                    ScrollAnchor::Start => *guard += delta,
                    ScrollAnchor::End => *guard -= delta,
                }
            },
            Scroll::Horizontal(_, _, _, _, _, m, a) => {
                let mut guard = m.lock().unwrap();
                match a {
                    ScrollAnchor::Start => *guard += delta,
                    ScrollAnchor::End => *guard -= delta,
                }
            }
        }
    }

    pub fn set_scroll(&mut self, val: f32) { 
        match self {
            Scroll::Vertical(_, _, _, _, _, m, _) =>  *m.lock().unwrap() = val,
            Scroll::Horizontal(_, _, _, _, _, m, _) => *m.lock().unwrap() = val,
        };
    }

    pub fn offset(&mut self) -> &mut Offset { 
        match self {
            Scroll::Vertical(_, o, _, _, _, _, _) => o,
            Scroll::Horizontal(o, _, _, _, _, _, _) => o,
        }
    }
}

impl Layout for Scroll {
    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        let (widths, heights): (Vec<_>, Vec<_>) = children.into_iter().map(|r|
            ((r.min_width(), r.max_width()), (r.min_height(), r.max_height()))
        ).unzip();

        let (width, height, padding) = match &self {
            Scroll::Vertical(_, _, s_x, s_y, padd, _, _) |
            Scroll::Horizontal(_, _, s_x, s_y, padd, _, _) => {
                let width = s_x.get(widths, Size::max);
                let height = s_y.get(heights, Size::max);
                (width, height, padd)
            }
        };
        
        padding.adjust_request(SizeRequest::new(width.0, height.0, width.1, height.1))
    }

    fn build(&self, _ctx: &mut Context, scroll_size: (f32, f32), children: Vec<SizeRequest>) -> Vec<Area> {
        match &self {
            Scroll::Vertical(x_off, y_off, _, _, padd, val, anch) => {
                let scroll_size = padd.adjust_size(scroll_size);
                let children_height: f32 = children.iter().map(|i| i.min_height()).sum();
                let max_scroll = (children_height - scroll_size.1).max(0.0);

                let mut scroll_val = val.lock().unwrap();
                *scroll_val = scroll_val.clamp(0.0, max_scroll);

                children.into_iter().map(|i| {
                    let size = i.get(scroll_size);
                    let y_offset = match anch {
                        ScrollAnchor::Start => y_off.get(scroll_size.1, size.1)-*scroll_val,
                        ScrollAnchor::End => scroll_size.1 - children_height + *scroll_val,
                    };
                    let offset = (x_off.get(scroll_size.0, size.0), y_offset);
                    Area {offset: padd.adjust_offset(offset), size }
                }).collect()
            }
            Scroll::Horizontal(x_off, y_off, _, _, padd, val, anch) => {
                let scroll_size = padd.adjust_size(scroll_size);
                let children_width: f32 = children.iter().map(|i| i.min_width()).sum();
                let max_scroll = (children_width - scroll_size.0).max(0.0);

                let mut scroll_val = val.lock().unwrap();
                *scroll_val = scroll_val.clamp(0.0, max_scroll);

                children.into_iter().map(|i| {
                    let size = i.get(scroll_size);
                    let x_offset = match anch {
                        ScrollAnchor::Start => x_off.get(scroll_size.0, size.0) - *scroll_val,
                        ScrollAnchor::End => scroll_size.0 - children_width + *scroll_val,
                    };
                    let offset = (x_offset, y_off.get(scroll_size.1, size.1));
                    Area {offset: padd.adjust_offset(offset), size }
                }).collect()
            }
        }
    }
}

/// A container pairing a layout with a drawable element.
#[derive(Debug, Component)]
pub struct Bin<L: Layout, D: Drawable>(pub L, pub D);

impl<L: Layout, D: Drawable> OnEvent for Bin<L, D> {}

impl<L: Layout, D: Drawable> Bin<L, D> {
    pub fn inner(&mut self) -> &mut D {
        &mut self.1
    }
    pub fn layout(&mut self) -> &mut L {
        &mut self.0
    }
}

/// A container that optionally displays a drawable item, toggling between visible and hidden states.
#[derive(Debug, Component)]
pub struct Opt<D: Drawable + 'static>(Stack, Option<D>, #[skip] Option<D>);
impl<D: Drawable + 'static> OnEvent for Opt<D> {}

impl<D: Drawable + 'static> Opt<D> {
    pub fn new(item: D, display: bool) -> Self {
        match display {
            true => Opt(Stack::default(), Some(item), None),
            false => Opt(Stack::default(), None, Some(item)),
        }
    }

    pub fn display(&mut self, display: bool) {
        match display {
            true if self.1.is_none() => self.1 = self.2.take(),
            false if self.2.is_none() => self.2 = self.1.take(),
            _ => {}
        }
    }

    pub fn inner(&mut self) -> &mut D {
        self.1.as_mut().unwrap_or_else(|| self.2.as_mut().unwrap())
    }

    pub fn is_showing(&self) -> bool {
        self.1.is_some()
    }
}

/// A container that holds two drawables but displays only one at a time, allowing toggling between them.
#[derive(Debug, Component)]
pub struct EitherOr<L: Drawable + 'static, R: Drawable + 'static>(Stack, Opt<L>, Opt<R>);

impl<L: Drawable + 'static, R: Drawable + 'static> OnEvent for EitherOr<L, R> {}

impl<L: Drawable + 'static, R: Drawable + 'static> EitherOr<L, R> {
    pub fn new(left: L, right: R) -> Self {
        EitherOr(Stack::default(), Opt::new(left, true), Opt::new(right, false))
    }

    pub fn display_left(&mut self, display_left: bool) {
        self.1.display(display_left);
        self.2.display(!display_left);
    }

    pub fn left(&mut self) -> &mut L { self.1.inner() }
    pub fn right(&mut self) -> &mut R { self.2.inner() }
}

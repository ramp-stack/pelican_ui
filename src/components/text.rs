use prism::events::{TickEvent, Key, KeyboardEvent, KeyboardState, DynEvent, MouseEvent, MouseState, MouseButton};
use ptsd::layout::{Stack, Size, Offset, Padding};
use ptsd::display::Opt;
use prism::drawable::{
    Drawable, Component, LayoutTree, RangeTree, SizeRange, Rect, Instruction, Item,
    Color as CanvasColor,
};
use prism::drawable::text::{Align, Span, Text as BasicText};
use prism::drawable::shape::{Shape as CanvasShape, Variant as ShapeType};
use prism::Context;

use std::time::{Instant, Duration};
use unicode_segmentation::UnicodeSegmentation;

use pelican_ui::components::Rectangle;
use pelican_ui::theme::{Theme, Color};

use ptsd::{theme, FontStyle};
pub use ptsd::TextSize;

#[derive(Component, Debug, Clone)]
#[on_dyn_event]
pub struct Text {
    layout: Stack,
    inner: BasicText,
    #[skip] pub spans: Vec<String>,
    #[skip] pub size: TextSize,
    #[skip] pub style: TextStyle,
    #[skip] pub align: Align,
    #[skip] pub max_lines: Option<u32>,
    #[skip] pub kerning: f32,
}

impl Text {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            self.inner.align = self.align;
            self.inner.max_lines = self.max_lines;
            self.inner.spans.iter_mut().enumerate().for_each(|(i, s)| {
                s.text = self.spans[i].to_string();
                s.line_height = Some(s.font_size * 1.25);
                s.kerning = self.kerning;
            });
        }
        vec![event]
    }
}

impl Text {
    pub fn new(theme: &Theme, text: &str, text_size: TextSize, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        let (color, font) = style.get(theme);
        let size = theme.fonts().get_size(text_size);
        let inner = BasicText {
            spans: vec![
                Span::new(text.to_string(), size, Some(size*1.25), font.into(), color.into(), 0.0)
            ],
            width: None, align, cursor: None, max_lines,
        };
        Text {layout: Stack::default(), inner, spans: vec![text.to_string()], size: text_size, style, align, max_lines, kerning: 0.0}
    }

    pub fn default(theme: &Theme, text: &str) -> Self {
        Self::new(theme, text, TextSize::H4, TextStyle::Heading, Align::Center, None)
    }

    pub fn inner(&self) -> &BasicText {&self.inner}
}

#[derive(Clone, Copy, Debug)]
pub enum TextStyle {
    Heading,
    Primary,
    Secondary,
    Error,
    Keyboard,
    Label(Color),
}

impl TextStyle {
    pub fn get(&self, theme: &Theme) -> (Color, prism::drawable::text::Font) {
        let heading = theme.fonts().get_font(FontStyle::Heading).unwrap();
        let text = theme.fonts().get_font(FontStyle::Text).unwrap();
        let label = theme.fonts().get_font(FontStyle::Label).unwrap();
        let keyboard = theme.fonts().get_font(FontStyle::Text).unwrap();
        let colors = theme.colors();
        match self {
            TextStyle::Heading => (colors.get(theme::Text::Heading), heading.clone()),
            TextStyle::Primary => (colors.get(theme::Text::Primary), text.clone()),
            TextStyle::Secondary => (colors.get(theme::Text::Secondary), text.clone()),
            TextStyle::Error => (colors.get(theme::Status::Danger), text.clone()),
            TextStyle::Keyboard => (colors.get(theme::Text::Heading), keyboard.clone()),
            TextStyle::Label(color) => (*color, label.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpandableText(pub Text);
impl ExpandableText {
    pub fn new(theme: &Theme, text: &str, size: TextSize, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        ExpandableText(Text::new(theme, text, size, style, align, max_lines),)
    }

    pub fn default(theme: &Theme, text: &str) -> Self {
        Self::new(theme, text, TextSize::H4, TextStyle::Heading, Align::Center, None)
    }
}

impl Drawable for ExpandableText {
    fn size_range(&self) -> RangeTree {
        let size = self.0.inner.size();
        RangeTree(SizeRange {
            min_width: 0.0, max_width: f32::MAX, min_height: size.1, max_height: size.1,
        }, vec![])
    }

    fn layout(&mut self, size: (f32, f32), request: &RangeTree) -> LayoutTree {
        let size = request.0.clamp(size);
        self.0.inner.width = Some(size.0);
        LayoutTree(size, vec![])
    }

    fn draw(&self, _sized: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        vec![Instruction {offset, bound, item: Item::Text(self.0.inner.clone())}]
    }

    fn event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) {
        self.0.event(ctx, sized, event)
    }
}

#[derive(Component, Clone, Debug)]
#[on_dyn_event]
pub struct TextEditor(Stack, pub ExpandableText, TextCursor, Magnifier, #[skip] Scrub, #[skip] FrameInput);
impl TextEditor {
    pub fn new(theme: &Theme, text: &str, size: TextSize, style: TextStyle, align: Align) -> Self {
        let mut built = ExpandableText::new(theme, text, size, style, align, None);
        built.0.inner.cursor = Some(text.len());
        let magnifier = Magnifier::new(theme, style, built.0.inner.clone());
        TextEditor(Stack::start(), built, TextCursor::new(theme, style, size), magnifier, Scrub::default(), FrameInput::default())
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "", TextSize::Md, TextStyle::Primary, Align::Left)
    }

    pub fn display_cursor(&mut self, display: bool) {
        self.2.1.display(display)
    }

    pub fn insert(&mut self, text: &str) {
        let content = &mut self.1.0.spans[0];
        let index = clamp_to_boundary(content, self.1.0.inner.cursor.unwrap_or(content.len()));
        content.insert_str(index, text);
        self.1.0.inner.cursor = Some(index + text.len());
        self.sync();
    }

    pub fn delete_back(&mut self) {
        let content = &mut self.1.0.spans[0];
        let index = clamp_to_boundary(content, self.1.0.inner.cursor.unwrap_or(content.len()));
        let cursor = match previous_grapheme(content, index) {
            Some(start) => { content.replace_range(start..index, ""); start },
            None => index,
        };
        self.1.0.inner.cursor = Some(cursor);
        self.sync();
    }

    pub fn set_text(&mut self, text: &str) {
        self.1.0.spans[0] = text.to_string();
        self.1.0.inner.cursor = Some(text.len());
        self.sync();
    }

    pub fn pointer_pressed(&mut self, pos: (f32, f32)) {
        self.place_cursor(pos);
        self.4 = match self.5.released {
            true => Scrub::default(),
            false => Scrub { pressed_at: Some(Instant::now()), pointer: pos, moved: (0.0, 0.0), active: false },
        };
        self.5.pressed = true;
        self.sync();
    }

    pub fn pointer_moved(&mut self, pos: Option<(f32, f32)>, delta: Option<(f32, f32)>) {
        if self.4.pressed_at.is_none() { return; }

        let step = match (delta, pos) {
            (Some(d), _) => d,
            (None, Some(p)) => (p.0 - self.4.pointer.0, p.1 - self.4.pointer.1),
            (None, None) => return,
        };
        self.4.pointer = (self.4.pointer.0 + step.0, self.4.pointer.1 + step.1);
        self.4.moved = (self.4.moved.0 + step.0, self.4.moved.1 + step.1);

        let distance = self.4.moved.0.hypot(self.4.moved.1);

        if !self.4.active {
            if cfg!(any(target_os = "ios", target_os = "android")) {
                if distance > 10.0 { self.4 = Scrub::default(); }
                return;
            }
            if distance < 3.0 { return; }
            self.4.active = true;
        }

        let pointer = self.4.pointer;
        self.place_cursor(pointer);
        self.sync();
    }

    pub fn pointer_released(&mut self) {
        if !self.5.pressed { self.4 = Scrub::default(); }
        self.5.released = true;
        self.sync();
    }

    fn place_cursor(&mut self, pos: (f32, f32)) {
        let inner = &mut self.1.0.inner;
        let lh = inner.spans.first().map(|s| s.line_height.unwrap_or(s.font_size * 1.25)).unwrap_or(1.0);
        let lines = ((inner.size().1 / lh).round() as usize).max(1);
        let line = ((pos.1 / lh).floor().max(0.0) as usize).min(lines - 1);
        inner.cursor_click(pos.0, line as f32 * lh + lh * 0.5);
    }

    fn sync(&mut self) {
        let text = &mut self.1.0;
        text.inner.spans.iter_mut().zip(&text.spans).for_each(|(span, s)| {
            if span.text != *s { span.text = s.clone(); }
        });

        if self.1.0.inner.cursor.is_some() {
            let cursor_pos = self.1.0.inner.cursor_position();
            *self.2.x_offset() = Offset::Static(cursor_pos.0);
            *self.2.y_offset() = Offset::Static(cursor_pos.1+2.0);
        }

        self.3.active = self.4.active;
        self.3.text = self.1.0.inner.clone();
    }
}

impl TextEditor {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            self.5 = FrameInput::default();

            if let Some(t) = self.4.pressed_at && !self.4.active && t.elapsed() >= Duration::from_millis(300) {
                self.4.active = true;
                let pointer = self.4.pointer;
                self.place_cursor(pointer);
            }
            self.sync();
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) => {
                    if let Some(position) = mouse.position {self.pointer_pressed(position);}
                }
                MouseState::Moved => self.pointer_moved(mouse.position, None),
                MouseState::Scroll(x, y) if cfg!(any(target_os = "ios", target_os = "android")) => {
                    self.pointer_moved(mouse.position, Some((-x, -y)));
                }
                MouseState::Released(MouseButton::Left) => self.pointer_released(),
                _ => {},
            }
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed | KeyboardState::Repeated, key, ..}) = event.downcast_ref() {
            match key {
                Key::Character(c) => self.insert(c),
                Key::Enter => self.insert("\n"),
                Key::Space => self.insert(" "),
                Key::Delete | Key::Backspace => self.delete_back(),
                _ => {},
            }
        }

        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
pub struct TextCursor(Stack, Opt<Rectangle>);


impl TextCursor {
    pub fn new(theme: &Theme, style: TextStyle, size: TextSize) -> Self {
        let (color, _) = style.get(theme);
        let size = theme.fonts().get_size(size);
        TextCursor(
            Stack(Offset::Start, Offset::End, Size::Static(2.0), Size::Static(size), Padding::default()),
            Opt::new(Rectangle::new(color, 0.0, None), true)
        )
    }

    pub fn x_offset(&mut self) -> &mut Offset { &mut self.0.0 }
    pub fn y_offset(&mut self) -> &mut Offset { &mut self.0.1 }
}

#[derive(Debug, Clone)]
pub struct Magnifier {
    text: BasicText,
    active: bool,
    background: CanvasColor,
    outline: CanvasColor,
    cursor: CanvasColor,
}

impl Magnifier {
    fn new(theme: &Theme, style: TextStyle, text: BasicText) -> Self {
        let colors = theme.colors();
        Magnifier {
            text,
            active: false,
            background: colors.get(ptsd::Background::Secondary).into(),
            outline: colors.get(ptsd::Outline::Secondary).into(),
            cursor: style.get(theme).0.into(),
        }
    }
}

impl Drawable for Magnifier {
    fn size_range(&self) -> RangeTree {
        let h = self.text.size().1;
        RangeTree(SizeRange {
            min_width: 0.0, max_width: f32::MAX, min_height: h, max_height: h,
        }, vec![])
    }

    fn draw(&self, _sized: &LayoutTree, offset: (f32, f32), _bound: Rect) -> Vec<Instruction> {
        if !self.active || self.text.cursor.is_none() { return vec![]; }
        let Some(span) = self.text.spans.first() else { return vec![] };

        let line_h = span.line_height.unwrap_or(span.font_size * 1.25);
        let (cx, cy) = self.text.cursor_position();
        let (w, h) = (120.0, line_h * 1.5 + 2.0 * 8.0);

        let lens = ((offset.0 + cx - w / 2.0).max(4.0), offset.1 + cy - h - 16.0);
        let center = (lens.0 + w / 2.0, lens.1 + h / 2.0);
        let cursor_x = (offset.0 + cx).clamp(lens.0 + 8.0, lens.0 + w - 8.0);
        let focus_y = cy + line_h / 2.0;

        let mut zoomed = self.text.clone();
        zoomed.cursor = None;
        zoomed.scale(1.5);

        let rect = |x: f32, y: f32, w: f32, h: f32, stroke: f32, radius: f32, color: CanvasColor| Instruction {
            offset: (x, y), bound: (x, y, w, h),
            item: Item::Shape(CanvasShape {
                stroke, width: w, height: h, angle: 0.0, color,
                variant: ShapeType::RoundedRectangle(radius),
            }),
        };

        let band = (lens.0, center.1 - line_h * 1.5 / 2.0, w, line_h * 1.5);

        vec![
            rect(lens.0, lens.1, w, h, 0.0, 8.0, self.background),
            Instruction {
                offset: (cursor_x - cx * 1.5, center.1 - focus_y * 1.5), bound: band,
                item: Item::Text(zoomed),
            },
            rect(cursor_x - 1.0, center.1 - line_h * 1.5 / 2.0, 2.0, line_h * 1.5, 0.0, 0.0, self.cursor),
            rect(lens.0, lens.1, w, h, 1.0, 8.0, self.outline),
        ]
    }
}

#[derive(Debug, Clone, Default)]
struct Scrub { pressed_at: Option<Instant>, pointer: (f32, f32), moved: (f32, f32), active: bool }

#[derive(Debug, Clone, Copy, Default)]
struct FrameInput { pressed: bool, released: bool }

fn clamp_to_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) { index -= 1; }
    index
}

fn previous_grapheme(text: &str, index: usize) -> Option<usize> {
    text[..index].grapheme_indices(true).next_back().map(|(start, _)| start)
}

use prism::event::{OnEvent, Event, TickEvent, Key, KeyboardEvent, KeyboardState};
use prism::layout::{Stack, Size, Offset, Padding, SizeRequest};
use prism::display::Opt;
use prism::drawable::{Drawable, Component, SizedTree, RequestTree, Rect}; 
use prism::canvas::{self, ShapeType, Shape as CanvasShape, Align, Span, Text as BasicText, Area as CanvasArea, Item as CanvasItem, Instruction};
use prism::Context;

use std::time::{Instant, Duration};
use unicode_segmentation::UnicodeSegmentation;

use pelican_ui::components::Rectangle;
use pelican_ui::theme::{Theme, Color};

use ptsd::{theme, FontStyle};
pub use ptsd::TextSize;

#[derive(Component, Debug, Clone)]
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

impl OnEvent for Text {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            // let (color, font) = self.style.get(ctx);
            self.inner.align = self.align;
            self.inner.max_lines = self.max_lines;
            self.inner.spans.iter_mut().enumerate().for_each(|(i, s)| {
                s.text = self.spans[i].to_string();
                // s.font_size = self.size.get(ctx);
                s.line_height = Some(s.font_size * 1.25);
                // s.color = color.into();
                // s.font = font.clone().into();
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
        let inner = BasicText{
            spans: vec![Span::new(text.to_string(), size, Some(size*1.25), font, color.into(), 0.0)], 
            align,
            max_lines,
            cursor: None,
            width: None,
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
    pub fn get(&self, theme: &Theme) -> (Color, canvas::Font) {
        let heading = theme.fonts().get_font(FontStyle::Heading).unwrap();
        let text = theme.fonts().get_font(FontStyle::Text).unwrap();
        let label = theme.fonts().get_font(FontStyle::Label).unwrap();
        let keyboard = theme.fonts().get_font(FontStyle::Text).unwrap(); // change to keybord (medium weight)
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
    fn request_size(&self) -> RequestTree {
        let size = self.0.inner.size();
        RequestTree(SizeRequest::new(0.0, size.1, f32::MAX, size.1), vec![])
    }

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let text = BasicText {spans: self.0.inner.spans.clone(), width: Some(sized.0.0), align: self.0.inner.align, cursor: self.0.inner.cursor, max_lines: self.0.inner.max_lines};
        vec![Instruction(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Text(text))]
    }

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) {
        self.0.inner.width = Some(sized.0.0);
        self.0.event(ctx, sized, event)
    }
}

#[derive(Component, Clone)]
pub struct TextEditor(Stack, pub ExpandableText, TextCursor, Magnifier, #[skip] Scrub, #[skip] FrameInput);
impl std::fmt::Debug for TextEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TextEditor").field(&self.0).field(&self.1).field(&self.2).finish()
    }
}

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

    /// Puts `text` in at the cursor, as typing it would.
    pub fn insert(&mut self, text: &str) {
        // `cursor` is a byte offset into the text (as the canvas lays it out).
        let content = &mut self.1.0.spans[0];
        let index = clamp_to_boundary(content, self.1.0.inner.cursor.unwrap_or(content.len()));
        content.insert_str(index, text);
        self.1.0.inner.cursor = Some(index + text.len());
        self.sync();
    }

    /// Deletes the character before the cursor, as backspace would.
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
            if prism::IS_MOBILE {
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

fn clamp_to_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) { index -= 1; }
    index
}

fn previous_grapheme(text: &str, index: usize) -> Option<usize> {
    text[..index].grapheme_indices(true).next_back().map(|(start, _)| start)
}

impl OnEvent for TextEditor {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            // A new frame: no pointer events handled in it yet.
            self.5 = FrameInput::default();

            // A press held long enough becomes a scrub.
            if let Some(t) = self.4.pressed_at && !self.4.active && t.elapsed() >= Duration::from_millis(300) {
                self.4.active = true;
                let pointer = self.4.pointer;
                self.place_cursor(pointer);
            }
            self.sync();
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed | KeyboardState::Repeated, key, ..}) = event.downcast_ref() {
            match key {
                Key::Character(c) => self.insert(c),
                Key::Enter => self.insert("\n"),
                Key::Space => self.insert(" "),
                // The on-screen keyboard and keypad send `Delete` for backspace,
                // so both keys delete the character before the cursor.
                Key::Delete | Key::Backspace => self.delete_back(),
                _ => {},
            }
        }

        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
pub struct TextCursor(Stack, Opt<Rectangle>);

impl OnEvent for TextCursor {}

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

// // /// # Bulleted Text
// // ///
// // /// A component for rendering lists with bullet points.
// // ///
// // /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/bulleted_text.png"
// // ///      alt="Bulleted Text Example"
// // ///      width="400">
// // ///
// // /// ## Example
// // ///```rust
// // /// let text_size = ctx.theme.fonts.size.md;
// // /// let items = vec!["Feed the chairs at midnight.", "Borrow a broom from the moon.", "Vacuum the car inside out.", "Hide the clock beneath the carpet."];
// // /// let list = BulletedText::new(ctx, items, TextStyle::Primary, text_size);
// // ///```
// // #[derive(Debug, Component)]
// // pub struct BulletedText(Column, Vec<BulletedTextContent>);

// // impl OnEvent for BulletedText {}

// // impl BulletedText {
// //     pub fn new(ctx: &mut Context, text: Vec<&str>, style: TextStyle, size: f32) -> Self {
// //         let color = style.get(ctx).0;
// //         let items = text.into_iter().map(|t| BulletedTextContent::new(ctx, t, color, style, size)).collect();
// //         BulletedText(Column::center(8.0), items)
// //     }
// // }

// // #[derive(Debug, Component)]
// // struct BulletedTextContent(Row, Shape, ExpandableText);
// // impl OnEvent for BulletedTextContent {}
// // impl BulletedTextContent {
// //     fn new(ctx: &mut Context, text: &str, color: Color, style: TextStyle, size: f32) -> Self {
// //         BulletedTextContent(
// //             Row::new(size*0.75, Offset::Center, Size::Fit, Padding::default()), // change this offset to be line_height - circle size / 2
// //             Circle::new(size*0.2, color, false),
// //             ExpandableText::new(ctx, text, style, size, Align::Left, None)
// //         )
// //     }
// // }

#[derive(Debug, Clone, Default)]
struct Scrub { pressed_at: Option<Instant>, pointer: (f32, f32), moved: (f32, f32), active: bool }

#[derive(Debug, Clone, Copy, Default)]
struct FrameInput { pressed: bool, released: bool }

#[derive(Debug, Clone)]
pub struct Magnifier {
    text: BasicText,
    active: bool,
    background: canvas::Color,
    outline: canvas::Color,
    cursor: canvas::Color,
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
    fn request_size(&self) -> RequestTree {
        let h = self.text.size().1;
        RequestTree(SizeRequest::new(0.0, h, f32::MAX, h), vec![])
    }

    fn draw(&self, _sized: &SizedTree, offset: (f32, f32), _bound: Rect) -> Vec<Instruction> {
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

        let rect = |x: f32, y: f32, w: f32, h: f32, stroke: f32, radius: f32, color: canvas::Color| Instruction(
            CanvasArea { offset: (x, y), bounds: None },
            CanvasItem::Shape(CanvasShape { shape: ShapeType::RoundedRectangle(stroke, (w, h), 0.0, radius), color }),
        );

        let band = (lens.0, center.1 - line_h * 1.5 / 2.0, w, line_h * 1.5);

        vec![
            rect(lens.0, lens.1, w, h, 0.0, 8.0, self.background),
            Instruction(
                CanvasArea { offset: (cursor_x - cx * 1.5, center.1 - focus_y * 1.5), bounds: Some(band) },
                CanvasItem::Text(zoomed),
            ),
            rect(cursor_x - 1.0, center.1 - line_h * 1.5 / 2.0, 2.0, line_h * 1.5, 0.0, 0.0, self.cursor),
            rect(lens.0, lens.1, w, h, 1.0, 8.0, self.outline),
        ]
    }
}
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
            spans: vec![Span::new(text.to_string(), size, Some(size*1.25), font.into(), color.into(), 0.0)], 
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
pub struct TextEditor(Stack, pub ExpandableText, TextCursor, Magnifier, #[skip] Scrub);
impl std::fmt::Debug for TextEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TextEditor").field(&self.0).field(&self.1).field(&self.2).finish()
    }
}

impl TextEditor {
    pub fn new(theme: &Theme, text: &str, size: TextSize, style: TextStyle, align: Align) -> Self {
        let mut built = ExpandableText::new(theme, text, size, style, align, None);
        // The canvas's cursor is a byte offset into the text.
        built.0.inner.cursor = Some(text.len());
        let magnifier = Magnifier::new(theme, style, built.0.inner.clone());
        TextEditor(Stack::start(), built, TextCursor::new(theme, style, size), magnifier, Scrub::default())
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "", TextSize::Md, TextStyle::Primary, Align::Left)
    }

    pub fn display_cursor(&mut self, display: bool) {
        self.2.1.display(display)
    }

    /// Replaces the text and moves the cursor to the end of it.
    pub fn set_text(&mut self, text: &str) {
        self.1.0.spans[0] = text.to_string();
        self.1.0.inner.cursor = Some(text.len());
        self.sync();
    }

    /// Pointer pressed at `pos`, in this editor's coordinates. The owning input
    /// calls this for presses anywhere in the field, not just on the text line.
    pub fn pointer_pressed(&mut self, pos: (f32, f32)) {
        self.place_cursor(pos);
        self.4 = Scrub { pressed_at: Some(Instant::now()), pointer: pos, travel: 0.0, active: false };
        self.sync();
    }

    /// Pointer moved while pressed. `pos` is the new position in this editor's
    /// coordinates when the pointer is over the field; `delta` is the movement
    /// (from touch drags), used to keep tracking once the finger leaves the field.
    pub fn pointer_moved(&mut self, pos: Option<(f32, f32)>, delta: Option<(f32, f32)>) {
        if self.4.pressed_at.is_none() { return; }
        let next = match (pos, delta) {
            (Some(p), _) => p,
            (None, Some(d)) => (self.4.pointer.0 + d.0, self.4.pointer.1 + d.1),
            (None, None) => return,
        };
        let (dx, dy) = (next.0 - self.4.pointer.0, next.1 - self.4.pointer.1);
        self.4.travel += (dx * dx + dy * dy).sqrt();
        self.4.pointer = next;

        if !self.4.active {
            if prism::IS_MOBILE {
                // On touch, moving before the hold completes is a swipe, not a scrub.
                if self.4.travel > SCRUB_CANCEL_DISTANCE { self.4 = Scrub::default(); }
                return;
            }
            // With a mouse, dragging starts scrubbing right away.
            if self.4.travel < DESKTOP_DRAG_START { return; }
            self.4.active = true;
        }

        self.place_cursor(next);
        self.sync();
    }

    /// Pointer released (anywhere). Ends any scrub and hides the magnifier.
    pub fn pointer_released(&mut self) {
        self.4 = Scrub::default();
        self.sync();
    }

    /// Moves the cursor to the character nearest `pos`. The y position is
    /// snapped to the middle of the nearest line, so a pointer above or below
    /// the text still scrubs along it instead of jumping to the end.
    fn place_cursor(&mut self, pos: (f32, f32)) {
        let inner = &mut self.1.0.inner;
        let lh = inner.spans.first().map(|s| s.line_height.unwrap_or(s.font_size * 1.25)).unwrap_or(1.0);
        // Count lines from the height, which includes the empty line after a
        // trailing newline, so the cursor can be scrubbed onto it.
        let lines = ((inner.size().1 / lh).round() as usize).max(1);
        let line = ((pos.1 / lh).floor().max(0.0) as usize).min(lines - 1);
        inner.cursor_click(pos.0, line as f32 * lh + lh * 0.5);
    }

    /// Brings the laid-out text, cursor bar and magnifier in line with the
    /// current text and scrub state, so they update in the same frame.
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

/// `index` clamped to the text and moved back onto a character boundary, so
/// it's always safe to insert or slice there.
fn clamp_to_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) { index -= 1; }
    index
}

/// Where the grapheme before byte `index` starts, if there is one. Deleting
/// from there removes a whole emoji (with its variation selector, ZWJ parts
/// or skin tone) or an accented letter, not just its last code point.
fn previous_grapheme(text: &str, index: usize) -> Option<usize> {
    text[..index].grapheme_indices(true).next_back().map(|(start, _)| start)
}

impl OnEvent for TextEditor {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            // A press held long enough becomes a scrub.
            if let Some(t) = self.4.pressed_at && !self.4.active && t.elapsed() >= SCRUB_HOLD {
                self.4.active = true;
                let pointer = self.4.pointer;
                self.place_cursor(pointer);
            }
            self.sync();
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed | KeyboardState::Repeated, key, ..}) = event.downcast_ref() {
            let character = match key {
                Key::Character(c) => Some(c.clone()),
                Key::Enter => Some("\n".to_string()),
                Key::Space => Some(" ".to_string()),
                // The on-screen keyboard and keypad send `Delete` for backspace,
                // so both keys delete the character before the cursor.
                Key::Delete | Key::Backspace => None,
                _ => {return vec![event];}
            };

            // `cursor` is a byte offset into the text (as the canvas lays it out).
            let text = &mut self.1.0.spans[0];
            let index = clamp_to_boundary(text, self.1.0.inner.cursor.unwrap_or(text.len()));

            let cursor = match character {
                Some(c) => {
                    text.insert_str(index, &c);
                    index + c.len()
                }
                None => match previous_grapheme(text, index) {
                    Some(start) => {
                        text.replace_range(start..index, "");
                        start
                    }
                    None => index,
                },
            };

            self.1.0.inner.cursor = Some(cursor);
            self.sync();
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

const SCRUB_HOLD: Duration = Duration::from_millis(300);
const SCRUB_CANCEL_DISTANCE: f32 = 10.0;
const DESKTOP_DRAG_START: f32 = 3.0;
const ZOOM: f32 = 1.5;
const LENS_WIDTH: f32 = 120.0;
const LENS_PADDING: f32 = 8.0;
const LENS_GAP: f32 = 16.0;
const LENS_RADIUS: f32 = 8.0;
const LENS_EDGE: f32 = 4.0;

#[derive(Debug, Clone, Default)]
struct Scrub { pressed_at: Option<Instant>, pointer: (f32, f32), travel: f32, active: bool }

/// A zoomed copy of the editor's text, drawn above the cursor while scrubbing.
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
    // Mirror the text's request so prism always draws this child
    // (it skips zero-size children) without changing the layout.
    fn request_size(&self) -> RequestTree {
        let h = self.text.size().1;
        RequestTree(SizeRequest::new(0.0, h, f32::MAX, h), vec![])
    }

    fn draw(&self, _sized: &SizedTree, offset: (f32, f32), _bound: Rect) -> Vec<Instruction> {
        if !self.active || self.text.cursor.is_none() { return vec![]; }
        let Some(span) = self.text.spans.first() else { return vec![] };

        let line_h = span.line_height.unwrap_or(span.font_size * 1.25);
        let (cx, cy) = self.text.cursor_position();
        let (w, h) = (LENS_WIDTH, line_h * ZOOM + 2.0 * LENS_PADDING);

        // Lens sits centered over the cursor, just above its line.
        let lens = ((offset.0 + cx - w / 2.0).max(LENS_EDGE), offset.1 + cy - h - LENS_GAP);
        let center = (lens.0 + w / 2.0, lens.1 + h / 2.0);
        let cursor_x = (offset.0 + cx).clamp(lens.0 + LENS_PADDING, lens.0 + w - LENS_PADDING);
        let focus_y = cy + line_h / 2.0;

        let mut zoomed = self.text.clone();
        zoomed.cursor = None;
        zoomed.scale(ZOOM);

        let rect = |x: f32, y: f32, w: f32, h: f32, stroke: f32, radius: f32, color: canvas::Color| Instruction(
            CanvasArea { offset: (x, y), bounds: None },
            CanvasItem::Shape(CanvasShape { shape: ShapeType::RoundedRectangle(stroke, (w, h), 0.0, radius), color }),
        );

        let band = (lens.0, center.1 - line_h * ZOOM / 2.0, w, line_h * ZOOM);

        vec![
            rect(lens.0, lens.1, w, h, 0.0, LENS_RADIUS, self.background),
            Instruction(
                CanvasArea { offset: (cursor_x - cx * ZOOM, center.1 - focus_y * ZOOM), bounds: Some(band) },
                CanvasItem::Text(zoomed),
            ),
            rect(cursor_x - 1.0, center.1 - line_h * ZOOM / 2.0, 2.0, line_h * ZOOM, 0.0, 0.0, self.cursor),
            rect(lens.0, lens.1, w, h, 1.0, LENS_RADIUS, self.outline),
        ]
    }
}
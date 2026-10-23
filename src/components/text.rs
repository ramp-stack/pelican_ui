use prism::drawable::{
    Caret, Drawable, Component, LayoutTree, RangeTree, SizeRange, Rect, Instruction, Item,
    Color as CanvasColor,
};
use prism::drawable::text::{Align, Span, Text as BasicText};
use prism::drawable::shape::{Shape as CanvasShape, Variant as ShapeType};
use prism::events::{
    Key, KeyboardEvent, KeyboardState, DynEvent, MouseEvent, MouseState, MouseButton, ChangeSource,
    OnChange, on_change, Timer, WindowEvent,
};
use prism::Context;
use ptsd::display::Opt;
use ptsd::layout::{Stack, Size, Offset, Padding};
use ptsd::{theme, FontStyle};
pub use ptsd::TextSize;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use crate::components::{Rectangle, TextInputEvent};
use crate::theme::{Theme, Color};

#[derive(Clone, Copy, Debug)]
pub enum TextStyle { Heading, Primary, Secondary, Error, Keyboard, Label(Color) }

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

#[derive(Component, Debug, Clone)]
pub struct Text {
    layout: Stack,
    inner: BasicText,
}

impl Text {
    pub fn new(theme: &Theme, text: &str, text_size: TextSize, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        let (color, font) = style.get(theme);
        let size = theme.fonts().get_size(text_size);
        let mut inner = BasicText::new(text, font.into(), size, color.into(), align);
        inner.spans[0].line_height = Some(size * 1.25);
        inner.max_lines = max_lines;
        Text { layout: Stack::default(), inner }
    }

    pub fn default(theme: &Theme, text: &str) -> Self {
        Self::new(theme, text, TextSize::H4, TextStyle::Heading, Align::Center, None)
    }
}
impl Deref for Text {
    type Target = BasicText;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

#[derive(Debug, Clone)]
pub struct ExpandableText(pub Text);
impl ExpandableText {
    pub fn new(theme: &Theme, text: &str, size: TextSize, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        ExpandableText(Text::new(theme, text, size, style, align, max_lines))
    }

    pub fn default(theme: &Theme, text: &str) -> Self {
        Self::new(theme, text, TextSize::H4, TextStyle::Heading, Align::Center, None)
    }
}

impl Drawable for ExpandableText {
    fn size_range(&self) -> RangeTree {
        let size = self.0.inner.size();
        RangeTree(
            SizeRange {
                min_width: 0.0, max_width: f32::MAX, min_height: size.1, max_height: size.1,
            }, vec![],
        )
    }

    fn layout(&mut self, size: (f32, f32), request: &RangeTree) -> LayoutTree {
        let size = request.0.clamp(size);
        self.0.inner.width = Some(size.0);
        LayoutTree(size, vec![])
    }

    fn draw(&self, _sized: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        vec![Instruction { offset, bound, item: Item::Text(self.0.inner.clone()) }]
    }

    fn event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) {
        self.0.event(ctx, sized, event)
    }
}

#[derive(Component, Clone, Debug)]
#[on_dyn_event]
#[caret]
#[after_layout]
pub struct TextEditor {
    layout: Stack,
    text: ExpandableText,
    caret: TextCursor,
    magnifier: Magnifier,
    #[skip]
    scrub: Scrub,
    #[skip]
    id: Uuid,
    #[skip]
    default_span: Span,
    #[skip]
    dirty: Timer,
}
impl TextEditor {
    pub fn new(theme: &Theme, id: Uuid, text: &str, size: TextSize, style: TextStyle, align: Align) -> Self {
        let mut built = ExpandableText::new(theme, "", size, style, align, None);
        let default_span = built.0.inner.spans[0].clone();
        built.0.inner.spans[0].text = text.to_string();
        built.0.inner.cursor = Some(text.len());
        let magnifier = Magnifier::new(theme, style, built.0.inner.clone());
        Self {
            layout: Stack::start(), text: built, caret: TextCursor::new(theme, style, size),
            magnifier, scrub: Scrub::default(), id, default_span, dirty: Timer::default(),
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, Uuid::new_v4(), "", TextSize::Md, TextStyle::Primary, Align::Left)
    }

    pub fn display_cursor(&mut self, display: bool) {
        self.caret.1.display(display);
        if !display {
            self.scrub = Scrub::default();
            self.sync();
        }
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn value(&self) -> Cow<'_, str> {
        match self.text.0.inner.spans.as_slice() {
            [] => Cow::Borrowed(""), [span] => Cow::Borrowed(&span.text),
            spans => Cow::Owned(spans.iter().map(|span| span.text.as_str()).collect()),
        }
    }

    pub fn insert(&mut self, ctx: &mut Context, text: &str) {
        if text.is_empty() { return; }
        let index = Self::clamp_to_boundary(&self.value(), self.cursor.unwrap_or(self.text_len()));
        let inner = &mut self.text.0.inner;
        if inner.spans.is_empty() { inner.spans.push(self.default_span.clone()); }
        let mut offset = index;
        for span in &mut inner.spans {
            if offset <= span.text.len() {
                span.text.insert_str(offset, text);
                break;
            }
            offset -= span.text.len();
        }
        inner.cursor = Some(index + text.len());
        self.changed(ctx);
    }

    pub fn delete_back(&mut self, ctx: &mut Context) {
        let value = self.value();
        let index = Self::clamp_to_boundary(&value, self.cursor.unwrap_or(value.len()));
        let Some((start, _)) = value[..index].grapheme_indices(true).next_back() else { return };
        let inner = &mut self.text.0.inner;
        let mut offset = 0;
        for span in &mut inner.spans {
            let end = offset + span.text.len();
            let from = start.max(offset);
            let to = index.min(end);
            if from < to { span.text.replace_range(from - offset..to - offset, ""); }
            offset = end;
        }
        inner.cursor = Some(start);
        self.changed(ctx);
    }

    pub fn set_text(&mut self, ctx: &mut Context, text: &str) {
        if self.value() == text { return; }
        let inner = &mut self.text.0.inner;
        if inner.spans.is_empty() { inner.spans.push(self.default_span.clone()); }
        inner.spans.truncate(1);
        inner.spans[0].text = text.to_string();
        inner.cursor = Some(text.len());
        self.changed(ctx);
    }

    pub fn pointer_pressed(&mut self, pos: (f32, f32)) {
        self.place_cursor(pos);
        self.scrub = Scrub {
            pressed: true, timer: Timer::new(Duration::from_millis(300)), pointer: pos,
            ..Scrub::default()
        };
        self.sync();
    }

    pub fn pointer_moved(&mut self, pos: Option<(f32, f32)>, delta: Option<(f32, f32)>, touch_controls: bool) {
        if !self.scrub.pressed { return; }

        let step = match (delta, pos) {
            (Some(d), _) => d,
            (None, Some(p)) => (p.0 - self.scrub.pointer.0, p.1 - self.scrub.pointer.1),
            (None, None) => {
                self.scrub = Scrub::default();
                self.sync();
                return;
            }
        };
        self.scrub.pointer = (self.scrub.pointer.0 + step.0, self.scrub.pointer.1 + step.1);
        self.scrub.moved = (self.scrub.moved.0 + step.0, self.scrub.moved.1 + step.1);

        let distance = self.scrub.moved.0.hypot(self.scrub.moved.1);

        if !self.scrub.active {
            if touch_controls {
                if distance > 10.0 { self.scrub = Scrub::default(); }
                return;
            }
            if distance < 3.0 { return; }
            self.scrub.active = true;
            self.scrub.timer.cancel();
        }

        let pointer = self.scrub.pointer;
        self.place_cursor(pointer);
        self.sync();
    }

    pub fn pointer_released(&mut self) {
        self.scrub = Scrub::default();
        self.sync();
    }

    fn caret(&self, layout: &LayoutTree) -> Option<Caret> {
        if !self.caret.1.is_showing() { return None; }
        let mut rect = self.text.0.inner.cursor_rect()?;
        rect.0 += layout.1[0].0.0;
        rect.1 += layout.1[0].0.1;
        Some(Caret { id: self.id, rect })
    }

    fn after_layout(&mut self, layout: &mut LayoutTree) {
        self.sync();
        let (x, y) = self.text.0.inner.cursor_position();
        layout.1[1].0 = (layout.1[0].0.0 + x, layout.1[0].0.1 + y + 2.0);
    }

    fn changed(&mut self, ctx: &mut Context) {
        self.dirty.cancel();
        self.sync();
        ctx.emit(TextInputEvent::Changed { id: self.id(), value: self.value().into_owned() });
    }

    fn clamp_to_boundary(text: &str, index: usize) -> usize {
        let mut index = index.min(text.len());
        while !text.is_char_boundary(index) { index -= 1; }
        index
    }

    fn place_cursor(&mut self, pos: (f32, f32)) { self.text.0.inner.cursor_click(pos.0, pos.1); }

    fn sync(&mut self) {
        if let Some(cursor) = self.text.0.inner.cursor {
            self.text.0.inner.cursor = Some(Self::clamp_to_boundary(&self.value(), cursor));
        }

        self.magnifier.active = self.scrub.active;
        self.magnifier.text = self.text.0.inner.clone();
    }
}

impl TextEditor {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if matches!(event.downcast_ref::<WindowEvent>(), Some(WindowEvent::Focused(false))) {
            self.scrub = Scrub::default();
            self.sync();
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) => {
                    if let Some(position) = mouse.position {
                        ctx.claim_pointer();
                        self.pointer_pressed(position);
                    }
                }
                MouseState::Moved => {
                    self.pointer_moved(mouse.position, None, ctx.window.touch_controls)
                }
                MouseState::Scroll(x, y) if ctx.window.touch_controls => {
                    self.pointer_moved(mouse.position, Some((-x, -y)), ctx.window.touch_controls);
                }
                MouseState::Released(MouseButton::Left) => self.pointer_released(),
                _ => {}
            }
        } else if let Some(KeyboardEvent {
            state: KeyboardState::Pressed | KeyboardState::Repeated, key, ..
        }) = event.downcast_ref()
        {
            match key {
                Key::Character(c) => self.insert(ctx, c), Key::Enter => self.insert(ctx, "\n"),
                Key::Space => self.insert(ctx, " "),
                Key::Delete | Key::Backspace => self.delete_back(ctx),
                _ => {}
            }
        }

        vec![event]
    }
}

impl Deref for TextEditor {
    type Target = BasicText;
    fn deref(&self) -> &Self::Target { &self.text.0.inner }
}

impl DerefMut for TextEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty.reset(Duration::ZERO);
        &mut self.text.0.inner
    }
}

#[on_change(ChangeSource::changed(&mut self.dirty))]
impl OnChange<()> for TextEditor {
    fn on_change(&mut self, ctx: &mut Context, _: ()) { self.changed(ctx); }
}
#[on_change(ChangeSource::changed(&mut self.scrub))]
impl OnChange<()> for TextEditor {
    fn on_change(&mut self, _ctx: &mut Context, _: ()) {
        self.scrub.active = true;
        self.place_cursor(self.scrub.pointer);
        self.sync();
    }
}

#[derive(Component, Debug, Clone)]
pub struct TextCursor(Stack, Opt<Rectangle>);

impl TextCursor {
    pub fn new(theme: &Theme, style: TextStyle, size: TextSize) -> Self {
        let (color, _) = style.get(theme);
        let size = theme.fonts().get_size(size);
        TextCursor(
            Stack(
                Offset::Start, Offset::End, Size::Static(2.0), Size::Static(size),
                Padding::default(),
            ), Opt::new(Rectangle::new(color, 0.0, None), true),
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
            text, active: false, background: colors.get(ptsd::Background::Secondary).into(),
            outline: colors.get(ptsd::Outline::Secondary).into(), cursor: style.get(theme).0.into(),
        }
    }
}

impl Drawable for Magnifier {
    fn size_range(&self) -> RangeTree {
        let h = self.text.size().1;
        RangeTree(
            SizeRange { min_width: 0.0, max_width: f32::MAX, min_height: h, max_height: h }, vec![],
        )
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

        let rect = |x: f32, y: f32, w: f32, h: f32, stroke: f32, radius: f32, color: CanvasColor| {
            Instruction {
                offset: (x, y), bound: (x, y, w, h),
                item: Item::Shape(CanvasShape {
                    stroke, width: w, height: h, angle: 0.0, color,
                    variant: ShapeType::RoundedRectangle(radius),
                }),
            }
        };

        let band = (lens.0, center.1 - line_h * 1.5 / 2.0, w, line_h * 1.5);

        vec![
            rect(lens.0, lens.1, w, h, 0.0, 8.0, self.background),
            Instruction {
                offset: (cursor_x - cx * 1.5, center.1 - focus_y * 1.5), bound: band,
                item: Item::Text(zoomed),
            },
            rect(
                cursor_x - 1.0, center.1 - line_h * 1.5 / 2.0, 2.0, line_h * 1.5, 0.0, 0.0,
                self.cursor,
            ), rect(lens.0, lens.1, w, h, 1.0, 8.0, self.outline),
        ]
    }
}

#[derive(Debug, Clone, Default)]
struct Scrub {
    pressed: bool,
    timer: Timer,
    pointer: (f32, f32),
    moved: (f32, f32),
    active: bool,
}
impl ChangeSource for Scrub {
    type Update = ();
    async fn next(&mut self) -> Option<()> { self.timer.next().await }
}

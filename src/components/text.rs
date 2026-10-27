use prism::drawable::{
    Drawable, Component, LayoutTree, Rect, Instruction, Item, Color as CanvasColor, Text as PrismText,
};
use prism::drawable::text::{Align, Span, Text as BasicText};
use prism::drawable::shape::{Shape as CanvasShape, Variant as ShapeType};
use prism::events::{
    Key, KeyboardEvent, KeyboardState, DynEvent, MouseEvent, MouseState, MouseButton,
    ChangeHandler, ListenCallback, ListenFuture, Timer, WindowEvent,
};
use prism::Context;
use ptsd::display::{Bin, Opt};
use ptsd::layout::{Stack, Size, Offset, Padding};
use ptsd::{theme, FontStyle};
pub use ptsd::TextSize;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use std::any::Any;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut, Range};
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
    inner: PrismText,
}

impl Text {
    pub fn new(theme: &Theme, text: &str, text_size: TextSize, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        let (color, font) = style.get(theme);
        let size = theme.fonts().get_size(text_size);
        let mut inner = PrismText::new(text, font.into(), size, color.into(), align);
        inner.spans[0].line_height = Some(size * 1.25);
        inner.max_lines = max_lines;
        Text { layout: Stack::default(), inner }
    }

    pub fn default(theme: &Theme, text: &str) -> Self {
        Self::new(theme, text, TextSize::H4, TextStyle::Heading, Align::Center, None)
    }

    pub fn fit(mut self) -> Self {self.inner = self.inner.fit(); self}
}
impl Deref for Text {
    type Target = BasicText;
    fn deref(&self) -> &Self::Target { &self.inner }
}
impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

#[derive(Clone, Debug)]
pub struct TextEditor {
    content: Bin<Stack, (Text, TextCursor, Magnifier)>,
    scrub: Scrub,
    default_span: Span,
    dirty: Timer,
}
impl TextEditor {
    pub fn new(theme: &Theme, id: Uuid, text: &str, size: TextSize, style: TextStyle, align: Align) -> Self {
        let mut built = Text::new(theme, "", size, style, align, None);
        built.inner.id = id;
        let default_span = built.inner.spans[0].clone();
        built.inner.spans[0].text = text.to_string();
        built.inner.cursor = Some(text.len());
        let magnifier = Magnifier::new(theme, style, (*built.inner).clone());
        Self {
            content: Bin(Stack::start(), (built, TextCursor::new(theme, style, size), magnifier)),
            scrub: Scrub::default(), default_span, dirty: Timer::default(),
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, Uuid::new_v4(), "", TextSize::Md, TextStyle::Primary, Align::Left)
    }

    pub fn display_cursor(&mut self, display: bool) {
        let inner = &mut self.content.1.0.inner;
        inner.focused = display;
        if !display {
            inner.anchor = None;
            self.scrub = Scrub::default();
        }
        self.sync();
    }

    pub fn id(&self) -> Uuid {self.content.1.0.inner.id}
    pub fn value(&self) -> Cow<'_, str> {
        match self.content.1.0.inner.spans.as_slice() {
            [] => Cow::Borrowed(""), [span] => Cow::Borrowed(&span.text),
            spans => Cow::Owned(spans.iter().map(|span| span.text.as_str()).collect()),
        }
    }

    pub fn insert(&mut self, ctx: &mut Context, text: &str) {
        if text.is_empty() { return; }
        let anchor = self.content.1.0.inner.anchor.take();
        let value = self.value();
        let cursor = Self::clamp_to_boundary(&value, self.cursor.unwrap_or(value.len()));
        let anchor = Self::clamp_to_boundary(&value, anchor.unwrap_or(cursor));
        let index = cursor.min(anchor);
        self.delete_range(index..cursor.max(anchor));
        let inner = &mut self.content.1.0.inner;
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
        let anchor = self.content.1.0.inner.anchor.take();
        let value = self.value();
        let cursor = Self::clamp_to_boundary(&value, self.cursor.unwrap_or(value.len()));
        let anchor = Self::clamp_to_boundary(&value, anchor.unwrap_or(cursor));
        let range = if anchor != cursor {anchor.min(cursor)..anchor.max(cursor)} else {
            let Some((start, _)) = value[..cursor].grapheme_indices(true).next_back() else {return};
            start..cursor
        };
        self.delete_range(range);
        self.changed(ctx);
    }

    pub fn set_text(&mut self, ctx: &mut Context, text: &str) {
        self.content.1.0.inner.anchor = None;
        if self.value() == text { self.sync(); return; }
        let inner = &mut self.content.1.0.inner;
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

    pub fn pointer_moved(&mut self, pos: Option<(f32, f32)>, delta: Option<(f32, f32)>, is_mobile: bool) {
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
            if is_mobile {
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

    fn changed(&mut self, ctx: &mut Context) {
        self.dirty.cancel();
        self.sync();
        ctx.emit(TextInputEvent::Changed { id: self.id(), value: self.value().into_owned() });
    }

    fn delete_range(&mut self, range: Range<usize>) {
        let inner = &mut self.content.1.0.inner;
        let mut offset = 0;
        for span in &mut inner.spans {
            let end = offset + span.text.len();
            let from = range.start.max(offset);
            let to = range.end.min(end);
            if from < to {span.text.replace_range(from - offset..to - offset, "");}
            offset = end;
        }
        inner.cursor = Some(range.start);
    }

    fn clamp_to_boundary(text: &str, index: usize) -> usize {
        let mut index = index.min(text.len());
        while !text.is_char_boundary(index) { index -= 1; }
        index
    }

    fn place_cursor(&mut self, pos: (f32, f32)) { self.content.1.0.inner.cursor_click(pos.0, pos.1); }

    fn sync(&mut self) {
        if let Some(cursor) = self.content.1.0.inner.cursor {
            self.content.1.0.inner.cursor = Some(Self::clamp_to_boundary(&self.value(), cursor));
        }
        if let Some(anchor) = self.content.1.0.inner.anchor {
            self.content.1.0.inner.anchor = Some(Self::clamp_to_boundary(&self.value(), anchor));
        }

        let (text, caret, magnifier) = &mut self.content.1;
        let inner = &*text.inner;
        // Re-enable the custom caret and magnifier when native text overlays are removed.
        let custom_cursor = false;
        caret.1.display(custom_cursor && inner.focused
            && inner.anchor.is_none_or(|anchor| inner.cursor == Some(anchor)));
        magnifier.active = custom_cursor && self.scrub.active;
        magnifier.text = inner.clone();
    }
}

impl Drawable for TextEditor {
    fn request_width(&mut self) -> (f32, f32) {self.content.request_width()}
    fn request_height(&mut self, width: f32) -> (f32, f32) {
        self.sync();
        self.content.request_height(width)
    }
    fn layout(&mut self, size: (f32, f32)) -> LayoutTree {
        let mut layout = Drawable::layout(&mut self.content, size);
        let (x, y) = self.content.1.0.inner.cursor_position();
        layout.1[1].0 = (layout.1[0].0.0 + x, layout.1[0].0.1 + y + 2.0);
        layout
    }
    fn draw(&self, sized: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.content.draw(sized, offset, bound)
    }

    fn event(&mut self, ctx: &mut Context, sized: &LayoutTree, event: Box<dyn DynEvent>) {
        if matches!(event.downcast_ref::<WindowEvent>(), Some(WindowEvent::Focused(false))) {
            self.scrub = Scrub::default();
            self.sync();
        } else if let Some(mouse) = event.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) => {
                    if let Some(position) = mouse.position {self.pointer_pressed(position);}
                }
                MouseState::Moved => {
                    self.pointer_moved(mouse.position, None, ctx.window.is_mobile)
                }
                MouseState::Scroll(x, y) if ctx.window.is_mobile => {
                    self.pointer_moved(mouse.position, Some((-x, -y)), ctx.window.is_mobile);
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

        if event.is::<MouseEvent>() || event.is::<KeyboardEvent>() || event.is::<WindowEvent>() {
            self.content.event(ctx, sized, event);
            return;
        }

        let value = self.value().into_owned();
        let cursor = self.cursor;
        let anchor = self.anchor;
        let focused = self.focused;
        self.content.event(ctx, sized, event);
        if !focused && self.focused {ctx.emit(TextInputEvent::Focus {id: self.id()});}
        if self.value() != value {self.changed(ctx);}
        else if self.cursor != cursor || self.anchor != anchor || self.focused != focused {self.sync();}
    }

    fn changed(&mut self) -> ListenFuture<'_> {
        let (content, dirty, scrub) = (&mut self.content, &mut self.dirty, &mut self.scrub);
        Box::pin(ChangeHandler::select(vec![
            Box::pin(async move {
                let callback = Component::listen(content).await;
                Box::new(move |target: &mut dyn Any, ctx: &mut Context| {
                    callback(&mut target.downcast_mut::<Self>().unwrap().content, ctx);
                }) as ListenCallback
            }),
            Box::pin(async move {
                dirty.next().await;
                Box::new(|target: &mut dyn Any, ctx: &mut Context| {
                    target.downcast_mut::<Self>().unwrap().changed(ctx);
                }) as ListenCallback
            }),
            Box::pin(async move {
                scrub.timer.next().await;
                Box::new(|target: &mut dyn Any, _: &mut Context| {
                    let target = target.downcast_mut::<Self>().unwrap();
                    target.scrub.active = true;
                    target.place_cursor(target.scrub.pointer);
                    target.sync();
                }) as ListenCallback
            }),
        ]))
    }
}

impl Deref for TextEditor {
    type Target = BasicText;
    fn deref(&self) -> &Self::Target { &self.content.1.0.inner }
}

impl DerefMut for TextEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty.reset(Duration::ZERO);
        &mut self.content.1.0.inner
    }
}

#[derive(Component, Debug, Clone)]
pub struct TextCursor(Stack, Opt<Rectangle>);

impl TextCursor {
    pub fn new(theme: &Theme, style: TextStyle, size: TextSize) -> Self {
        let (color, _) = style.get(theme);
        let size = theme.fonts().get_size(size);
        TextCursor(
            Stack::new(
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
    fn request_width(&mut self) -> (f32, f32) {(0.0, f32::MAX)}
    fn request_height(&mut self, width: f32) -> (f32, f32) {
        self.text.width = Some(width);
        let height = self.text.size().1;
        (height, height)
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
        zoomed.anchor = None;
        zoomed.focused = false;
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

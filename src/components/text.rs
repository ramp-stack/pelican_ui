use prism::event::{OnEvent, MouseState, MouseEvent, Event, TickEvent, Key, NamedKey, KeyboardEvent, KeyboardState};
use prism::layout::{Stack, Size, Offset, Padding, SizeRequest};
use prism::display::Opt;
use prism::drawable::{Drawable, Component, SizedTree, RequestTree, Rect}; 
use prism::canvas::{self, Align, Span, Text as BasicText, Area as CanvasArea, Item as CanvasItem};
use prism::Context;

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
        let inner = BasicText::new(vec![Span::new(text.to_string(), size, Some(size*1.25), font.into(), color.into(), 0.0)], None, align, max_lines);
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
pub struct ExpandableText(pub Text, f32);
impl ExpandableText {
    pub fn new(theme: &Theme, text: &str, size: TextSize, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        ExpandableText(Text::new(theme, text, size, style, align, max_lines), 0.0)
    }

    pub fn default(theme: &Theme, text: &str) -> Self {
        Self::new(theme, text, TextSize::H4, TextStyle::Heading, Align::Center, None)
    }
}

impl Drawable for ExpandableText {
    fn request_size(&self) -> RequestTree {
        let size = self.0.inner.size();

        // println!("LINES {:?}, size: {:?}", self.0.inner.len(), size);
        // request needs to be max so that larger texts can know when to wrap
        // but the max should also just be the width of the text itself so that there's not a bunch of extra length
        
        RequestTree(SizeRequest::new(0.0, size.1, f32::MAX, size.1), vec![])
    }

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<(CanvasArea, CanvasItem)> {
        let text = BasicText {spans: self.0.inner.spans.clone(), width: Some(sized.0.0), align: self.0.inner.align, cursor: self.0.inner.cursor, max_lines: self.0.inner.max_lines};
        vec![(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Text(text))]
    }

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) {
        self.0.inner.width = Some(sized.0.0);
        self.0.event(ctx, sized, event)
    }
}

#[derive(Component, Clone)]
pub struct TextEditor(Stack, pub ExpandableText, TextCursor);
impl std::fmt::Debug for TextEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TextEditor").field(&self.0).field(&self.1).field(&self.2).finish()
    }
}

impl TextEditor {
    pub fn new(theme: &Theme, text: &str, size: TextSize, style: TextStyle, align: Align) -> Self {
        let mut built = ExpandableText::new(theme, text, size, style, align, None);
        built.0.inner.cursor = Some(text.len());
        TextEditor(Stack::start(), built, TextCursor::new(theme, style, size))
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "", TextSize::Md, TextStyle::Primary, Align::Left)
    }

    pub fn display_cursor(&mut self, display: bool) {
        self.2.1.display(display)
    }
}

impl OnEvent for TextEditor {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() && self.1.0.inner.cursor.is_some() {
            let cursor_pos = self.1.0.inner.cursor_position();
            *self.2.x_offset() = Offset::Static(cursor_pos.0);
            *self.2.y_offset() = Offset::Static(cursor_pos.1+2.0);
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() && let Some(pos) = event.position && event.state == MouseState::Pressed {
            self.1.0.inner.cursor_click(pos.0, pos.1) 
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref() {
            let index = self.1.0.inner.cursor.unwrap();
            
            let character = match key {
                Key::Character(c) => Some(c.chars().next().unwrap_or_default()),
                Key::Named(NamedKey::Enter) => Some('\n'),
                Key::Named(NamedKey::Space) => Some(' '),
                Key::Named(NamedKey::Delete) => None,
                _ => {return vec![event];}
            };

            match character {
                Some(c) => {
                    match index >= self.1.0.spans[0].len() {
                        true => self.1.0.spans[0].push(c),
                        false => self.1.0.spans[0].insert(index, c),
                    };
                    if let Some(c) = self.1.0.inner.cursor.as_mut() {*c += 1;}
                }
                None => {
                    self.1.0.spans[0] = {
                        let mut chars: Vec<char> = self.1.0.spans[0].chars().collect();

                        match chars.len() {
                            1 => chars.clear(),
                            _ if index >= chars.len() => {chars.pop();},
                            _ => {chars.remove(index);}
                        }

                        chars.into_iter().collect()
                    };
                    if let Some(c) = self.1.0.inner.cursor.as_mut() { *c = c.saturating_sub(1); }
                }
            }

            // (self.3)(ctx, &mut self.1.0.spans[0])
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
use mustache::events::{OnEvent, MouseState, MouseEvent, Event, TickEvent, Key, NamedKey, KeyboardEvent, KeyboardState};
use mustache::layout::{Area, SizeRequest};
use mustache::drawable::{Drawable, Color, Align, Span, Cursor}; //Shape, Cursor
use mustache::drawable::Text as BasicText;
use mustache::{Context, Component, resources};

use crate::layout::{Stack, Offset, Size, Padding, Opt};
use crate::components::Rectangle;
use crate::plugin::PelicanUI;

/// # Text Style
///
/// Represents the different text styles supported by Pelican UI.
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
    pub fn get(&self, ctx: &mut Context) -> (Color, resources::Font) {
        let mut plugin = ctx.get::<PelicanUI>();
        let theme = plugin.get().0.theme();
        let fonts = theme.fonts.fonts.clone();
        match self {
            TextStyle::Heading => (theme.colors.text.heading, fonts.heading.clone()),
            TextStyle::Primary => (theme.colors.text.primary, fonts.text.clone()),
            TextStyle::Secondary => (theme.colors.text.secondary, fonts.text.clone()),
            TextStyle::Error => (theme.colors.status.danger, fonts.text.clone()),
            TextStyle::Keyboard => (theme.colors.text.heading, fonts.keyboard.clone()),
            TextStyle::Label(color) => (*color, fonts.label.clone()),
        }
    }
}

#[derive(Component, Debug)]
pub struct Text {
    layout: Stack,
    inner: BasicText,
    #[skip] pub spans: Vec<String>,
    #[skip] pub size: f32,
    #[skip] pub style: TextStyle,
    #[skip] pub align: Align,
    #[skip] pub max_lines: Option<u32>,
    #[skip] pub kerning: f32,
}

impl OnEvent for Text {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TickEvent>().is_some() {
            let (color, font) = self.style.get(ctx);
            self.inner.align = self.align;
            self.inner.max_lines = self.max_lines;
            self.inner.spans.iter_mut().enumerate().for_each(|(i, s)| {
                s.text = self.spans[i].to_string();
                s.font_size = self.size;
                s.color = color;
                s.font = font.clone();
                s.kerning = self.kerning;
            });
        }
        true
    }
}

impl Text {
    pub fn new(ctx: &mut Context, text: &str, size: f32, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        let (color, font) = style.get(ctx);
        let inner = BasicText::new(vec![Span::new(text.to_string(), size, Some(size*1.25), font, color, 0.0)], None, align, max_lines);
        Text {layout: Stack::default(), inner, spans: vec![text.to_string()], size, style, align, max_lines, kerning: 0.0}
    }
}

/// # Expandable Text
///
/// A text component that expands to take as much horizontal space as possible,  
/// enabling automatic line wrapping and custom text alignment.  
/// Unlike [`Text`], which only sizes to fit its content.
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/expandable_text.png"
///      alt="Expandable Text Example"
///      width="400">
///
/// ## Example
/// ```rust
/// let text = "Greyhounds are gentle, affectionate dogs that love to run.
/// They are known for their incredible speed and calm temperament,
/// making them excellent companions.";
///
/// let text_size = ctx.theme.fonts.size.md;
///
/// let expandable = ExpandableText::new(
///     ctx,
///     text,
///     text_size,
///     TextStyle::Primary,
///     Align::Start,
///     Some(3), // limit to 3 lines before truncation
/// );
/// ```
#[derive(Debug)]
pub struct ExpandableText(pub Text);
impl OnEvent for ExpandableText {}

impl ExpandableText {
    pub fn new(ctx: &mut Context, text: &str, size: f32, style: TextStyle, align: Align, max_lines: Option<u32>) -> Self {
        ExpandableText(Text::new(ctx, text, size, style, align, max_lines))
    }
}

impl Component for ExpandableText {
    fn children_mut(&mut self) -> Vec<&mut dyn Drawable> { vec![&mut self.0] }
    fn children(&self) -> Vec<&dyn Drawable> { vec![&self.0] }

    fn request_size(&self, _ctx: &mut Context, children: Vec<SizeRequest>) -> SizeRequest {
        SizeRequest::new(0.0, children[0].min_height(), f32::MAX, children[0].max_height())
    }

    fn build(&mut self, _ctx: &mut Context, size: (f32, f32), _children: Vec<SizeRequest>) -> Vec<Area> {
        self.0.inner.width = Some(size.0);
        vec![Area{offset: (0.0, 0.0), size}]
    }
}

#[derive(Component, Debug)]
pub struct TextEditor(Stack, pub ExpandableText, TextCursor);

impl TextEditor {
    pub fn new(ctx: &mut Context, text: &str, size: f32, style: TextStyle, align: Align) -> Self {
        let mut text = ExpandableText::new(ctx, text, size, style, align, None);
        text.0.inner.cursor = Some(Cursor::default());
        TextEditor(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default()), text, TextCursor::new(ctx, style, size))
    }


    pub fn apply_edit(&mut self, key: &Key) {
        let index = self.1.0.inner.cursor.unwrap();
        match key {
            Key::Named(NamedKey::Enter) => {
                match index >= self.1.0.spans[0].len() {
                    true => self.1.0.spans[0].push('\n'),
                    false => self.1.0.spans[0].insert(index, '\n'),
                };
                if let Some(c) = self.1.0.inner.cursor.as_mut() {*c += 1};
            },
            Key::Named(NamedKey::Space) => {
                match index >= self.1.0.spans[0].len() {
                    true => self.1.0.spans[0].push(' '),
                    false => self.1.0.spans[0].insert(index, ' '),
                };
                if let Some(c) = self.1.0.inner.cursor.as_mut() {*c += 1};
            },
            Key::Named(NamedKey::Delete | NamedKey::Backspace) => {
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
            },
            Key::Character(c) => {
                match index >= self.1.0.spans[0].len() {
                    true => c.chars().next().map(|ch| self.1.0.spans[0].push(ch)),
                    false => c.chars().next().map(|ch| self.1.0.spans[0].insert(index, ch)),
                };
                if let Some(c) = self.1.0.inner.cursor.as_mut() {*c += 1;}
            },
            _ => {}
        };
    }

    pub fn display_cursor(&mut self, display: bool) {
        self.2.1.display(display)
    }
}

impl OnEvent for TextEditor {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if event.downcast_ref::<TickEvent>().is_some() && self.1.0.inner.cursor.is_some() {
            let cursor_pos = self.1.0.inner.cursor_position();
            *self.2.x_offset() = Offset::Static(cursor_pos.0);
            *self.2.y_offset() = Offset::Static(cursor_pos.1+2.0);
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if event.state == MouseState::Pressed && event.position.is_some() {
                self.1.0.inner.cursor_click(event.position.unwrap().0, event.position.unwrap().1) 
            }
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref() {
            self.apply_edit(key);
        }
        
        true
    }
}

#[derive(Component, Debug)]
pub struct TextCursor(Stack, Opt<Rectangle>);

impl OnEvent for TextCursor {}

impl TextCursor {
    pub fn new(ctx: &mut Context, style: TextStyle, size: f32) -> Self {
        let (color, _) = style.get(ctx);
        TextCursor(
            Stack(Offset::Start, Offset::End, Size::Static(2.0), Size::Static(size), Padding::default()), 
            Opt::new(Rectangle::new(color, 0.0, None), true)
        )
    }

    pub fn x_offset(&mut self) -> &mut Offset { &mut self.0.0 }
    pub fn y_offset(&mut self) -> &mut Offset { &mut self.0.1 }
}

// /// # Bulleted Text
// ///
// /// A component for rendering lists with bullet points.
// ///
// /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/bulleted_text.png"
// ///      alt="Bulleted Text Example"
// ///      width="400">
// ///
// /// ## Example
// ///```rust
// /// let text_size = ctx.theme.fonts.size.md;
// /// let items = vec!["Feed the chairs at midnight.", "Borrow a broom from the moon.", "Vacuum the car inside out.", "Hide the clock beneath the carpet."];
// /// let list = BulletedText::new(ctx, items, TextStyle::Primary, text_size);
// ///```
// #[derive(Debug, Component)]
// pub struct BulletedText(Column, Vec<BulletedTextContent>);

// impl OnEvent for BulletedText {}

// impl BulletedText {
//     pub fn new(ctx: &mut Context, text: Vec<&str>, style: TextStyle, size: f32) -> Self {
//         let color = style.get(ctx).0;
//         let items = text.into_iter().map(|t| BulletedTextContent::new(ctx, t, color, style, size)).collect();
//         BulletedText(Column::center(8.0), items)
//     }
// }

// #[derive(Debug, Component)]
// struct BulletedTextContent(Row, Shape, ExpandableText);
// impl OnEvent for BulletedTextContent {}
// impl BulletedTextContent {
//     fn new(ctx: &mut Context, text: &str, color: Color, style: TextStyle, size: f32) -> Self {
//         BulletedTextContent(
//             Row::new(size*0.75, Offset::Center, Size::Fit, Padding::default()), // change this offset to be line_height - circle size / 2
//             Circle::new(size*0.2, color, false),
//             ExpandableText::new(ctx, text, style, size, Align::Left, None)
//         )
//     }
// }
use rust_on_rails::prelude::*;
use rust_on_rails::prelude::Text as BasicText;
use crate::layout::{Stack, Offset, Size, Padding, Opt, Row};
use crate::elements::shapes::{Rectangle, Circle};
use crate::plugin::PelicanUI;

/// Enumeration of text styles used in the UI.
#[derive(Clone, Copy, Debug)]
pub enum TextStyle {
    /// Represents the heading or title text style. Used for prominent titles or section headers.
    Heading,
    /// Represents the primary body text style. Typically used for main content text.
    Primary,
    /// Represents the secondary text style. Used for secondary or less prominent content.
    Secondary,
    /// Represents the error text style. Often used to indicate error messages or alerts.
    Error,
    /// Represents the white text style. Used when white-colored text is needed.
    White,
    /// Represents the keyboard text style. Used for the keyboard key/label text.
    Keyboard,
    /// Represents a label text style with a custom color. The `Color` parameter allows customization of the text color.
    Label(Color),
}

impl TextStyle {
    /// Retrieves the color and font associated with the `TextStyle`.
    pub fn get(&self, ctx: &mut Context) -> (Color, resources::Font) {
        let theme = &ctx.get::<PelicanUI>().theme;
        match self {
            TextStyle::Heading => (theme.colors.text.heading, theme.fonts.fonts.heading.clone()),
            TextStyle::Primary => (theme.colors.text.primary, theme.fonts.fonts.text.clone()),
            TextStyle::Secondary => (theme.colors.text.secondary, theme.fonts.fonts.text.clone()),
            TextStyle::Error => (theme.colors.status.danger, theme.fonts.fonts.text.clone()),
            TextStyle::White => (theme.colors.text.heading, theme.fonts.fonts.text.clone()),
            TextStyle::Keyboard => (theme.colors.text.heading, theme.fonts.fonts.keyboard.clone()),
            TextStyle::Label(color) => (*color, theme.fonts.fonts.label.clone()),
        }
    }
}

/// Component representing a text element.
#[derive(Component, Debug)]
pub struct Text(Stack, BasicText);
impl OnEvent for Text {}

impl Text {
    /// Creates a new `Text` component with the given text, style, size, and alignment.
    pub fn new(ctx: &mut Context, text: &str, style: TextStyle, size: f32, align: Align) -> Self {
        let (color, font) = style.get(ctx);
        let text = BasicText::new(vec![Span::new(text.to_string(), size, size*1.25, font, color)], None, align, None);
        Text(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default()), text)
    }

    pub fn new_with_cursor(ctx: &mut Context, text: &str, style: TextStyle, size: f32, align: Align) -> Self {
        let (color, font) = style.get(ctx);
        let text = BasicText::new(vec![Span::new(text.to_string(), size, size*1.25, font, color)], None, align, Some(Cursor::default()));
        Text(Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default()), text)
    }

    pub fn text(&mut self) -> &mut BasicText { &mut self.1 }
}


/// Component representing a text element that can expand.
#[derive(Debug)]
pub struct ExpandableText(pub Text);
impl OnEvent for ExpandableText {}

impl ExpandableText {
    /// Creates a new `ExpandableText` component with the given text, style, size, and alignment.
    pub fn new(ctx: &mut Context, text: &str, style: TextStyle, size: f32, align: Align) -> Self {
        ExpandableText(Text::new(ctx, text, style, size, align))
    }

    pub fn new_with_cursor(ctx: &mut Context, text: &str, style: TextStyle, size: f32, align: Align) -> Self {
        ExpandableText(Text::new_with_cursor(ctx, text, style, size, align))
    }

    /// Returns a mutable reference to the [`BasicText`] of the `ExpandableText` component.
    pub fn text(&mut self) -> &mut BasicText { self.0.text() }
}

impl Component for ExpandableText {
    fn children_mut(&mut self) -> Vec<&mut dyn Drawable> { vec![&mut self.0] }
    fn children(&self) -> Vec<&dyn Drawable> { vec![&self.0] }

    fn request_size(&self, ctx: &mut Context, _children: Vec<SizeRequest>) -> SizeRequest {
        let height = self.0.1.size(ctx).1;
        SizeRequest::new(0.0, height, f32::MAX, height)
    }

    fn build(&mut self, _ctx: &mut Context, size: (f32, f32), _children: Vec<SizeRequest>) -> Vec<Area> {
        self.0.text().width = Some(size.0);
        vec![Area{offset: (0.0, 0.0), size}]
    }
}

#[derive(Component, Debug)]
pub struct TextEditor(Stack, Option<Text>, Option<ExpandableText>, TextCursor);

impl TextEditor {
    pub fn new(ctx: &mut Context, text: &str, style: TextStyle, size: f32, align: Align, can_expand: bool) -> Self {
        let (t, et) = match can_expand {
            true => (None, Some(ExpandableText::new_with_cursor(ctx, text, style, size, align))),
            false => (Some(Text::new_with_cursor(ctx, text, style, size, align)), None)
        };

        TextEditor(
            Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default()),
            t, et, TextCursor::new(ctx, style, size)
        )
    }
    
    pub fn text(&mut self) -> &mut BasicText {
        if let Some(text) = &mut self.1 {
            return text.text();
        }

        self.2.as_mut().unwrap().text()
    }

    pub fn cursor(&mut self) -> &mut TextCursor { &mut self.3 }

    pub fn display_cursor(&mut self, display: bool) {
        self.3.display(display);
    }

    pub fn apply_edit(&mut self, ctx: &mut Context, key: &Key) {
        if let Some((i, _)) = self.text().cursor_action(ctx.as_canvas(), CursorAction::GetIndex) {
            let new_text = self.text().spans[0].text.clone();
            println!("HERE {:?}", new_text);
            match key {
                Key::Named(NamedKey::Enter) => {
                    self.text().spans[0].text = Self::insert_char(new_text, '\n', i as usize);
                    self.text().cursor_action(ctx.as_canvas(), CursorAction::MoveNewline);
                },
                Key::Named(NamedKey::Space) => {
                    self.text().spans[0].text = Self::insert_char(new_text, ' ', i as usize);
                    self.text().cursor_action(ctx.as_canvas(), CursorAction::MoveRight);
                },
                Key::Named(NamedKey::Delete | NamedKey::Backspace) => {
                    self.text().cursor_action(ctx.as_canvas(), CursorAction::MoveLeft);
                    self.text().spans[0].text = Self::remove_char(new_text, (i as usize).saturating_sub(1));
                },
                Key::Character(c) => {
                    // self.2.text().text().spans[0].text.insert_str(i as usize , c);
                    let c = c.to_string().chars().next().unwrap();
                    self.text().spans[0].text = Self::insert_char(new_text, c, i as usize);
                    self.text().cursor_action(ctx.as_canvas(), CursorAction::MoveRight);
                },
                _ => {}
            };
        }
    }

    fn insert_char(text: String, new: char, index: usize) -> String {
        let mut chars: Vec<char> = text.chars().collect();
        match index >= chars.len() {
            true => chars.push(new),
            false => chars.insert(index, new)
        }
    
        chars.into_iter().collect()
    }
    
    fn remove_char(text: String, index: usize) -> String {
        let mut chars: Vec<char> = text.chars().collect();
        match chars.len() == 1 {
            true => {chars.clear();},
            false if index >= chars.len() => {chars.pop();},
            false => {chars.remove(index);},
        }
    
        chars.into_iter().collect()
    }
}


impl OnEvent for TextEditor {
    /// Handles events, such as cursor movements or mouse clicks, for the `Text` component.
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        let mut text = self.text().clone();
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(cords) = text.cursor_action(ctx.as_canvas(), CursorAction::GetPosition) {
                *self.3.x_offset() = Offset::Static(cords.0);
                *self.3.y_offset() = Offset::Static(cords.1-(text.spans[0].line_height/1.2));
            }
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if event.state == MouseState::Pressed && event.position.is_some() {
                text.set_cursor(ctx.as_canvas(), (event.position.unwrap().0, event.position.unwrap().1));
                text.cursor_action(ctx.as_canvas(), CursorAction::GetPosition);
            }
        }
        *self.text() = text;
        
        true
    }
}

/// Component representing a text cursor.
#[derive(Component, Debug)]
pub struct TextCursor(Stack, Opt<Rectangle>);

impl OnEvent for TextCursor {}

impl TextCursor {
    /// Creates a new `TextCursor` with the specified style and size.
    pub fn new(ctx: &mut Context, style: TextStyle, size: f32) -> Self {
        let (color, _) = style.get(ctx);
        TextCursor(
            Stack(Offset::Start, Offset::End, Size::Static(2.0), Size::Static(size), Padding::default()), 
            Opt::new(Rectangle::new(color), false)
        )
    }

    /// Displays or hides the cursor.
    pub fn display(&mut self, display: bool) { self.1.display(display) }

    /// Returns the X offset of the cursor.
    pub fn x_offset(&mut self) -> &mut Offset { &mut self.0.0 }

    /// Returns the Y offset of the cursor.
    pub fn y_offset(&mut self) -> &mut Offset { &mut self.0.1 }
}

/// A component that represents bulleted text, combining a row layout, circular bullet, and expandable text.
///
/// The `BulletedText` component is designed to display a piece of text with a bullet (circle) preceding it.
/// The bullet and text are arranged in a row layout, and the text is expandable. This component supports styling
/// through `TextStyle` and alignment via `Align`.
///
/// # Fields
/// - `Row`: A layout component that arranges the bullet and text in a row, with configurable size, alignment, and padding.
/// - `Shape`: A circular bullet shape that precedes the text. The size of the bullet is proportional to the provided `size`.
/// - `ExpandableText`: The text component that holds the actual text, with support for styling and expandable behavior.
///
/// # Example
/// ```rust
/// let ctx: &mut Context = ...;
/// let bulleted_text = BulletedText::new(ctx, "This is a bulleted item.", TextStyle::default(), 20.0, Align::Left);
/// ```
#[derive(Debug, Component)]
pub struct BulletedText(Row, Shape, ExpandableText);

impl OnEvent for BulletedText {}

impl BulletedText {
    /// Creates a new `BulletedText` component.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `text`: The static text to display next to the bullet.
    /// - `style`: The text style used for styling the text.
    /// - `size`: The size of the bullet and text.
    /// - `align`: The alignment of the text.
    ///
    /// # Returns
    /// A new `BulletedText` component containing a row with a bullet and expandable text.
    pub fn new(ctx: &mut Context, text: &str, style: TextStyle, size: f32, align: Align) -> Self {
        let (color, _) = style.get(ctx);
        BulletedText(
            Row::new(size*0.75, Offset::Center, Size::Fit, Padding::default()), // change this offset to be line_height - circle size / 2
            Circle::new(size*0.5, color),
            ExpandableText::new(ctx, text, style, size, align)
        )
    }

    /// Returns a mutable reference to the `BasicText` component inside the `ExpandableText` part of `BulletedText`.
    ///
    /// This method allows direct manipulation of the text within the `ExpandableText` component.
    ///
    /// # Returns
    /// A mutable reference to the `BasicText` component for modifying the text.
    pub fn text(&mut self) -> &mut BasicText { self.2.text() }
}

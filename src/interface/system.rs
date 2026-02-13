use prism::event::Key as WinitKey;
// use roost_ui::maverick_os::hardware::ImageOrientation;
use prism::event::{self, MouseState, TickEvent, KeyboardState, KeyboardEvent, MouseEvent, OnEvent, Event, NamedKey};
use prism::canvas::{self, Align, Image};
use prism::{Context, Request, Hardware};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Stack, Column, Row, Offset, Size, Padding};
use prism::display::Bin;

use ptsd::interfaces::ShowKeyboard;

use crate::theme::{Theme, Color};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon};
use crate::components::button::GhostIconButton;

use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Copy, Clone, Debug, PartialEq)]
enum ButtonState {Default, Pressed}

#[derive(Component, Debug, Clone)]
pub struct MobileKeyboard(Stack, Rectangle, KeyboardContent);
impl OnEvent for MobileKeyboard {}

impl MobileKeyboard {
    pub fn new(theme: &Theme, actions: bool) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| heights[1]);
        let color = theme.colors().get(ptsd::Background::Secondary);
        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()), 
            Rectangle::new(color, 0.0, None),
            KeyboardContent::new(theme, actions)
        )
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardHeader(Column, KeyboardIcons, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardHeader {}

impl KeyboardHeader {
    fn new(theme: &Theme, actions: bool) -> Self {
        let layout = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0,0.0,0.0,2.0));
        KeyboardHeader(Column::start(0.0),
            KeyboardIcons::new(theme, actions),
            Bin(layout, Rectangle::new(theme.colors().get(ptsd::Outline::Secondary), 0.0, None))
        )
    }
}

#[derive(Component, Debug, Clone)]
pub struct KeyboardActions(Stack, Vec<GhostIconButton>);
impl OnEvent for KeyboardActions {}

#[derive(Component, Debug, Clone)]
struct KeyboardIcons(Row, Option<KeyboardActions>, Bin<Stack, Rectangle>, GhostIconButton);

impl KeyboardIcons {
    fn new(theme: &Theme, icons: bool) -> Self {
        // let (sender, _) = mpsc::channel();
        let actions = vec![
            // IconButton::keyboard(ctx, "emoji", |_ctx: &mut Context| ()),
            // IconButton::keyboard(ctx, "gif", |_ctx: &mut Context| ()),
            GhostIconButton::new(theme, "photos", |_ctx: &mut Context, _: &Theme| {}) //ctx.send(Request::Hardware(Hardware::PhotoPicker(sender.clone())))),
            // IconButton::keyboard(ctx, "camera", |_ctx: &mut Context| ()),
        ];

        KeyboardIcons(
            Row::new(16.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)), 
            icons.then(|| KeyboardActions(Stack::default(), actions)),
            Bin (
                Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(1.0),  Padding::default()), 
                Rectangle::new(Color::TRANSPARENT, 0.0, None)
            ),
            GhostIconButton::new(theme, "down_arrow", |ctx: &mut Context, _: &Theme| {
                ctx.send(Request::Event(Box::new(ShowKeyboard(false))));
                ctx.send(Request::Event(Box::new(event::TextInput::Focused(false))));
            }),
        )
    }
}

impl OnEvent for KeyboardIcons {
    // fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
    //     if event.downcast_ref::<TickEvent>().is_some() {
    //         if let Ok((bytes, orientation)) = self.4.try_recv() {
    //             if let Some(s) = EncodedImage::encode(bytes, orientation) {ctx.trigger_event(AttachmentEvent(s));}
    //         }
    //     }
    //     true
    // }
}


// TODO: remove receiver use event instead, default impl for PartialEq
#[derive(Component, Debug, Clone)]
struct KeyboardContent(Column, KeyboardHeader, KeyboardRow, KeyboardRow, KeyboardRow, KeyboardRow);

impl KeyboardContent {
    fn new(theme: &Theme, actions: bool) -> Self {
        let (sender, receiver) = mpsc::channel();
        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0), None),
            KeyboardHeader::new(theme, actions),
            KeyboardRow::top(theme),
            KeyboardRow::middle(theme),
            KeyboardRow::bottom(theme, sender.clone()),
            KeyboardRow::modifier(theme, sender),
        )
    }

    fn update(&mut self) {
        let caps = *self.4.capslock().as_mut().unwrap().status();
        let page = *self.5.paginator().as_mut().unwrap().status();
        self.2.update(top_keys(&page), caps);
        self.3.update(mid_keys(&page), caps);
        self.4.update(bot_keys(&page), caps);
        self.5.update(vec![], caps);
    }
}

impl OnEvent for KeyboardContent {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
        //     match self.6.try_recv() {
        //         Ok(0) => {println!("CAPSLOCK"); self.update();},
        //         Ok(1) => {println!("PAGINATOR"); self.update();},
        //         _ => {}
        //     }
        // }

        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
struct KeyRow(Row, Vec<Key>);
impl OnEvent for KeyRow {}

impl KeyRow {
    fn new(theme: &Theme, keys: Vec<&str>) -> Self {
        let keys = keys.iter().map(|k| Key::character(theme, k)).collect();
        KeyRow(Row::center(0.0), keys)
    }

    fn keys(&mut self) -> &mut Vec<Key> {&mut self.1}
}

#[derive(Component, Debug, Clone)]
struct KeyboardRow(Row, Option<Capslock>, Option<Paginator>, Option<KeyRow>, Option<Key>, Option<Key>);
// Capslock, Paginator, Character Row, Spacebar, Return
impl OnEvent for KeyboardRow {}

impl KeyboardRow {
    fn top(theme: &Theme) -> Self {
        let key_row = KeyRow::new(theme, top_keys(&0));
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn middle(theme: &Theme) -> Self {
        let key_row = KeyRow::new(theme, mid_keys(&0));
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn bottom(theme: &Theme, sender: Sender<u8>) -> Self {
        let capslock = Capslock::new(theme, sender);
        let backspace = Key::backspace(theme);
        let key_row = KeyRow::new(theme, bot_keys(&0));
        KeyboardRow(Row::center(6.0), Some(capslock), None, Some(key_row), None, Some(backspace))
    }

    fn modifier(theme: &Theme, sender: Sender<u8>) -> Self {
        let paginator = Paginator::new(theme, sender);
        let spacebar = Key::spacebar(theme);
        let newline = Key::newline(theme);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }

    fn update(&mut self, new: Vec<&str>, caps_on: bool) {
        let format_text = |text: &str| {
            match caps_on {
                true => text.to_uppercase(),
                false => text.to_lowercase(),
            }
        };
    
        if let Some(spacebar) = &mut self.4 && let Some(text) = spacebar.1.character().get_text().as_mut() {
            text.spans = vec![format_text("space")];
        }
    
        if let Some(newline) = &mut self.5 && let Some(text) = newline.1.character().get_text().as_mut() {
            text.spans = vec![format_text("return")];
        }

        if let Some(keys) = &mut self.3 {
            keys.keys().iter_mut().enumerate().for_each(|(i, k)| {
                if let Some(text) = k.1.character().get_text().as_mut() {
                    text.spans = vec![format_text(new[i])];
                }
                let key = format_text(new[i]);
                k.3 = WinitKey::Character(key.to_string());
            });
        }
    }

    fn capslock(&mut self) -> &mut Option<Capslock> {&mut self.1}
    fn paginator(&mut self) -> &mut Option<Paginator> {&mut self.2}
}

#[derive(Component, Debug, Clone)]
struct Key(Stack, KeyContent, #[skip] ButtonState, #[skip] WinitKey);

impl Key {
    fn character(theme: &Theme, c: &str) -> Self {
        let character = KeyCharacter::char(theme, c);
        let content = KeyContent::new(33.0, Offset::End, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Character(c.to_string()))
    }

    fn spacebar(theme: &Theme) -> Self {
        let character = KeyCharacter::text(theme, "space");
        let content = KeyContent::new(f32::MAX, Offset::Center, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Named(NamedKey::Space))
    }

    fn backspace(theme: &Theme) -> Self {
        let character = KeyCharacter::icon(theme, "backspace");
        let content = KeyContent::new(42.0, Offset::Center, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Named(NamedKey::Delete))
    }

    fn newline(theme: &Theme) -> Self {
        let character = KeyCharacter::text(theme, "return");
        let content = KeyContent::new(92.0, Offset::Center, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Named(NamedKey::Enter))
    }
}

impl OnEvent for Key {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.2 = handle_state(self.2, *event);

            *self.1.background() = match self.2 {
                ButtonState::Default => Color::from_hex("ffffff", 110).into(),
                ButtonState::Pressed => Color::from_hex("ffffff", 180).into(),
            };

            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                ctx.send(Request::Hardware(Hardware::Haptic));
                ctx.send(Request::Event(Box::new(KeyboardEvent{state: KeyboardState::Pressed, key: self.3.clone()})))
            }

            return Vec::new();
        }
        vec![event]
    }
}

#[derive(Component, Clone)]
struct Capslock(Stack, KeyContent, #[skip] ButtonState, #[skip] bool, #[skip] Sender<u8>);

impl Capslock {
    fn new(theme: &Theme, sender: Sender<u8>) -> Self {
        let character = KeyCharacter::icon(theme, "capslock");
        let content = KeyContent::new(42.0, Offset::Center, character);
        Capslock(Stack::default(), content, ButtonState::Default, false, sender)
    }

    fn status(&mut self) -> &mut bool {&mut self.3}
}

impl std::fmt::Debug for Capslock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Capslock(...)")
    }
}

impl OnEvent for Capslock {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.2 = handle_state(self.2, *event);

            *self.1.background() = match self.2 {
                ButtonState::Default => Color::from_hex("ffffff", 110).into(),
                ButtonState::Pressed => Color::from_hex("ffffff", 180).into(),
            };

            // if event.state == MouseState::Pressed && event.position.is_some() {
            //     self.3 = !self.3;
            //     let icon = if self.3 { "capslock_on" } else { "capslock" };
            //     *self.1.character() = KeyCharacter::icon(ctx, icon);
            // }

            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                self.4.send(0).unwrap();
            }

            return Vec::new();
        }
        vec![event]
    }
}

#[derive(Component, Clone)]
struct Paginator(Stack, KeyContent, #[skip] ButtonState, #[skip] u32, #[skip] Sender<u8>);

impl Paginator {
    fn new(theme: &Theme, sender: Sender<u8>) -> Self {
        let character = KeyCharacter::paginator(theme, 0);
        let content = KeyContent::new(92.0, Offset::Center, character);
        Paginator(Stack::default(), content, ButtonState::Default, 0, sender)
    }

    fn status(&mut self) -> &mut u32 {&mut self.3}
}

impl std::fmt::Debug for Paginator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Paginator(...)")
    }
}

impl OnEvent for Paginator {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.2 = handle_state(self.2, *event);

            *self.1.background() = match self.2 {
                ButtonState::Default => Color::from_hex("ffffff", 110).into(),
                ButtonState::Pressed => Color::from_hex("ffffff", 180).into(),
            };

            if event.state == MouseState::Pressed && event.position.is_some() {
                let highlight = TextStyle::Keyboard;
                let dim = TextStyle::Secondary;
                let next = if self.3 == 2 { 0 } else { self.3 + 1 };
                self.3 = next;

                let styles = match next {
                    0 => (highlight, dim, dim),
                    1 => (dim, highlight, dim),
                    _ => (dim, dim, highlight),
                };

                self.1.character().2.as_mut().unwrap().style = styles.0;
                self.1.character().3.as_mut().unwrap().style = styles.1;
                self.1.character().4.as_mut().unwrap().style = styles.2;
            }

            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                self.4.send(1).unwrap()
            }

            return Vec::new();
        }

        vec![event]
    }
}

#[derive(Component, Debug, Clone)]
struct KeyContent(Stack, Rectangle, KeyCharacter);
impl OnEvent for KeyContent {}

impl KeyContent {
    fn new(size: f32, offset: Offset, content: KeyCharacter) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, size));
        KeyContent(
            Stack(Offset::Center, offset, width, Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            Rectangle::new(Color::from_hex("ffffff", 110), 4.0, None),
            content
        )
    }

    fn background(&mut self) -> &mut canvas::Color {self.1.background()}
    fn character(&mut self) -> &mut KeyCharacter {&mut self.2}
}

#[derive(Component, Debug, Clone)]
struct KeyCharacter(Row, Option<Image>, Option<Text>, Option<Text>, Option<Text>);
impl OnEvent for KeyCharacter {}

impl KeyCharacter {
    fn char(theme: &Theme, key: &str) -> Self {
        KeyCharacter(
            Row::new(0.0, Offset::Center, Size::Fit, Padding(0.0, 0.0, 0.0, 10.0)),
            None,
            Some(Text::new(theme, key, TextSize::Xl, TextStyle::Keyboard, Align::Left, None)),
            None, None
        )
    }

    fn text(theme: &Theme, key: &str) -> Self {
        KeyCharacter(Row::center(0.0), None, Some(Text::new(theme, key, TextSize::Md, TextStyle::Keyboard, Align::Left, None)), None, None)
    }

    fn icon(theme: &Theme, i: &'static str) -> Self {
        let c = theme.colors().get(ptsd::Text::Heading);
        KeyCharacter(Row::center(0.0), Some(Icon::new(theme, i, Some(c), 36.0)), None, None, None)
    }

    fn paginator(theme: &Theme, page: u32) -> Self {
        let (highlight, dim) = (TextStyle::Keyboard, TextStyle::Secondary);

        let styles = match page {
            0 => (highlight, dim, dim),
            1 => (dim, highlight, dim),
            _ => (dim, dim, highlight),
        };

        KeyCharacter(
            Row::center(1.0),
            None,
            Some(Text::new(theme, "•", TextSize::H2, styles.0, Align::Left, None)),
            Some(Text::new(theme, "•", TextSize::H2, styles.1, Align::Left, None)),
            Some(Text::new(theme, "•", TextSize::H2, styles.2, Align::Left, None)),
        )
    }

    fn get_text(&mut self) -> &mut Option<Text> {&mut self.2}
}

fn handle_state(state: ButtonState, event: MouseEvent) -> ButtonState {
    match state {
        ButtonState::Default if event.position.is_some() => {
            match event.state {
                MouseState::Pressed => Some(ButtonState::Pressed),
                MouseState::Released => Some(ButtonState::Default),
                _ => None,
            }
        },
        ButtonState::Pressed => {
            match event.state {
                MouseState::Released => Some(ButtonState::Default),
                MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
                _ => None,
            }
        },
        _ => None
    }.unwrap_or(state)
}

fn top_keys(page: &u32) -> Vec<&str> {
    match page {
        0 => vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
        1 => vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
        _ => vec!["[", "]", "{", "}", "(", ")", "<", ">", "+", "="]
    }
}

fn mid_keys(page: &u32) -> Vec<&str> {
    match page {
        0 => vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"],
        1 => vec!["/", "\\", "\"", "'", "~", ".", ",", "?", "!"],
        _ => vec!["-", ":", ";", "#", "%", "$", "&", "^", "*",]
    }  
}

fn bot_keys(page: &u32) -> Vec<&str> {
    match page {
        0 => vec!["z", "x", "c", "v", "b", "n", "m"],
        1 => vec!["@", "|", "`", "˚", "€", "£", "¥"],
        _ => vec!["™", "©", "•", "¶", "€", "£", "¥"]
    }  
}

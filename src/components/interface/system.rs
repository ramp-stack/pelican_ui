use mustache::events::Key as WinitKey;
// use mustache::maverick_os::hardware::ImageOrientation;
use mustache::events::{MouseState, TickEvent, KeyboardState, KeyboardEvent, MouseEvent, OnEvent, Event, NamedKey, SmolStr};
use mustache::drawable::{Align, Image, Color};
use mustache::{Context, Component};

use crate::components::interface::mobile::ShowKeyboard;
use crate::components::{Text, TextStyle, Rectangle, Icon};
use crate::layout::{Stack, Bin, Column, Row, Offset, Size, Padding};
use crate::components::interactions::ButtonState;
use crate::components::button::GhostIconButton;
use crate::plugin::PelicanUI;

use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Component, Debug)]
pub struct MobileKeyboard(Stack, Rectangle, KeyboardContent);
impl OnEvent for MobileKeyboard {}

impl MobileKeyboard {
    pub fn new(ctx: &mut Context, actions: bool) -> Self {
        let height = Size::custom(|heights: Vec<(f32, f32)>| heights[1]);
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.background.secondary;
        MobileKeyboard(
            Stack(Offset::Start, Offset::Start, Size::Fill, height, Padding::default()), 
            Rectangle::new(color, 0.0, None),
            KeyboardContent::new(ctx, actions)
        )
    }
}

#[derive(Component, Debug)]
struct KeyboardHeader(Column, KeyboardIcons, Bin<Stack, Rectangle>);
impl OnEvent for KeyboardHeader {}

impl KeyboardHeader {
    fn new(ctx: &mut Context, actions: bool) -> Self {
        let layout = Stack(Offset::default(), Offset::default(), Size::Fit, Size::Static(1.0), Padding(0.0,0.0,0.0,2.0));
        KeyboardHeader(Column::start(0.0),
            KeyboardIcons::new(ctx, actions),
            Bin(layout, Rectangle::new(ctx.get::<PelicanUI>().get().0.theme().colors.outline.secondary, 0.0, None))
        )
    }
}

#[derive(Component, Debug)]
pub struct KeyboardActions(Stack, Vec<GhostIconButton>);
impl OnEvent for KeyboardActions {}

#[derive(Component, Debug)]
struct KeyboardIcons(Row, Option<KeyboardActions>, Bin<Stack, Rectangle>, GhostIconButton);

impl KeyboardIcons {
    fn new(ctx: &mut Context, icons: bool) -> Self {
        let (sender, _) = mpsc::channel();
        let actions = vec![
            // IconButton::keyboard(ctx, "emoji", |_ctx: &mut Context| ()),
            // IconButton::keyboard(ctx, "gif", |_ctx: &mut Context| ()),
            GhostIconButton::new(ctx, "photos", move |ctx: &mut Context| ctx.hardware.open_photo_picker(sender.clone())),
            // IconButton::keyboard(ctx, "camera", |_ctx: &mut Context| ()),
        ];

        KeyboardIcons(
            Row::new(16.0, Offset::Start, Size::Fit, Padding(12.0, 6.0, 12.0, 6.0)), 
            icons.then(|| KeyboardActions(Stack::default(), actions)),
            Bin (
                Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(1.0),  Padding::default()), 
                Rectangle::new(Color::TRANSPARENT, 0.0, None)
            ),
            GhostIconButton::new(ctx, "down_arrow", |ctx: &mut Context| ctx.trigger_event(ShowKeyboard(false))),
        )
    }
}

impl OnEvent for KeyboardIcons {
    // fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
    //     if event.downcast_ref::<TickEvent>().is_some() {
    //         if let Ok((bytes, orientation)) = self.4.try_recv() {
    //             if let Some(s) = EncodedImage::encode(bytes, orientation) {ctx.trigger_event(AttachmentEvent(s));}
    //         }
    //     }
    //     true
    // }
}


#[derive(Component, Debug)]
struct KeyboardContent(Column, KeyboardHeader, KeyboardRow, KeyboardRow, KeyboardRow, KeyboardRow, #[skip] Receiver<u8>);

impl KeyboardContent {
    fn new(ctx: &mut Context, actions: bool) -> Self {
        let (sender, receiver) = mpsc::channel();
        KeyboardContent(
            Column::new(0.0, Offset::Center, Size::Fit, Padding(8.0, 8.0, 8.0, 8.0)),
            KeyboardHeader::new(ctx, actions),
            KeyboardRow::top(ctx),
            KeyboardRow::middle(ctx),
            KeyboardRow::bottom(ctx, sender.clone()),
            KeyboardRow::modifier(ctx, sender),
            receiver
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
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            match self.6.try_recv() {
                Ok(0) => {println!("CAPSLOCK"); self.update();},
                Ok(1) => {println!("PAGINATOR"); self.update();},
                _ => {}
            }
            
            true
        } else {true}
    }
}

#[derive(Component, Debug)]
struct KeyRow(Row, Vec<Key>);
impl OnEvent for KeyRow {}

impl KeyRow {
    fn new(ctx: &mut Context, keys: Vec<&str>) -> Self {
        let keys = keys.iter().map(|k| Key::character(ctx, k)).collect();
        KeyRow(Row::center(0.0), keys)
    }

    fn keys(&mut self) -> &mut Vec<Key> {&mut self.1}
}

#[derive(Component, Debug)]
struct KeyboardRow(Row, Option<Capslock>, Option<Paginator>, Option<KeyRow>, Option<Key>, Option<Key>);
// Capslock, Paginator, Character Row, Spacebar, Return
impl OnEvent for KeyboardRow {}

impl KeyboardRow {
    fn top(ctx: &mut Context) -> Self {
        let key_row = KeyRow::new(ctx, top_keys(&0));
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn middle(ctx: &mut Context) -> Self {
        let key_row = KeyRow::new(ctx, mid_keys(&0));
        KeyboardRow(Row::center(0.0), None, None, Some(key_row), None, None)
    }

    fn bottom(ctx: &mut Context, sender: Sender<u8>) -> Self {
        let capslock = Capslock::new(ctx, sender);
        let backspace = Key::backspace(ctx);
        let key_row = KeyRow::new(ctx, bot_keys(&0));
        KeyboardRow(Row::center(6.0), Some(capslock), None, Some(key_row), None, Some(backspace))
    }

    fn modifier(ctx: &mut Context, sender: Sender<u8>) -> Self {
        let paginator = Paginator::new(ctx, sender);
        let spacebar = Key::spacebar(ctx);
        let newline = Key::newline(ctx);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }

    fn update(&mut self, new: Vec<&str>, caps_on: bool) {
        let format_text = |text: &str| {
            match caps_on {
                true => text.to_uppercase(),
                false => text.to_lowercase(),
            }
        };
    
        if let Some(spacebar) = &mut self.4 {
            if let Some(text) = spacebar.1.character().get_text().as_mut() {
                text.spans = vec![format_text("space")];
            }
        }
    
        if let Some(newline) = &mut self.5 {
            if let Some(text) = newline.1.character().get_text().as_mut() {
                text.spans = vec![format_text("return")];
            }
        }

        if let Some(keys) = &mut self.3 {
            keys.keys().iter_mut().enumerate().for_each(|(i, k)| {
                if let Some(text) = k.1.character().get_text().as_mut() {
                    text.spans = vec![format_text(new[i])];
                }
                let key = format_text(new[i]);
                k.3 = WinitKey::Character(SmolStr::new(key.as_str()));
            });
        }
    }

    fn capslock(&mut self) -> &mut Option<Capslock> {&mut self.1}
    fn paginator(&mut self) -> &mut Option<Paginator> {&mut self.2}
}

#[derive(Component, Debug)]
struct Key(Stack, KeyContent, #[skip] ButtonState, #[skip] WinitKey);

impl Key {
    fn character(ctx: &mut Context, c: &str) -> Self {
        let character = KeyCharacter::char(ctx, c);
        let content = KeyContent::new(ctx, 33.0, Offset::End, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Character(SmolStr::new_inline(c)))
    }

    fn spacebar(ctx: &mut Context) -> Self {
        let character = KeyCharacter::text(ctx, "space");
        let content = KeyContent::new(ctx, f32::MAX, Offset::Center, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Named(NamedKey::Space))
    }

    fn backspace(ctx: &mut Context) -> Self {
        let character = KeyCharacter::icon(ctx, "backspace");
        let content = KeyContent::new(ctx, 42.0, Offset::Center, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Named(NamedKey::Backspace))
    }

    fn newline(ctx: &mut Context) -> Self {
        let character = KeyCharacter::text(ctx, "return");
        let content = KeyContent::new(ctx, 92.0, Offset::Center, character);
        Key(Stack::default(), content, ButtonState::Default, WinitKey::Named(NamedKey::Enter))
    }
}

impl OnEvent for Key {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.2 = handle_state(ctx, self.2, *event);

            *self.1.background() = match self.2 {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 180),
                _ => Color::from_hex("ffffff", 110),
            };

            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                match self.2 {
                    ButtonState::Default | ButtonState::Hover | ButtonState::Pressed => {
                        ctx.hardware.haptic();
                        ctx.trigger_event(KeyboardEvent{state: KeyboardState::Pressed, key: self.3.clone()})
                    },
                    _ => {}
                }
            }

            false
        } else {true}

    }
}

#[derive(Component)]
struct Capslock(Stack, KeyContent, #[skip] ButtonState, #[skip] bool, #[skip] Sender<u8>);

impl Capslock {
    fn new(ctx: &mut Context, sender: Sender<u8>) -> Self {
        let character = KeyCharacter::icon(ctx, "capslock");
        let content = KeyContent::new(ctx, 42.0, Offset::Center, character);
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
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.2 = handle_state(ctx, self.2, *event);

            *self.1.background() = match self.2 {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 180),
                _ => Color::from_hex("ffffff", 110),
            };

            if event.state == MouseState::Pressed && event.position.is_some() {
                self.3 = !self.3;
                let icon = if self.3 { "capslock_on" } else { "capslock" };
                *self.1.character() = KeyCharacter::icon(ctx, icon);
            }

            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                match self.2 {
                    ButtonState::Default | ButtonState::Hover | ButtonState::Pressed => {
                        // ctx.hardware.vibrate();
                        self.4.send(0).unwrap();
                    }
                    _ => {}
                }
            }
            false
        } else {true}
    }
}

#[derive(Component)]
struct Paginator(Stack, KeyContent, #[skip] ButtonState, #[skip] u32, #[skip] Sender<u8>);

impl Paginator {
    fn new(ctx: &mut Context, sender: Sender<u8>) -> Self {
        let character = KeyCharacter::paginator(ctx, 0);
        let content = KeyContent::new(ctx, 92.0, Offset::Center, character);
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
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            self.2 = handle_state(ctx, self.2, *event);

            *self.1.background() = match self.2 {
                ButtonState::Default => Color::from_hex("ffffff", 110),
                ButtonState::Pressed => Color::from_hex("ffffff", 180),
                _ => Color::from_hex("ffffff", 110),
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
                match self.2 {
                    ButtonState::Default | ButtonState::Hover | ButtonState::Pressed => self.4.send(1).unwrap(),
                    _ => {}
                }
            }
            false
        } else {true}
    }
}

#[derive(Component, Debug)]
struct KeyContent(Stack, Rectangle, KeyCharacter);
impl OnEvent for KeyContent {}

impl KeyContent {
    fn new(_ctx: &mut Context, size: f32, offset: Offset, content: KeyCharacter) -> Self {
        let width = Size::custom(move |widths: Vec<(f32, f32)>|(widths[0].0, size));
        KeyContent(
            Stack(Offset::Center, offset, width, Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0)),
            Rectangle::new(Color::from_hex("ffffff", 110), 4.0, None),
            content
        )
    }

    fn background(&mut self) -> &mut Color {self.1.background()}
    fn character(&mut self) -> &mut KeyCharacter {&mut self.2}
}

#[derive(Component, Debug)]
struct KeyCharacter(Row, Option<Image>, Option<Text>, Option<Text>, Option<Text>);
impl OnEvent for KeyCharacter {}

impl KeyCharacter {
    fn char(ctx: &mut Context, key: &str) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.xl;
        KeyCharacter(
            Row::new(0.0, Offset::Center, Size::Fit, Padding(0.0, 0.0, 0.0, 10.0)),
            None,
            Some(Text::new(ctx, key, size, TextStyle::Keyboard, Align::Left, None)),
            None, None
        )
    }

    fn text(ctx: &mut Context, key: &str) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.md;
        KeyCharacter(Row::center(0.0), None, Some(Text::new(ctx, key, size, TextStyle::Keyboard, Align::Left, None)), None, None)
    }

    fn icon(ctx: &mut Context, i: &'static str) -> Self {
        let c = ctx.get::<PelicanUI>().get().0.theme().colors.text.heading;
        KeyCharacter(Row::center(0.0), Some(Icon::new(ctx, i, c, 36.0)), None, None, None)
    }

    fn paginator(ctx: &mut Context, page: u32) -> Self {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size.h2;
        let (highlight, dim) = (TextStyle::Keyboard, TextStyle::Secondary);

        let styles = match page {
            0 => (highlight, dim, dim),
            1 => (dim, highlight, dim),
            _ => (dim, dim, highlight),
        };

        KeyCharacter(
            Row::center(1.0),
            None,
            Some(Text::new(ctx, "•", size, styles.0, Align::Left, None)),
            Some(Text::new(ctx, "•", size, styles.1, Align::Left, None)),
            Some(Text::new(ctx, "•", size, styles.2, Align::Left, None)),
        )
    }

    fn get_text(&mut self) -> &mut Option<Text> {&mut self.2}
}

fn handle_state(_ctx: &mut Context, state: ButtonState, event: MouseEvent) -> ButtonState {
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

use prism::events::{DynEvent, ListenFuture, KeyboardState, KeyboardEvent};
use prism::drawable::text::Align;
use prism::{Context, Window};
use ptsd::emitters;
use prism::drawable::{Drawable, Component, RangeTree, SizeRange, LayoutTree, Rect, Instruction, Item, Area};
use prism::drawable::shape::{Shape, Variant};
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding};
use ptsd::display::{Bin, Enum, Dismissible, DismissEdge};
use ptsd::interfaces::ShowKeyboard;
use ptsd::interactions;

use std::any::Any;

use crate::theme::{Theme, Color, Icons};
use crate::components::text::{Text, TextStyle, TextSize};
use crate::components::{Rectangle, Icon};

use super::key::InputKey;

#[derive(Component, Debug, Clone)]
#[on_dyn_event]
#[listen]
pub struct CustomKeyboard {
    layout: Stack,
    panel: Dismissible<KeyboardBody>,
    #[skip] caps: bool,
    #[skip] keys_page: usize,
    #[skip] theme: Theme,
}

impl CustomKeyboard {
    pub fn new(ctx: &Context, theme: &Theme) -> Self {
        let color = theme.colors().get(ptsd::Background::Secondary);
        Self {
            layout: Stack::default(),
            panel: Dismissible::new(KeyboardBody {
                layout: Stack::new(Offset::Start, Offset::Start, Size::Fill,
                    Size::custom(|heights| heights[1]), Padding::default()),
                background: KeyboardBackground {color: color.into(), window: ctx.window},
                content: Bin(
                    Stack(Offset::Center, Offset::Start, Size::Fill, Size::Fit, Padding(8.0, 22.0, 8.0, 8.0)),
                    KeyRows::new(theme, 0, false),
                ),
            }, DismissEdge::Bottom),
            caps: false, keys_page: 0, theme: theme.clone(),
        }
    }

    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let refresh = match event.downcast_ref::<KeyAction>() {
            Some(KeyAction::Capslock(caps)) => {self.caps = *caps; true}
            Some(KeyAction::Paginator(page)) => {self.keys_page = *page; true}
            None => if self.caps && matches!(event.downcast_ref::<KeyboardEvent>(),
                Some(KeyboardEvent {
                    key: prism::events::Key::Character(_), state: KeyboardState::Pressed, ..
                })) {
                self.caps = false;
                true
            } else {false},
        };
        if refresh {self.panel.child.content.1 = KeyRows::new(&self.theme, self.keys_page, self.caps);}
        vec![event]
    }

    pub fn on_interface_event(&mut self, ctx: &Context, event: &mut dyn DynEvent, showing: bool) {
        self.panel.child.background.window = ctx.window;
        if event.downcast_ref::<ptsd::navigation::NavigationEvent>().is_some()
            || event.downcast_ref::<ShowKeyboard>().is_some() {self.panel.reset();}
        let size = self.panel.size();
        let area = showing.then_some(Area {offset: (0.0, ctx.window.area().size.1 - size.1), size});
        self.panel.observe(event, area, None);
    }
}

impl CustomKeyboard {
    fn listen(&mut self) -> ListenFuture<'_> {Drawable::listen(&mut self.panel)}
    fn update(&mut self, ctx: &mut Context, path: Vec<usize>, update: Box<dyn Any>) {
        Drawable::update(&mut self.panel, ctx, path, update);
        if self.panel.advance() {
            ctx.emit(ShowKeyboard(false));
            ctx.emit(ptsd::events::TextInput::Focused(false));
        }
    }
}

#[derive(Debug, Clone)]
enum KeyAction {Capslock(bool), Paginator(usize)}
impl prism::events::Event for KeyAction {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum ButtonState {Default, Pressed}

impl InputKey {
    fn character(theme: &Theme, character: char) -> Self {
        let default = _Key::character(theme, &character.to_string(), ButtonState::Default, true);
        let pressed = _Key::character(theme, &character.to_string(), ButtonState::Pressed, true);
        Self::new(default, pressed, prism::events::Key::Character(character.to_string()), false)
    }

    fn spacebar(theme: &Theme, caps: bool) -> Self {
        Self::new(_Key::spacebar(theme, caps, ButtonState::Default),
            _Key::spacebar(theme, caps, ButtonState::Pressed), prism::events::Key::Space, false)
    }

    fn newline(theme: &Theme, caps: bool) -> Self {
        Self::new(_Key::newline(theme, caps, ButtonState::Default, "return"),
            _Key::newline(theme, caps, ButtonState::Pressed, "return"), prism::events::Key::Enter, false)
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardBody {
    layout: Stack,
    background: KeyboardBackground,
    content: Bin<Stack, KeyRows>,
}

#[derive(Component, Debug, Clone)]
struct KeyRow(Row, Vec<InputKey>);

impl KeyRow {
    fn new(theme: &Theme, keys: Vec<&str>, caps_on: bool) -> Self {
        let keys = keys.iter().map(|k| {
            InputKey::character(theme, match caps_on {
                true => k.to_uppercase(),
                false => k.to_lowercase(),
            }.chars().next().unwrap_or_default())
        }).collect();
        KeyRow(Row::center(0.0), keys)
    }
}

#[derive(Component, Debug, Clone)]
struct KeyboardRow(
    Row, Option<Capslock>, Option<Paginator>, Option<KeyRow>, Option<InputKey>, Option<InputKey>,
);

impl KeyboardRow {
    fn top(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let keys = match num {
            0 => vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
            1 => vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
            _ => vec!["[", "]", "{", "}", "(", ")", "<", ">", "+", "="]
        };
        KeyboardRow(Row::center(0.0), None, None, Some(KeyRow::new(theme, keys, caps_on)), None, None)
    }

    fn middle(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let keys = match num {
            0 => vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"],
            1 => vec!["/", "\\", "\"", "'", "~", ".", ",", "?", "!"],
            _ => vec!["-", ":", ";", "#", "%", "$", "&", "^", "*",]
        };
        KeyboardRow(Row::center(0.0), None, None, Some(KeyRow::new(theme, keys, caps_on)), None, None)
    }

    fn bottom(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let keys = match num {
            0 => vec!["z", "x", "c", "v", "b", "n", "m"],
            1 => vec!["@", "|", "`", "˚", "€", "£", "¥"],
            _ => vec!["™", "©", "•", "¶", "€", "£", "¥"]
        };
        let capslock = Capslock::new(theme, caps_on);
        let backspace = InputKey::backspace(theme, 48.0);
        KeyboardRow(
            Row::center(6.0), Some(capslock), None,
            Some(KeyRow::new(theme, keys, caps_on)), None, Some(backspace),
        )
    }

    fn modifier(theme: &Theme, num: usize, caps_on: bool) -> Self {
        let paginator = Paginator::new(theme, num);
        let spacebar = InputKey::spacebar(theme, caps_on);
        let newline = InputKey::newline(theme, caps_on);
        KeyboardRow(Row::center(6.0), None, Some(paginator), None, Some(spacebar), Some(newline))
    }
}

#[derive(Component, Debug, Clone)]
struct KeyRows(Column, KeyboardRow, KeyboardRow, KeyboardRow, KeyboardRow);

impl KeyRows {
    fn new(theme: &Theme, page: usize, caps: bool) -> Self {
        KeyRows(
            Column::new(0.0, Offset::Center, Size::Fit, Padding::default(), None),
            KeyboardRow::top(theme, page, caps),
            KeyboardRow::middle(theme, page, caps),
            KeyboardRow::bottom(theme, page, caps),
            KeyboardRow::modifier(theme, page, caps),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Capslock(Stack, interactions::Selectable);
impl Capslock {
    fn new(theme: &Theme, is_on: bool) -> Self {
        let key = |state| interactions::Button::new(
            _Key::capslock(theme, state), None::<_Key>, None::<_Key>, None::<_Key>, None::<_Key>,
            move |ctx: &mut Context| ctx.emit(KeyAction::Capslock(state != ButtonState::Pressed)), false,
        );
        let selected = key(ButtonState::Pressed);
        let default = key(ButtonState::Default);

        let selectable = interactions::Selectable::new(
            default, selected, is_on, true, Box::new(|_: &mut Context| {}), uuid::Uuid::new_v4(),
        );

        Capslock(Stack::default(), selectable)
    }
}

#[derive(Component, Debug, Clone)]
enum _Key {
    Character {layout: Stack, background: Option<Rectangle>, text: Bin<Stack, Text>},
    Spacebar {layout: Stack, background: Rectangle, text: Text},
    Capslock {layout: Stack, background: Rectangle, icon: Icon},
    Paginator {layout: Stack, background: Rectangle, content: PaginatorContent},
    Newline {layout: Stack, background: Rectangle, text: Text},
}

impl _Key {
    fn character(theme: &Theme, character: &str, state: ButtonState, background: bool) -> Self {
        _Key::Character {
            layout: Stack(
                Offset::Center, Offset::End, Size::Static(30.0), Size::Static(48.0),
                Padding(3.0, 6.0, 3.0, 6.0),
            ),
            background: background.then_some(Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 40),
                ButtonState::Pressed => Color::from_hex("ffffff", 57)
            }, 4.0, None)),
            text: Bin(
                Stack(
                    Offset::default(), Offset::default(), Size::default(), Size::default(),
                    Padding(0.0, 0.0, 0.0, 10.0),
                ),
                Text::new(theme, character, TextSize::Xl, TextStyle::Keyboard, Align::Left, None)
            )
        }
    }

    fn spacebar(theme: &Theme, caps_on: bool, state: ButtonState) -> Self {
        _Key::Spacebar {
            layout: Stack(
                Offset::Center, Offset::Center,
                Size::custom(|widths: Vec<(f32, f32)>| (widths[1].0, f32::MAX)),
                Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0),
            ),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 40),
                ButtonState::Pressed => Color::from_hex("ffffff", 57)
            }, 4.0, None),
            text: Text::new(theme, match caps_on {
                true => "SPACE",
                false => "space",
            }, TextSize::Md, TextStyle::Keyboard, Align::Left, None)
        }
    }

    fn newline(theme: &Theme, caps_on: bool, state: ButtonState, label: &str) -> Self {
        _Key::Newline {
            layout: Stack(
                Offset::Center, Offset::Center,
                Size::custom(|widths: Vec<(f32, f32)>| (widths[1].0, 92.0)),
                Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0),
            ),
            background: Rectangle::new(match state {
                ButtonState::Default => Color::from_hex("ffffff", 40),
                ButtonState::Pressed => Color::from_hex("ffffff", 57)
            }, 4.0, None),
            text: Text::new(theme, &match caps_on {
                true => label.to_uppercase(),
                false => label.to_string(),
            }, TextSize::Md, TextStyle::Keyboard, Align::Left, None)
        }
    }

    fn capslock(theme: &Theme, state: ButtonState) -> Self {
        let icon = match state {
            ButtonState::Default => Icons::Capslock,
            ButtonState::Pressed => Icons::CapslockOn
        };

        _Key::Capslock {
            layout: Stack(
                Offset::Center, Offset::Center,
                Size::custom(|widths: Vec<(f32, f32)>| (widths[1].0, 42.0)),
                Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0),
            ),
            background: Rectangle::new(Color::from_hex("ffffff", 40), 4.0, None),
            icon: Icon::new(theme, icon, Some(Color::white()), 36.0),
        }
    }

    fn paginator(theme: &Theme, page: usize) -> Self {
        _Key::Paginator {
            layout: Stack(
                Offset::Center, Offset::Center,
                Size::custom(|widths: Vec<(f32, f32)>| (widths[1].0, 92.0)),
                Size::Static(48.0), Padding(3.0, 6.0, 3.0, 6.0),
            ),
            background: Rectangle::new(Color::from_hex("ffffff", 40), 4.0, None),
            content: PaginatorContent::new(theme, page)
        }
    }
}

#[derive(Debug, Clone)]
struct KeyboardBackground {
    color: prism::drawable::Color,
    window: Window,
}

impl Drawable for KeyboardBackground {
    fn size_range(&self) -> RangeTree {
        RangeTree(SizeRange {
            min_width: 0.0, max_width: f32::INFINITY,
            min_height: 0.0, max_height: f32::INFINITY,
        }, vec![])
    }

    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), mut bound: Rect) -> Vec<Instruction> {
        let (width, height) = self.window.size;
        let safe_bottom = height - self.window.insets.2;
        let reaches_bottom = (bound.1 + bound.3 - safe_bottom).abs() <= f32::EPSILON * height;
        let mut size = layout.0;
        if height > width && reaches_bottom {
            // Extend only the paint; layout and key hit areas stay inside the safe area.
            size.1 = height - offset.1;
            bound.3 = height - bound.1;
        }
        let shape = Shape::new(size.0, size.1, self.color, Variant::Rectangle);
        vec![Instruction {offset, bound, item: Item::Shape(shape)}]
    }

    fn event(&mut self, ctx: &mut Context, _layout: &LayoutTree, _event: Box<dyn DynEvent>) {
        self.window = ctx.window;
    }
}

#[derive(Debug, Component, Clone)]
struct PaginatorContent(Row, Text, Text, Text);
impl PaginatorContent {
    fn new(theme: &Theme, page: usize) -> Self {
        let (highlight, dim) = (TextStyle::Keyboard, TextStyle::Secondary);

        let styles = match page {
            0 => (highlight, dim, dim),
            1 => (dim, highlight, dim),
            _ => (dim, dim, highlight),
        };

        PaginatorContent(
            Row::center(1.0),
            Text::new(theme, "•", TextSize::H2, styles.0, Align::Left, None),
            Text::new(theme, "•", TextSize::H2, styles.1, Align::Left, None),
            Text::new(theme, "•", TextSize::H2, styles.2, Align::Left, None),
        )
    }
}

#[derive(Component, Debug, Clone)]
struct Paginator(Stack, emitters::Selectable<_Paginator>);
impl Paginator {
    fn new(theme: &Theme, page: usize) -> Self {
        let first = _Key::paginator(theme, 0);
        let second = _Key::paginator(theme, 1);
        let third = _Key::paginator(theme, 2);

        let selectable = _Paginator::new(page, first, second, third);

        Self(Stack::default(), emitters::Selectable::new(selectable, uuid::Uuid::new_v4()))
    }
}

#[derive(Component, Debug, Clone)]
#[on_dyn_event]
struct _Paginator(Stack, Enum<Box<dyn Drawable>>, #[skip] usize);

impl _Paginator {
    fn new(page: usize, first: impl Drawable + 'static, second: impl Drawable + 'static, third: impl Drawable + 'static) -> Self {
        let start = match page {
            0 => "first",
            1 => "second",
            _ => "third"
        };

        _Paginator(Stack::default(), Enum::new(vec![
            ("first".to_string(), Box::new(first)),
            ("second".to_string(), Box::new(second)),
            ("third".to_string(), Box::new(third))
        ], start.to_string()), 0)
    }
}

impl _Paginator {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(ptsd::events::Selectable::Selected(true)) =
            event.downcast_ref::<ptsd::events::Selectable>() {
            if &self.1.current() == "first" {
                self.1.display("second");
                self.2 = 1;
            } else if &self.1.current() == "second" {
                self.1.display("third");
                self.2 = 2;
            } else {
                self.1.display("first");
                self.2 = 0;
            }

            ctx.hardware.trigger_haptic();
            ctx.emit(KeyAction::Paginator(self.2));
        }
        vec![event]
    }
}

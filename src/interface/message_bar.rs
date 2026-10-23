use prism::drawable::{Area, Component, Drawable, LayoutTree, RangeTree, Rect, Instruction};
use prism::events::{DynEvent, MouseEvent, MouseState, MouseButton, KeyboardEvent, KeyboardState, Key};
use prism::events::{ChangeHandler, ListenFuture};
use prism::Context;
use ptsd::layout::{Column, Row, Stack, Offset, Size, Padding};
use ptsd::display::{Bin, Opt, Dismissible, DismissEdge};
use ptsd::interfaces::{ShowKeyboard, CustomKeyboardVisibilityChanged};
use ptsd::interactions;

use std::any::Any;

use crate::theme::{self, Theme, Icons, Variant, ButtonColorScheme};
use crate::components::{TextInput, TextInputEvent, AttachButton, SubmitFn, ValidFn};
use crate::components::button::{GhostIconButton, IconButton, ButtonSize, ButtonStyle};
use crate::interface::system::{Panel, Panels};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBarEvent {Keyboard, Panel(Panel), Hide}
impl prism::events::Event for MessageBarEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardMode {Request, Native, Custom, Physical}
impl prism::events::Event for KeyboardMode {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone)]
pub struct MessageBar(MessageBarContent);
impl MessageBar {
    pub fn new(
        theme: &Theme, placeholder: &str,
        on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static, valid: impl ValidFn,
    ) -> Self {Self(MessageBarContent::new(theme, placeholder, on_submit, valid))}

    pub fn field(&mut self) -> &mut TextInput {&mut self.0.field}
}
impl Drawable for MessageBar {
    fn listen(&mut self) -> ListenFuture<'_> {Drawable::listen(&mut self.0)}
    fn update(&mut self, ctx: &mut Context, path: Vec<usize>, update: Box<dyn Any>) {
        Drawable::update(&mut self.0, ctx, path, update);
    }
    fn size_range(&self) -> RangeTree {self.0.size_range()}
    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        Drawable::layout(&mut self.0, size, range)
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.0.draw(layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        for event in self.0.on_event(ctx, layout, event) {self.0.event(ctx, layout, event);}
    }
}

#[derive(Debug, Component, Clone)]
#[listen]
struct MessageBarContent {
    layout: Column,
    field: Bin<Stack, TextInput>,
    toolbar: Toolbar,
    panels: Opt<Dismissible<Bin<Stack, Panels>>>,
    #[skip] active: Option<Panel>,
    #[skip] custom_keyboard_visible: bool,
    #[skip] initial: bool,
}
impl MessageBarContent {
    fn new(
        theme: &Theme, placeholder: &str,
        on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static, valid: impl ValidFn,
    ) -> Self {
        let send = (Icons::Send, Box::new(on_submit) as Box<dyn SubmitFn>, Box::new(valid) as Box<dyn ValidFn>);
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let highlighted = ButtonColorScheme {background: colors.pressed.background, ..colors.default};
        let pages = [
            (Icons::Keyboard, MessageBarEvent::Keyboard),
            (Icons::Emoji, MessageBarEvent::Panel(Panel::Emoji)),
            (Icons::Sticker, MessageBarEvent::Panel(Panel::Stickers)),
            (Icons::Gif, MessageBarEvent::Panel(Panel::Gifs)),
        ];
        let buttons = pages.into_iter().map(|(icon, action)| {
            let looks = [colors.default, highlighted].map(|c| {
                let layout = Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit,
                    Padding(4.0, 6.0, 4.0, 8.0));
                let face = Bin(layout, IconButton::new(
                    theme, icon, ButtonStyle::Ghost, ButtonSize::Medium,
                    c.background, c.outline, c.label,
                ));
                interactions::Button::new_triggers_on_release(
                    face, None::<IconButton>, None::<IconButton>, None::<IconButton>,
                    None::<IconButton>, move |ctx: &mut Context| ctx.emit(action), false,
                )
            });
            let visible = match action {
                MessageBarEvent::Keyboard | MessageBarEvent::Panel(Panel::Emoji) => false,
                _ => true,
            };
            Opt::new(PageButton {
                layout: Stack::default(), button: looks[0].clone(), action, looks, selected: false,
            }, visible)
        }).collect();
        let mut hide = GhostIconButton::new(
            theme, Icons::DownArrow,
            Stack(Offset::End, Offset::Center, Size::Fit, Size::Fit, Padding(4.0, 6.0, 0.0, 8.0)),
            |ctx, _| ctx.emit(MessageBarEvent::Hide),
        );
        hide.1.set_trigger_on_release();
        Self {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            field: Bin(
                Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 8.0, 0.0, 6.0)),
                TextInput::new(theme, uuid::Uuid::new_v4(), None, None, Some(placeholder), None, Some(send)),
            ),
            toolbar: Toolbar {
                layout: Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), buttons,
                attach: AttachButton::new(theme, Stack(
                    Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(4.0, 6.0, 4.0, 8.0))),
                hide: Bin(Stack(Offset::End, Offset::Center, Size::custom(|_| (0.0, f32::MAX)),
                    Size::Fit, Padding::default()), Opt::new(hide, false)),
            },
            panels: Opt::new(Dismissible::new(Bin(
                Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, Padding(0.0, 4.0, 0.0, 8.0)),
                Panels::new(theme),
            ), DismissEdge::Bottom), false),
            active: None, custom_keyboard_visible: false, initial: true,
        }
    }

    fn show_panel(&mut self, ctx: &Context, panel: Option<Panel>) {
        self.panels.inner().reset();
        self.panels.inner().child.close_search();
        if let Some(panel) = panel {self.panels.inner().child.show(panel);}
        self.active = panel;
        self.panels.display(panel.is_some() && !ctx.window.keyboard_visible);
        self.refresh_toolbar(ctx);
    }

    fn refresh_toolbar(&mut self, ctx: &Context) {
        let keyboard_visible = self.custom_keyboard_visible || ctx.window.keyboard_visible;
        for button in &mut self.toolbar.buttons {
            let button = button.inner();
            let selected = match button.action {
                MessageBarEvent::Keyboard => self.active.is_none() && keyboard_visible,
                MessageBarEvent::Panel(panel) => self.active == Some(panel),
                MessageBarEvent::Hide => false,
            };
            if button.selected != selected {
                button.button = button.looks[selected as usize].clone();
                button.selected = selected;
            }
        }
        self.toolbar.hide.1.display(self.active.is_some() || keyboard_visible);
    }

    fn on_event(&mut self, ctx: &mut Context, sized: &LayoutTree, mut event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let searching = self.panels.is_showing() && self.panels.inner_ref().child.searching();
        let area = self.panels.is_showing().then(|| sized.1.get(2).map(|(offset, layout)| {
            let size = self.panels.inner_ref().size();
            Area {offset: (offset.0, offset.1 + layout.0.1 - size.1 - sized.0.1), size}
        })).flatten();
        let handle = area.and_then(|_| sized.1.get(1)).map(|(offset, layout)| {
            Area {offset: (offset.0, offset.1 - sized.0.1), size: layout.0}
        });
        if let Some(mouse) = event.downcast_mut::<MouseEvent>() {
            // The bumper's bottom stays fixed while dragging changes its local origin.
            let mut observed = mouse.clone();
            if let Some(position) = &mut observed.position {position.1 -= sized.0.1;}
            self.panels.inner().observe(&mut observed, area, handle);
            if observed.position.is_none() {mouse.position = None;}
            mouse.state = observed.state;
        } else {
            self.panels.inner().observe(&mut *event, area, handle);
        }

        if let Some(mode) = event.downcast_ref::<KeyboardMode>() && *mode != KeyboardMode::Request {
            for button in &mut self.toolbar.buttons {
                button.display(match button.inner_ref().action {
                    MessageBarEvent::Keyboard => *mode == KeyboardMode::Custom,
                    MessageBarEvent::Panel(Panel::Emoji) => *mode != KeyboardMode::Native,
                    _ => true,
                });
            }
        } else if let Some(MouseEvent {position: Some(p), state: MouseState::Pressed(MouseButton::Left)}) =
            event.downcast_ref::<MouseEvent>() {
            let on = |child: usize| sized.1.get(child).is_some_and(|(at, size)| {
                p.0 >= at.0 && p.0 < at.0 + size.0.0 && p.1 >= at.1 && p.1 < at.1 + size.0.1
            });
            if on(0) {
                self.show_panel(ctx, None);
            } else if on(1) || on(2) && !searching {
                // Preserve the message's selection while using the toolbar or picker.
                let press = |(at, child): &((f32, f32), LayoutTree)| {
                    let local = (p.0 - at.0, p.1 - at.1);
                    let inside = local.0 >= 0.0 && local.1 >= 0.0 && local.0 < child.0.0 && local.1 < child.0.1;
                    Box::new(MouseEvent {position: inside.then_some(local), state: MouseState::Pressed(MouseButton::Left)})
                };
                if let Some(toolbar) = sized.1.get(1) {self.toolbar.event(ctx, &toolbar.1, press(toolbar));}
                if let Some(panels) = sized.1.get(2) {self.panels.event(ctx, &panels.1, press(panels));}
                return vec![];
            }
        } else if let Some(action) = event.downcast_ref::<MessageBarEvent>() {
            match action {
                MessageBarEvent::Panel(panel) => {
                    self.show_panel(ctx, (self.active != Some(*panel)).then_some(*panel));
                    ctx.emit(ShowKeyboard(false));
                }
                MessageBarEvent::Keyboard => {
                    self.show_panel(ctx, None);
                    ctx.emit(TextInputEvent::Focus {id: self.field.id()});
                    ctx.emit(ShowKeyboard(true));
                }
                MessageBarEvent::Hide => {
                    self.show_panel(ctx, None);
                    ctx.emit(ShowKeyboard(false));
                    ctx.emit(ptsd::events::TextInput::Focused(false));
                }
            }
        } else if let Some(ShowKeyboard(showing)) = event.downcast_ref() {
            if *showing && !searching {self.show_panel(ctx, None);}
        } else if let Some(CustomKeyboardVisibilityChanged(visible)) = event.downcast_ref() {
            self.custom_keyboard_visible = *visible;
        } else if matches!(event.downcast_ref::<ptsd::navigation::NavigationEvent>(), Some(_)) {
            self.show_panel(ctx, None);
        } else if self.active == Some(Panel::Emoji)
            && let Some(KeyboardEvent {key, state: KeyboardState::Pressed | KeyboardState::Repeated, ..}) = event.downcast_ref() {
            // The tray owns this destination even when a picker was opened before typing.
            match key {
                Key::Character(text) => self.field.insert(ctx, text),
                Key::Delete | Key::Backspace => self.field.delete_back(ctx),
                _ => return vec![event],
            }
            return vec![];
        }
        let searching = self.panels.inner_ref().child.searching();
        self.panels.display(self.active.is_some() && (searching || !ctx.window.keyboard_visible));
        self.refresh_toolbar(ctx);
        vec![event]
    }
}

impl MessageBarContent {
    fn listen(&mut self) -> ListenFuture<'_> {
        let initial = &mut self.initial;
        Box::pin(ChangeHandler::select(vec![
            Drawable::listen(&mut self.field), Drawable::listen(&mut self.toolbar),
            Drawable::listen(&mut self.panels), Box::pin(async move {
                if !std::mem::take(initial) {std::future::pending::<()>().await;}
                (Vec::new(), Box::new(()) as Box<dyn Any>)
            }),
        ]))
    }
    fn update(&mut self, ctx: &mut Context, mut path: Vec<usize>, update: Box<dyn Any>) {
        match path.pop().unwrap() {
            0 => Drawable::update(&mut self.field, ctx, path, update),
            1 => Drawable::update(&mut self.toolbar, ctx, path, update),
            2 => {
                Drawable::update(&mut self.panels, ctx, path, update);
                if self.panels.inner().advance() {
                    self.show_panel(ctx, None);
                    ctx.emit(ShowKeyboard(false));
                }
            }
            3 => ctx.emit(KeyboardMode::Request),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Component, Clone)]
struct Toolbar {
    layout: Row,
    buttons: Vec<Opt<PageButton>>,
    attach: AttachButton,
    hide: Bin<Stack, Opt<GhostIconButton>>,
}
#[derive(Debug, Component, Clone)]
struct PageButton {
    layout: Stack,
    button: interactions::Button,
    #[skip] action: MessageBarEvent,
    #[skip] looks: [interactions::Button; 2],
    #[skip] selected: bool,
}

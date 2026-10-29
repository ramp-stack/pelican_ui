use prism::drawable::{Area, Component, Drawable, LayoutTree, Rect, Instruction};
use prism::events::{DynEvent, MouseEvent, MouseState, MouseButton, KeyboardEvent, KeyboardState, Key};
use prism::events::{OnChange, on_change, ListenCallback, ListenFuture};
use prism::Context;
use ptsd::layout::{Column, Row, Stack, Offset, Size, Padding};
use ptsd::display::{Bin, Opt, Dismissible, DismissEdge};
use ptsd::interfaces::{ShowKeyboard, CustomKeyboardVisibilityChanged};
use ptsd::interactions;

use std::any::Any;

use crate::theme::{self, Theme, Icons, Variant, ButtonColorScheme};
use crate::components::{TextInput, AttachButton, SubmitFn, ValidFn};
use crate::components::button::{GhostIconButton, IconButton, ButtonSize, ButtonStyle};
use crate::interface::system::{Panel, Panels};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageBarEvent {Panel(Panel), Hide}
impl prism::events::Event for MessageBarEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone)]
pub struct MessageBar(MessageBarContent);
impl MessageBar {
    pub fn new(
        theme: &Theme, placeholder: &str, bottom_padding: f32,
        on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static, valid: impl ValidFn,
    ) -> Self {
        Self(MessageBarContent::new(theme, placeholder, bottom_padding, on_submit, valid))
    }

    pub fn field(&mut self) -> &mut TextInput {&mut self.0.field}
}

impl Drawable for MessageBar {
    fn request_width(&mut self) -> (f32, f32) {
        self.0.toolbar.inner().child.request_width();
        self.0.request_width()
    }
    fn request_height(&mut self, width: f32) -> (f32, f32) {
        let height = self.0.toolbar.inner().child.request_height(width).0;
        self.0.field.inner.keyboard_dismiss_padding(height + self.0.bottom_padding);
        self.0.request_height(width)
    }
    fn layout(&mut self, size: (f32, f32)) -> LayoutTree {
        Drawable::layout(&mut self.0, size)
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        self.0.draw(layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        for event in self.0.on_event(ctx, layout, event) {self.0.event(ctx, layout, event);}
    }

    fn changed(&mut self) -> ListenFuture<'_> {
        Box::pin(async move {
            let callback = Component::listen(&mut self.0).await;
            Box::new(move |target: &mut dyn Any, ctx: &mut Context| {
                callback(&mut target.downcast_mut::<Self>().unwrap().0, ctx);
            }) as ListenCallback
        })
    }
}

#[derive(Debug, Component, Clone)]
struct MessageBarContent {
    layout: Column,
    field: Bin<Stack, TextInput>,
    toolbar: Opt<Dismissible<Toolbar>>,
    panels: Opt<Dismissible<Bin<Stack, Panels>>>,
    #[skip] active: Option<Panel>,
    #[skip] custom_keyboard_visible: bool,
    #[skip] bottom_padding: f32,
    #[skip] initial: bool,
}
impl MessageBarContent {
    fn new(
        theme: &Theme, placeholder: &str, bottom_padding: f32,
        on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static, valid: impl ValidFn,
    ) -> Self {
        let send = (Icons::Send, Box::new(on_submit) as Box<dyn SubmitFn>, Box::new(valid) as Box<dyn ValidFn>);
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let highlighted = ButtonColorScheme {background: colors.pressed.background, ..colors.default};
        let pages = [
            (Icons::Emoji, MessageBarEvent::Panel(Panel::Emoji)),
            (Icons::Sticker, MessageBarEvent::Panel(Panel::Stickers)),
            (Icons::Gif, MessageBarEvent::Panel(Panel::Gifs)),
        ];
        let buttons = pages.into_iter().map(|(icon, action)| {
            let looks = [colors.default, highlighted].map(|c| {
                let layout = Stack::new(
                    Offset::Center, Offset::Center, Size::Fit, Size::Fit,
                    Padding(4.0, 6.0, 4.0, 8.0),
                );
                let face = Bin(layout, IconButton::new(
                    theme, icon, ButtonStyle::Ghost, ButtonSize::Medium,
                    c.background, c.outline, c.label,
                ));
                interactions::Button::new_triggers_on_release(
                    face, None::<IconButton>, None::<IconButton>, None::<IconButton>,
                    None::<IconButton>, move |ctx: &mut Context| ctx.emit(action), false,
                )
            });
            let visible = action != MessageBarEvent::Panel(Panel::Emoji);
            Opt::new(PageButton {
                layout: Stack::default(), button: looks[0].clone(), action, looks, selected: false,
            }, visible)
        }).collect();
        let mut hide = GhostIconButton::new(
            theme, Icons::DownArrow,
            Stack::new(Offset::End, Offset::Center, Size::Fit, Size::Fit, Padding(4.0, 6.0, 0.0, 8.0)),
            |ctx, _| ctx.emit(MessageBarEvent::Hide),
        );
        hide.1.set_trigger_on_release();
        Self {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            field: Bin(
                Stack::new(
                    Offset::Start, Offset::Start, Size::Fit, Size::Fit,
                    Padding(0.0, 8.0, 0.0, 6.0),
                ),
                TextInput::new(theme, uuid::Uuid::new_v4(), None, None, Some(placeholder), None, Some(send)),
            ),
            toolbar: Opt::new(Dismissible::new(Toolbar {
                layout: Row::new(0.0, Offset::Center, Size::Fit, Padding::default()), buttons,
                attach: AttachButton::new(theme, Stack::new(
                    Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding(4.0, 6.0, 4.0, 8.0),
                )),
                hide: Bin(Stack::new(
                    Offset::End, Offset::Center, Size::custom(|_| (0.0, f32::MAX)),
                    Size::Fit, Padding::default(),
                ), Opt::new(hide, false)),
            }, DismissEdge::Bottom), false),
            panels: Opt::new(Dismissible::new(Bin(
                Stack::new(
                    Offset::Start, Offset::Start, Size::Fill, Size::Fit,
                    Padding(0.0, 4.0, 0.0, 8.0),
                ),
                Panels::new(theme),
            ), DismissEdge::Bottom), false),
            active: None, custom_keyboard_visible: false, bottom_padding, initial: true,
        }
    }

    fn show_panel(&mut self, ctx: &Context, panel: Option<Panel>) {
        self.panels.inner().reset();
        self.panels.inner().child.close_search();
        if let Some(panel) = panel {self.panels.inner().child.show(panel);}
        self.active = panel;
        self.panels.display(panel.is_some() && !ctx.hardware.keyboard().visible());
        self.refresh_toolbar(ctx);
    }

    fn refresh_toolbar(&mut self, ctx: &Context) {
        let keyboard_visible = self.custom_keyboard_visible || ctx.hardware.keyboard().visible();
        let showing = !ctx.window.is_mobile || self.active.is_some() || keyboard_visible;
        if !ctx.window.is_mobile || self.active.is_some() || self.custom_keyboard_visible {
            self.toolbar.inner().reset();
        } else if !showing && self.toolbar.is_showing() {
            self.toolbar.inner().dismiss();
        }
        self.toolbar.display(showing || self.toolbar.inner_ref().is_dismissing());
        for button in &mut self.toolbar.inner().child.buttons {
            button.display(
                button.inner_ref().action != MessageBarEvent::Panel(Panel::Emoji) || !ctx.window.is_mobile,
            );
            let button = button.inner();
            let selected = match button.action {
                MessageBarEvent::Panel(panel) => self.active == Some(panel),
                MessageBarEvent::Hide => false,
            };
            if button.selected != selected {
                button.button = button.looks[selected as usize].clone();
                button.selected = selected;
            }
        }
        let hide = self.active.is_some() || keyboard_visible || self.toolbar.inner_ref().is_dismissing();
        self.toolbar.inner().child.hide.1.display(hide);
    }

    fn on_event(&mut self, ctx: &mut Context, sized: &LayoutTree, mut event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let searching = self.panels.is_showing() && self.panels.inner_ref().child.searching();
        let origin = (
            ctx.window.insets.1 + (ctx.window.area().size.0 - sized.0.0) / 2.0,
            ctx.window.size.1 - ctx.window.insets.2 - self.bottom_padding - sized.0.1,
        );
        let area = self.panels.is_showing().then(|| sized.1.get(2).map(|(offset, layout)| {
            let size = self.panels.inner_ref().size();
            Area {offset: (offset.0 + origin.0, offset.1 + layout.0.1 - size.1 + origin.1), size}
        })).flatten();
        let handle = area.and_then(|_| sized.1.get(1)).map(|(offset, layout)| {
            Area {offset: (offset.0 + origin.0, offset.1 + origin.1), size: layout.0}
        });
        let toolbar = (self.active.is_none() && self.toolbar.is_showing()
            && (ctx.hardware.keyboard().visible() || self.toolbar.inner_ref().is_dismissing()))
            .then(|| sized.1.get(1).map(|(offset, layout)| {
                let size = self.toolbar.inner_ref().size();
                Area {offset: (offset.0 + origin.0, offset.1 + layout.0.1 - size.1 + origin.1), size}
            })).flatten();
        if let Some(mouse) = event.downcast_mut::<MouseEvent>() {
            // Window coordinates separate native keyboard motion from the panel's own drag.
            let mut observed = mouse.clone();
            if let Some(position) = &mut observed.position {
                position.0 += origin.0;
                position.1 += origin.1;
            }
            self.panels.inner().observe(&mut observed, area, handle);
            self.toolbar.inner().observe(&mut observed, toolbar, toolbar);
            if observed.position.is_none() {mouse.position = None;}
            mouse.state = observed.state;
        } else {
            self.panels.inner().observe(&mut *event, area, handle);
            self.toolbar.inner().observe(&mut *event, toolbar, toolbar);
        }

        if let Some(MouseEvent {position: Some(p), state: MouseState::Pressed(MouseButton::Left)}) =
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
                    ctx.emit(ShowKeyboard::Hide);
                }
                MessageBarEvent::Hide => {
                    if self.panels.is_showing() {self.panels.inner().dismiss();}
                    else {self.show_panel(ctx, None);}
                    self.toolbar.inner().dismiss();
                    ctx.emit(ShowKeyboard::Hide);
                    ctx.emit(ptsd::events::TextInput::Focused(false));
                }
            }
        } else if let Some(ShowKeyboard::Show {..}) = event.downcast_ref() {
            self.toolbar.inner().reset();
            if !ctx.hardware.keyboard().visible() {self.toolbar.display(false);}
            if !searching {self.show_panel(ctx, None);}
        } else if let Some(CustomKeyboardVisibilityChanged(visible)) = event.downcast_ref() {
            self.custom_keyboard_visible = *visible;
        } else if event.downcast_ref::<ptsd::navigation::NavigationEvent>().is_some() {
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
        self.panels.display(self.active.is_some()
            && (searching || !ctx.hardware.keyboard().visible()));
        self.refresh_toolbar(ctx);
        vec![event]
    }
}

#[on_change(Component::listen(self.toolbar.inner()))]
impl OnChange<ListenCallback<Dismissible<Toolbar>>> for MessageBarContent {
    fn on_change(&mut self, ctx: &mut Context, callback: ListenCallback<Dismissible<Toolbar>>) {
        callback(self.toolbar.inner(), ctx);
        if self.toolbar.inner().advance() {
            self.toolbar.inner().reset();
            self.toolbar.display(!ctx.window.is_mobile || self.active.is_some()
                || self.custom_keyboard_visible || ctx.hardware.keyboard().visible());
        }
    }
}

#[on_change(Drawable::changed(&mut self.panels))]
impl OnChange<ListenCallback> for MessageBarContent {
    fn on_change(&mut self, ctx: &mut Context, callback: ListenCallback) {
        callback(&mut self.panels, ctx);
        if self.panels.inner().advance() {
            self.show_panel(ctx, None);
            ctx.emit(ShowKeyboard::Hide);
        }
    }
}

#[on_change(async {
    if !self.initial {std::future::pending::<()>().await;}
})]
impl OnChange<()> for MessageBarContent {
    fn on_change(&mut self, ctx: &mut Context, _: ()) {
        self.initial = false;
        self.refresh_toolbar(ctx);
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

use prism::{Context, IS_MOBILE};
use prism::event::{self, Event, OnEvent, MouseEvent, MouseState, MouseButton, TickEvent};
use prism::drawable::{Component, Drawable, SizedTree};
use prism::layout::{Column, Row, Stack, Offset, Size, Padding};
use prism::display::{Bin, Opt};

use ptsd::interactions;
use ptsd::interfaces::ShowKeyboard;

use crate::theme::{self, Theme, Icons, Variant, ButtonColorScheme};
use crate::components::{TextInput, AttachButton, SendEvent, SubmitFn, ValidFn};
use crate::components::button::{GhostIconButton, IconButton, ButtonSize, ButtonStyle};
use crate::interface::system::{KeyboardPages, KeyboardPage, MobileKeyboardEvent, Panels, TEXT_FIELD_GROUP};

#[derive(Debug, Component, Clone)]
pub struct MessageBar {
    layout: Column,
    field: TextInput,
    toolbar: Toolbar,
    panels: Option<Opt<Panels>>,
    #[skip] field_id: Option<String>,
    #[skip] learning_id: bool,
    #[skip] checking: Option<u8>,
}

impl MessageBar {
    pub fn new(
        theme: &Theme,
        placeholder: &str,
        on_submit: impl FnMut(&mut Context, &mut String) + Clone + 'static,
        valid: impl FnMut(&mut Context, &mut String) -> bool + Clone + 'static,
    ) -> Self {
        let send = (Icons::Send, Box::new(on_submit) as Box<dyn SubmitFn>, Box::new(valid) as Box<dyn ValidFn>);

        let group = uuid::Uuid::new_v4();
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let highlighted = ButtonColorScheme { background: colors.pressed.background, ..colors.default };
        let pages = [(Icons::Keyboard, KeyboardPage::Keys), (Icons::Emoji, KeyboardPage::Emoji), (Icons::Sticker, KeyboardPage::Stickers), (Icons::Gif, KeyboardPage::Gifs)];
        let buttons = pages.into_iter().filter(|(_, page)| IS_MOBILE || *page != KeyboardPage::Keys).map(|(icon, page)| {
            let [default, selected] = [colors.default, highlighted].map(|c| {
                IconButton::new(theme, icon, ButtonStyle::Ghost, ButtonSize::Medium, c.background, c.outline, c.label)
            });
            let looks = [false, true].map(|is_selected| interactions::Selectable::new(
                default.clone(), selected.clone(), is_selected, page != KeyboardPage::Keys,
                move |ctx: &mut Context| ctx.emit(MessageBarEvent::Page(page)), group,
            ));
            PageButton { layout: Stack::default(), button: looks[0].clone(), page, looks }
        }).collect();

        let mut hide = GhostIconButton::new(theme, Icons::DownArrow, |ctx: &mut Context, _: &Theme| ctx.emit(MessageBarEvent::Hide));
        hide.1.set_trigger_on_release();

        MessageBar {
            layout: Column::new(12.0, Offset::Start, Size::Fill, Padding(0.0, 8.0, 0.0, 8.0), None),
            field: TextInput::new(theme, None, None, Some(placeholder), None, Some(send), KeyboardPages::TOOLBAR),
            toolbar: Toolbar {
                layout: Row::new(8.0, Offset::Center, Size::Fit, Padding::default()),
                buttons,
                attach: AttachButton::new(theme),
                hide: Bin(Stack(Offset::End, Offset::Center, Size::custom(|_: Vec<(f32, f32)>| (0.0, f32::MAX)), Size::Fit, Padding::default()), Opt::new(hide, false)),
            },
            panels: (!IS_MOBILE).then(|| Opt::new(Panels::new(theme), false)),
            field_id: None,
            learning_id: false,
            checking: None,
        }
    }

    pub fn field(&mut self) -> &mut TextInput { &mut self.field }
}

impl OnEvent for MessageBar {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        let searching_gifs = self.panels.as_mut().is_some_and(|panels| panels.is_showing() && panels.inner().gifs.inner().searching());
        let mut focus = false;
        let mut swallow = false;

        if let Some(MouseEvent { position: Some(p), state: MouseState::Pressed(MouseButton::Left) }) = event.downcast_ref::<MouseEvent>() {
            let on = |child: usize| sized.1.get(child).is_some_and(|(at, size)| {
                p.0 >= at.0 && p.0 < at.0 + size.0.0 && p.1 >= at.1 && p.1 < at.1 + size.0.1
            });
            if on(0) {
                self.learning_id = true;
            } else if !IS_MOBILE && (on(1) || (on(2) && !searching_gifs)) {
                let press = |(at, child): &((f32, f32), SizedTree)| {
                    let local = (p.0 - at.0, p.1 - at.1);
                    let inside = local.0 >= 0.0 && local.1 >= 0.0 && local.0 < child.0.0 && local.1 < child.0.1;
                    Box::new(MouseEvent { position: inside.then_some(local), state: MouseState::Pressed(MouseButton::Left) })
                };
                if let Some(toolbar) = sized.1.get(1) { self.toolbar.event(ctx, &toolbar.1, press(toolbar)); }
                if let (Some(panels), Some(child)) = (&mut self.panels, sized.1.get(2)) { panels.event(ctx, &child.1, press(child)); }
                focus = !self.field.has_focus();
                swallow = true;
            }
        } else if let Some(event::Selectable::Pressed(id, group)) = event.downcast_ref::<event::Selectable>()
            && group == TEXT_FIELD_GROUP && std::mem::take(&mut self.learning_id)
        {
            self.field_id = Some(id.clone());
        } else if event.downcast_ref::<TickEvent>().is_some() && let Some(frames) = self.checking {
            self.checking = match (self.field.has_focus(), frames) {
                (true, _) => None,
                (false, 0) => { self.field_id = None; focus = true; None },
                (false, n) => Some(n - 1),
            };
        } else if let Some(MessageBarEvent::Key(key)) = event.downcast_ref::<MessageBarEvent>() {
            match key {
                event::Key::Character(text) => self.field.insert(text),
                event::Key::Delete | event::Key::Backspace => self.field.delete_back(),
                _ => {},
            }
            focus = !self.field.has_focus();
        } else if let Some(toolbar) = event.downcast_ref::<MessageBarEvent>() {
            let page = match toolbar { MessageBarEvent::Page(page) => Some(*page), _ => None };
            let open = page.is_some_and(|page| self.toolbar.buttons.iter().any(|button| button.page == page && button.button.is_selected()));
            match (&mut self.panels, page) {
                (Some(panels), page) => {
                    match page {
                        Some(page) if open => panels.inner().show(page),
                        _ => panels.inner().gifs.inner().close_search(),
                    }
                    panels.display(open);
                    self.toolbar.hide.1.display(open);
                    if !open {
                        for button in &mut self.toolbar.buttons {
                            if button.button.is_selected() { button.button = button.looks[0].clone(); }
                        }
                    }
                },
                (None, None) => {
                    ctx.emit(ShowKeyboard(false));
                    ctx.emit(event::TextInput::Focused(false));
                },
                (None, Some(page)) => {
                    focus = !self.field.has_focus();
                    ctx.emit(MobileKeyboardEvent::Show(if open { page } else { KeyboardPage::Keys }));
                    ctx.emit(ShowKeyboard(true));
                },
            }
        } else if let Some(MobileKeyboardEvent::Showing(showing)) = event.downcast_ref::<MobileKeyboardEvent>() {
            for button in &mut self.toolbar.buttons {
                let selected = *showing == Some(button.page);
                if button.button.is_selected() != selected { button.button = button.looks[selected as usize].clone(); }
            }
            self.toolbar.hide.1.display(showing.is_some());
        } else if searching_gifs
            && (matches!(event.downcast_ref::<MobileKeyboardEvent>(), Some(MobileKeyboardEvent::CancelGifSearch)) || matches!(event.downcast_ref::<SendEvent>(), Some(SendEvent::Gif(_))))
        {
            focus = true;
        }

        if focus {
            if let Some(id) = &self.field_id {
                ctx.emit(event::Selectable::Pressed(id.clone(), TEXT_FIELD_GROUP.to_string()));
                self.checking = Some(3);
            } else if let Some((_, field)) = sized.1.first() {
                self.learning_id = true;
                let at = Some(((field.0.0 - 60.0).max(0.0), (field.0.1 / 2.0).min(24.0)));
                for state in [MouseState::Pressed(MouseButton::Left), MouseState::Released(MouseButton::Left)] {
                    self.field.event(ctx, field, Box::new(MouseEvent { position: at, state }));
                }
            }
        }

        if swallow { return vec![]; }
        vec![event]
    }
}

#[derive(Debug, Component, Clone)]
struct Toolbar {
    layout: Row,
    buttons: Vec<PageButton>,
    attach: AttachButton,
    hide: Bin<Stack, Opt<GhostIconButton>>,
}
impl OnEvent for Toolbar {}

#[derive(Debug, Component, Clone)]
struct PageButton {
    layout: Stack,
    button: interactions::Selectable,
    #[skip] page: KeyboardPage,
    #[skip] looks: [interactions::Selectable; 2],
}
impl OnEvent for PageButton {}

#[derive(Debug, Clone)]
pub(crate) enum MessageBarEvent {
    Page(KeyboardPage),
    Hide,
    Key(event::Key),
}

prism::impl_event_all_children!(MessageBarEvent);

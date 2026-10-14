use prism::{Context, IS_MOBILE};
use prism::event::{self, Event, OnEvent, MouseEvent, MouseState, MouseButton, TickEvent};
use prism::drawable::{Component, Drawable, SizedTree};
use prism::layout::{Area, Column, Row, Stack, Offset, Size, Padding};
use prism::display::{Bin, Opt};

use ptsd::interactions;
use ptsd::interfaces::ShowKeyboard;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::theme::{self, Theme, Icons, Variant, ButtonColorScheme};
use crate::components::{TextInput, AttachButton, SendGifEvent};
use crate::components::button::{GhostIconButton, IconButton, ButtonSize, ButtonStyle};
use crate::interface::system::{KeyboardPages, KeyboardPage, KeyboardShowing, MobileKeyboardEvent, DesktopPanels, BarKey, TEXT_FIELD_GROUP};

/// Space between the message field and the toolbar, and between the
/// toolbar's buttons.
const SPACING: f32 = 8.0;

/// The chat's message bar: the message field, and under it a toolbar with
/// buttons for emoji, stickers, GIFs, and photos and videos.
///
/// On a phone the toolbar opens those pages in the on-screen keyboard, in
/// place of the letter keys. It also has a button back to the letter keys,
/// and, while the keyboard is up, one at its end that hides it. On a
/// computer it opens them under itself (tap the button again to close
/// one); emoji go straight into the message field, which keeps the focus
/// while you use them.
#[derive(Debug, Component, Clone)]
pub struct MessageBar(Column, Toolbar, TextInput, Option<Opt<DesktopPanels>>, #[skip] BarState);

#[derive(Debug, Clone)]
struct BarState {
    /// Tells this bar's toolbar events from another's.
    token: u64,
    /// The message field's id in prism's group of text fields, once it's
    /// known: selecting it gives the field the focus without moving its cursor.
    field_id: Option<String>,
    /// The message field was just pressed: the next text field selected is it.
    learning_id: bool,
    /// What the on-screen keyboard is showing (on a phone).
    keyboard: Option<KeyboardPage>,
    /// The field was just given the focus by selecting it: the frames left
    /// to see that it has it. If it doesn't, the id was wrong.
    checking: Option<u8>,
}

/// What a toolbar button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolbarAction {
    /// Shows a page: on a phone the letter keys (`Keys`), emoji, stickers or
    /// GIFs; on a computer emoji, stickers or GIFs.
    Page(KeyboardPage),
    /// Hides the keyboard (on a phone).
    Hide,
}

/// A button on the toolbar of the message bar with this number was tapped.
#[derive(Debug, Clone)]
struct ToolbarEvent(u64, ToolbarAction);

impl Event for ToolbarEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Where a press on the message bar landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part { Field, Toolbar, Pages }

impl MessageBar {
    pub fn new(
        theme: &Theme,
        placeholder: &str,
        on_submit: impl FnMut(&mut Context, &mut String) + 'static,
        valid: impl FnMut(&mut Context, &mut String) -> bool + 'static,
    ) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let token = NEXT.fetch_add(1, Ordering::Relaxed);
        let send = (Icons::Send, Arc::new(Mutex::new(on_submit)) as _, Arc::new(Mutex::new(valid)) as _);
        let field = TextInput::new(theme, None, None, Some(placeholder), None, Some(send), KeyboardPages::TOOLBAR);
        let pages = (!IS_MOBILE).then(|| Opt::new(DesktopPanels::new(theme, token), false));
        MessageBar(
            Column::new(12.0, Offset::Start, Size::Fill, Padding(0.0, 8.0, 0.0, 8.0), None),
            Toolbar::new(theme, token),
            field,
            pages,
            BarState { token, field_id: None, learning_id: false, keyboard: None, checking: None },
        )
    }

    /// The message field.
    pub fn field(&mut self) -> &mut TextInput { &mut self.2 }

    /// The page open under the toolbar (on a computer).
    fn open_page(&mut self) -> Option<KeyboardPage> {
        self.3.as_mut().filter(|pages| pages.is_showing()).map(|pages| pages.inner().page())
    }

    fn searching_gifs(&mut self) -> bool {
        self.3.as_mut().is_some_and(|pages| pages.is_showing() && pages.inner().searching())
    }

    /// What's at `p` (in the bar's own coordinates).
    fn part_at(sized: &SizedTree, p: (f32, f32)) -> Option<Part> {
        let inside = |(offset, child): &((f32, f32), SizedTree)| {
            p.0 >= offset.0 && p.0 < offset.0 + child.0.0 && p.1 >= offset.1 && p.1 < offset.1 + child.0.1
        };
        [Part::Field, Part::Toolbar, Part::Pages].into_iter().zip(&sized.1).find(|(_, child)| inside(child)).map(|(part, _)| part)
    }

    /// Gives the message field the focus: by selecting it if its id is
    /// known, so its cursor stays where it was; if not, with a tap near its
    /// end (left of the send button), which puts the cursor after the text.
    fn focus_field(&mut self, ctx: &mut Context, sized: &SizedTree) {
        if let Some(id) = &self.4.field_id {
            ctx.emit(event::Selectable::Pressed(id.clone(), TEXT_FIELD_GROUP.to_string()));
            self.4.checking = Some(3);
        } else if let Some((_, field)) = sized.1.first() {
            self.4.learning_id = true;
            let at = Some(((field.0.0 - 60.0).max(0.0), (field.0.1 / 2.0).min(24.0)));
            for state in [MouseState::Pressed(MouseButton::Left), MouseState::Released(MouseButton::Left)] {
                self.2.event(ctx, field, Box::new(MouseEvent { position: at, state }));
            }
        }
    }

    /// Passes a press at `p` on to the toolbar and the pages, not the message
    /// field (which would lose the focus).
    fn press_beside_field(&mut self, ctx: &mut Context, sized: &SizedTree, p: (f32, f32)) {
        let press = |(offset, child): &((f32, f32), SizedTree)| {
            let local = (p.0 - offset.0, p.1 - offset.1);
            let inside = local.0 >= 0.0 && local.1 >= 0.0 && local.0 < child.0.0 && local.1 < child.0.1;
            Box::new(MouseEvent { position: inside.then_some(local), state: MouseState::Pressed(MouseButton::Left) })
        };
        if let Some(toolbar) = sized.1.get(1) { self.2.event(ctx, &toolbar.1, press(toolbar)); }
        if let (Some(pages), Some(sized)) = (&mut self.3, sized.1.get(2)) { pages.event(ctx, &sized.1, press(sized)); }
    }

    fn act(&mut self, ctx: &mut Context, sized: &SizedTree, action: ToolbarAction) {
        match (action, IS_MOBILE) {
            (ToolbarAction::Hide, _) => {
                ctx.emit(ShowKeyboard(false));
                ctx.emit(event::TextInput::Focused(false));
            },
            // A phone: the page goes in the keyboard. Tapping the page that's
            // showing goes back to the letter keys. The message field gets the
            // focus, so emoji go into it (and the keyboard comes up).
            (ToolbarAction::Page(page), true) => {
                let page = match self.4.keyboard == Some(page) { true => KeyboardPage::Keys, false => page };
                if !self.2.has_focus() { self.focus_field(ctx, sized); }
                ctx.emit(MobileKeyboardEvent::Show(page));
                ctx.emit(ShowKeyboard(true));
            },
            // A computer: the page opens under the toolbar, or closes if it's open.
            (ToolbarAction::Page(page), false) => {
                let open = self.open_page();
                if let Some(pages) = &mut self.3 {
                    match open == Some(page) {
                        true => { pages.inner().close_search(); pages.display(false); },
                        false => { pages.inner().show(page); pages.display(true); },
                    }
                }
                self.show_state();
            },
        }
    }

    /// Highlights the toolbar button for what's showing.
    fn show_state(&mut self) {
        match IS_MOBILE {
            true => self.1.set_showing(self.4.keyboard, self.4.keyboard.is_some()),
            false => { let open = self.open_page(); self.1.set_showing(open, false) },
        }
    }
}

impl OnEvent for MessageBar {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(MouseEvent { position: Some(p), state: MouseState::Pressed(MouseButton::Left) }) = event.downcast_ref::<MouseEvent>() {
            match Self::part_at(sized, *p) {
                Some(Part::Field) => self.4.learning_id = true,
                // On a computer a press anywhere but a text field takes the
                // focus from it. Presses on the toolbar and the pages go to
                // them alone, so the message field keeps the focus (and gets
                // it if it hadn't), and emoji go into it. Not in the GIF
                // search, whose own field has the focus.
                Some(part) if !IS_MOBILE && (part == Part::Toolbar || !self.searching_gifs()) => {
                    if !self.2.has_focus() { self.focus_field(ctx, sized); }
                    self.press_beside_field(ctx, sized, *p);
                    return vec![];
                },
                _ => {},
            }
        } else if let Some(event::Selectable::Pressed(id, group)) = event.downcast_ref::<event::Selectable>()
            && group == TEXT_FIELD_GROUP && std::mem::take(&mut self.4.learning_id)
        {
            self.4.field_id = Some(id.clone());
        } else if let Some(BarKey(token, key)) = event.downcast_ref::<BarKey>() && *token == self.4.token {
            // From the emoji page (on a computer): straight into the field,
            // which gets the focus if it had lost it.
            match key {
                event::Key::Character(text) => self.2.insert(text),
                event::Key::Delete | event::Key::Backspace => self.2.delete_back(),
                _ => {},
            }
            if !self.2.has_focus() { self.focus_field(ctx, sized); }
        } else if event.downcast_ref::<TickEvent>().is_some() && let Some(frames) = self.4.checking {
            // Selecting it didn't give it the focus: forget the id, and tap it.
            self.4.checking = match (self.2.has_focus(), frames) {
                (true, _) => None,
                (false, 0) => { self.4.field_id = None; self.focus_field(ctx, sized); None },
                (false, n) => Some(n - 1),
            };
        } else if let Some(ToolbarEvent(token, action)) = event.downcast_ref::<ToolbarEvent>() && *token == self.4.token {
            self.act(ctx, sized, *action);
        } else if let Some(KeyboardShowing(showing)) = event.downcast_ref::<KeyboardShowing>() {
            self.4.keyboard = *showing;
            self.show_state();
        } else if !IS_MOBILE && self.searching_gifs()
            && (matches!(event.downcast_ref::<MobileKeyboardEvent>(), Some(MobileKeyboardEvent::CancelGifSearch)) || event.downcast_ref::<SendGifEvent>().is_some())
        {
            // The GIF search is ending (the pages end it): the message field
            // gets the focus back from the search field.
            self.focus_field(ctx, sized);
        }
        vec![event]
    }
}

/// The buttons under the message field. On a phone: the letter keys, emoji,
/// stickers, GIFs, and photos and videos, then at the end, while the keyboard
/// is up, the button that hides it. On a computer: emoji, stickers, GIFs,
/// and photos and videos.
#[derive(Debug, Component, Clone)]
struct Toolbar(Row, Option<ToolbarButton>, ToolbarButton, ToolbarButton, ToolbarButton, AttachButton, Bin<Stack, Opt<GhostIconButton>>);
impl OnEvent for Toolbar {}

impl Toolbar {
    fn new(theme: &Theme, token: u64) -> Self {
        let page = |icon: Icons, page: KeyboardPage| ToolbarButton::new(theme, icon, page, token);
        let mut hide = GhostIconButton::new(theme, Icons::DownArrow, move |ctx: &mut Context, _: &Theme| ctx.emit(ToolbarEvent(token, ToolbarAction::Hide)));
        hide.1.set_trigger_on_release();
        Toolbar(
            Row::new(SPACING, Offset::Center, Size::Fit, Padding::default()),
            IS_MOBILE.then(|| page(Icons::Keyboard, KeyboardPage::Keys)),
            page(Icons::Emoji, KeyboardPage::Emoji),
            page(Icons::Sticker, KeyboardPage::Stickers),
            page(Icons::Gif, KeyboardPage::Gifs),
            AttachButton::new(theme),
            // The rest of the row, with the hide button at its end.
            Bin(Stack(Offset::End, Offset::Center, Size::custom(|_: Vec<(f32, f32)>| (0.0, f32::MAX)), Size::Fit, Padding::default()), Opt::new(hide, false)),
        )
    }

    /// Highlights the button for the page showing (none for `None`), and
    /// shows the hide button while the keyboard is up.
    fn set_showing(&mut self, showing: Option<KeyboardPage>, keyboard_up: bool) {
        for button in [self.1.as_mut(), Some(&mut self.2), Some(&mut self.3), Some(&mut self.4)].into_iter().flatten() {
            button.set_active(showing == Some(button.2.page));
        }
        self.6.1.display(keyboard_up);
    }
}

/// A toolbar button that shows a page. While its page is showing it's
/// highlighted (a ghost button's pressed background behind its icon).
#[derive(Debug, Component, Clone)]
struct ToolbarButton(Stack, interactions::Button, #[skip] ToolbarButtonInfo);
impl OnEvent for ToolbarButton {}

#[derive(Debug, Clone)]
struct ToolbarButtonInfo { page: KeyboardPage, icon: Icons, token: u64, active: bool, theme: Theme }

impl ToolbarButton {
    fn new(theme: &Theme, icon: Icons, page: KeyboardPage, token: u64) -> Self {
        let info = ToolbarButtonInfo { page, icon, token, active: false, theme: theme.clone() };
        ToolbarButton(Stack::default(), Self::button(&info), info)
    }

    fn set_active(&mut self, active: bool) {
        if self.2.active == active { return; }
        self.2.active = active;
        self.1 = Self::button(&self.2);
    }

    fn button(info: &ToolbarButtonInfo) -> interactions::Button {
        let colors = theme::Button::get(info.theme.colors(), Variant::Ghost);
        // Highlighted: the pressed background, with the icon at full strength.
        let highlighted = ButtonColorScheme { background: colors.pressed.background, ..colors.default };
        let schemes = match info.active {
            true => [highlighted, highlighted, highlighted, colors.disabled],
            false => [colors.default, colors.hover, colors.pressed, colors.disabled],
        };
        let [default, hover, pressed, disabled] = schemes.map(|c| {
            IconButton::new(&info.theme, info.icon, ButtonStyle::Ghost, ButtonSize::Medium, c.background, c.outline, c.label)
        });
        let (token, page) = (info.token, info.page);
        let callback = move |ctx: &mut Context| ctx.emit(ToolbarEvent(token, ToolbarAction::Page(page)));
        interactions::Button::new_triggers_on_release(default, Some(hover), Some(pressed), Some(disabled), None::<IconButton>, callback, false)
    }
}

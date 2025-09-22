use pelican_ui::events::{MouseState, MouseEvent};
use pelican_ui::theme::ButtonColorScheme;
use pelican_ui::Context;

#[allow(clippy::module_inception)]
mod button;
pub use button::{Button, ButtonWidth, QuickActions};

mod icon;
pub use icon::IconButton;

/// The three styles of a [`Button`] or [`IconButton`].
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Ghost
}

/// The various states a [`Button`] or [`IconButton`] can be.
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum ButtonState {
    /// The normal, un-interacted state of the button.
    Default,
    /// The state of the button when it is  unclickable.
    Disabled,
    /// The state of the button when it has been selected or toggled on.
    Selected,
    /// The state of the button when it is toggled off.
    UnSelected,
    /// The state of the button when it is being clicked or pressed.
    Pressed,
    /// The state of the button when the mouse is hovering over it.
    Hover,
}

impl ButtonState {
    pub fn handle(&mut self, _ctx: &mut Context, event: MouseEvent) -> Option<Self> {
        let state = match self {
            ButtonState::Default | ButtonState::UnSelected if event.position.is_some() => {
                match event.state {
                    MouseState::Pressed => {
                        // ctx.hardware.vibrate();
                        Some(ButtonState::Pressed)
                    },
                    MouseState::Moved | MouseState::Scroll(..) => Some(if crate::config::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
                    _ => None
                }
            },
            ButtonState::Pressed => {
                match event.state {
                    MouseState::Released if event.position.is_some() => Some(if crate::config::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
                    MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
                    _ => None
                }
            },
            ButtonState::Hover => {
                match event.state {
                    MouseState::Pressed if event.position.is_some() => Some(ButtonState::Pressed),
                    MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
                    _ => None
                }
            }
            _ => None
        };
        if let Some(state) = state { *self = state; }
        state
    }

    pub fn color(&self, ctx: &mut Context, style: ButtonStyle) -> ButtonColorScheme {
        let schemes = &ctx.theme.colors.button;
        match (style, self) {
            (ButtonStyle::Primary, ButtonState::Default) => schemes.primary_default,
            (ButtonStyle::Primary, ButtonState::Disabled) => schemes.primary_disabled,
            (ButtonStyle::Primary, ButtonState::Hover) => schemes.primary_hover,
            (ButtonStyle::Primary, ButtonState::Pressed) => schemes.primary_pressed,
            (ButtonStyle::Primary, ButtonState::Selected) => schemes.primary_selected,
            (ButtonStyle::Primary, ButtonState::UnSelected) => schemes.ghost_disabled,

            (ButtonStyle::Secondary, ButtonState::Default) => schemes.secondary_default,
            (ButtonStyle::Secondary, ButtonState::Disabled) => schemes.secondary_disabled,
            (ButtonStyle::Secondary, ButtonState::Hover) => schemes.secondary_hover,
            (ButtonStyle::Secondary, ButtonState::Pressed) => schemes.secondary_pressed,
            (ButtonStyle::Secondary, ButtonState::Selected) => schemes.secondary_selected,
            (ButtonStyle::Secondary, ButtonState::UnSelected) => schemes.ghost_disabled,

            (ButtonStyle::Ghost, ButtonState::Default) => schemes.ghost_default,
            (ButtonStyle::Ghost, ButtonState::Disabled) => schemes.ghost_disabled,
            (ButtonStyle::Ghost, ButtonState::Hover) => schemes.ghost_hover,
            (ButtonStyle::Ghost, ButtonState::Pressed) => schemes.ghost_pressed,
            (ButtonStyle::Ghost, ButtonState::Selected) => schemes.ghost_selected,
            (ButtonStyle::Ghost, ButtonState::UnSelected) => schemes.ghost_disabled,
        }
    }
}

/// The set sizes of a [`Button`] or [`IconButton`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Large,
    Medium,
}

impl ButtonSize {
    /// Returns the button's text size, icon size, and content spacing.
    pub fn content(&self, ctx: &mut Context) -> (f32, f32, f32) { // text size, icon size, spacing
        let font_size = ctx.theme.fonts.size;
        match self {
            ButtonSize::Medium => (font_size.md, 16., 4.),
            ButtonSize::Large => (font_size.lg, 24., 12.)
        }
    }

    /// Returns the button's height and padding.
    pub fn sizes(&self) -> (f32, f32) { // height, padding
        match self {
            ButtonSize::Medium => (32., 12.),
            ButtonSize::Large => (48., 24.)
        }
    }
}

use rust_on_rails::prelude::*;
use crate::components::avatar::{Avatar, AvatarContent};
use crate::elements::images::Icon;
use crate::elements::shapes::OutlinedRectangle;
use crate::elements::text::{Text, TextStyle};
use crate::layout::{Offset, Padding, Row, Size, Stack, Wrap};

use super::{ButtonSize, ButtonState, ButtonStyle};

/// Defines the width behavior for the button.
#[derive(Debug, Clone, Copy)]
pub enum ButtonWidth {
    /// The button expands to fill the available space.
    Expand,
    
    /// The button's width hugs its content.
    Hug,
}

/// A clickable button component with customizable content, size, and styles.
#[derive(Component)]
pub struct Button(
    Stack, 
    OutlinedRectangle, 
    ButtonContent, 
    #[skip] ButtonStyle, 
    #[skip] ButtonState,
    #[skip] pub Box<dyn FnMut(&mut Context)>, 
);

impl Button {
    /// Creates a new `Button` component.
    ///
    /// # Parameters:
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `avatar`: An optional [`Avatar`] to display inside the button.
    /// - `icon_l`: An optional [`Icon`] to display on the left side of the button.
    /// - `label`: An optional [`Text`] label.
    /// - `icon_r`: An optional [`Icon`] to display on the right side of the button.
    /// - `size`: Defines the size of the button.
    /// - `width`: Defines the width of the button.
    /// - `style`: Defines the style of the button.
    /// - `state`: Specifies the initial state of the button.
    /// - `offset`: Specifies the button's content's offset. (usually [`Offset::Center`])
    /// - `on_click`: A closure that will be executed when the button is clicked.
    ///
    /// # Returns:
    /// A UI ready [`Button`] instance.
    ///
    /// # Example:
    /// ```
    /// let button = Button::new(
    ///     ctx, 
    ///     Some(avatar_data),
    ///     Some("left"),
    ///     Some("Click Me"),
    ///     Some("right"),
    ///     ButtonSize::Medium,
    ///     ButtonWidth::Hug,
    ///     ButtonStyle::Primary,
    ///     ButtonState::Default,
    ///     Offset::Center,
    ///     |ctx: &mut Context| println!("Button clicked!"),
    /// );
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        icon_l: Option<&'static str>,
        label: Option<&str>,
        icon_r: Option<&'static str>,
        size: ButtonSize,
        width: ButtonWidth,
        style: ButtonStyle,
        state: ButtonState,
        offset: Offset,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let (height, padding) = size.background();
        let colors = state.color(ctx, style);
        let content = ButtonContent::new(ctx, avatar, icon_l, label, icon_r, size, colors.label, padding);

        let width = match width {
            ButtonWidth::Hug => Size::custom(move |widths: Vec<(f32, f32)>|
                (widths[1].0, widths[1].1)
            ),
            ButtonWidth::Expand => Size::custom(move |widths: Vec<(f32, f32)>|
                (widths[1].0, f32::MAX)
            ),
        };

        let background = OutlinedRectangle::new(colors.background, colors.outline, height/2.0, 1.0);
        let layout = Stack(offset, Offset::Center, width, Size::Static(height), Padding::default());

        Button(layout, background, content, style, state, Box::new(on_click))
    }

    /// Updates the color of the button based on its current state and style.
    pub fn color(&mut self, ctx: &mut Context) {
        let colors = self.4.color(ctx, self.3);
        self.2.set_color(colors.label);
        *self.1.outline() = colors.outline;
        *self.1.background() = colors.background;
    }

    /// Returns a mutable reference to the current state of the button.
    pub fn status(&mut self) -> &mut ButtonState {&mut self.4}
}

impl OnEvent for Button {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if self.4.handle(ctx, *event).is_some() {
                self.color(ctx);
            }

            if let MouseEvent{state: MouseState::Pressed, position: Some(_)} = event {
                match self.4 {
                    ButtonState::Default | ButtonState::Hover | ButtonState::Pressed => (self.5)(ctx),
                    _ => {}
                }
            }
        }
        false
    }
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Button")
    }
}

#[derive(Debug, Component)]
struct ButtonContent(Row, Option<Avatar>, Option<Image>, Option<Text>, Option<Image>);
impl OnEvent for ButtonContent {}

impl ButtonContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        icon_l: Option<&'static str>,
        label: Option<&str>,
        icon_r: Option<&'static str>,
        size: ButtonSize,
        color: Color,
        padding: f32,
    ) -> Self {
        let (text_size, icon_size, spacing) = size.content(ctx);
        ButtonContent(
            Row::new(spacing, Offset::Center, Size::Fit, Padding(padding, 0.0, padding, 0.0)),
            avatar.map(|content| Avatar::new(ctx, content, None, false, icon_size, None)),
            icon_l.map(|icon| Icon::new(ctx, icon, color, icon_size)),
            label.map(|label| Text::new(ctx, label, TextStyle::Label(color), text_size, Align::Left)),
            icon_r.map(|icon| Icon::new(ctx, icon, color, icon_size)),
        )
    }

    fn set_color(&mut self, color: Color) {
        if let Some(icon) = &mut self.2 { icon.color = Some(color); }
        if let Some(text) = &mut self.3 { text.text().set_color(color); }
        if let Some(icon) = &mut self.4 { icon.color = Some(color); }
    }
}

impl Button {
    /// Creates a primary style button. Typically used for the main actions in the UI.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `label`: The text displayed on the button.
    /// - `on_click`:  A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Primary`] and state [`ButtonState::Default`].
    pub fn primary (
        ctx: &mut Context,
        label: &str,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            None,
            Some(label),
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Primary,
            ButtonState::Default,
            Offset::Center,
            on_click,
        )
    }

    /// Creates a secondary style button with optional left and right icons.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `icon_l`: An optional [`Icon`] to display on the left side of the button.
    /// - `label`: The text displayed on the button.
    /// - `icon_r`: An optional [`Icon`] to display on the right side of the button.
    /// - `on_click`: A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Secondary`] and state [`ButtonState::Default`].
    pub fn secondary(
        ctx: &mut Context,
        icon_l: Option<&'static str>,
        label: &str,
        icon_r: Option<&'static str>,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            icon_l,
            Some(label),
            icon_r,
            ButtonSize::Medium,
            ButtonWidth::Hug,
            ButtonStyle::Secondary,
            ButtonState::Default,
            Offset::Center,
            on_click,
        )
    }

    /// Creates a ghost style button, typically used for non-intrusive actions.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `label`: The text displayed on the button.
    /// - `on_click` A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Ghost`] and state [`ButtonState::Default`].
    pub fn ghost(
        ctx: &mut Context,
        label: &str,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            None,
            Some(label),
            None,
            ButtonSize::Medium,
            ButtonWidth::Hug,
            ButtonStyle::Ghost,
            ButtonState::Default,
            Offset::Center,
            on_click,
        )
    }

    /// Creates a disabled button, which cannot be interacted with.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `label`: The text displayed on the button.
    /// - `on_click`: A closure that defines the action when the button is clicked (this will not be triggered as the button is disabled).
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Primary`] and state [`ButtonState::Disabled`].
    pub fn disabled(
        ctx: &mut Context,
        label: &str,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            None,
            Some(label),
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Primary,
            ButtonState::Disabled,
            Offset::Center,
            on_click,
        )
    }

    /// Creates a numeric keypad style button, typically used for numbers or symbols on a keypad.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `label`: The text displayed on the button.
    /// - `icon`: An optional icon displayed on the button.
    /// - `on_click` A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Ghost`], state [`ButtonState::Default`], and size [`ButtonSize::Large`].
    pub fn keypad(
        ctx: &mut Context,
        label: Option<&str>,
        icon: Option<&'static str>,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            icon,
            label,
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Ghost,
            ButtonState::Default,
            Offset::Center,
            on_click,
        )
    }

    /// Creates a navigation button for desktop-style navigators, with optional selection.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `icon`: The icon to display on the button.
    /// - `label`: The text displayed on the button.
    /// - `selected`: A flag that determines if the button should be in the selected state.
    /// - `on_click` A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Ghost`] and either state [`ButtonState::Selected`] or [`ButtonState::Default`].
    pub fn navigation(
        ctx: &mut Context,
        icon: &'static str,
        label: &str,
        selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            Some(icon),
            Some(label),
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Ghost,
            if selected {ButtonState::Selected} else {ButtonState::Default},
            Offset::Start,
            on_click,
        )
    }

    /// Creates a profile photo button for desktop-style navigation with a photo.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `label`: The text displayed on the button.
    /// - `photo`: The photo or avatar content for the button.
    /// - `selected`: A flag that determines if the button should be in the pressed state.
    /// - `on_click` A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Ghost`] and either state [`ButtonState::Selected`] or [`ButtonState::Default`].
    pub fn photo(
        ctx: &mut Context,
        label: &str,
        photo: AvatarContent,
        selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            Some(photo),
            None,
            Some(label),
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Ghost,
            if selected {ButtonState::Selected} else {ButtonState::Default},
            Offset::Start,
            on_click,
        )
    }

    /// Creates a close page button, typically used for closing dialogs or pages.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `label`: The text displayed on the button.
    /// - `on_click` A closure that will be executed when the button is clicked.
    ///
    /// # Returns
    /// A [`Button`] with style [`ButtonStyle::Secondary`] and state [`ButtonState::Default`].
    pub fn close(
        ctx: &mut Context,
        label: &str,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            None,
            Some(label),
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Secondary,
            ButtonState::Default,
            Offset::Center,
            on_click,
        )
    }
}

/// A component that represents a set of quick action buttons displayed in a wrap layout.
///
/// The [`QuickActions`] component is used to display multiple buttons in a flexible wrap layout, where the buttons
/// can be customized and interact with various events. The component is designed to allow easy addition and removal
/// of buttons. Each button is displayed with a default margin between them for easy organization.
///
/// # Fields
/// - `Wrap`: A layout component that arranges the buttons in a wrap style, with adjustable spacing and alignment.
/// - `Vec<Button>`: A vector of buttons that represent the quick actions. Each button can trigger different actions when clicked.
///
/// # Example
/// ```rust
/// let quick_actions = QuickActions::new(vec![Button::primary(...), Button::secondary(...)]);
/// ```
#[derive(Debug, Component)]
pub struct QuickActions(Wrap, Vec<Button>);

impl OnEvent for QuickActions {}

impl QuickActions {
    /// Creates a new `QuickActions` component with a list of action buttons.
    pub fn new(buttons: Vec<Button>) -> Self {
        QuickActions(Wrap(8.0, 8.0, Offset::Start, Offset::Center, Padding::default()), buttons)
    }
}

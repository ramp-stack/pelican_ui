use pelican_ui::events::{MouseState, MouseEvent, OnEvent, Event, TickEvent};
use pelican_ui::drawable::{Drawable, Component, Image, Color, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::components::avatar::{Avatar, AvatarContent};
use crate::components::{Icon, Rectangle, Text, TextStyle};
use crate::layout::{Offset, Padding, Row, Size, Stack, Wrap, Opt};

use super::{ButtonSize, ButtonState, ButtonStyle};

use std::time::Instant;

/// ## Button
///
/// A clickable button component.  
///  
/// See various examples below.
#[derive(Component)]
pub struct Button(
    Stack, 
    Rectangle, 
    ButtonContent, 
    #[skip] ButtonStyle, 
    #[skip] ButtonState,
    #[skip] Box<dyn FnMut(&mut Context)>, 
    #[skip] Option<String>,
    #[skip] Option<Instant>,
    #[skip] Option<String>,
    #[skip] bool,
);

impl Button {
    #[allow(clippy::too_many_arguments)]
    /// Creates a new button.
    pub fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        icon_l: Option<(&'static str, ButtonFlair)>,
        label: Option<&str>,
        icon_r: Option<(&'static str, ButtonFlair)>,
        size: ButtonSize,
        width: ButtonWidth,
        style: ButtonStyle,
        state: ButtonState,
        offset: Offset,
        on_click: impl FnMut(&mut Context) + 'static,
        active_label: Option<String>,
    ) -> Self {
        let (height, padding) = size.sizes();
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

        let background = Rectangle::new(colors.background, height/2.0, Some((1.0, colors.outline)));
        let layout = Stack(offset, Offset::Center, width, Size::Static(height), Padding::default());

        Button(layout, background, content, style, state, Box::new(on_click), label.map(|l| l.to_string()), None, active_label, true)
    }

    /// Update the button's colors.
    pub fn color(&mut self, ctx: &mut Context) {
        let colors = self.4.color(ctx, self.3);
        self.2.set_color(colors.label);
        *self.1.background() = colors.background;
        if let Some(c) = self.1.outline() { *c = colors.outline; }
    }

    /// Update the state of the button based off the current state and two booleans.
    pub fn update_state(&mut self, ctx: &mut Context, should_disable: bool, should_enable: bool, status: &mut ButtonState) {
        let disabled = *self.status() == ButtonState::Disabled;
        let current_status = *self.status();
        if !disabled {*status = current_status;}
        if should_disable && !disabled {*self.status() = ButtonState::Disabled;}
        if should_enable {*self.status() = *status;}
        self.color(ctx);
    }

    /// Show or hide the flair on the left icon.
    pub fn show_flair_left(&mut self, hide: bool) {self.2.2.as_mut().map(|b| b.flair().as_mut().map(|i| i.display(hide)));}
    /// Show or hide the flair on the right icon.
    pub fn show_flair_right(&mut self, hide: bool) {self.2.4.as_mut().map(|b| b.flair().as_mut().map(|i| i.display(hide)));}
    /// Returns a mutable reference to the optional avatar.
    pub fn avatar(&mut self) -> &mut Option<Avatar> { &mut self.2.1 }
    /// Returns a mutable reference to the ButtonState.
    pub fn status(&mut self) -> &mut ButtonState {&mut self.4}
    /// Returns a mutable reference to the button's optional label.
    pub fn label(&mut self) -> &mut Option<Text> {&mut self.2.3}
    /// Sets the trigger of the on_click to either `On Press` or `On Release`
    pub fn set_trigger_on_press(&mut self, on_press: bool) {self.9 = on_press;}
}

impl OnEvent for Button {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some(timer) = self.7 {
                if timer.elapsed().as_millis() > 750 {
                    let new = self.6.clone().unwrap();
                    if let Some(l) = self.label().as_mut() { l.text().spans[0].text = new; }
                    self.7 = None;
                    *self.status() = ButtonState::Default;
                    self.color(ctx);
                }
            }
        } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if self.4.handle(ctx, *event).is_some() { self.color(ctx); }

            if (matches!(event, MouseEvent { state: MouseState::Pressed, position: Some(_) }) && self.9
            || matches!(event, MouseEvent { state: MouseState::Released, .. }) && !self.9) 
            && matches!(self.4, ButtonState::Default | ButtonState::Hover | ButtonState::Pressed) {
                if let Some(label) = self.8.clone() {
                    self.7 = Some(Instant::now());
                    *self.status() = ButtonState::Selected;
                    if let Some(l) = self.label().as_mut() {
                        l.text().spans[0].text = label.to_string();
                    }
                }
                ctx.hardware.haptic();
                (self.5)(ctx);
            
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

/// Defines the width of of the button.
#[derive(Debug, Clone, Copy)]
pub enum ButtonWidth {
    /// Button's width will expand to take up as much space as possible.
    Expand,
    /// Button's width will hug it's children.
    Hug,
}

#[derive(Debug, Component)]
struct ButtonContent(Row, Option<Avatar>, Option<ButtonIcon>, Option<Text>, Option<ButtonIcon>);
impl OnEvent for ButtonContent {}

impl ButtonContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ctx: &mut Context,
        avatar: Option<AvatarContent>,
        icon_l: Option<(&'static str, ButtonFlair)>,
        label: Option<&str>,
        icon_r: Option<(&'static str, ButtonFlair)>,
        size: ButtonSize,
        color: Color,
        padding: f32,
    ) -> Self {
        let (text_size, icon_size, spacing) = size.content(ctx);
        ButtonContent(
            Row::new(spacing, Offset::Center, Size::Fit, Padding(padding, 0.0, padding, 0.0)),
            avatar.map(|content| Avatar::new(ctx, content, None, false, icon_size, None)),
            icon_l.map(|(icon, flair)| ButtonIcon::new(ctx, (icon, color, icon_size), flair)),
            label.map(|label| Text::new(ctx, label, TextStyle::Label(color), text_size, Align::Left)),
            icon_r.map(|(icon, flair)| ButtonIcon::new(ctx, (icon, color, icon_size), flair)),
        )
    }

    fn set_color(&mut self, color: Color) {
        if let Some(icon) = &mut self.2 { icon.1.color = Some(color); }
        if let Some(text) = &mut self.3 { text.text().spans[0].color = color; }
        if let Some(icon) = &mut self.4 { icon.1.color = Some(color); }
    }
}

type ButtonFlair = Option<(&'static str, Color, bool)>;

#[derive(Debug, Component)]
struct ButtonIcon(Stack, Image, Option<Opt<Image>>);
impl OnEvent for ButtonIcon {}

impl ButtonIcon {
    #[allow(clippy::too_many_arguments)]
    fn new(ctx: &mut Context, icon: (&'static str, Color, f32), flair: ButtonFlair) -> Self {
        let i = Icon::new(ctx, icon.0, icon.1, icon.2);
        let flair = flair.map(|(i, c, h)| Opt::new(Icon::new(ctx, i, c, icon.2 / 1.8), h));
        ButtonIcon(Stack(Offset::End, Offset::Start, Size::Fit, Size::Fit, Padding::default()), i, flair)
    }

    fn flair(&mut self) -> &mut Option<Opt<Image>> {&mut self.2}
}

impl Button {
    /// ## Primary Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/primary_buttons.png"
    ///      alt="Primary Button Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = Button::primary(ctx, "Label", |ctx: &mut Context| println!("This button has been clicked!"));
    /// ```
    pub fn primary(
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
            None,
        )
    }

    /// ## Secondary Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/secondary_buttons.png"
    ///      alt="Secondary Button Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = Button::secondary(ctx, Some("copy") "Copy", None, |ctx: &mut Context| println!("This button has been clicked!"), Some("Copied"));
    /// ```
    pub fn secondary(
        ctx: &mut Context,
        icon_l: Option<&'static str>,
        label: &str,
        icon_r: Option<&'static str>,
        on_click: impl FnMut(&mut Context) + 'static,
        active_label: Option<&str>,
    ) -> Self {
        Button::new(
            ctx,
            None,
            icon_l.map(|i| (i, None)),
            Some(label),
            icon_r.map(|i| (i, None)),
            ButtonSize::Medium,
            ButtonWidth::Hug,
            ButtonStyle::Secondary,
            ButtonState::Default,
            Offset::Center,
            on_click,
            active_label.map(|a| a.to_string()),
        )
    }

    /// ## Ghost Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/ghost_buttons.png"
    ///      alt="Ghost Button Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = Button::ghost(ctx, "Next", |ctx: &mut Context| println!("This button has been clicked!"));
    /// ```
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
            None,
        )
    }

    /// Creates a primary button defaulting to the disabled state.
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
            None,
        )
    }

    /// Creates a button designed for keypads components.
    pub fn keypad(
        ctx: &mut Context,
        label: Option<&str>,
        icon: Option<&'static str>,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        Button::new(
            ctx,
            None,
            icon.map(|i| (i, None)),
            label,
            None,
            ButtonSize::Large,
            ButtonWidth::Expand,
            ButtonStyle::Ghost,
            ButtonState::Default,
            Offset::Center,
            on_click,
            None,
        )
    }

    /// Creates a button designed for the interface navigators.
    pub fn navigation(
        ctx: &mut Context,
        icon: &'static str,
        label: &str,
        selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let color = ctx.theme.colors.brand.primary;
        let icon = (!crate::config::IS_WEB).then_some((icon, Some(("notification", color, false))));
        let offset = if crate::config::IS_WEB {Offset::Center} else {Offset::Start};
        let size = if crate::config::IS_WEB {ButtonWidth::Hug} else {ButtonWidth::Expand};
        Button::new(
            ctx,
            None,
            icon,
            Some(label),
            None,
            ButtonSize::Large,
            size,
            ButtonStyle::Ghost,
            if selected {ButtonState::Selected} else {ButtonState::Default},
            offset,
            on_click,
            None,
        )
    }

    /// Creates a button with a user avatar.
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
            None,
        )
    }

    /// Creates a button designed for the ending or closing page of a flow.
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
            None,
        )
    }
}

/// # Quick Actions
///
/// A wrapped group of quick action buttons.
///
/// # Example
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/quick_actions.png"
///      alt="Quick Action Example"
///      width="400">
///
/// ```rust
/// let button1 = Button::secondary(ctx, Some("qr_code"), "Scan QR Code", None, |ctx: &mut Context| println!("Save clicked"));
/// let button2 = Button::new(ctx, Some("paste"), "Paste", None, |ctx: &mut Context| println!("Cancel clicked"));
/// let button3 = Button::new(ctx, Some("accounts"), "Select Contact", None, |ctx: &mut Context| println!("Delete clicked"));
///
/// let quick_actions = QuickActions::new(vec![button1, button2, button3]);
/// ```
#[derive(Debug, Component)]
pub struct QuickActions(Wrap, Vec<Button>);

impl OnEvent for QuickActions {}

impl QuickActions {
    pub fn new(buttons: Vec<Button>) -> Self {
        QuickActions(Wrap::new(8.0, 8.0), buttons)
    }
}

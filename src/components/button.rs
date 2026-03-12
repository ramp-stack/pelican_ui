use prism::event::OnEvent;
use prism::{Context, drawables};
use prism::canvas::{Image, Align};
use prism::drawable::{Drawable, Component};
use prism::layout::{Wrap, Offset, Padding, Row, Size, Stack};

use ptsd::interactions;
use ptsd::theme::{Color, TextSize};

use crate::theme::{self, Variant, Theme, ButtonColorScheme, Icons};
use crate::components::text::{Text, TextStyle};
use crate::components::{Icon, Rectangle};

#[derive(Debug, Component, Clone)]
pub struct QuickActions{
    layout: Wrap, 
    buttons: Vec<SecondaryButton>
}

impl OnEvent for QuickActions {}
impl QuickActions {
    pub fn new(theme: &Theme, actions: Vec<(String, Option<String>, impl FnMut(&mut Context, &Theme) + Clone + 'static)>) -> Self {
        let buttons = actions.into_iter().map(|(l, o, a)| {
            SecondaryButton::medium(theme, Icons::Edit, &l, o.as_deref(), a)
        }).collect();
        QuickActions{layout: Wrap::start(8.0, 8.0), buttons}
    }
}

/// ## Primary Button
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/primary_buttons.png"
///      alt="Primary Button Example"
///      width="250">
///
/// ### Example
/// ```rust
/// let button = PrimaryButton::new(ctx, "Label", |ctx: &mut Context| println!("This button has been clicked!"), false);
/// ```
#[derive(Debug, Component, Clone)]
pub struct PrimaryButton(Stack, pub interactions::Button);
impl OnEvent for PrimaryButton {}
impl PrimaryButton {
    pub fn new(theme: &Theme, label: &str, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Primary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            Button::new(drawables![text], ButtonSize::Large, ButtonWidth::Fill, Offset::Center, colors.background, colors.outline)
        });
        
        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        PrimaryButton(Stack::default(), interactions::Button::new(default, Some(hover), Some(pressed), Some(disabled), callback, true))
    }

    pub fn default(theme: &Theme) -> Self { 
        Self::new(theme, "Primary Button", |_: &mut Context, _: &Theme| println!("Pressed...."))
    }
}

/// ## Secondary Button
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/secondary_buttons.png"
///      alt="Secondary Button Example"
///      width="250">
///
/// ### Example
/// ```rust
/// let button = SecondaryButton::medium(ctx, "edit", "Copy", Some("Copied"), |ctx: &mut Context| println!("This button has been clicked!"));
/// ```
#[derive(Debug, Component, Clone)]
pub struct SecondaryButton(Stack, pub interactions::Button);
impl OnEvent for SecondaryButton {}
impl SecondaryButton {
    pub fn medium(theme: &Theme, icon: Icons, label: &str, active_label: Option<&str>, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.disabled];
        let [default, hover, disabled] = buttons.map(|colors| Self::_medium(theme, icon, label, colors));
        let pressed = Self::_medium(theme, icon, active_label.unwrap_or(label), colors.pressed);

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryButton(Stack::default(), interactions::Button::new(default, Some(hover), Some(pressed), Some(disabled), callback, false))
    }

    fn _medium(theme: &Theme, icon: Icons, label: &str, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Medium.font();
        let icon_size = ButtonSize::Medium.icon();
        let text = Text::new(theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
        let icon = Icon::new(theme, icon, Some(colors.label), icon_size);
        Button::new(drawables![icon, text], ButtonSize::Medium, ButtonWidth::Fit, Offset::Center, colors.background, colors.outline)
    }

    pub fn large(theme: &Theme, label: &str,  mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None);
            Button::new(drawables![text], ButtonSize::Large, ButtonWidth::Fill, Offset::Center, colors.background, colors.outline)
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryButton(Stack::default(), interactions::Button::new(default, Some(hover), Some(pressed), Some(disabled), callback, false))
    }

    pub fn default(theme: &Theme) -> Self { 
        Self::medium(theme, Icons::Edit, "Secondary", None, |_: &mut Context, _: &Theme| println!("Pressed...."))
    }
}

/// ## Secondary Icon Button
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/secondary_icons.png"
///      alt="Secondary Icons Example"
///      width="250">
///
/// ### Example
/// ```rust
/// let button = SecondaryIconButton::new(ctx, "info", |ctx: &mut Context| println!("This button has been clicked!"));
/// ```
#[derive(Debug, Component, Clone)]
pub struct SecondaryIconButton(Stack, pub interactions::Button);
impl OnEvent for SecondaryIconButton {}
impl SecondaryIconButton {
    pub fn large(theme: &Theme, icon: Icons, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            IconButton::new(theme, icon, ButtonStyle::Secondary, ButtonSize::Large, colors.background, colors.outline, colors.label)
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryIconButton(Stack::default(), interactions::Button::new(default, Some(hover), Some(pressed), Some(disabled), callback, false))
    }

    pub fn medium(theme: &Theme, icon: Icons, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            IconButton::new(theme, icon, ButtonStyle::Secondary, ButtonSize::Medium, colors.background, colors.outline, colors.label)
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryIconButton(Stack::default(), interactions::Button::new(default, Some(hover), Some(pressed), Some(disabled), callback, false))
    }

    pub fn default(theme: &Theme) -> Self { 
        Self::medium(theme, Icons::Explore, |_: &mut Context, _: &Theme| println!("Pressed...."))
    }
}

/// ## Ghost Icon Button
///
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/ghost_icons.png"
///      alt="Ghost Icons Example"
///      width="250">
///
/// ### Example
/// ```rust
/// let button = GhostIconButton::new(ctx, "explore", |ctx: &mut Context| println!("This button has been clicked!"));
/// ```
#[derive(Debug, Component, Clone)]
pub struct GhostIconButton(Stack, pub interactions::Button);
impl OnEvent for GhostIconButton {}
impl GhostIconButton {
    pub fn new(theme: &Theme, icon: Icons, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            IconButton::new(theme, icon, ButtonStyle::Ghost, ButtonSize::Medium, colors.background, colors.outline, colors.label)
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        GhostIconButton(Stack::default(), interactions::Button::new(default, Some(hover), Some(pressed), Some(disabled), callback, false))
    }

    pub fn default(theme: &Theme) -> Self { 
        Self::new(theme, Icons::Left, |_: &mut Context, _: &Theme| println!("Pressed...."))
    }
}

#[derive(Debug, Component, Clone)]
pub(crate) struct IconButton(Stack, Rectangle, Image);
impl OnEvent for IconButton {}

impl IconButton {
    pub(crate) fn new(
        theme: &Theme,
        icon: Icons,
        style: ButtonStyle,
        size: ButtonSize,
        background: Color,
        outline: Color,
        label: Color,
    ) -> Self {
        let (size, icon_size, radius) = size.icon_button(style);
        let icon = Icon::new(theme, icon, Some(label), icon_size);
        let background = Rectangle::new(background, radius, Some((1.0, outline)));
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(size), Size::Static(size), Padding::default());
        IconButton(layout, background, icon)
    }
}

#[derive(Debug, Component, Clone)]
pub(crate) struct Button(Stack, Rectangle, ButtonContent);
impl OnEvent for Button {}

impl Button {
    pub(crate) fn new(
        content: Vec<Box<dyn Drawable>>,
        size: ButtonSize,
        width: ButtonWidth,
        offset: Offset,
        background: Color,
        outline: Color,
    ) -> Self {
        let (spacing, height, padding) = size.get();
        let content = ButtonContent::new(content, padding, spacing);
        let background = Rectangle::new(background, height / 2.0, Some((1.0, outline)));
        let layout = Stack(offset, Offset::Center, width.get(), Size::Static(height), Padding::default());
        Button(layout, background, content)
    }
}

#[derive(Debug, Component, Clone)]
struct ButtonContent(Row, Vec<Box<dyn Drawable>>);
impl OnEvent for ButtonContent {}
impl ButtonContent {
    fn new(content: Vec<Box<dyn Drawable>>, padding: Padding, spacing: f32) -> Self {
        ButtonContent(Row::new(spacing, Offset::Center, Size::Fit, padding), content)
    }
}

/// Various button styles.
#[derive(Eq, Clone, Copy, Debug, PartialEq)]
pub enum ButtonStyle {Primary, Secondary, Ghost}

/// Available button width behaviors.
#[derive(Eq, Clone, Copy, Debug, PartialEq)]
pub enum ButtonWidth {Fit, Fill}
impl ButtonWidth{
    pub(crate) fn get(&self) -> Size {
        match self {
            ButtonWidth::Fit => Size::custom(move |w: Vec<(f32, f32)>| (w[1].0, w[1].1)),
            ButtonWidth::Fill => Size::Fill,
        }
    }
}

/// Available button sizes and their corresponding layout, font, and icon properties.
#[derive(Eq, Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {Large, Medium}
impl ButtonSize {
    /// Regular button sizing
    pub fn get(&self) -> (f32, f32, Padding) {
        match self {
            ButtonSize::Medium => (4.0, 32.0, Padding(12.0, 0.0, 12.0, 0.0)),
            ButtonSize::Large => (12.0, 48.0, Padding(24.0, 0.0, 24.0, 0.0))
        }
    }

    /// Regular button font size
    pub fn font(&self) -> TextSize {
        match self {
            ButtonSize::Medium => TextSize::Md,
            ButtonSize::Large => TextSize::Lg,
        }
    }

    /// Regular button icon size
    pub fn icon(&self) -> f32 {
        match self {
            ButtonSize::Medium => 16.0,
            ButtonSize::Large => 24.0,
        }
    }

    /// Icon button outer size, inner icon size, and corner radius
    pub fn icon_button(&self, style: ButtonStyle) -> (f32, f32, f32) {
        match (style, self) {
            (ButtonStyle::Secondary, ButtonSize::Large) => (52.0, 32.0, 12.0),
            (ButtonStyle::Secondary, ButtonSize::Medium) => (36.0, 20.0, 8.0),
            (ButtonStyle::Ghost, ButtonSize::Large) => (52.0, 48.0, 12.0),
            (ButtonStyle::Ghost, ButtonSize::Medium) => (36.0, 32.0, 8.0),
            (ButtonStyle::Primary, ButtonSize::Large) => (52.0, 48.0, 12.0),
            (ButtonStyle::Primary, ButtonSize::Medium) => (36.0, 32.0, 8.0),
        }
    }
}

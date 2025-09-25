use mustache::events::OnEvent;
use mustache::drawable::{Drawable, Component, Color, Align};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component};

use crate::Callback;
use crate::components::{Icon, Rectangle, Text, TextStyle};
use crate::layout::{Offset, Padding, Row, Size, Stack};
use crate::theme::{ButtonColorScheme, ButtonColorSet};
use crate::components::interactions::ButtonState;
use crate::components::interactions;
use crate::plugin::PelicanUI;

#[derive(Debug, Component)]
pub struct PrimaryButton(Stack, interactions::Button);
impl OnEvent for PrimaryButton {}
impl PrimaryButton {
    pub fn new(ctx: &mut Context, label: &str, on_click: Callback, is_disabled: bool) -> Self {
        let state = if is_disabled {ButtonState::Disabled} else {ButtonState::Default};
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.primary;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|c| Self::_button(ctx, label, c));
        PrimaryButton(Stack::default(), interactions::Button::new(on_click, default, hover, pressed, selected, disabled, state))
    }

    fn _button(ctx: &mut Context, label: &str, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Large.font(ctx);
        let text = Text::new(ctx, label, TextStyle::Label(colors.label), font_size, Align::Left);
        Button::new(vec![Box::new(text)], ButtonSize::Large, ButtonWidth::Fill, Offset::Center, colors.background, colors.label)
    }
}

#[derive(Debug, Component)]
pub struct SecondaryButton(Stack, interactions::Button);
impl OnEvent for SecondaryButton {}

//TODO: Implement the active_label again.

impl SecondaryButton {
    pub fn medium(ctx: &mut Context, icon: &'static str, label: &str, active_label: Option<&str>, on_click: Callback) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.secondary;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|c| Self::_medium(ctx, icon, label, c));
        let selected = Self::_medium(ctx, icon, active_label.unwrap_or(label), colors.pressed);
        SecondaryButton(Stack::default(), interactions::Button::new(on_click, default, hover, pressed, selected, disabled, ButtonState::Default))
    }

    fn _medium(ctx: &mut Context, icon: &'static str, label: &str, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Medium.font(ctx);
        let icon_size = ButtonSize::Medium.icon();
        let text = Text::new(ctx, label, TextStyle::Label(colors.label), font_size, Align::Left);
        let icon = Icon::new(ctx, icon, colors.label, icon_size);
        Button::new(vec![Box::new(icon), Box::new(text)], ButtonSize::Medium, ButtonWidth::Fit, Offset::Center, colors.background, colors.label)
    }

    pub fn large(ctx: &mut Context, label: &str, on_click: Callback) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.secondary;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|c| Self::_large(ctx, label, c));
        SecondaryButton(Stack::default(), interactions::Button::new(on_click, default, hover, pressed, selected, disabled, ButtonState::Default))
    }

    fn _large(ctx: &mut Context, label: &str, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Large.font(ctx);
        let text = Text::new(ctx, label, TextStyle::Label(colors.label), font_size, Align::Left);
        Button::new(vec![Box::new(text)], ButtonSize::Large, ButtonWidth::Fill, Offset::Center, colors.background, colors.label)
    }
}

#[derive(Debug, Component)]
pub struct GhostButton(Stack, interactions::Button);
impl OnEvent for GhostButton {}
impl GhostButton {
    pub fn navigation(ctx: &mut Context, icon: &'static str, label: &str, on_click: Callback, is_selected: bool) -> Self {
        let state = if is_selected {ButtonState::Selected} else {ButtonState::Default};
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|c| Self::_navigation(ctx, icon, label, c));
        GhostButton(Stack::default(), interactions::Button::new(on_click, default, hover, pressed, selected, disabled, state))
    }

    fn _navigation(ctx: &mut Context, icon: &'static str, label: &str, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Large.font(ctx);
        let icon_size = ButtonSize::Large.icon();
        let text = Text::new(ctx, label, TextStyle::Label(colors.label), font_size, Align::Left);
        let icon = Icon::new(ctx, icon, colors.label, icon_size);
        Button::new(vec![Box::new(icon), Box::new(text)], ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.label)
    }

    pub fn keypad_number(ctx: &mut Context, label: &str, on_click: Callback) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|c| Self::_keypad(ctx, None, Some(label), c));
        GhostButton(Stack::default(), interactions::Button::new(on_click, default, hover, pressed, selected, disabled, ButtonState::Default))
    }

    pub fn keypad_backspace(ctx: &mut Context, on_click: Callback) -> Self {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button.ghost;
        let buttons = [colors.default, colors.hover, colors.pressed, colors.pressed, colors.disabled];
        let [default, hover, pressed, selected, disabled] = buttons.map(|c| Self::_keypad(ctx, Some("back"), None, c));
        GhostButton(Stack::default(), interactions::Button::new(on_click, default, hover, pressed, selected, disabled, ButtonState::Default))
    }

    fn _keypad(ctx: &mut Context, icon: Option<&'static str>, label: Option<&str>, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Large.font(ctx);
        let icon_size = ButtonSize::Large.icon();
        let mut content: Vec<Box<dyn Drawable>> = Vec::new();
        if let Some(l) = label {content.push(Box::new(Text::new(ctx, l, TextStyle::Label(colors.label), font_size, Align::Left)));}
        if let Some(i) = icon {content.push(Box::new(Icon::new(ctx, i, colors.label, icon_size)));}
        Button::new(content, ButtonSize::Large, ButtonWidth::Fill, Offset::Start, colors.background, colors.label)
    }
}

#[derive(Debug, Component)]
pub struct Button(Stack, Rectangle, ButtonContent);
impl OnEvent for Button {}

impl Button {
    pub fn new(
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

#[derive(Debug, Component)]
struct ButtonContent(Row, Vec<Box<dyn Drawable>>);
impl OnEvent for ButtonContent {}
impl ButtonContent {
    fn new(content: Vec<Box<dyn Drawable>>, padding: Padding, spacing: f32) -> Self {
        ButtonContent(Row::new(spacing, Offset::Center, Size::Fit, padding), content)
    }
}


#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum ButtonWidth {Fit, Fill}
impl ButtonWidth{
    pub fn get(&self) -> Size {
        match self {
            ButtonWidth::Fit => Size::Fit,
            ButtonWidth::Fill => Size::Fill,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum ButtonStyle {Primary, Secondary, Ghost}

impl ButtonStyle {
    pub fn get(&self, ctx: &mut Context) -> ButtonColorSet {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors.button;
        match self {
            ButtonStyle::Primary => colors.primary,
            ButtonStyle::Secondary => colors.secondary,
            ButtonStyle::Ghost => colors.ghost,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum ButtonSize {Large, Medium}
impl ButtonSize {
    pub fn get(&self) -> (f32, f32, Padding) {
        match self {
            ButtonSize::Medium => (4.0, 32.0, Padding(12.0, 0.0, 12.0, 0.0)),
            ButtonSize::Large => (12.0, 48.0, Padding(24.0, 0.0, 24.0, 0.0))
        }
    }

    pub fn font(&self, ctx: &mut Context) -> f32 {
        let size = ctx.get::<PelicanUI>().get().0.theme().fonts.size;
        match self {
            ButtonSize::Medium => size.md,
            ButtonSize::Large => size.lg,
        }
    }

    pub fn icon(&self) -> f32 {
        match self {
            ButtonSize::Medium => 16.0,
            ButtonSize::Large => 24.0,
        }
    }
}
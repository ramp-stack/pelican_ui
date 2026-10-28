use prism::Context;
use prism::drawable::text::Align;
use prism::drawable::{Drawable, Component};
use ptsd::layout::{Wrap, Offset, Padding, Row, Size, Stack};
use ptsd::interactions;
use ptsd::display::Bin;
use ptsd::theme::{Color, TextSize};

use crate::Callback;
use crate::theme::{self, Variant, Theme, ButtonColorScheme, Icons};
use crate::components::text::{Text, TextStyle};
use crate::components::{Icon, Rectangle};

#[derive(Eq, Clone, Copy, Debug, PartialEq)]
pub enum ButtonStyle {Primary, Secondary, Ghost}

#[derive(Eq, Clone, Copy, Debug, PartialEq)]
pub enum ButtonWidth {Fit, Fill}
impl ButtonWidth {
    pub fn get(&self) -> Size {
        match self {
            ButtonWidth::Fit => Size::custom(move |w: Vec<(f32, f32)>| (w[1].0, w[1].1)),
            ButtonWidth::Fill => Size::Fill,
        }
    }
}

#[derive(Eq, Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {Large, Medium}
impl ButtonSize {
    pub fn get(&self) -> (f32, f32, Padding) {
        match self {
            ButtonSize::Medium => (4.0, 32.0, Padding(12.0, 0.0, 12.0, 0.0)),
            ButtonSize::Large => (12.0, 48.0, Padding(24.0, 0.0, 24.0, 0.0))
        }
    }

    pub fn font(&self) -> TextSize {
        match self {
            ButtonSize::Medium => TextSize::Md,
            ButtonSize::Large => TextSize::Lg,
        }
    }

    pub fn icon(&self) -> f32 {
        match self {
            ButtonSize::Medium => 16.0,
            ButtonSize::Large => 24.0,
        }
    }

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

pub struct ActionData {
    pub label: String,
    pub active: Option<String>,
    pub icon: Icons,
    pub on_click: Box<dyn Callback>,
}
impl ActionData {
    pub fn new(label: &str, active: Option<&str>, icon: Icons, on_click: Box<dyn Callback>) -> Self {
        ActionData {label: label.to_string(), active: active.map(|l| l.to_string()), icon, on_click}
    }
}

#[derive(Debug, Component, Clone)]
pub struct QuickActions {
    layout: Wrap,
    buttons: Vec<SecondaryButton>,
}

impl QuickActions {
    pub fn new(theme: &Theme, offset: Offset, actions: Vec<ActionData>) -> Self {
        QuickActions {
            layout: match offset {
                Offset::Center => Wrap::center(8.0, 8.0),
                Offset::Start => Wrap::start(8.0, 8.0),
                _ => Wrap::end(8.0, 8.0),
            },
            buttons: actions.into_iter().map(|action| {
                let mut a = action.on_click.clone();
                SecondaryButton::medium(
                    theme, action.icon, &action.label, action.active.as_deref(), Stack::default(),
                    move |ctx: &mut Context, theme: &Theme| (a)(ctx, theme),
                )
            }).collect()
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct IconButtonGroup(Row, Vec<SecondaryIconButton>);
impl IconButtonGroup {
    pub fn new(theme: &Theme, actions: Vec<(Icons, Box<dyn Callback>)>) -> Self {
        let last = actions.len().saturating_sub(1);
        IconButtonGroup(Row::center(0.0), actions.into_iter().enumerate().map(|(index, (icon, action))| {
            let padding = Padding(
                if index == 0 {0.0} else {12.0}, 0.0,
                if index == last {0.0} else {12.0}, 0.0,
            );
            let layout = Stack::new(Offset::Center, Offset::Center, Size::Fit, Size::Fit, padding);
            SecondaryIconButton::large(theme, icon, layout, action)
        }).collect())
    }
}

#[derive(Debug, Component, Clone)]
pub struct PrimaryButton(Stack, pub interactions::Button);
impl PrimaryButton {
    pub fn new(theme: &Theme, label: &str, layout: Stack, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Primary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(
                theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None,
            ).fit();
            Bin(layout.clone(), Button::new(
                vec![Box::new(text)], ButtonSize::Large, ButtonWidth::Fill,
                Offset::Center, colors.background, colors.outline,
            ))
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        PrimaryButton(Stack::default(), interactions::Button::new(
            default, Some(hover), Some(pressed), Some(disabled), None::<Bin<Stack, Button>>, callback, true,
        ))
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, "Primary Button", Stack::default(), |_: &mut Context, _: &Theme| {
            println!("Pressed....")
        })
    }
}

#[derive(Debug, Component, Clone)]
pub struct SecondaryButton(Stack, pub interactions::Button);
impl SecondaryButton {
    pub fn medium(
        theme: &Theme, icon: Icons, label: &str, active_label: Option<&str>, layout: Stack,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
    ) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            Bin(layout.clone(), Self::_medium(theme, icon, label, colors))
        });
        let feedback = active_label.map(|al| Bin(layout, Self::_medium(theme, icon, al, colors.default)));

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryButton(Stack::default(), interactions::Button::new(
            default, Some(hover), Some(pressed), Some(disabled), feedback, callback, false,
        ))
    }

    fn _medium(theme: &Theme, icon: Icons, label: &str, colors: ButtonColorScheme) -> Button {
        let font_size = ButtonSize::Medium.font();
        let icon_size = ButtonSize::Medium.icon();
        let text = Text::new(
            theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None,
        ).fit();
        let icon = Icon::new(theme, icon, Some(colors.label), icon_size);
        Button::new(
            vec![Box::new(icon), Box::new(text)], ButtonSize::Medium, ButtonWidth::Fit,
            Offset::Center, colors.background, colors.outline,
        )
    }

    pub fn large(theme: &Theme, label: &str, layout: Stack, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            let font_size = ButtonSize::Large.font();
            let text = Text::new(
                theme, label, font_size, TextStyle::Label(colors.label), Align::Left, None,
            ).fit();
            Bin(layout.clone(), Button::new(
                vec![Box::new(text)], ButtonSize::Large, ButtonWidth::Fill,
                Offset::Center, colors.background, colors.outline,
            ))
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryButton(Stack::default(), interactions::Button::new(
            default, Some(hover), Some(pressed), Some(disabled), None::<Bin<Stack, Button>>, callback, false,
        ))
    }

    pub fn default(theme: &Theme) -> Self {
        Self::medium(theme, Icons::Edit, "Secondary", None, Stack::default(), |_: &mut Context, _: &Theme| {
            println!("Pressed....")
        })
    }
}

#[derive(Debug, Component, Clone)]
pub struct SecondaryIconButton(Stack, pub interactions::Button);
impl SecondaryIconButton {
    pub fn large(theme: &Theme, icon: Icons, layout: Stack, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            Bin(layout.clone(), IconButton::new(
                theme, icon, ButtonStyle::Secondary, ButtonSize::Large,
                colors.background, colors.outline, colors.label,
            ))
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryIconButton(Stack::default(), interactions::Button::new(
            default, Some(hover), Some(pressed), Some(disabled),
            None::<Bin<Stack, IconButton>>, callback, false,
        ))
    }

    pub fn medium(theme: &Theme, icon: Icons, layout: Stack, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Secondary);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            Bin(layout.clone(), IconButton::new(
                theme, icon, ButtonStyle::Secondary, ButtonSize::Medium,
                colors.background, colors.outline, colors.label,
            ))
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        SecondaryIconButton(Stack::default(), interactions::Button::new(
            default, Some(hover), Some(pressed), Some(disabled),
            None::<Bin<Stack, IconButton>>, callback, false,
        ))
    }

    pub fn default(theme: &Theme) -> Self {
        Self::medium(theme, Icons::Explore, Stack::default(), |_: &mut Context, _: &Theme| {
            println!("Pressed....")
        })
    }
}

#[derive(Debug, Component, Clone)]
pub struct GhostIconButton(Stack, pub interactions::Button);
impl GhostIconButton {
    pub fn new(theme: &Theme, icon: Icons, layout: Stack, mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static) -> Self {
        let colors = theme::Button::get(theme.colors(), Variant::Ghost);
        let buttons = [colors.default, colors.hover, colors.pressed, colors.disabled];
        let [default, hover, pressed, disabled] = buttons.map(|colors| {
            Bin(layout.clone(), IconButton::new(
                theme, icon, ButtonStyle::Ghost, ButtonSize::Medium,
                colors.background, colors.outline, colors.label,
            ))
        });

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        GhostIconButton(Stack::default(), interactions::Button::new(
            default, Some(hover), Some(pressed), Some(disabled),
            None::<Bin<Stack, IconButton>>, callback, false,
        ))
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, Icons::Left, Stack::default(), |_, _| println!("Pressed...."))
    }
}

#[derive(Debug, Component, Clone)]
pub struct IconButton(Stack, Rectangle, Icon);

impl IconButton {
    pub fn new(theme: &Theme, icon: Icons, style: ButtonStyle, size: ButtonSize, background: Color, outline: Color, label: Color) -> Self {
        let (size, icon_size, radius) = size.icon_button(style);
        let icon = Icon::new(theme, icon, Some(label), icon_size);
        let background = Rectangle::new(background, radius, Some((1.0, outline)));
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::Static(size), Size::Static(size), Padding::default(),
        );
        IconButton(layout, background, icon)
    }
}

#[derive(Debug, Component, Clone)]
pub struct Button(Stack, Rectangle, ButtonContent);

impl Button {
    pub fn new(content: Vec<Box<dyn Drawable>>, size: ButtonSize, width: ButtonWidth, offset: Offset, background: Color, outline: Color) -> Self {
        let (spacing, height, padding) = size.get();
        let content = ButtonContent::new(content, padding, spacing);
        let background = Rectangle::new(background, height / 2.0, Some((1.0, outline)));
        let layout = Stack::new(
            offset, Offset::Center, width.get(), Size::Static(height), Padding::default(),
        );
        Button(layout, background, content)
    }
}

#[derive(Debug, Component, Clone)]
struct ButtonContent(Row, Vec<Box<dyn Drawable>>);
impl ButtonContent {
    fn new(content: Vec<Box<dyn Drawable>>, padding: Padding, spacing: f32) -> Self {
        ButtonContent(Row::new(spacing, Offset::Center, Size::Fit, padding), content)
    }
}

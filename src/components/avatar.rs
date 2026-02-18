use prism::event::{OnEvent, Event};
use prism::canvas::{Image, Shape, ShapeType};
use prism::layout::{Stack, Offset, Size, Padding};
use prism::Context;
use prism::drawable::{Component, SizedTree};

use crate::Callback;
use crate::theme::Theme;
use crate::theme::Color;
use crate::components::{Icon, Circle};

use image::RgbaImage;
use std::sync::Arc;

/// ## Avatar
///
/// Displays a user avatar.  
///  
/// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/avatar.png"
///      alt="Avatar Example"
///      width="400">
///
/// ### Example
/// ```rust
/// let avatar = Avatar::new(
///     ctx, 
///     AvatarContent::Icon("profile", AvatarIconStyle::Secondary), 
///     None, 
///     false, 
///     AvatarSize::Lg,
///     None
/// );
/// ```
#[derive(Component, Clone)]
pub struct Avatar {
    _layout: Stack,
    _avatar: PrimaryAvatar,
    _flair: Option<Flair>,
    #[skip] _size: AvatarSize,
    #[skip] _on_click: Option<Box<dyn Callback>>,
    #[skip] pub content: AvatarContent,
    #[skip] pub flair: Option<(String, AvatarIconStyle)>,
    #[skip] pub outline: bool,
}

impl std::fmt::Debug for Avatar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Avatar...")
    }
}

impl Avatar {
    pub fn new(
        theme: &Theme, 
        content: AvatarContent, 
        flair: Option<(String, AvatarIconStyle)>, 
        outline: bool, 
        size: AvatarSize,
        on_click: Option<Box<dyn Callback>>
    ) -> Self {
        Avatar {
            _layout: Stack(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default()),
            _avatar: PrimaryAvatar::new(theme, content.clone(), outline, size),
            _flair: flair.clone().map(|(name, style)| Flair::new(theme, &name, style, size)),
            _size: size,
            _on_click: on_click,
            content,
            flair,
            outline,
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, AvatarContent::icon("profile", AvatarIconStyle::Secondary), None, false, AvatarSize::Lg, None)
    }
}

impl OnEvent for Avatar {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // TODO: should be ina interactions button instead
        // if let Some(MouseEvent{state: MouseState::Pressed, position: Some(_)}) = event.as_any_mut().downcast_mut::<MouseEvent>() {
        //     if let Some(on_click) = &mut self.on_click {
        //         ctx.send(Request::Hardware(Hardware::Haptic));
        //         (on_click)(ctx, )
        //     }
        // if event.as_any().downcast_ref::<TickEvent>().is_some() {
        //     // TODO: allow this
        //     // let (circle_icon, image) = match &self.content {
        //     //     AvatarContent::Image(image) => (None, Some(Image{shape: ShapeType::Ellipse(0.0, (self._size.get(), self._size.get()), 0.0), image: image.clone(), color: None})),
        //     //     AvatarContent::Icon(name, style) => (Some(AvatarIcon::new(theme, name, *style, self._size.get())), None)
        //     // };
            
        //     // self._avatar.1 = circle_icon;
        //     // self._avatar.2 = image;
        //     // self._avatar.3 = self.outline.then(|| Circle::new(self._size.get(), Color::BLACK, true));
        //     // self._flair = self.flair.clone().map(|(name, style)| Flair::new(theme, &name, style, self._size));
        // }
        vec![event]
    }
}

#[derive(Clone, Component, Debug, PartialEq)]
struct PrimaryAvatar(Stack, Option<AvatarIcon>, Option<Image>, Option<Shape>);
impl OnEvent for PrimaryAvatar {}

impl PrimaryAvatar {
    fn new(theme: &Theme, content: AvatarContent, outline: bool, size: AvatarSize) -> Self {
        let (circle_icon, image) = match content {
            AvatarContent::Image(image) => (None, Some(Image{shape: ShapeType::Ellipse(0.0, (size.get(), size.get()), 0.0), image, color: None})),
            AvatarContent::Icon(name, style) => (Some(AvatarIcon::new(theme, &name, style, size.get())), None)
        };

        PrimaryAvatar(
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default()),
            circle_icon, image, outline.then(|| Circle::new(size.get(), Color::BLACK, true)),
        )
    }
}

#[derive(Clone, Debug, Component, PartialEq)]
struct AvatarIcon(Stack, Shape, Image);
impl OnEvent for AvatarIcon {}
impl AvatarIcon {
    fn new(theme: &Theme, name: &str, style: AvatarIconStyle, size: f32) -> Self {
        let icon_size = size * 0.75;
        let (background, icon_color) = style.get(theme);
        AvatarIcon(
            Stack::center(),
            Circle::new(size - 2.0, background, false), 
            Icon::new(theme, name, Some(icon_color), icon_size)
        )
    }
}

#[derive(Debug, Component, PartialEq, Clone)]
struct Flair(Stack, AvatarIcon, Shape);
impl OnEvent for Flair {}
impl Flair {
    fn new(theme: &Theme, name: &str, style: AvatarIconStyle, size: AvatarSize) -> Self {
        Flair(
            Stack::center(),
            AvatarIcon::new(theme, name, style, size.get() / 3.0),
            Circle::new(size.get() / 3.0,  Color::BLACK, true)
        )
    }
}

/// Variations of avatar content.
#[derive(Debug, Clone, PartialEq)]
pub enum AvatarContent {
    /// Display an icon on a circle background.
    Icon(String, AvatarIconStyle),
    /// Display a circular image .
    Image(Arc<RgbaImage>)
}

impl AvatarContent {
    pub fn icon(icon: &str, style: AvatarIconStyle) -> Self {
        AvatarContent::Icon(icon.to_string(), style)
    }

    pub fn image(image: Arc<RgbaImage>) -> Self {
        AvatarContent::Image(image)
    }
}

/// Style presets for avatar icons and backgrounds.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AvatarIconStyle {
    Primary,
    Secondary,
    Brand,
    Success,
    Warning,
    Danger,
}

impl AvatarIconStyle {
    fn get(&self, theme: &Theme) -> (Color, Color) {
        let colors = theme.colors();
        match self {
            AvatarIconStyle::Primary => (colors.get(ptsd::Background::Primary), colors.get(ptsd::Background::Secondary)),
            AvatarIconStyle::Secondary => (colors.get(ptsd::Background::Secondary), colors.get(ptsd::Text::Secondary)),
            AvatarIconStyle::Brand => (colors.get(ptsd::Brand), Color::WHITE),
            AvatarIconStyle::Success => (colors.get(ptsd::Status::Success), Color::WHITE),
            AvatarIconStyle::Warning => (colors.get(ptsd::Status::Warning), Color::WHITE),
            AvatarIconStyle::Danger => (colors.get(ptsd::Status::Danger), Color::WHITE),
        }
    }
}

/// Size presets for avatars.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AvatarSize {
    /// `128.0`
    Xxl, 
    /// `96.0`
    Xl, 
    /// `64.0`
    Lg, 
    /// `48.0`
    Md, 
    /// `32.0`
    Sm, 
    /// `24.0`
    Xs
}

impl AvatarSize {
    /// returns the corresponding size.
    pub fn get(&self) -> f32 {
        match self {
            AvatarSize::Xxl => 128.0,
            AvatarSize::Xl => 96.0,
            AvatarSize::Lg => 64.0,
            AvatarSize::Md => 48.0,
            AvatarSize::Sm => 32.0,
            AvatarSize::Xs => 24.0
        }
    }
}

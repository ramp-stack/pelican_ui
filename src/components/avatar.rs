use mustache::events::{TickEvent, OnEvent, MouseState, Event, MouseEvent};
use mustache::drawable::{Drawable, Component, Image, Color, Shape, ShapeType};
use mustache::layout::{Area, SizeRequest, Layout};
use mustache::{Context, Component, resources};

use crate::components::{Icon, Circle};
use crate::layout::{Stack, Offset, Size, Padding};
use crate::utils::Callback;
use crate::plugin::PelicanUI;

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
///     48.0, 
///     None
/// );
/// ```
#[derive(Component)]
pub struct Avatar {
    _layout: Stack,
    _avatar: PrimaryAvatar,
    _flair: Option<Flair>,
    #[skip] _size: f32,
    #[skip] on_click: Option<Callback>,
    #[skip] pub content: AvatarContent,
    #[skip] pub flair: Option<(&'static str, AvatarIconStyle)>,
    #[skip] pub outline: bool,
}

impl std::fmt::Debug for Avatar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Avatar...")
    }
}

impl Avatar {
    pub fn new(
        ctx: &mut Context, 
        content: AvatarContent, 
        flair: Option<(&'static str, AvatarIconStyle)>, 
        outline: bool, 
        size: f32,
        on_click: Option<Callback>
    ) -> Self {
        Avatar {
            _layout: Stack(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default()),
            _avatar: PrimaryAvatar::new(ctx, content.clone(), outline, size),
            _flair: flair.map(|(name, style)| Flair::new(ctx, name, style, size)),
            _size: size,
            on_click,
            content,
            flair,
            outline,
        }
    }
}

impl OnEvent for Avatar {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent{state: MouseState::Pressed, position: Some(_)}) = event.as_any_mut().downcast_mut::<MouseEvent>() {
            if let Some(on_click) = &mut self.on_click {
                ctx.hardware.haptic();
                (on_click)(ctx)
            }
        } else if event.as_any().downcast_ref::<TickEvent>().is_some() {
            let (circle_icon, image) = match &self.content {
                AvatarContent::Image(image) => (None, Some(Image{shape: ShapeType::Ellipse(0.0, (self._size, self._size), 0.0), image: image.clone(), color: None})),
                AvatarContent::Icon(name, style) => (Some(AvatarIcon::new(ctx, name, *style, self._size)), None)
            };
            
            self._avatar.1 = circle_icon;
            self._avatar.2 = image;
            self._avatar.3 = self.outline.then(|| Circle::new(self._size, Color::BLACK, true));
            self._flair = self.flair.map(|(name, style)| Flair::new(ctx, name, style, self._size));
        }
        false
    }
}

#[derive(Component, Debug)]
struct PrimaryAvatar(Stack, Option<AvatarIcon>, Option<Image>, Option<Shape>);
impl OnEvent for PrimaryAvatar {}

impl PrimaryAvatar {
    fn new(ctx: &mut Context, content: AvatarContent, outline: bool, size: f32) -> Self {
        let (circle_icon, image) = match content {
            AvatarContent::Image(image) => (None, Some(Image{shape: ShapeType::Ellipse(0.0, (size, size), 0.0), image, color: None})),
            AvatarContent::Icon(name, style) => (Some(AvatarIcon::new(ctx, name, style, size)), None)
        };

        PrimaryAvatar(
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default()),
            circle_icon, image, outline.then(|| Circle::new(size, Color::BLACK, true)),
        )
    }
}


#[derive(Debug, Clone)]
pub enum AvatarContent {
    Icon(&'static str, AvatarIconStyle),
    Image(resources::Image)
}

#[derive(Debug, Copy, Clone)]
pub enum AvatarIconStyle {
    Primary,
    Secondary,
    Brand,
    Success,
    Warning,
    Danger,
}

impl AvatarIconStyle {
    fn get(&self, ctx: &mut Context) -> (Color, Color) {
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;
        match self {
            AvatarIconStyle::Primary => (colors.text.heading, colors.background.primary),
            AvatarIconStyle::Secondary => (colors.background.secondary, colors.text.secondary),
            AvatarIconStyle::Brand => (colors.brand, Color::WHITE),
            AvatarIconStyle::Success => (colors.status.success, Color::WHITE),
            AvatarIconStyle::Warning => (colors.status.warning, Color::WHITE),
            AvatarIconStyle::Danger => (colors.status.danger, Color::WHITE),
        }
    }
}

#[derive(Debug, Component)]
struct AvatarIcon(Stack, Shape, Image);
impl OnEvent for AvatarIcon {}
impl AvatarIcon {
    fn new(ctx: &mut Context, name: &'static str, style: AvatarIconStyle, size: f32) -> Self {
        let icon_size = size * 0.75;
        let (background, icon_color) = style.get(ctx);
        AvatarIcon(
            Stack::center(),
            Circle::new(size - 2.0, background, false), 
            Icon::new(ctx, name, icon_color, icon_size)
        )
    }
}

#[derive(Debug, Component)]
struct Flair(Stack, AvatarIcon, Shape);
impl OnEvent for Flair {}
impl Flair {
    fn new(ctx: &mut Context, name: &'static str, style: AvatarIconStyle, size: f32) -> Self {
        Flair(
            Stack::center(),
            AvatarIcon::new(ctx, name, style, size / 3.0),
            Circle::new(size / 3.0,  Color::BLACK, true)
        )
    }
}
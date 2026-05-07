use prism::event::{OnEvent, Event, PickedPhoto, MouseState, MouseEvent, TickEvent};
use prism::canvas::{Image, Shape, ShapeType};
use prism::layout::{Stack, Offset, Size, Padding, Row};
use prism::Context;
use prism::drawable::{Component, SizedTree};

use crate::Callback;
use crate::theme::Theme;
use crate::theme::Color;
use crate::components::{Icon, Circle};

use image::RgbaImage;
use std::sync::Arc;
use crate::theme::Icons;

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
    #[skip] pub flair: Option<(Icons, AvatarIconStyle)>,
    #[skip] pub outline: bool,
    #[skip] waiting_on_photo: bool,
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
        flair: Option<(Icons, AvatarIconStyle)>, 
        outline: bool, 
        size: AvatarSize,
        on_click: Option<Box<dyn Callback>>
    ) -> Self {
        Avatar {
            _layout: Stack(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default()),
            _avatar: PrimaryAvatar::new(theme, content.clone(), outline, size),
            _flair: flair.map(|(name, style)| Flair::new(theme, name, style, size)),
            _size: size,
            _on_click: on_click,
            content,
            flair,
            outline,
            waiting_on_photo: false,
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, AvatarContent::icon(Icons::Profile, AvatarIconStyle::Secondary), None, false, AvatarSize::Lg, None)
    }
}

impl OnEvent for Avatar {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(PickedPhoto(img)) = event.downcast_ref::<PickedPhoto>() && self.waiting_on_photo {
            self.waiting_on_photo = false;
            self.content = AvatarContent::image(Arc::new(img.clone()));
        } else if let Some(MouseEvent{state: MouseState::Pressed, position: Some(_)}) = event.downcast_ref::<MouseEvent>() {
            if let Some(on_click) = &mut self._on_click {
                ctx.trigger_haptic();
                ctx.pick_photo();
                self.waiting_on_photo = true;
            }
        }

        if event.as_any().downcast_ref::<TickEvent>().is_some() {
            self._avatar.update(self.content.clone(), self._size);
        }

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
            AvatarContent::Icon(icon, style) => (Some(AvatarIcon::new(theme, icon, style, size.get())), None)
        };

        PrimaryAvatar(
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default()),
            circle_icon, image, outline.then(|| Circle::new(size.get(), Color::BLACK, true)),
        )
    }

    fn update(&mut self, content: AvatarContent, size: AvatarSize) {
        match content {
            AvatarContent::Image(image) => {
                self.1 = None;
                self.3 = None;
                self.2 = Some(Image{shape: ShapeType::Ellipse(0.0, (size.get(), size.get()), 0.0), image, color: None});
            },
            AvatarContent::Icon(icon, style) => {}
        }
    }
}

#[derive(Clone, Debug, Component, PartialEq)]
struct AvatarIcon(Stack, Shape, Image);
impl OnEvent for AvatarIcon {}
impl AvatarIcon {
    fn new(theme: &Theme, name: Icons, style: AvatarIconStyle, size: f32) -> Self {
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
    fn new(theme: &Theme, name: Icons, style: AvatarIconStyle, size: AvatarSize) -> Self {
        Flair(
            Stack::center(),
            AvatarIcon::new(theme, name, style, size.get() / 3.0),
            Circle::new(size.get() / 3.0,  Color::BLACK, true)
        )
    }
}

#[derive(Debug, Component, Clone)]
pub struct AvatarGroup(Row, Vec<Avatar>);
impl OnEvent for AvatarGroup {}
impl AvatarGroup {
    pub fn new(theme: &Theme, contents: Vec<AvatarContent>) -> Self {
        if contents.is_empty() {
            let content = AvatarContent::default();
            return AvatarGroup(Row::center(-8.0), vec![Avatar::new(theme, content, None, true, AvatarSize::Sm, None)]);
        }

        let avatars = contents.into_iter().map(|content| {
            Avatar::new(theme, content, None, true, AvatarSize::Sm, None)
        }).collect::<Vec<_>>();
        AvatarGroup(Row::center(-8.0), avatars)
    }
}

/// Variations of avatar content.
#[derive(Debug, Clone, PartialEq)]
pub enum AvatarContent {
    /// Display an icon on a circle background.
    Icon(Icons, AvatarIconStyle),
    /// Display a circular image .
    Image(Arc<RgbaImage>)
}

impl Default for AvatarContent {
    fn default() -> Self {
        AvatarContent::Icon(Icons::Profile, AvatarIconStyle::Secondary)
    }
}

impl AvatarContent {
    pub fn icon(icon: Icons, style: AvatarIconStyle) -> Self {
        AvatarContent::Icon(icon, style)
    }

    pub fn image(image: Arc<RgbaImage>) -> Self {
        AvatarContent::Image(image)
    }

    pub fn from_string(s: &str) -> Self {
        use std::sync::Arc;
        use base64::{engine::general_purpose, Engine as _};

        let bytes = match general_purpose::STANDARD.decode(s) {
            Ok(bytes) => bytes,
            Err(_) => return AvatarContent::default(),
        };

        let mut img = match image::load_from_memory(&bytes) {
            Ok(img) => img.to_rgba8(),
            Err(_) => return AvatarContent::default(),
        };

        AvatarContent::image(Arc::new(img))
    }

    pub fn get_image(&self) -> Option<String> {
        use std::io::Cursor;
        use image::{DynamicImage, ImageFormat, RgbaImage};
        use base64::{engine::general_purpose, Engine as _};
        match &self {
            AvatarContent::Image(img) => {
                let mut bytes: Vec<u8> = Vec::new();
                let img = if img.width() > 256 || img.height() > 256 {
                    image::imageops::thumbnail(&(**img), 256, 256)
                } else { (**img).clone() };
                DynamicImage::ImageRgba8(img).write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).ok()?;
                Some(general_purpose::STANDARD.encode(bytes))
            }
            _ => None
        }
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
            AvatarIconStyle::Primary => (colors.get(ptsd::Outline::Primary), colors.get(ptsd::Background::Secondary)),
            AvatarIconStyle::Secondary => (colors.get(ptsd::Background::Secondary), colors.get(ptsd::Text::Secondary)),
            AvatarIconStyle::Brand => (colors.get(ptsd::Brand), colors.get(ptsd::Background::Primary)),
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

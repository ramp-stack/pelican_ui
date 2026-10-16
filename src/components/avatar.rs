use prism::event::{OnEvent, Event, TickEvent};
use prism::canvas::{Image, Shape, ShapeType};
use prism::layout::{Stack, Offset, Size, Padding, Row};
use prism::Context;
use prism::drawable::{Component, SizedTree};

use ptsd::interactions;

use crate::Callback;
use crate::theme::{Theme, Color, Icons};
use crate::components::{Icon, Circle};

use image::{RgbaImage, DynamicImage, ImageFormat};
use base64::{engine::general_purpose, Engine as _};

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Component, Clone)]
pub struct Avatar {
    _layout: Stack,
    _button: Option<interactions::Button>,
    _face: Option<AvatarFace>,
    #[skip] _size: AvatarSize,
    #[skip] _on_click: Option<Box<dyn Callback>>,
    #[skip] pub content: AvatarContent,
    #[skip] pub flair: Option<(Icons, AvatarIconStyle)>,
    #[skip] pub outline: bool,
    #[skip] theme: Theme,
    #[skip] shown: AvatarContent,
}

impl fmt::Debug for Avatar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        let face = AvatarFace(
            Stack(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default()),
            PrimaryAvatar::new(theme, content.clone(), outline, size),
            flair.map(|(name, style)| Flair::new(theme, name, style, size)),
        );
        let (button, face) = match on_click.clone() {
            Some(mut on_click) => {
                let theme = theme.clone();
                let callback = move |ctx: &mut Context| on_click(ctx, &theme);
                (Some(interactions::Button::new_triggers_on_release(face, None::<AvatarFace>, None::<AvatarFace>, None::<AvatarFace>, None::<AvatarFace>, callback, false)), None)
            },
            None => (None, Some(face)),
        };
        Avatar {
            _layout: Stack::default(),
            _button: button,
            _face: face,
            _size: size,
            _on_click: on_click,
            shown: content.clone(),
            content,
            flair,
            outline,
            theme: theme.clone(),
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, AvatarContent::default(), None, false, AvatarSize::Lg, None)
    }
}

impl OnEvent for Avatar {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            let changed = match (&self.content, &self.shown) {
                (AvatarContent::Image(a), AvatarContent::Image(b)) => !Arc::ptr_eq(a, b),
                (a, b) => a != b,
            };
            if changed {
                *self = Avatar::new(&self.theme, self.content.clone(), self.flair, self.outline, self._size, self._on_click.clone());
            }
        }
        vec![event]
    }
}

#[derive(Clone, Component, Debug)]
struct AvatarFace(Stack, PrimaryAvatar, Option<Flair>);
impl OnEvent for AvatarFace {}

#[derive(Clone, Component, Debug, PartialEq)]
struct PrimaryAvatar(Stack, Option<AvatarIcon>, Option<Image>, Option<Shape>);
impl OnEvent for PrimaryAvatar {}

impl PrimaryAvatar {
    fn new(theme: &Theme, content: AvatarContent, outline: bool, size: AvatarSize) -> Self {
        let (icon, image) = match content {
            AvatarContent::Image(image) => (None, Some(Image { shape: ShapeType::Ellipse(0.0, (size.get(), size.get()), 0.0), image, color: None })),
            AvatarContent::Icon(icon, style) => (Some(AvatarIcon::new(theme, icon, style, size.get())), None),
        };
        let background = theme.colors().get(ptsd::Background::Primary);
        PrimaryAvatar(
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::Fit, Padding::default()),
            icon,
            image,
            outline.then(|| Circle::new(size.get(), background, true)),
        )
    }
}

#[derive(Clone, Debug, Component, PartialEq)]
struct AvatarIcon(Stack, Shape, Image);
impl OnEvent for AvatarIcon {}

impl AvatarIcon {
    fn new(theme: &Theme, name: Icons, style: AvatarIconStyle, size: f32) -> Self {
        let colors = theme.colors();
        let (background, icon_color) = match style {
            AvatarIconStyle::Primary => (colors.get(ptsd::Outline::Primary), colors.get(ptsd::Background::Secondary)),
            AvatarIconStyle::Secondary => (colors.get(ptsd::Background::Secondary), colors.get(ptsd::Text::Secondary)),
            AvatarIconStyle::Brand => (colors.get(ptsd::Brand), colors.get(ptsd::Background::Primary)),
            AvatarIconStyle::Success => (colors.get(ptsd::Status::Success), Color::WHITE),
            AvatarIconStyle::Warning => (colors.get(ptsd::Status::Warning), Color::WHITE),
            AvatarIconStyle::Danger => (colors.get(ptsd::Status::Danger), Color::WHITE),
        };
        AvatarIcon(
            Stack::center(),
            Circle::new(size - 2.0, background, false),
            Icon::new(theme, name, Some(icon_color), size * 0.75),
        )
    }
}

#[derive(Debug, Component, PartialEq, Clone)]
struct Flair(Stack, AvatarIcon, Shape);
impl OnEvent for Flair {}

impl Flair {
    fn new(theme: &Theme, name: Icons, style: AvatarIconStyle, size: AvatarSize) -> Self {
        let background = theme.colors().get(ptsd::Background::Primary);
        Flair(
            Stack::center(),
            AvatarIcon::new(theme, name, style, size.get() / 3.0),
            Circle::new(size.get() / 3.0, background, true),
        )
    }
}

#[derive(Debug, Component, Clone)]
pub struct AvatarGroup(Row, Vec<Avatar>);
impl OnEvent for AvatarGroup {}

impl AvatarGroup {
    pub fn new(theme: &Theme, contents: Vec<AvatarContent>) -> Self {
        let contents = if contents.is_empty() { vec![AvatarContent::default()] } else { contents };
        AvatarGroup(Row::center(-8.0), contents.into_iter().map(|content| Avatar::new(theme, content, None, true, AvatarSize::Sm, None)).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AvatarContent {
    Icon(Icons, AvatarIconStyle),
    Image(Arc<RgbaImage>),
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
        general_purpose::STANDARD.decode(s).ok()
            .and_then(|bytes| image::load_from_memory(&bytes).ok())
            .map(|image| AvatarContent::Image(Arc::new(image.to_rgba8())))
            .unwrap_or_default()
    }

    pub fn get_image(&self) -> Option<String> {
        let AvatarContent::Image(image) = self else { return None };
        let image = match image.width() > 256 || image.height() > 256 {
            true => image::imageops::thumbnail(&**image, 256, 256),
            false => (**image).clone(),
        };
        let mut bytes = Vec::new();
        DynamicImage::ImageRgba8(image).write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).ok()?;
        Some(general_purpose::STANDARD.encode(bytes))
    }
}

impl Serialize for AvatarContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            AvatarContent::Icon(icon, style) => map.serialize_entry("Icon", &(icon, style))?,
            AvatarContent::Image(_) => map.serialize_entry("Image", &self.get_image().unwrap_or_default())?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AvatarContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AvatarContentVisitor;

        impl<'de> Visitor<'de> for AvatarContentVisitor {
            type Value = AvatarContent;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("AvatarContent")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<AvatarContent, M::Error> {
                match map.next_key::<String>()?.as_deref() {
                    Some("Icon") => {
                        let (icon, style) = map.next_value()?;
                        Ok(AvatarContent::Icon(icon, style))
                    }
                    Some("Image") => Ok(AvatarContent::from_string(&map.next_value::<String>()?)),
                    Some(other) => Err(de::Error::unknown_field(other, &["Icon", "Image"])),
                    None => Err(de::Error::custom("empty AvatarContent")),
                }
            }
        }

        deserializer.deserialize_map(AvatarContentVisitor)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AvatarIconStyle {
    Primary,
    Secondary,
    Brand,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AvatarSize {
    Xxl,
    Xl,
    Lg,
    Md,
    Sm,
    Xs,
}

impl AvatarSize {
    pub fn get(&self) -> f32 {
        match self {
            AvatarSize::Xxl => 128.0,
            AvatarSize::Xl => 96.0,
            AvatarSize::Lg => 64.0,
            AvatarSize::Md => 48.0,
            AvatarSize::Sm => 32.0,
            AvatarSize::Xs => 24.0,
        }
    }
}
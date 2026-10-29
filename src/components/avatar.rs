use prism::drawable::shape::Variant;
use ptsd::layout::{Stack, Offset, Size, Padding, Row};
use prism::Context;
use media::{Image, ImageFormat, Pixel};
use prism::drawable::{Component, Drawable, Shape};
use ptsd::display::Bin;
use ptsd::interactions;
use image::{ImageBuffer, Rgba};
use base64::{engine::general_purpose, Engine as _};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt;
use std::hash::{Hash, Hasher};

use crate::Callback;
use crate::theme::{Theme, Color, Icons};
use crate::components::{Icon, Circle, MaskedImage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvatarContent {
    Icon(Icons, AvatarIconStyle),
    Image(Image),
}

impl Hash for AvatarContent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Icon(icon, style) => {icon.hash(state); style.hash(state);}
            Self::Image(image) => {
                // AIR signs before serialization and verifies after decoding.
                // Hash the same thumbnail we transmit, not the original photo.
                let image = Self::thumbnail(image);
                image.dimensions.hash(state);
                image.as_ref().hash(state);
            }
        }
    }
}

impl Default for AvatarContent {
    fn default() -> Self {AvatarContent::Icon(Icons::Profile, AvatarIconStyle::Secondary)}
}

impl AvatarContent {
    pub fn icon(icon: Icons, style: AvatarIconStyle) -> Self {AvatarContent::Icon(icon, style)}

    pub fn image(image: Image) -> Self {AvatarContent::Image(Self::thumbnail(&image))}

    pub fn from_string(s: &str) -> Self {
        general_purpose::STANDARD.decode(s).ok()
            .and_then(|bytes| Image::read(bytes.as_slice()).ok())
            .map(Self::image)
            .unwrap_or_default()
    }

    pub fn get_image(&self) -> Option<String> {
        let AvatarContent::Image(image) = self else { return None };
        let mut bytes = Vec::new();
        Self::thumbnail(image).write(&mut bytes, ImageFormat::Png).unwrap();
        Some(general_purpose::STANDARD.encode(bytes))
    }

    fn thumbnail(image: &Image) -> Image {
        let (width, height) = image.dimensions;
        if width <= 256 && height <= 256 {return image.clone();}
        let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, image.as_ref()).unwrap();
        let image = image::imageops::thumbnail(&image, 256, 256);
        let pixels = image.pixels().map(|pixel| {
            let [red, green, blue, alpha] = pixel.0;
            Pixel {red, green, blue, alpha}
        }).collect();
        Image::new(image.dimensions(), pixels)
    }
}

impl Serialize for AvatarContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            AvatarContent::Icon(icon, style) => map.serialize_entry("Icon", &(icon, style))?,
            AvatarContent::Image(_) => map.serialize_entry("Image", &self.get_image().unwrap())?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AvatarContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
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
#[derive(Component, Clone, Debug)]
pub struct Avatar {
    _layout: Stack,
    face: Box<dyn Drawable>,
    #[skip] _size: AvatarSize,
    #[skip] _on_click: Option<Box<dyn Callback>>,
    #[skip] content: AvatarContent,
    #[skip] pub flair: Option<(Icons, AvatarIconStyle)>,
    #[skip] theme: Theme,
}

impl Avatar {
    pub fn new(
        theme: &Theme, content: AvatarContent, flair: Option<(Icons, AvatarIconStyle)>,
        size: AvatarSize, on_click: Option<Box<dyn Callback>>,
    ) -> Self {
        let face = AvatarFace(
            Stack::new(Offset::End, Offset::End, Size::Fit, Size::Fit, Padding::default()),
            PrimaryAvatar::new(theme, content.clone(), size),
            flair.map(|(name, style)| Flair::new(theme, name, style, size)),
        );
        let face: Box<dyn Drawable> = match on_click.clone() {
            Some(mut on_click) => {
                let theme = theme.clone();
                let callback = move |ctx: &mut Context| on_click(ctx, &theme);
                Box::new(interactions::Button::new_triggers_on_release(
                    face, None::<AvatarFace>, None::<AvatarFace>, None::<AvatarFace>,
                    None::<AvatarFace>, callback, false,
                ))
            },
            None => Box::new(face),
        };
        Avatar {
            _layout: Stack::default(), face, _size: size, _on_click: on_click,
            content, flair, theme: theme.clone(),
        }
    }

    pub fn default(theme: &Theme) -> Self {
        Self::new(theme, AvatarContent::default(), None, AvatarSize::Lg, None)
    }

    pub fn content(&self) -> &AvatarContent {&self.content}

    pub fn set_content(&mut self, content: AvatarContent) {
        let changed = match (&self.content, &content) {
            (AvatarContent::Image(a), AvatarContent::Image(b)) => a != b,
            (a, b) => a != b,
        };
        if changed {
            *self = Self::new(
                &self.theme, content, self.flair, self._size,
                self._on_click.clone(),
            );
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct AvatarGroup(Row, Vec<Avatar>);

impl AvatarGroup {
    pub fn new(theme: &Theme, contents: Vec<AvatarContent>) -> Self {
        let contents = if contents.is_empty() { vec![AvatarContent::default()] } else { contents };
        AvatarGroup(Row::center(-8.0), contents.into_iter().map(|content| {
            Avatar::new(theme, content, None, AvatarSize::Sm, None)
        }).collect())
    }
}

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
            Some("Image") => {
                let encoded = map.next_value::<String>()?;
                let bytes = general_purpose::STANDARD.decode(encoded).map_err(de::Error::custom)?;
                let image = Image::read(bytes.as_slice()).map_err(de::Error::custom)?;
                Ok(AvatarContent::Image(image))
            }
            Some(other) => Err(de::Error::unknown_field(other, &["Icon", "Image"])),
            None => Err(de::Error::custom("empty AvatarContent")),
        }
    }
}

#[derive(Clone, Component, Debug)]
struct AvatarFace(Stack, PrimaryAvatar, Option<Flair>);

#[derive(Clone, Component, Debug)]
struct PrimaryAvatar(Stack, Box<dyn Drawable>);

impl PrimaryAvatar {
    fn new(theme: &Theme, content: AvatarContent, size: AvatarSize) -> Self {
        let content: Box<dyn Drawable> = match content {
            AvatarContent::Image(image) => Box::new(MaskedImage(image, Variant::Ellipse)),
            AvatarContent::Icon(icon, style) => Box::new(AvatarIcon::new(theme, icon, style, size.get())),
        };
        PrimaryAvatar(
            Stack::new(
                Offset::Center, Offset::Center, Size::Static(size.get()),
                Size::Static(size.get()), Padding::default(),
            ),
            content,
        )
    }
}

#[derive(Clone, Debug, Component)]
struct AvatarIcon(Stack, Bin<Stack, Shape>, Icon);

impl AvatarIcon {
    fn new(theme: &Theme, name: Icons, style: AvatarIconStyle, size: f32) -> Self {
        let colors = theme.colors();
        let (background, icon_color) = match style {
            AvatarIconStyle::Primary => (
                colors.get(ptsd::Outline::Primary), colors.get(ptsd::Background::Secondary),
            ),
            AvatarIconStyle::Secondary => (
                colors.get(ptsd::Background::Secondary), colors.get(ptsd::Text::Secondary),
            ),
            AvatarIconStyle::Brand => (colors.get(ptsd::Brand), colors.get(ptsd::Background::Primary)),
            AvatarIconStyle::Success => (colors.get(ptsd::Status::Success), Color::white()),
            AvatarIconStyle::Warning => (colors.get(ptsd::Status::Warning), Color::white()),
            AvatarIconStyle::Danger => (colors.get(ptsd::Status::Danger), Color::white()),
        };
        AvatarIcon(
            Stack::center(),
            Circle::new(size - 2.0, background, false),
            Icon::new(theme, name, Some(icon_color), size * 0.75),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Flair(Stack, AvatarIcon);

impl Flair {
    fn new(theme: &Theme, name: Icons, style: AvatarIconStyle, size: AvatarSize) -> Self {
        Flair(
            Stack::center(),
            AvatarIcon::new(theme, name, style, size.get() / 3.0),
        )
    }
}

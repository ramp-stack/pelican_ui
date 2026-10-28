use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageBuffer, Rgba, ColorType};
use image::codecs::jpeg::JpegEncoder;
use image::imageops;
use prism::air::{Metadata, Contract, Id, Name, Instance};
use prism::drawable::{Component, shape::Variant};
use prism::Context;
use media::Image;
use ptsd::layout::{Stack, Offset, Size, Padding};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use std::sync::Arc;
use std::hash::{Hash, Hasher};

use crate::components::images::MaskedImage;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum StickerPackAction {Add(Sticker), Remove(Sticker)}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sticker {
    bytes: Arc<[u8]>,
    image: Image,
}

impl Hash for Sticker {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
        self.image.dimensions.hash(state);
        self.image.as_ref().hash(state);
    }
}

impl Serialize for Sticker {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(&self.bytes))
    }
}

impl<'de> Deserialize<'de> for Sticker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        let bytes = STANDARD.decode(encoded).map_err(serde::de::Error::custom)?;
        Sticker::from_bytes(bytes)
            .ok_or_else(|| serde::de::Error::custom("Invalid sticker image"))
    }
}

impl Sticker {
    pub fn from_photo(photo: &Image) -> Option<Self> {
        let (w, h) = photo.dimensions;
        let pixels = photo.as_ref();
        let photo = ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, pixels).unwrap();
        let side = w.min(h);
        let square = imageops::crop_imm(&photo, (w - side) / 2, (h - side) / 2, side, side);
        let size = side.min(320);
        let square = imageops::resize(&*square, size, size, imageops::FilterType::Triangle);
        let rgb: Vec<u8> = square.as_raw().as_chunks::<4>().0.iter().flat_map(|pixel| {
            let alpha = pixel[3] as u32;
            [0, 1, 2].map(|i| ((pixel[i] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8)
        }).collect();
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, 80)
            .encode(&rgb, size, size, ColorType::Rgb8).ok()?;
        Sticker::from_bytes(jpeg)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let image = Image::read(bytes.as_slice()).ok()?;
        Some(Sticker {bytes: bytes.into(), image})
    }

    pub fn image(&self) -> Image { self.image.clone() }
    pub fn bytes(&self) -> &[u8] { &self.bytes }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StickerPack {
    pub owner: Name,
    pub stickers: Vec<Sticker>,
}

impl Contract for StickerPack {
    type Init = Name;
    type Message = StickerPackAction;
    type Result = ();

    fn id() -> Id {Id::hash("StickerPack")}

    fn init(owner: Self::Init, _metadata: Metadata) -> Self {StickerPack {owner, stickers: Vec::new()}}

    fn apply(&mut self, message: Self::Message, metadata: Metadata) -> Self::Result {
        if metadata.signer != self.owner { return; }

        match message {
            StickerPackAction::Add(sticker) => {
                self.stickers.retain(|s| *s != sticker);
                self.stickers.insert(0, sticker);
            },
            StickerPackAction::Remove(sticker) => self.stickers.retain(|s| *s != sticker),
        }
    }
}

impl StickerPack {
    pub fn mine(ctx: &mut Context) -> Instance<StickerPack> {
        let me = ctx.air.me();
        let existing = ctx.air.instances::<StickerPack>().list().into_values()
            .find(|pack| pack.pending().owner == me);
        existing.unwrap_or_else(|| ctx.air.instances::<StickerPack>().create(me))
    }
}

#[derive(Debug, Clone, Component)]
pub struct StickerView(Stack, MaskedImage);

impl StickerView {
    pub fn message(sticker: &Sticker) -> Self {
        StickerView(
            Stack::new(
                Offset::Center, Offset::Center, Size::Static(120.0), Size::Static(120.0), Padding::default(),
            ),
            MaskedImage(sticker.image(), Variant::RoundedRectangle(16.0)),
        )
    }
}

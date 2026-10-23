use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{RgbaImage, ColorType};
use image::codecs::jpeg::JpegEncoder;
use image::imageops;
use prism::air::{Metadata, Contract, Id, Name, Instance};
use prism::drawable::{Component, shape::Variant};
use prism::Context;
use prism::hardware::Hardware;
use ptsd::layout::{Stack, Offset, Size, Padding};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use std::sync::Arc;
use std::hash::Hash;

use crate::components::images::MaskedImage;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum StickerPackAction {Add(Sticker), Remove(Sticker)}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sticker {
    bytes: Arc<[u8]>,
    image: Arc<RgbaImage>,
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
    pub fn from_photo(photo: &RgbaImage) -> Option<Self> {
        let (w, h) = photo.dimensions();
        let side = w.min(h);
        let square = imageops::crop_imm(photo, (w - side) / 2, (h - side) / 2, side, side).to_image();
        if side == 0 {return None;}
        let size = side.min(320);
        let square = imageops::resize(&square, size, size, imageops::FilterType::Triangle);
        let rgb: Vec<u8> = square.as_raw().chunks_exact(4).flat_map(|pixel| {
            let alpha = pixel[3] as u32;
            [0, 1, 2].map(|i| ((pixel[i] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8)
        }).collect();
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, 80)
            .encode(&rgb, size, size, ColorType::Rgb8).ok()?;
        Sticker::from_bytes(jpeg)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
        Some(Sticker { bytes: bytes.into(), image: Arc::new(image) })
    }

    pub fn image(&self) -> Arc<RgbaImage> { self.image.clone() }
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
    pub fn message(hardware: &Hardware, sticker: &Sticker) -> Self {
        StickerView(
            Stack(
                Offset::Center, Offset::Center, Size::Static(120.0), Size::Static(120.0), Padding::default(),
            ),
            MaskedImage(
                hardware.image_from_rgba(sticker.image()), Variant::RoundedRectangle(16.0),
            ),
        )
    }
}

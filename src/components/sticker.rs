use prism::event::{OnEvent, Event};
use prism::drawable::Component;
use prism::layout::{Area, Stack, Offset, Size, Padding};
use prism::canvas::{Image, ShapeType};
use prism::Context;

use image::{DynamicImage, RgbaImage};
use image::imageops::{self, FilterType};
use image::codecs::jpeg::JpegEncoder;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

use maverick_os::air::{Metadata, Contract, Id, Name, Instance};

use std::sync::Arc;
use std::hash::{Hash, Hasher};

use crate::theme::Theme;
use crate::components::Rectangle;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sticker {
    bytes: Arc<[u8]>,
    image: Arc<RgbaImage>,
}


impl Serialize for Sticker {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use base64::{engine::general_purpose, Engine as _};
        serializer.serialize_str(&general_purpose::STANDARD.encode(&self.bytes))
    }
}

impl<'de> Deserialize<'de> for Sticker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use base64::{engine::general_purpose, Engine as _};
        let encoded = String::deserialize(deserializer)?;
        let bytes = general_purpose::STANDARD.decode(encoded).map_err(serde::de::Error::custom)?;
        Ok(Sticker::from_bytes(bytes.clone()).unwrap_or_else(|| Sticker {
            bytes: bytes.into(),
            image: Arc::new(RgbaImage::new(1, 1)),
        }))
    }
}

impl Sticker {
    pub fn from_photo(photo: &RgbaImage) -> Option<Self> {
        let (w, h) = photo.dimensions();
        if w == 0 || h == 0 { return None; }

        let side = w.min(h);
        let square = imageops::crop_imm(photo, (w - side) / 2, (h - side) / 2, side, side).to_image();
        let image = imageops::resize(&square, 320, 320, FilterType::Triangle);

        let mut bytes = Vec::new();
        let rgb = DynamicImage::ImageRgba8(image.clone()).to_rgb8();
        JpegEncoder::new_with_quality(&mut bytes, 80).encode_image(&rgb).ok()?;

        Some(Sticker { bytes: bytes.into(), image: Arc::new(image) })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
        Some(Sticker { bytes: bytes.into(), image: Arc::new(image) })
    }

    pub fn image(&self) -> Arc<RgbaImage> { self.image.clone() }
    pub fn bytes(&self) -> &[u8] { &self.bytes }
}

#[derive(Debug, Clone)]
pub struct SendStickerEvent(pub Sticker);

impl Event for SendStickerEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
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

    fn init(owner: Self::Init, _metadata: Metadata) -> Self {
        StickerPack { owner, stickers: Vec::new() }
    }

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
        let me = ctx.me();
        let existing = ctx.list::<StickerPack>().into_values().find(|pack| pack.pending().owner == me);
        existing.unwrap_or_else(|| ctx.create::<StickerPack>(me))
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum StickerPackAction {
    Add(Sticker),
    Remove(Sticker),
}

use prism::drawable::Component;
use ptsd::layout::{Stack, Offset, Size, Padding};
use prism::drawable::shape::Variant;
use crate::components::images::MaskedImage;
use prism::Context;

use image::RgbaImage;
use image::imageops;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

use prism::air::{Metadata, Contract, Id, Name, Instance};

use std::sync::Arc;
use std::hash::Hash;

use crate::components::Photo;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sticker {
    bytes: Arc<[u8]>,
    image: Arc<RgbaImage>,
}

impl Serialize for Sticker {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::base64_bytes::serialize(&self.bytes, serializer)
    }
}

impl<'de> Deserialize<'de> for Sticker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes: Arc<[u8]> = crate::base64_bytes::deserialize(deserializer)?;
        Ok(Sticker::from_bytes(bytes.to_vec()).unwrap_or_else(|| Sticker { bytes, image: Arc::new(RgbaImage::new(1, 1)) }))
    }
}

impl Sticker {
    pub fn from_photo(photo: &RgbaImage) -> Option<Self> {
        let (w, h) = photo.dimensions();
        let side = w.min(h);
        let square = imageops::crop_imm(photo, (w - side) / 2, (h - side) / 2, side, side).to_image();
        Sticker::from_bytes(Photo::encode(&square, 320, 80)?.jpeg().to_vec())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
        Some(Sticker { bytes: bytes.into(), image: Arc::new(image) })
    }

    pub fn image(&self) -> Arc<RgbaImage> { self.image.clone() }
    pub fn bytes(&self) -> &[u8] { &self.bytes }
}

#[derive(Debug, Clone, Component)]
pub struct StickerView(Stack, MaskedImage);

impl StickerView {
    pub fn message(sticker: &Sticker) -> Self {
        StickerView(
            Stack(Offset::Center, Offset::Center, Size::Static(120.0), Size::Static(120.0), Padding::default()),
            MaskedImage(sticker.image(), Variant::RoundedRectangle(16.0)),
        )
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
        let me = ctx.air.me();
        let existing = ctx.air.instances::<StickerPack>().list().into_values().find(|pack| pack.pending().owner == me);
        existing.unwrap_or_else(|| ctx.air.instances::<StickerPack>().create(me))
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum StickerPackAction {
    Add(Sticker),
    Remove(Sticker),
}

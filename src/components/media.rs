
use prism::event::{OnEvent, Event, TickEvent, PickedMedia};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Stack, Offset, Size, Padding};
use prism::canvas::{Align, Image, ShapeType};
use prism::display::Bin;
use prism::Context;

use ptsd::interactions;

use image::{RgbaImage, DynamicImage, ColorType};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;

use maverick_os::air::{Contract, Metadata, Id, Name};

use serde::{Serialize, Deserialize};

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use crate::theme::{Theme, Color, Icons};
use crate::components::{Rectangle, Icon, Task};
use crate::components::button::GhostIconButton;
use crate::components::text::{Text, TextSize, TextStyle};

/// A photo is sent at most this many pixels on its longest side.
const PHOTO_SIDE: u32 = 1600;
const PHOTO_QUALITY: u8 = 82;
/// A video's still, sent in its message.
const POSTER_SIDE: u32 = 640;
const POSTER_QUALITY: u8 = 75;
/// A photo or video shows in a chat at most this wide and tall.
const MESSAGE_WIDTH: f32 = 240.0;
const MESSAGE_MAX_HEIGHT: f32 = 320.0;
/// Photos are decoded for showing at most this many pixels on their longest
/// side (enough for the sharpest screens at message size).
const SHOWN_SIDE: u32 = 960;
/// How much memory decoded photos may use before the least recently shown go.
const CACHE_BYTES: usize = 64 * 1024 * 1024;
/// Photos decoded at once, at most.
const DECODING: usize = 2;

/// A photo in a chat: a JPEG and its size.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Photo {
    #[serde(with = "base64_bytes")]
    jpeg: Arc<[u8]>,
    width: u32,
    height: u32,
}

impl Photo {
    /// `image` made small enough to send. Slow for a big photo, so it's best
    /// done away from the interface (the [`AttachButton`] does).
    pub fn from_image(image: &RgbaImage) -> Option<Self> {
        Self::encode(image, PHOTO_SIDE, PHOTO_QUALITY)
    }

    /// A JPEG of `image`, at most `side` pixels on its longest side.
    fn encode(image: &RgbaImage, side: u32, quality: u8) -> Option<Self> {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 { return None; }
        let scale = (side as f32 / w.max(h) as f32).min(1.0);
        let (width, height) = (((w as f32 * scale).round() as u32).max(1), ((h as f32 * scale).round() as u32).max(1));
        let resized;
        let image = match (width, height) == (w, h) {
            true => image,
            // `DynamicImage`'s resize is built with the image crate, so it's
            // fast in a debug build when that crate is optimized.
            false => { resized = DynamicImage::ImageRgba8(image.clone()).resize_exact(width, height, FilterType::Triangle).into_rgba8(); &resized }
        };
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, quality).encode(&rgb_over_white(image.as_raw()), width, height, ColorType::Rgb8).ok()?;
        Some(Photo { jpeg: jpeg.into(), width, height })
    }

    /// Its size in pixels.
    pub fn size(&self) -> (u32, u32) { (self.width, self.height) }

    pub fn jpeg(&self) -> &[u8] { &self.jpeg }
}

/// RGBA pixels as RGB, over white (JPEGs have no transparency).
fn rgb_over_white(rgba: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
    for pixel in rgba.chunks_exact(4) {
        let a = pixel[3] as u32;
        match a {
            255 => rgb.extend_from_slice(&pixel[..3]),
            _ => rgb.extend(pixel[..3].iter().map(|c| ((*c as u32 * a + 255 * (255 - a)) / 255) as u8)),
        }
    }
    rgb
}

impl std::fmt::Debug for Photo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Photo({}x{}, {} bytes)", self.width, self.height, self.jpeg.len())
    }
}

/// A video in a chat: a still from its start (shown until it's played), how
/// long it is, and the id of the [`VideoClip`] with the video itself.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Video {
    pub clip: Id,
    pub poster: Photo,
    pub duration_ms: u64,
}

/// A video, as its own instance: kept out of the chat it's sent in (which is
/// saved again, whole, whenever it changes) and shared with its members.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: Id,
    mp4: Mp4,
}

/// An MP4's bytes.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Mp4(#[serde(with = "base64_bytes")] Arc<[u8]>);

impl Mp4 {
    pub fn new(mp4: Arc<[u8]>) -> Self { Mp4(mp4) }
    /// Its id: the same for the same video.
    pub fn id(&self) -> Id { Id::hash(&self.0[..]) }
}

impl std::fmt::Debug for Mp4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Mp4({} bytes)", self.0.len()) }
}

impl std::fmt::Debug for VideoClip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "VideoClip({:?}, {:?})", self.id, self.mp4) }
}

impl Contract for VideoClip {
    type Init = Mp4;
    /// It never changes.
    type Message = ();
    type Result = ();

    fn id() -> Id { Id::hash("VideoClip") }

    fn init(mp4: Mp4, _metadata: Metadata) -> Self {
        VideoClip { id: mp4.id(), mp4 }
    }

    fn apply(&mut self, _message: (), _metadata: Metadata) {}
}

impl VideoClip {
    /// Starts listening for clips shared with you. Call it when the app
    /// starts (where its chats are listed): clips shared before something
    /// is listening aren't picked up.
    pub fn listen(ctx: &mut Context) {
        let _ = ctx.list::<VideoClip>();
    }

    /// The video of the clip with this `id`, once it's here (a clip someone
    /// shared takes a while to arrive).
    pub fn find(ctx: &mut Context, id: Id) -> Option<Arc<[u8]>> {
        ctx.list::<VideoClip>().values().find_map(|clip| {
            let clip = clip.pending();
            (clip.id == id).then(|| clip.mp4.0.clone())
        })
    }
}

/// A video picked to send: the MP4, its still, and how long it is.
#[derive(Clone)]
pub struct NewVideo {
    pub mp4: Arc<[u8]>,
    pub poster: Photo,
    pub duration: Duration,
}

impl std::fmt::Debug for NewVideo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NewVideo({} bytes, {:?}, {:?})", self.mp4.len(), self.duration, self.poster)
    }
}

impl NewVideo {
    /// Makes the video's [`VideoClip`] and shares it with `members` (the
    /// chat's), giving the message to send for it. It's shared with you too,
    /// so your own videos are found the same way as everyone else's.
    pub fn share(&self, ctx: &mut Context, members: &[Name]) -> Video {
        let mp4 = Mp4(self.mp4.clone());
        let clip = mp4.id();
        let instance = ctx.create::<VideoClip>(mp4);
        let me = ctx.me();
        for member in members.iter().chain((!members.contains(&me)).then_some(&me)) {
            instance.share(*member);
        }
        Video { clip, poster: self.poster.clone(), duration_ms: self.duration.as_millis() as u64 }
    }
}

/// A photo to send as a message.
#[derive(Debug, Clone)]
pub struct SendPhotoEvent(pub Photo);

/// A video to send as a message: share it with [`NewVideo::share`] and send
/// the [`Video`] that gives.
#[derive(Debug, Clone)]
pub struct SendVideoEvent(pub NewVideo);

impl Event for SendPhotoEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

impl Event for SendVideoEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// What a photo or video takes up in a chat: at most 240 wide and 320 tall,
/// its own shape.
fn message_size((width, height): (u32, u32)) -> (f32, f32) {
    let (w, h) = (width.max(1) as f32, height.max(1) as f32);
    let scale = (MESSAGE_WIDTH / w).min(MESSAGE_MAX_HEIGHT / h);
    ((w * scale).round().max(1.0), (h * scale).round().max(1.0))
}

/// A photo in a chat: a box of its size until it's decoded, then the photo.
#[derive(Debug, Clone, Component)]
pub struct PhotoView(Stack, Option<Image>, Option<Rectangle>, #[skip] Photo, #[skip] (f32, f32), #[skip] f32);
impl PhotoView {
    pub fn message(theme: &Theme, photo: &Photo) -> Self {
        Self::new(theme, photo, message_size(photo.size()), 16.0)
    }

    fn new(theme: &Theme, photo: &Photo, size: (f32, f32), radius: f32) -> Self {
        let placeholder = theme.colors().get(ptsd::Background::Secondary);
        PhotoView(
            Stack(Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            None,
            Some(Rectangle::new(placeholder, radius, None)),
            photo.clone(),
            size,
            radius,
        )
    }
}

impl OnEvent for PhotoView {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if self.1.is_none() && event.downcast_ref::<TickEvent>().is_some() && let Some(image) = load_photo(&self.3) {
            let shape = ShapeType::RoundedRectangle(0.0, self.4, 0.0, self.5);
            self.1 = Some(Image { shape, image, color: None });
            self.2 = None;
        }
        vec![event]
    }
}

/// A video in a chat: its still, with a play button and how long it is. Tap
/// it to play it (once its clip is here: until then it says it's loading,
/// and plays when it arrives).
#[derive(Debug, Clone, Component)]
pub struct VideoView(Stack, interactions::Button, Option<Bin<Stack, Badge>>, #[skip] Video, #[skip] u64, #[skip] bool, #[skip] Theme);

impl VideoView {
    pub fn message(theme: &Theme, video: &Video) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let token = NEXT.fetch_add(1, Ordering::Relaxed);
        let size = message_size(video.poster.size());
        let face = VideoFace::new(theme, video, size);
        let callback = move |ctx: &mut Context| ctx.emit(PlayVideo(token));
        VideoView(
            Stack(Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            // On release, so a finger scrolling the chat over it doesn't play it.
            interactions::Button::new_triggers_on_release(face, None::<VideoFace>, None::<VideoFace>, None::<VideoFace>, None::<VideoFace>, callback, false),
            None,
            video.clone(),
            token,
            false,
            theme.clone(),
        )
    }

    /// Plays it if its clip is here; if not, says so until it is.
    fn play(&mut self, ctx: &mut Context) -> bool {
        match VideoClip::find(ctx, self.3.clip) {
            Some(mp4) => {
                self.5 = false;
                self.2 = None;
                ctx.play_video(mp4);
                true
            },
            None => {
                self.5 = true;
                let loading = Badge::label(&self.6, "Loading\u{2026}");
                self.2 = Some(Bin(Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default()), loading));
                false
            },
        }
    }
}

impl OnEvent for VideoView {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(PlayVideo(token)) = event.downcast_ref::<PlayVideo>() && *token == self.4 {
            self.play(ctx);
        } else if self.5 && let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            // Tapped before its clip was here: plays once it is.
            self.play(ctx);
        }
        vec![event]
    }
}

/// Tapping a [`VideoView`] (the one with this number).
#[derive(Debug, Clone)]
struct PlayVideo(u64);

impl Event for PlayVideo {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// What a video shows: its still, a play button in the middle and how long
/// it is in the corner.
#[derive(Debug, Clone, Component)]
struct VideoFace(Stack, PhotoView, Bin<Stack, Badge>, Bin<Stack, Badge>);
impl OnEvent for VideoFace {}

impl VideoFace {
    fn new(theme: &Theme, video: &Video, size: (f32, f32)) -> Self {
        // The whole video (a static size is inside the padding).
        let fill = |x: Offset, y: Offset, p: Padding| Stack(x, y, Size::Static(size.0 - p.0 - p.2), Size::Static(size.1 - p.1 - p.3), p);
        VideoFace(
            Stack(Offset::Start, Offset::Start, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            PhotoView::new(theme, &video.poster, size, 16.0),
            Bin(fill(Offset::Center, Offset::Center, Padding::default()), Badge::play(theme)),
            Bin(fill(Offset::Start, Offset::End, Padding(10.0, 10.0, 10.0, 10.0)), Badge::label(theme, &duration_text(video.duration_ms))),
        )
    }
}

/// A dark, see-through badge over a video: the play button, or a label.
#[derive(Debug, Clone, Component)]
pub struct Badge(Stack, Rectangle, Option<Image>, Option<Text>);
impl OnEvent for Badge {}

impl Badge {
    fn play(theme: &Theme) -> Self {
        Badge(
            Stack(Offset::Center, Offset::Center, Size::Static(52.0), Size::Static(52.0), Padding::default()),
            Rectangle::new(Color::from_hex("000000", 140), 26.0, None),
            Some(Icon::new(theme, Icons::Play, Some(Color::WHITE), 32.0)),
            None,
        )
    }

    fn label(theme: &Theme, text: &str) -> Self {
        // Its text and a margin (the background alone would take all the
        // room there is), with the text in the middle.
        let around_text = |margin: f32| Size::custom(move |children: Vec<(f32, f32)>| {
            let text = children.last().map(|c| c.0).unwrap_or_default() + 2.0 * margin;
            (text, text)
        });
        Badge(
            Stack(Offset::Center, Offset::Center, around_text(8.0), around_text(3.0), Padding::default()),
            Rectangle::new(Color::from_hex("000000", 140), 10.0, None),
            None,
            Some(Text::new(theme, text, TextSize::Sm, TextStyle::Label(Color::WHITE), Align::Left, None)),
        )
    }
}

/// "0:42", "12:05", "1:02:03".
fn duration_text(ms: u64) -> String {
    let seconds = (ms + 500) / 1000;
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    match h {
        0 => format!("{m}:{s:02}"),
        _ => format!("{h}:{m:02}:{s:02}"),
    }
}

/// The message bar's photo button: picks a photo or a video from the library
/// and sends it, with a [`SendPhotoEvent`] or [`SendVideoEvent`] for the chat
/// (once it's been made small enough, away from the interface). It can be
/// tapped again while something's on its way: everything picked is sent, in
/// the order it arrived.
#[derive(Debug, Clone, Component)]
pub struct AttachButton(Stack, GhostIconButton, #[skip] u64, #[skip] Attaching);

#[derive(Debug, Clone, Default)]
struct Attaching {
    /// How many times this button opened the library without hearing back
    /// (cancelling isn't heard, so it can be more than are really open).
    open: usize,
    /// What was picked, being made ready to send, oldest first.
    preparing: VecDeque<Task<Option<Prepared>>>,
}

#[derive(Debug)]
enum Prepared { Photo(Photo), Video(NewVideo) }

/// The [`AttachButton`] with this number was tapped.
#[derive(Debug, Clone)]
struct Attach(u64);

impl Event for Attach {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

impl AttachButton {
    pub fn new(theme: &Theme) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let token = NEXT.fetch_add(1, Ordering::Relaxed);
        let mut button = GhostIconButton::new(theme, Icons::Photos, move |ctx: &mut Context, _: &Theme| ctx.emit(Attach(token)));
        button.1.set_trigger_on_release();
        AttachButton(Stack::default(), button, token, Attaching::default())
    }

    /// Makes what was picked ready to send.
    fn prepare(media: &PickedMedia) -> Task<Option<Prepared>> {
        let media = media.clone();
        Task::spawn(move || match media {
            PickedMedia::Photo(image) => Photo::from_image(&image).map(Prepared::Photo),
            PickedMedia::Video(video) => {
                let poster = Photo::encode(&video.poster, POSTER_SIDE, POSTER_QUALITY)?;
                Some(Prepared::Video(NewVideo { mp4: video.mp4.as_slice().into(), poster, duration: video.duration }))
            },
        })
    }
}

impl OnEvent for AttachButton {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        let attaching = &mut self.3;
        if let Some(Attach(token)) = event.downcast_ref::<Attach>() && *token == self.2 {
            attaching.open += 1;
            ctx.pick_media();
        } else if let Some(media) = event.downcast_ref::<PickedMedia>() && attaching.open > 0 {
            attaching.open -= 1;
            attaching.preparing.push_back(Self::prepare(media));
        } else if event.downcast_ref::<TickEvent>().is_some() {
            // Sent in order: a photo waits for anything picked before it.
            while let Some(prepared) = attaching.preparing.front().and_then(|task| task.take()) {
                attaching.preparing.pop_front();
                match prepared {
                    Some(Prepared::Photo(photo)) => ctx.emit(SendPhotoEvent(photo)),
                    Some(Prepared::Video(video)) => ctx.emit(SendVideoEvent(video)),
                    None => {},
                }
            }
        }
        vec![event]
    }
}

/// Decoded photos, for showing.
#[derive(Default)]
struct PhotoCache {
    /// By where the JPEG is in memory (checked against the JPEG itself).
    entries: HashMap<usize, (Weak<[u8]>, Decoded)>,
    /// Decoded photos, least recently shown first.
    recent: Vec<usize>,
    bytes: usize,
}

enum Decoded { Loading, Ready(Arc<RgbaImage>), Failed }

fn cache() -> &'static Mutex<PhotoCache> {
    static CACHE: OnceLock<Mutex<PhotoCache>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

static DECODES: AtomicUsize = AtomicUsize::new(0);

/// `photo`, decoded for showing, once it is. The first call starts decoding it
/// (a couple at a time, so a chat full of photos doesn't hold everything up).
fn load_photo(photo: &Photo) -> Option<Arc<RgbaImage>> {
    let key = Arc::as_ptr(&photo.jpeg) as *const u8 as usize;
    {
        let mut photos = cache().lock().ok()?;
        let same = |weak: &Weak<[u8]>| weak.upgrade().is_some_and(|jpeg| Arc::ptr_eq(&jpeg, &photo.jpeg));
        match photos.entries.get(&key) {
            Some((weak, Decoded::Ready(image))) if same(weak) => {
                let image = image.clone();
                if let Some(i) = photos.recent.iter().position(|k| *k == key) {
                    let k = photos.recent.remove(i);
                    photos.recent.push(k);
                }
                return Some(image);
            }
            Some((weak, Decoded::Loading | Decoded::Failed)) if same(weak) => return None,
            _ => {}
        }
        if DECODES.load(Ordering::Relaxed) >= DECODING { return None; }
        DECODES.fetch_add(1, Ordering::Relaxed);
        if let Some((_, Decoded::Ready(old))) = photos.entries.insert(key, (Arc::downgrade(&photo.jpeg), Decoded::Loading)) {
            photos.bytes -= old.as_raw().len();
            photos.recent.retain(|k| *k != key);
        }
    }

    let jpeg = photo.jpeg.clone();
    std::thread::spawn(move || {
        let image = image::load_from_memory(&jpeg).ok().map(|image| {
            let (w, h) = (image.width(), image.height());
            let scale = (SHOWN_SIDE as f32 / w.max(h) as f32).min(1.0);
            match scale < 1.0 {
                true => image.resize_exact(((w as f32 * scale) as u32).max(1), ((h as f32 * scale) as u32).max(1), FilterType::Triangle).into_rgba8(),
                false => image.into_rgba8(),
            }
        });
        DECODES.fetch_sub(1, Ordering::Relaxed);
        let Ok(mut photos) = cache().lock() else { return };
        let weak = Arc::downgrade(&jpeg);
        match image {
            Some(image) => {
                photos.bytes += image.as_raw().len();
                photos.entries.insert(key, (weak, Decoded::Ready(Arc::new(image))));
                photos.recent.push(key);
                while photos.bytes > CACHE_BYTES && photos.recent.len() > 1 {
                    let old = photos.recent.remove(0);
                    if let Some((_, Decoded::Ready(image))) = photos.entries.remove(&old) { photos.bytes -= image.as_raw().len(); }
                }
            },
            None => { photos.entries.insert(key, (weak, Decoded::Failed)); },
        }
    });
    None
}

/// Bytes as base64 text (like stickers), which suits every way messages are
/// stored.
mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    pub fn serialize<S: Serializer>(bytes: &Arc<[u8]>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Arc<[u8]>, D::Error> {
        let text = String::deserialize(deserializer)?;
        STANDARD.decode(text).map(Into::into).map_err(serde::de::Error::custom)
    }
}

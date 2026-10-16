use prism::event::{self, OnEvent, Event, TickEvent, PickedMedia};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Stack, Offset, Size, Padding};
use prism::canvas::{Align, Image, ShapeType};
use prism::display::Bin;
use prism::{emitters, Context};

use image::{RgbaImage, DynamicImage, ColorType};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;

use maverick_os::air::{Contract, Metadata, Id, Name};

use serde::{Serialize, Deserialize};

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::theme::{Theme, Color, Icons};
use crate::components::{Rectangle, Icon, Task};
use crate::components::button::GhostIconButton;
use crate::components::text::{Text, TextSize, TextStyle};

static CACHE: Mutex<PhotoCache> = Mutex::new(PhotoCache { entries: BTreeMap::new(), recent: Vec::new(), bytes: 0, decoding: 0 });

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Photo {
    #[serde(with = "base64_bytes")]
    jpeg: Arc<[u8]>,
    width: u32,
    height: u32,
}

impl Photo {
    pub fn from_image(image: &RgbaImage) -> Option<Self> {
        Self::encode(image, 1600, 82)
    }

    fn encode(image: &RgbaImage, side: u32, quality: u8) -> Option<Self> {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 { return None; }
        let scale = (side as f32 / w.max(h) as f32).min(1.0);
        let (width, height) = (((w as f32 * scale).round() as u32).max(1), ((h as f32 * scale).round() as u32).max(1));
        let resized;
        let image = match (width, height) == (w, h) {
            true => image,
            false => { resized = DynamicImage::ImageRgba8(image.clone()).resize_exact(width, height, FilterType::Triangle).into_rgba8(); &resized }
        };
        let rgb: Vec<u8> = image.as_raw().chunks_exact(4).flat_map(|pixel| {
            let a = pixel[3] as u32;
            [0, 1, 2].map(|i| ((pixel[i] as u32 * a + 255 * (255 - a)) / 255) as u8)
        }).collect();
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, quality).encode(&rgb, width, height, ColorType::Rgb8).ok()?;
        Some(Photo { jpeg: jpeg.into(), width, height })
    }

    pub fn size(&self) -> (u32, u32) { (self.width, self.height) }

    pub fn jpeg(&self) -> &[u8] { &self.jpeg }
}

impl std::fmt::Debug for Photo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Photo({}x{}, {} bytes)", self.width, self.height, self.jpeg.len())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Video {
    pub clip: Id,
    pub poster: Photo,
    pub duration_ms: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: Id,
    mp4: Mp4,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Mp4(#[serde(with = "base64_bytes")] Arc<[u8]>);

impl Mp4 {
    pub fn new(mp4: Arc<[u8]>) -> Self { Mp4(mp4) }
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
    type Message = ();
    type Result = ();

    fn id() -> Id { Id::hash("VideoClip") }

    fn init(mp4: Mp4, _metadata: Metadata) -> Self {
        VideoClip { id: mp4.id(), mp4 }
    }

    fn apply(&mut self, _message: (), _metadata: Metadata) {}
}

impl VideoClip {
    pub fn listen(ctx: &mut Context) {
        let _ = ctx.list::<VideoClip>();
    }

    pub fn find(ctx: &mut Context, id: Id) -> Option<Arc<[u8]>> {
        ctx.list::<VideoClip>().values().find_map(|clip| {
            let clip = clip.pending();
            (clip.id == id).then(|| clip.mp4.0.clone())
        })
    }
}

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

#[derive(Debug, Clone)]
pub struct SendPhotoEvent(pub Photo);

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

#[derive(Debug, Clone, Component)]
pub struct PhotoView(Stack, Option<Image>, Option<Rectangle>, #[skip] Photo, #[skip] (f32, f32));

impl PhotoView {
    pub fn message(theme: &Theme, photo: &Photo) -> Self {
        let (w, h) = (photo.width.max(1) as f32, photo.height.max(1) as f32);
        let scale = (240.0 / w).min(320.0 / h);
        let size = ((w * scale).round().max(1.0), (h * scale).round().max(1.0));
        PhotoView(
            Stack(Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            None,
            Some(Rectangle::new(theme.colors().get(ptsd::Background::Secondary), 16.0, None)),
            photo.clone(),
            size,
        )
    }
}

impl OnEvent for PhotoView {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if self.1.is_none() && event.downcast_ref::<TickEvent>().is_some() && let Ok(mut photos) = CACHE.lock() {
            let key = Arc::as_ptr(&self.3.jpeg) as *const u8 as usize;
            let same = |weak: &Weak<[u8]>| weak.upgrade().is_some_and(|jpeg| Arc::ptr_eq(&jpeg, &self.3.jpeg));
            match photos.entries.get(&key) {
                Some((weak, Decoded::Ready(image))) if same(weak) => {
                    let shape = ShapeType::RoundedRectangle(0.0, self.4, 0.0, 16.0);
                    self.1 = Some(Image { shape, image: image.clone(), color: None });
                    self.2 = None;
                    if let Some(i) = photos.recent.iter().position(|k| *k == key) {
                        let k = photos.recent.remove(i);
                        photos.recent.push(k);
                    }
                }
                Some((weak, Decoded::Loading | Decoded::Failed)) if same(weak) => {}
                _ if photos.decoding < 2 => {
                    photos.decoding += 1;
                    if let Some((_, Decoded::Ready(old))) = photos.entries.insert(key, (Arc::downgrade(&self.3.jpeg), Decoded::Loading)) {
                        photos.bytes -= old.as_raw().len();
                        photos.recent.retain(|k| *k != key);
                    }

                    let jpeg = self.3.jpeg.clone();
                    std::thread::spawn(move || {
                        let image = image::load_from_memory(&jpeg).ok().map(|image| {
                            let (w, h) = (image.width(), image.height());
                            let scale = (960.0 / w.max(h) as f32).min(1.0);
                            match scale < 1.0 {
                                true => image.resize_exact(((w as f32 * scale) as u32).max(1), ((h as f32 * scale) as u32).max(1), FilterType::Triangle).into_rgba8(),
                                false => image.into_rgba8(),
                            }
                        });
                        let Ok(mut photos) = CACHE.lock() else { return };
                        photos.decoding -= 1;
                        let weak = Arc::downgrade(&jpeg);
                        match image {
                            Some(image) => {
                                photos.bytes += image.as_raw().len();
                                photos.entries.insert(key, (weak, Decoded::Ready(Arc::new(image))));
                                photos.recent.push(key);
                                while photos.bytes > 64 * 1024 * 1024 && photos.recent.len() > 1 {
                                    let old = photos.recent.remove(0);
                                    if let Some((_, Decoded::Ready(image))) = photos.entries.remove(&old) { photos.bytes -= image.as_raw().len(); }
                                }
                            },
                            None => { photos.entries.insert(key, (weak, Decoded::Failed)); },
                        }
                    });
                }
                _ => {}
            }
        }
        vec![event]
    }
}

#[derive(Debug, Clone, Component)]
pub struct VideoView(Stack, emitters::Button<_VideoView>);
impl OnEvent for VideoView {}

impl VideoView {
    pub fn message(theme: &Theme, video: &Video) -> Self {
        let poster = PhotoView::message(theme, &video.poster);
        let size = poster.4;
        let fill = |x: Offset, y: Offset, p: Padding| Stack(x, y, Size::Static(size.0 - p.0 - p.2), Size::Static(size.1 - p.1 - p.3), p);
        let seconds = (video.duration_ms + 500) / 1000;
        let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
        let duration = match h {
            0 => format!("{m}:{s:02}"),
            _ => format!("{h}:{m:02}:{s:02}"),
        };
        VideoView(Stack::default(), emitters::Button::new(_VideoView(
            Stack(Offset::Start, Offset::Start, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            poster,
            Bin(fill(Offset::Center, Offset::Center, Padding::default()), Badge::play(theme)),
            Bin(fill(Offset::Start, Offset::End, Padding(10.0, 10.0, 10.0, 10.0)), Badge::label(theme, &duration)),
            None,
            video.clone(),
            false,
            theme.clone(),
        )))
    }
}

#[derive(Debug, Clone, Component)]
struct _VideoView(Stack, PhotoView, Bin<Stack, Badge>, Bin<Stack, Badge>, Option<Bin<Stack, Badge>>, #[skip] Video, #[skip] bool, #[skip] Theme);

impl OnEvent for _VideoView {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        let tapped = matches!(event.downcast_ref::<event::Button>(), Some(event::Button::Pressed(false)));
        if tapped || (self.6 && event.downcast_ref::<TickEvent>().is_some()) {
            match VideoClip::find(ctx, self.5.clip) {
                Some(mp4) => {
                    self.6 = false;
                    self.4 = None;
                    ctx.play_video(mp4);
                },
                None => {
                    self.6 = true;
                    let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default());
                    self.4 = Some(Bin(layout, Badge::label(&self.7, "Loading\u{2026}")));
                },
            }
        }
        vec![event]
    }
}

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

#[derive(Debug, Clone, Component)]
pub struct AttachButton(Stack, emitters::Button<_AttachButton>);
impl OnEvent for AttachButton {}

impl AttachButton {
    pub fn new(theme: &Theme) -> Self {
        let button = GhostIconButton::new(theme, Icons::Photos, |_: &mut Context, _: &Theme| {});
        AttachButton(Stack::default(), emitters::Button::new(_AttachButton(Stack::default(), button, 0, VecDeque::new())))
    }
}

#[derive(Debug, Clone, Component)]
struct _AttachButton(Stack, GhostIconButton, #[skip] usize, #[skip] VecDeque<Task<Option<Prepared>>>);

#[derive(Debug)]
enum Prepared { Photo(Photo), Video(NewVideo) }

impl OnEvent for _AttachButton {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if matches!(event.downcast_ref::<event::Button>(), Some(event::Button::Pressed(false))) {
            self.2 += 1;
            ctx.pick_media();
        } else if let Some(media) = event.downcast_ref::<PickedMedia>() && self.2 > 0 {
            self.2 -= 1;
            let media = media.clone();
            self.3.push_back(Task::spawn(move || match media {
                PickedMedia::Photo(image) => Photo::from_image(&image).map(Prepared::Photo),
                PickedMedia::Video(video) => {
                    let poster = Photo::encode(&video.poster, 640, 75)?;
                    Some(Prepared::Video(NewVideo { mp4: video.mp4.as_slice().into(), poster, duration: video.duration }))
                },
            }));
        } else if event.downcast_ref::<TickEvent>().is_some() {
            while let Some(prepared) = self.3.front().and_then(|task| task.take()) {
                self.3.pop_front();
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

struct PhotoCache {
    entries: BTreeMap<usize, (Weak<[u8]>, Decoded)>,
    recent: Vec<usize>,
    bytes: usize,
    decoding: usize,
}

enum Decoded { Loading, Ready(Arc<RgbaImage>), Failed }

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
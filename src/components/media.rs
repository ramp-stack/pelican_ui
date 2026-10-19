use prism::events::{Event, DynEvent, TickEvent};
use prism::drawable::Component;
use ptsd::layout::{Stack, Offset, Size, Padding};
use prism::drawable::{text::Align, shape::Variant};
use prism::hardware::{Media, Picked, Player};
use crate::components::images::MaskedImage;
use std::cell::RefCell;
use std::rc::Rc;
use ptsd::display::Bin;
use prism::Context;

use ptsd::interactions;

use image::{RgbaImage, DynamicImage, ColorType};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;

use prism::air::{Contract, Metadata, Id, Name};

use serde::{Serialize, Deserialize};

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::Task;
use crate::theme::{Theme, Color, Icons};
use crate::components::{Rectangle, Icon, SendEvent};
use crate::components::button::GhostIconButton;
use crate::components::text::{Text, TextSize, TextStyle};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Photo {
    #[serde(with = "crate::base64_bytes")]
    jpeg: Arc<[u8]>,
    width: u32,
    height: u32,
}

impl Photo {
    pub fn from_image(image: &RgbaImage) -> Option<Self> {
        Self::encode(image, 1600, 82)
    }

    pub fn encode(image: &RgbaImage, side: u32, quality: u8) -> Option<Self> {
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
pub struct Mp4(#[serde(with = "crate::base64_bytes")] Arc<Vec<u8>>);

impl Mp4 {
    pub fn new(mp4: Arc<Vec<u8>>) -> Self { Mp4(mp4) }
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
        let _ = ctx.air.instances::<VideoClip>().list();
    }

    pub fn find(ctx: &mut Context, id: Id) -> Option<Arc<Vec<u8>>> {
        ctx.air.instances::<VideoClip>().list().values().find_map(|clip| {
            let clip = clip.pending();
            (clip.id == id).then(|| clip.mp4.0.clone())
        })
    }
}

#[derive(Clone)]
pub struct NewVideo {
    pub mp4: Arc<Vec<u8>>,
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
        let instance = ctx.air.instances::<VideoClip>().create(mp4);
        let me = ctx.air.me();
        for member in members.iter().chain((!members.contains(&me)).then_some(&me)) {
            instance.share(*member);
        }
        Video { clip, poster: self.poster.clone(), duration_ms: self.duration.as_millis() as u64 }
    }
}

#[derive(Debug, Clone, Component)]
#[on_dyn_event]
pub struct PhotoView {
    layout: Stack,
    image: Option<MaskedImage>,
    placeholder: Option<Rectangle>,
    #[skip] photo: Photo,
    #[skip] size: (f32, f32),
}

impl PhotoView {
    pub fn message(theme: &Theme, photo: &Photo) -> Self {
        let (w, h) = (photo.width.max(1) as f32, photo.height.max(1) as f32);
        let scale = (240.0 / w).min(320.0 / h);
        let size = ((w * scale).round().max(1.0), (h * scale).round().max(1.0));
        PhotoView {
            layout: Stack(Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            image: None,
            placeholder: Some(Rectangle::new(theme.colors().get(ptsd::Background::Secondary), 16.0, None)),
            photo: photo.clone(),
            size,
        }
    }
}

impl PhotoView {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if self.image.is_none() && event.downcast_ref::<TickEvent>().is_some() && let Ok(mut photos) = CACHE.lock() {
            let key = Arc::as_ptr(&self.photo.jpeg) as *const u8 as usize;
            let same = |weak: &Weak<[u8]>| weak.upgrade().is_some_and(|jpeg| Arc::ptr_eq(&jpeg, &self.photo.jpeg));
            match photos.entries.get(&key) {
                Some((weak, Decoded::Ready(image))) if same(weak) => {
                    self.image = Some(MaskedImage(image.clone(), Variant::RoundedRectangle(16.0)));
                    self.placeholder = None;
                    if let Some(i) = photos.recent.iter().position(|k| *k == key) && i + 1 != photos.recent.len() {
                        let shown = photos.recent.remove(i).unwrap();
                        photos.recent.push_back(shown);
                    }
                }
                Some((weak, Decoded::Loading | Decoded::Failed)) if same(weak) => {}
                _ if photos.decoding < 2 => {
                    photos.decoding += 1;
                    if let Some((_, Decoded::Ready(old))) = photos.entries.insert(key, (Arc::downgrade(&self.photo.jpeg), Decoded::Loading)) {
                        photos.bytes -= old.as_raw().len();
                        photos.recent.retain(|k| *k != key);
                    }

                    let jpeg = self.photo.jpeg.clone();
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
                        let Some(image) = image else {
                            photos.entries.insert(key, (weak, Decoded::Failed));
                            return;
                        };
                        photos.bytes += image.as_raw().len();
                        photos.entries.insert(key, (weak, Decoded::Ready(Arc::new(image))));
                        photos.recent.push_back(key);
                        while photos.bytes > 64 * 1024 * 1024 && photos.recent.len() > 1 {
                            let Some(old) = photos.recent.pop_front() else { break };
                            if let Some((_, Decoded::Ready(image))) = photos.entries.remove(&old) { photos.bytes -= image.as_raw().len(); }
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
#[on_dyn_event]
pub struct VideoView {
    layout: Stack,
    button: Option<interactions::Button>,
    player: Option<Player>,
    loading: Option<Bin<Stack, Badge>>,
    #[skip] id: uuid::Uuid,
    #[skip] video: Video,
    #[skip] waiting: bool,
    #[skip] theme: Theme,
}

impl VideoView {
    pub fn message(theme: &Theme, video: &Video) -> Self {
        let id = uuid::Uuid::new_v4();
        let poster = PhotoView::message(theme, &video.poster);
        let size = poster.size;
        let fill = |x: Offset, y: Offset, p: Padding| Stack(x, y, Size::Static(size.0 - p.0 - p.2), Size::Static(size.1 - p.1 - p.3), p);
        let seconds = (video.duration_ms + 500) / 1000;
        let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
        let duration = match h {
            0 => format!("{m}:{s:02}"),
            _ => format!("{h}:{m:02}:{s:02}"),
        };
        let face = VideoFace(
            Stack(Offset::Start, Offset::Start, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            poster,
            Bin(fill(Offset::Center, Offset::Center, Padding::default()), Badge::play(theme)),
            Bin(fill(Offset::Start, Offset::End, Padding(10.0, 10.0, 10.0, 10.0)), Badge::label(theme, &duration)),
        );
        let callback = move |ctx: &mut Context| ctx.emit(VideoViewEvent::Play(id));
        VideoView {
            layout: Stack(Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1), Padding::default()),
            button: Some(interactions::Button::new_triggers_on_release(
                face, None::<VideoFace>, None::<VideoFace>, None::<VideoFace>, None::<VideoFace>,
                callback, false,
            )),
            player: None,
            loading: None,
            id,
            video: video.clone(),
            waiting: false,
            theme: theme.clone(),
        }
    }
}

impl VideoView {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let tapped = matches!(event.downcast_ref::<VideoViewEvent>(), Some(VideoViewEvent::Play(id)) if *id == self.id);
        if tapped || (self.waiting && event.downcast_ref::<TickEvent>().is_some()) {
            match VideoClip::find(ctx, self.video.clip) {
                Some(mp4) => {
                    self.waiting = false;
                    self.loading = None;
                    let player = ctx.hardware.video(mp4.as_ref().clone())
                        .ok_or_else(|| "Invalid video".to_string())
                        .and_then(|video| Player::new(&video));
                    match player {
                        Ok(player) => {
                            player.start();
                            self.button = None;
                            self.player = Some(player);
                        }
                        Err(_) => {
                            self.loading = Some(Bin(
                                Stack::fill(), Badge::label(&self.theme, "Unable to play video"),
                            ));
                        }
                    }
                },
                None => {
                    self.waiting = true;
                    let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default());
                    self.loading = Some(Bin(layout, Badge::label(&self.theme, "Loading\u{2026}")));
                },
            }
        }
        vec![event]
    }
}

#[derive(Clone, Component)]
#[on_dyn_event]
pub struct AttachButton {
    layout: Stack,
    button: GhostIconButton,
    #[skip] id: uuid::Uuid,
    #[skip] picked: Rc<RefCell<Option<Picked<Media>>>>,
    #[skip] preparing: VecDeque<Task<Option<SendEvent>>>,
}

impl AttachButton {
    pub fn new(theme: &Theme) -> Self {
        let id = uuid::Uuid::new_v4();
        let mut button = GhostIconButton::new(theme, Icons::Photos, move |ctx: &mut Context, _: &Theme| ctx.emit(AttachButtonEvent::Pick(id)));
        button.1.set_trigger_on_release();
        AttachButton { layout: Stack::default(), button, id, picked: Rc::new(RefCell::new(None)), preparing: VecDeque::new() }
    }
}

impl std::fmt::Debug for AttachButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachButton").field("button", &self.button).finish()
    }
}

impl AttachButton {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if matches!(event.downcast_ref::<AttachButtonEvent>(), Some(AttachButtonEvent::Pick(id)) if *id == self.id) {
            *self.picked.borrow_mut() = Some(ctx.hardware.pick::<Media>());
        } else if event.downcast_ref::<TickEvent>().is_some() {
            let picked = self.picked.borrow_mut().as_mut().and_then(Picked::try_get);
            if let Some(result) = picked {
                self.picked.borrow_mut().take();
                if let Ok(media) = result {
                    self.preparing.push_back(Task::spawn(move || match media {
                        Media::Image(image) => Photo::from_image(&image).map(SendEvent::Photo),
                        Media::Video(video) => {
                            let poster = Photo::encode(&video.thumbnail(), 640, 75)?;
                            Some(SendEvent::Video(NewVideo {
                                mp4: video.bytes().clone(), poster,
                                duration: video.duration().unwrap_or_default(),
                            }))
                        },
                    }));
                }
            }
            while let Some(prepared) = self.preparing.front().and_then(|task| task.take()) {
                self.preparing.pop_front();
                if let Some(send) = prepared {ctx.emit(send);}
            }
        }
        vec![event]
    }
}

#[derive(Debug, Clone)]
enum VideoViewEvent {
    Play(uuid::Uuid),
}

impl Event for VideoViewEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Debug, Clone, Component)]
struct VideoFace(Stack, PhotoView, Bin<Stack, Badge>, Bin<Stack, Badge>);

#[derive(Debug, Clone, Component)]
struct Badge(Stack, Rectangle, Option<Icon>, Option<Text>);

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

#[derive(Debug, Clone)]
enum AttachButtonEvent {
    Pick(uuid::Uuid),
}

impl Event for AttachButtonEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

struct PhotoCache {
    entries: BTreeMap<usize, (Weak<[u8]>, Decoded)>,
    recent: VecDeque<usize>,
    bytes: usize,
    decoding: usize,
}

enum Decoded { Loading, Ready(Arc<RgbaImage>), Failed }

static CACHE: Mutex<PhotoCache> = Mutex::new(PhotoCache { entries: BTreeMap::new(), recent: VecDeque::new(), bytes: 0, decoding: 0 });

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageBuffer, Rgba, ColorType};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use prism::Context;
use prism::air::{Contract, Metadata, Id, Name, Instances, Instance};
use prism::drawable::{Component, text::Align, shape::Variant};
use prism::events::{Event, DynEvent, OnEvent, on_event, OnChange, on_change};
use media::{Media, Format, Image};
use prism::hardware::{Picked, Player, Visual};
use ptsd::display::Bin;
use ptsd::interactions;
use ptsd::layout::{Stack, Offset, Size, Padding};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use std::collections::VecDeque;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::Task;
use crate::theme::{Theme, Color, Icons};
use crate::components::{Rectangle, Icon, SendEvent};
use crate::components::button::GhostIconButton;
use crate::components::images::MaskedImage;
use crate::components::text::{Text, TextSize, TextStyle};

type JpegData = (Arc<[u8]>, (u32, u32));

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Photo {
    #[serde(serialize_with = "Photo::save_jpeg", deserialize_with = "Photo::read_jpeg")]
    jpeg: JpegData,
    // Preserve transmitted metadata for existing serialized messages and AIR signatures.
    width: u32,
    height: u32,
}

impl Photo {
    pub fn from_image(image: &Image) -> Option<Self> { Self::encode(image, 1600, 82) }

    pub fn encode(image: &Image, side: u32, quality: u8) -> Option<Self> {
        let (w, h) = image.dimensions;
        let pixels = image.as_ref();
        let scale = (side as f32 / w.max(h) as f32).min(1.0);
        let width = ((w as f32 * scale).round() as u32).max(1);
        let height = ((h as f32 * scale).round() as u32).max(1);
        let resized;
        let pixels = if (width, height) == (w, h) {
            pixels
        } else {
            let image = ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, pixels).unwrap();
            resized = image::imageops::resize(&image, width, height, FilterType::Triangle);
            resized.as_raw()
        };
        let rgb: Vec<u8> = pixels.as_chunks::<4>().0.iter().flat_map(|pixel| {
            let alpha = pixel[3] as u32;
            [0, 1, 2].map(|i| ((pixel[i] as u32 * alpha + 255 * (255 - alpha)) / 255) as u8)
        }).collect();
        let mut bytes = Vec::new();
        JpegEncoder::new_with_quality(&mut bytes, quality)
            .encode(&rgb, width, height, ColorType::Rgb8).unwrap();
        Some(Self { jpeg: (bytes.into(), (width, height)), width, height })
    }

    pub fn image(&self) -> Image {Image::read(&*self.jpeg.0).unwrap()}
    pub fn size(&self) -> (u32, u32) {self.jpeg.1}
    pub fn jpeg(&self) -> &[u8] { &self.jpeg.0 }

    fn save_jpeg<S: Serializer>(jpeg: &JpegData, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(&jpeg.0))
    }

    fn read_jpeg<'de, D: Deserializer<'de>>(deserializer: D) -> Result<JpegData, D::Error> {
        let text = String::deserialize(deserializer)?;
        let bytes = STANDARD.decode(text).map_err(serde::de::Error::custom)?;
        let image = image::load_from_memory(&bytes).map_err(serde::de::Error::custom)?;
        Ok((bytes.into(), (image.width(), image.height())))
    }
}

impl Hash for Photo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.jpeg.0.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl std::fmt::Debug for Photo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Photo({}x{}, {} bytes)", self.width, self.height, self.jpeg.0.len())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Video {
    pub clip: Id,
    pub poster: Photo,
    pub duration_ms: u64,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Mp4(Arc<[u8]>);

impl Serialize for Mp4 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for Mp4 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        STANDARD.decode(text).map(|bytes| Self(bytes.into())).map_err(serde::de::Error::custom)
    }
}

impl Mp4 {
    pub fn new(mp4: Arc<[u8]>) -> Self { Mp4(mp4) }
    pub fn id(&self) -> Id { Id::hash(&self.0[..]) }
}

impl std::fmt::Debug for Mp4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mp4({} bytes)", self.0.len())
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VideoClip {
    pub id: Id,
    mp4: Mp4,
}

impl std::fmt::Debug for VideoClip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VideoClip({:?}, {:?})", self.id, self.mp4)
    }
}

impl Contract for VideoClip {
    type Init = Mp4;
    type Message = ();
    type Result = ();

    fn id() -> Id { Id::hash("VideoClip") }

    fn init(mp4: Mp4, _metadata: Metadata) -> Self { VideoClip { id: mp4.id(), mp4 } }

    fn apply(&mut self, _message: (), _metadata: Metadata) {}
}

impl VideoClip {
    pub fn listen(ctx: &mut Context) { let _ = ctx.air.instances::<VideoClip>().list(); }

    pub fn find(ctx: &mut Context, id: Id) -> Option<Arc<[u8]>> {
        ctx.air.instances::<VideoClip>().list().values().find_map(|clip| {
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
        let instance = ctx.air.instances::<VideoClip>().create(mp4);
        let me = ctx.air.me();
        for member in members.iter().filter(|name| **name != me) {
            instance.share(*member);
        }
        Video { clip, poster: self.poster.clone(), duration_ms: self.duration.as_millis() as u64 }
    }
}

#[derive(Debug, Clone, Component)]
pub struct PhotoView {
    layout: Stack,
    image: MaskedImage,
    #[skip]
    size: (f32, f32),
}

impl PhotoView {
    pub fn message(photo: &Photo) -> Self {
        let (width, height) = (photo.size().0 as f32, photo.size().1 as f32);
        let scale = (240.0 / width).min(320.0 / height);
        let size = ((width * scale).round(), (height * scale).round());
        Self {
            layout: Stack::new(
                Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1),
                Padding::default(),
            ), image: MaskedImage(photo.image(), Variant::RoundedRectangle(16.0)), size,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct VideoView {
    layout: Stack,
    button: Option<interactions::Button>,
    player: Option<Player>,
    loading: Option<Bin<Stack, Badge>>,
    #[skip]
    id: uuid::Uuid,
    #[skip]
    video: Video,
    #[skip]
    clips: Option<Instances<VideoClip>>,
    #[skip]
    theme: Theme,
}

impl VideoView {
    pub fn message(theme: &Theme, video: &Video) -> Self {
        let id = uuid::Uuid::new_v4();
        let poster = PhotoView::message(&video.poster);
        let size = poster.size;
        let fill = |x: Offset, y: Offset, p: Padding| {
            Stack::new(x, y, Size::Static(size.0 - p.0 - p.2), Size::Static(size.1 - p.1 - p.3), p)
        };
        let seconds = (video.duration_ms + 500) / 1000;
        let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
        let duration = match h {
            0 => format!("{m}:{s:02}"),
            _ => format!("{h}:{m:02}:{s:02}"),
        };
        let face = VideoFace(
            Stack::new(
                Offset::Start, Offset::Start, Size::Static(size.0), Size::Static(size.1),
                Padding::default(),
            ), poster,
            Bin(fill(Offset::Center, Offset::Center, Padding::default()), Badge::play(theme)),
            Bin(
                fill(Offset::Start, Offset::End, Padding(10.0, 10.0, 10.0, 10.0)),
                Badge::label(theme, &duration),
            ),
        );
        let callback = move |ctx: &mut Context| ctx.emit(VideoViewEvent::Play(id));
        VideoView {
            layout: Stack::new(
                Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1),
                Padding::default(),
            ),
            button: Some(interactions::Button::new_triggers_on_release(
                face, None::<VideoFace>, None::<VideoFace>, None::<VideoFace>, None::<VideoFace>,
                callback, false,
            )), player: None, loading: None, id, video: video.clone(), clips: None,
            theme: theme.clone(),
        }
    }
}

#[on_event]
impl OnEvent<VideoViewEvent> for VideoView {
    fn on(&mut self, ctx: &mut Context, event: VideoViewEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, VideoViewEvent::Play(id) if id == self.id) {
            self.clips = Some(ctx.air.instances::<VideoClip>());
            self.load(ctx);
        }
        prism::events![event]
    }
}

impl VideoView {
    fn load(&mut self, ctx: &mut Context) {
        match VideoClip::find(ctx, self.video.clip) {
            Some(mp4) => {
                self.clips = None;
                self.loading = None;
                let player = Media::read(std::io::Cursor::new(mp4))
                    .and_then(|media| {
                        media.video.dimensions().ok_or("Media has no video track")?;
                        ctx.hardware.player(media.player())
                    });
                match player {
                    Ok(player) => {
                        player.start();
                        self.button = None;
                        self.player = Some(player);
                    }
                    Err(_) => {
                        self.loading = Some(Bin(
                            Stack::fill(), Badge::label(&self.theme, "Unable to play video"),
                        ))
                    }
                }
            }
            None => {
                let layout = Stack::new(
                    Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default(),
                );
                self.loading = Some(Bin(layout, Badge::label(&self.theme, "Loading…")));
            }
        }
    }
}
#[on_change(async {
    match &mut self.clips {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<Instance<VideoClip>> for VideoView {
    fn on_change(&mut self, ctx: &mut Context, clip: Instance<VideoClip>) {
        if clip.pending().id == self.video.clip { self.load(ctx); }
    }
}

#[derive(Clone, Component)]
pub struct AttachButton {
    layout: Stack,
    button: GhostIconButton,
    #[skip]
    id: uuid::Uuid,
    #[skip]
    picked: Option<Picked<Visual>>,
    #[skip]
    preparing: VecDeque<Task<Option<SendEvent>>>,
}

impl AttachButton {
    pub fn new(theme: &Theme, layout: Stack) -> Self {
        let id = uuid::Uuid::new_v4();
        let mut button = GhostIconButton::new(theme, Icons::Photos, layout, move |ctx, _| {
            ctx.emit(AttachButtonEvent::Pick(id))
        });
        button.1.set_trigger_on_release();
        AttachButton {
            layout: Stack::default(), button, id, picked: None, preparing: VecDeque::new(),
        }
    }
}

impl std::fmt::Debug for AttachButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachButton").field("button", &self.button).finish()
    }
}

#[on_event]
impl OnEvent<AttachButtonEvent> for AttachButton {
    fn on(&mut self, ctx: &mut Context, event: AttachButtonEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, AttachButtonEvent::Pick(id) if id == self.id) {
            self.picked = Some(ctx.hardware.pick::<Visual>());
        }
        prism::events![event]
    }
}

#[on_change(async {
    match &mut self.picked {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<prism::hardware::PickResult<Visual>> for AttachButton {
    fn on_change(&mut self, _ctx: &mut Context, result: prism::hardware::PickResult<Visual>) {
        self.picked = None;
        let prepare = match result {
            Ok(Visual::Image(image)) => {
                Task::spawn(move || Photo::from_image(&image).map(SendEvent::Photo))
            }
            Ok(Visual::Video(video)) => Task::spawn(move || {
                let thumbnail = video.video.cursor().video()?;
                let poster = Photo::encode(&thumbnail.image, 640, 75)?;
                let format = video.video.format()
                    .filter(|format| matches!(format, Format::Mp4_Mpeg4p2_Aac {..}))
                    .unwrap_or(Format::Mp4_Mpeg4p2_Aac {bitrate: 8_000_000});
                let mut output = Vec::new();
                for chunk in video.encode(format).expect("Could not encode selected video") {
                    output.extend(chunk.expect("Could not encode selected video"));
                }
                Some(SendEvent::Video(NewVideo {
                    mp4: output.into(), poster, duration: video.duration(),
                }))
            }), Err(_) => return,
        };
        self.preparing.push_back(prepare);
    }
}
#[on_change(async {
    loop {
        let prepared = match self.preparing.front_mut() {
            Some(task) => task.next().await,
            None => std::future::pending().await,
        };
        self.preparing.pop_front();
        if let Some(prepared) = prepared {break prepared}
    }
})]
impl OnChange<Option<SendEvent>> for AttachButton {
    fn on_change(&mut self, ctx: &mut Context, prepared: Option<SendEvent>) {
        if let Some(send) = prepared { ctx.emit(send); }
    }
}

#[derive(Debug, Clone)]
enum VideoViewEvent {
    Play(uuid::Uuid),
}

impl Event for VideoViewEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

#[derive(Debug, Clone, Component)]
struct VideoFace(Stack, PhotoView, Bin<Stack, Badge>, Bin<Stack, Badge>);

#[derive(Debug, Clone, Component)]
struct Badge(Stack, Rectangle, Option<Icon>, Option<Text>);

impl Badge {
    fn play(theme: &Theme) -> Self {
        Badge(
            Stack::new(
                Offset::Center, Offset::Center, Size::Static(52.0), Size::Static(52.0),
                Padding::default(),
            ), Rectangle::new(Color::from_hex("000000", 67), 26.0, None),
            Some(Icon::new(theme, Icons::Play, Some(Color::white()), 32.0)), None,
        )
    }

    fn label(theme: &Theme, text: &str) -> Self {
        let around_text = |margin: f32| {
            Size::custom(move |children: Vec<(f32, f32)>| {
                let text = children.last().map(|c| c.1).unwrap_or_default() + 2.0 * margin;
                (text, text)
            })
        };
        Badge(
            Stack::new(
                Offset::Center, Offset::Center, around_text(8.0), around_text(3.0),
                Padding::default(),
            ), Rectangle::new(Color::from_hex("000000", 67), 10.0, None), None,
            Some(Text::new(
                theme, text, TextSize::Sm, TextStyle::Label(Color::white()), Align::Left, None,
            ).fit()),
        )
    }
}

#[derive(Debug, Clone)]
enum AttachButtonEvent {
    Pick(uuid::Uuid),
}

impl Event for AttachButtonEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children { *child.event = Some(self.clone()); }
    }
}

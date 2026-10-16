use prism::event::{OnEvent, Event, TickEvent};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Stack, Offset, Size, Padding};
use prism::canvas::{Image, ShapeType};
use prism::Context;

use image::{RgbaImage, AnimationDecoder};
use image::codecs::gif::GifDecoder;
use image::imageops;

use serde::{Serialize, Deserialize};

use std::collections::{BTreeMap, VecDeque};
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::theme::{Theme, Color};
use crate::components::Rectangle;

static API_KEY: OnceLock<String> = OnceLock::new();

static CACHE: Mutex<GifCache> = Mutex::new(GifCache { entries: BTreeMap::new(), recent: VecDeque::new(), bytes: 0 });

lazy_static::lazy_static! {
    static ref AGENT: ureq::Agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
}

pub fn set_giphy_api_key(key: &str) {
    let _ = API_KEY.set(key.to_string());
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Gif {
    pub id: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub struct SendGifEvent(pub Gif);

impl Event for SendGifEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

pub struct Task<T>(Arc<Mutex<Option<T>>>);

impl<T: Send + 'static> Task<T> {
    pub fn spawn(work: impl FnOnce() -> T + Send + 'static) -> Self {
        let slot = Arc::new(Mutex::new(None));
        let result = slot.clone();
        std::thread::spawn(move || {
            let value = work();
            if let Ok(mut slot) = result.lock() { *slot = Some(value); }
        });
        Task(slot)
    }

    pub fn take(&self) -> Option<T> { self.0.lock().ok()?.take() }
}

impl<T> Clone for Task<T> {
    fn clone(&self) -> Self { Task(self.0.clone()) }
}

impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Task") }
}

#[derive(Debug, Clone)]
pub struct GifBatch {
    pub gifs: Vec<Gif>,
    pub more: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GifError {
    NoApiKey,
    Unavailable,
}

pub fn find_gifs(query: Option<String>, offset: usize) -> Task<Result<GifBatch, GifError>> {
    Task::spawn(move || {
        let key = API_KEY.get().cloned()
            .or_else(|| option_env!("GIPHY_API_KEY").map(str::to_string))
            .filter(|key| !key.is_empty())
            .ok_or(GifError::NoApiKey)?;
        let endpoint = if query.is_some() { "search" } else { "trending" };
        let mut request = AGENT.get(&format!("https://api.giphy.com/v1/gifs/{endpoint}"))
            .query("api_key", &key)
            .query("limit", "24")
            .query("offset", &offset.to_string())
            .query("rating", "g");
        if let Some(query) = &query { request = request.query("q", query); }

        let body: serde_json::Value = request.call().ok()
            .and_then(|response| response.into_json().ok())
            .ok_or(GifError::Unavailable)?;
        let data = body.get("data").and_then(|d| d.as_array()).ok_or(GifError::Unavailable)?;
        let total = body.get("pagination").and_then(|p| p.get("total_count")).and_then(|t| t.as_u64());

        let text = |v: &serde_json::Value, key: &str| v.get(key)?.as_str().map(str::to_string);
        let number = |v: &serde_json::Value, key: &str| v.get(key)?.as_str()?.parse::<u32>().ok();

        Ok(GifBatch {
            gifs: data.iter().filter_map(|value| {
                let images = value.get("images")?;
                let full = images.get("fixed_width")?;
                let small = images.get("fixed_width_small").unwrap_or(full);
                Some(Gif {
                    id: text(value, "id")?,
                    url: text(full, "url")?,
                    width: number(full, "width")?,
                    height: number(full, "height")?,
                    preview: text(small, "url")?,
                })
            }).collect(),
            more: !data.is_empty() && total.is_none_or(|total| ((offset + data.len()) as u64) < total),
        })
    })
}

pub struct GifFrames {
    frames: Vec<(Arc<RgbaImage>, Duration)>,
    length: Duration,
    bytes: usize,
}

impl GifFrames {
    pub fn frame_at(&self, elapsed: Duration) -> (usize, &Arc<RgbaImage>) {
        if self.frames.len() == 1 || self.length.is_zero() { return (0, &self.frames[0].0); }
        let mut t = Duration::from_nanos((elapsed.as_nanos() % self.length.as_nanos()) as u64);
        for (i, (frame, delay)) in self.frames.iter().enumerate() {
            if t < *delay { return (i, frame); }
            t -= *delay;
        }
        (self.frames.len() - 1, &self.frames[self.frames.len() - 1].0)
    }
}

impl std::fmt::Debug for GifFrames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GifFrames({} frames)", self.frames.len())
    }
}

enum CacheEntry {
    Loading,
    Ready(Arc<GifFrames>),
    Failed(Instant),
}

struct GifCache {
    entries: BTreeMap<String, CacheEntry>,
    recent: VecDeque<String>,
    bytes: usize,
}

pub fn load_gif(url: &str, crop: Option<(u32, u32)>) -> Option<Arc<GifFrames>> {
    let key = match crop {
        Some((w, h)) => format!("{url}#{w}x{h}"),
        None => url.to_string(),
    };

    {
        let mut gifs = CACHE.lock().ok()?;
        match gifs.entries.get(&key) {
            Some(CacheEntry::Ready(frames)) => {
                let frames = frames.clone();
                if let Some(i) = gifs.recent.iter().position(|k| *k == key) && i + 1 != gifs.recent.len() {
                    let shown = gifs.recent.remove(i).unwrap();
                    gifs.recent.push_back(shown);
                }
                return Some(frames);
            }
            Some(CacheEntry::Loading) => return None,
            Some(CacheEntry::Failed(at)) if at.elapsed() < Duration::from_secs(30) => return None,
            _ => {}
        }
        gifs.entries.insert(key.clone(), CacheEntry::Loading);
    }

    let url = url.to_string();
    std::thread::spawn(move || {
        let bytes = AGENT.get(&url).call().ok().and_then(|response| {
            let mut bytes = Vec::new();
            response.into_reader().take(8 * 1024 * 1024).read_to_end(&mut bytes).ok().map(|_| bytes)
        });
        let frames: Vec<(Arc<RgbaImage>, Duration)> = bytes
            .and_then(|bytes| GifDecoder::new(std::io::Cursor::new(bytes)).ok())
            .map(|decoder| decoder.into_frames().take(90).map_while(Result::ok).map(|frame| {
                let (numer, denom) = frame.delay().numer_denom_ms();
                let ms = if denom == 0 { 0 } else { numer / denom };
                let image = frame.into_buffer();
                let image = match crop {
                    Some((aw, ah)) => {
                        let (w, h) = image.dimensions();
                        let (cw, ch) = match (w as u64 * ah as u64) > (h as u64 * aw as u64) {
                            true => ((h as u64 * aw as u64 / ah as u64).max(1) as u32, h),
                            false => (w, (w as u64 * ah as u64 / aw as u64).max(1) as u32),
                        };
                        imageops::crop_imm(&image, (w - cw) / 2, (h - ch) / 2, cw, ch).to_image()
                    }
                    None => image,
                };
                (Arc::new(image), Duration::from_millis(if ms < 20 { 100 } else { ms as u64 }))
            }).collect())
            .unwrap_or_default();

        let Ok(mut gifs) = CACHE.lock() else { return };
        if frames.is_empty() {
            gifs.entries.insert(key, CacheEntry::Failed(Instant::now()));
            return;
        }
        let frames = GifFrames {
            length: frames.iter().map(|(_, delay)| *delay).sum(),
            bytes: frames.iter().map(|(image, _)| image.as_raw().len()).sum(),
            frames,
        };
        gifs.bytes += frames.bytes;
        gifs.entries.insert(key.clone(), CacheEntry::Ready(Arc::new(frames)));
        gifs.recent.push_back(key);
        while gifs.bytes > 96 * 1024 * 1024 && gifs.recent.len() > 1 {
            let Some(old) = gifs.recent.pop_front() else { break };
            if let Some(CacheEntry::Ready(frames)) = gifs.entries.remove(&old) { gifs.bytes -= frames.bytes; }
        }
    });
    None
}

#[derive(Debug, Clone, Component)]
pub struct GifPlayer(Stack, Option<Image>, Option<Rectangle>, #[skip] GifSource, #[skip] Option<Arc<GifFrames>>, #[skip] Instant, #[skip] usize, #[skip] bool);

#[derive(Debug, Clone)]
struct GifSource { url: String, crop: Option<(u32, u32)>, size: (f32, f32), radius: f32, placeholder: Color }

impl GifPlayer {
    pub fn message(theme: &Theme, gif: &Gif) -> Self {
        let (w, h) = (gif.width.max(1) as f32, gif.height.max(1) as f32);
        let scale = (200.0 / w).min(280.0 / h);
        let placeholder = theme.colors().get(ptsd::Background::Secondary);
        Self::new(&gif.url, None, ((w * scale).round(), (h * scale).round()), 16.0, placeholder)
    }

    pub fn thumbnail(gif: &Gif, size: (f32, f32)) -> Self {
        let (w, h) = (size.0.round().max(1.0) as u32, size.1.round().max(1.0) as u32);
        let (mut common, mut rest) = (w, h);
        while rest != 0 { (common, rest) = (rest, common % rest); }
        let url = if size.0 > 125.0 { &gif.url } else { &gif.preview };
        Self::new(url, Some((w / common, h / common)), size, 8.0, Color::from_hex("ffffff", 40))
    }

    fn new(url: &str, crop: Option<(u32, u32)>, size: (f32, f32), radius: f32, placeholder: Color) -> Self {
        let layout = Stack(Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1), Padding::default());
        GifPlayer(
            layout,
            None,
            Some(Rectangle::new(placeholder, radius, None)),
            GifSource { url: url.to_string(), crop, size, radius, placeholder },
            None,
            Instant::now(),
            usize::MAX,
            true,
        )
    }

    pub fn set_live(&mut self, live: bool) {
        if live == self.7 { return; }
        self.7 = live;
        if !live {
            self.1 = None;
            self.2 = Some(Rectangle::new(self.3.placeholder, self.3.radius, None));
            self.4 = None;
            self.6 = usize::MAX;
        }
    }
}

impl OnEvent for GifPlayer {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() && self.7 {
            if self.4.is_none() && let Some(frames) = load_gif(&self.3.url, self.3.crop) {
                self.4 = Some(frames);
                self.5 = Instant::now();
                self.2 = None;
            }

            if let Some(frames) = &self.4 {
                let (index, frame) = frames.frame_at(self.5.elapsed());
                if index != self.6 {
                    self.6 = index;
                    match &mut self.1 {
                        Some(image) => image.image = frame.clone(),
                        None => {
                            let shape = ShapeType::RoundedRectangle(0.0, self.3.size, 0.0, self.3.radius);
                            self.1 = Some(Image { shape, image: frame.clone(), color: None });
                        }
                    }
                }
            }
        }
        vec![event]
    }
}
//! GIFs from GIPHY: finding them, downloading them, and playing them.
//!
//! A chat only carries a GIF's id, URLs and size ([`Gif`]); everyone who sees
//! it downloads the GIF itself from GIPHY. Requests and downloads run on
//! background threads, and decoded GIFs are kept in a small cache, so a GIF
//! is downloaded once however often the chat or keyboard showing it is rebuilt.
//!
//! GIPHY needs an API key: call [`set_giphy_api_key`] when the app starts, or
//! build with the `GIPHY_API_KEY` environment variable set.

use prism::event::{OnEvent, Event, TickEvent};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Stack, Offset, Size, Padding};
use prism::canvas::{Image, ShapeType};
use prism::Context;

use image::{RgbaImage, AnimationDecoder};
use image::codecs::gif::GifDecoder;
use image::imageops;

use serde::{Serialize, Deserialize};

use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::theme::{Theme, Color};
use crate::components::Rectangle;

const GIPHY_API: &str = "https://api.giphy.com/v1/gifs";
/// How many GIFs to ask GIPHY for at a time.
const BATCH: usize = 24;
/// The most mature content results may have (GIPHY's ratings: g, pg, pg-13, r).
const RATING: &str = "pg-13";
/// GIFs bigger than this aren't downloaded.
const MAX_DOWNLOAD: u64 = 8 * 1024 * 1024;
/// Frames past this are left off (most GIFs have far fewer).
const MAX_FRAMES: usize = 90;
/// How much memory decoded GIFs may use before the least recently shown are dropped.
const CACHE_BYTES: usize = 96 * 1024 * 1024;
/// A GIF that failed to download is tried again after this long.
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// How big a GIF is shown in a chat: GIPHY's 200px-wide version, no taller than this.
const MESSAGE_WIDTH: f32 = 200.0;
/// The width of GIPHY's small preview (`fixed_width_small`), for keyboard thumbnails.
const PREVIEW_WIDTH: f32 = 100.0;
const MESSAGE_MAX_HEIGHT: f32 = 280.0;

static API_KEY: OnceLock<String> = OnceLock::new();

/// Sets the GIPHY API key (from developers.giphy.com). Call it once when the
/// app starts. Without it, the `GIPHY_API_KEY` set at build time is used.
pub fn set_giphy_api_key(key: &str) {
    let _ = API_KEY.set(key.to_string());
}

fn api_key() -> Option<String> {
    API_KEY.get().cloned()
        .or_else(|| option_env!("GIPHY_API_KEY").map(str::to_string))
        .filter(|key| !key.is_empty())
}

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build())
}

/// A GIF from GIPHY, as it's sent in a chat.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Gif {
    /// GIPHY's id for it.
    pub id: String,
    /// The 200px-wide version, shown in chats.
    pub url: String,
    pub width: u32,
    pub height: u32,
    /// The 100px-wide version, shown on the keyboard.
    pub preview: String,
}

impl Gif {
    fn from_giphy(value: &serde_json::Value) -> Option<Self> {
        let text = |v: &serde_json::Value, key: &str| v.get(key)?.as_str().map(str::to_string);
        // GIPHY gives sizes as strings.
        let number = |v: &serde_json::Value, key: &str| v.get(key)?.as_str()?.parse::<u32>().ok();

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
    }
}

/// Sent by the keyboard's GIF page when you tap a GIF. A chat handles it by
/// sending the GIF.
#[derive(Debug, Clone)]
pub struct SendGifEvent(pub Gif);

impl Event for SendGifEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Work running on a background thread. Check [`Task::take`] each frame.
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

    /// The result, once the work is done. It's only given out once.
    pub fn take(&self) -> Option<T> { self.0.lock().ok()?.take() }
}

impl<T> Clone for Task<T> {
    fn clone(&self) -> Self { Task(self.0.clone()) }
}

impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Task") }
}

/// A batch of GIFs from GIPHY, and whether there are more after it.
#[derive(Debug, Clone)]
pub struct GifBatch {
    pub gifs: Vec<Gif>,
    pub more: bool,
}

/// Why GIFs couldn't be fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GifError {
    /// No GIPHY API key has been set.
    NoApiKey,
    /// GIPHY couldn't be reached, or answered with something unexpected.
    Unavailable,
}

/// Fetches trending GIFs (`query` is `None`) or search results, starting
/// `offset` GIFs in, on a background thread.
pub fn find_gifs(query: Option<String>, offset: usize) -> Task<Result<GifBatch, GifError>> {
    Task::spawn(move || {
        let key = api_key().ok_or(GifError::NoApiKey)?;
        let endpoint = if query.is_some() { "search" } else { "trending" };
        let mut request = agent().get(&format!("{GIPHY_API}/{endpoint}"))
            .query("api_key", &key)
            .query("limit", &BATCH.to_string())
            .query("offset", &offset.to_string())
            .query("rating", RATING);
        if let Some(query) = &query { request = request.query("q", query); }

        let body: serde_json::Value = request.call().ok()
            .and_then(|response| response.into_json().ok())
            .ok_or(GifError::Unavailable)?;
        let data = body.get("data").and_then(|d| d.as_array()).ok_or(GifError::Unavailable)?;
        let total = body.get("pagination").and_then(|p| p.get("total_count")).and_then(|t| t.as_u64());

        Ok(GifBatch {
            gifs: data.iter().filter_map(Gif::from_giphy).collect(),
            more: !data.is_empty() && total.is_none_or(|total| ((offset + data.len()) as u64) < total),
        })
    })
}

/// A decoded GIF: its frames, and how long each is shown.
pub struct GifFrames {
    frames: Vec<(Arc<RgbaImage>, Duration)>,
    length: Duration,
    bytes: usize,
}

impl GifFrames {
    /// The frame showing `elapsed` after the GIF started, and its number.
    pub fn frame_at(&self, elapsed: Duration) -> (usize, &Arc<RgbaImage>) {
        if self.frames.len() == 1 || self.length.is_zero() { return (0, &self.frames[0].0); }
        let mut t = Duration::from_nanos((elapsed.as_nanos() % self.length.as_nanos()) as u64);
        for (i, (frame, delay)) in self.frames.iter().enumerate() {
            if t < *delay { return (i, frame); }
            t -= *delay;
        }
        (self.frames.len() - 1, &self.frames[self.frames.len() - 1].0)
    }

    fn decode(bytes: &[u8], crop: Option<(u32, u32)>) -> Option<Self> {
        let decoder = GifDecoder::new(std::io::Cursor::new(bytes)).ok()?;
        let mut frames = Vec::new();
        for frame in decoder.into_frames().take(MAX_FRAMES) {
            let Ok(frame) = frame else { break };
            let (numer, denom) = frame.delay().numer_denom_ms();
            let ms = if denom == 0 { 0 } else { numer / denom };
            // Browsers show delays this short as 100ms, and GIFs are made to look right there.
            let delay = Duration::from_millis(if ms < 20 { 100 } else { ms as u64 });
            let image = frame.into_buffer();
            let image = match crop { Some(aspect) => crop_to(&image, aspect), None => image };
            frames.push((Arc::new(image), delay));
        }
        if frames.is_empty() { return None; }
        Some(GifFrames {
            length: frames.iter().map(|(_, delay)| *delay).sum(),
            bytes: frames.iter().map(|(image, _)| image.as_raw().len()).sum(),
            frames,
        })
    }
}

impl std::fmt::Debug for GifFrames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GifFrames({} frames)", self.frames.len())
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    match b { 0 => a.max(1), _ => gcd(b, a % b) }
}

/// The middle of `image`, cut to the shape of `aspect` (width, height).
fn crop_to(image: &RgbaImage, (aw, ah): (u32, u32)) -> RgbaImage {
    let (w, h) = image.dimensions();
    let (cw, ch) = match (w as u64 * ah as u64) > (h as u64 * aw as u64) {
        true => ((h as u64 * aw as u64 / ah as u64).max(1) as u32, h),
        false => (w, (w as u64 * ah as u64 / aw as u64).max(1) as u32),
    };
    imageops::crop_imm(image, (w - cw) / 2, (h - ch) / 2, cw, ch).to_image()
}

enum CacheEntry {
    Loading,
    Ready(Arc<GifFrames>),
    Failed(Instant),
}

#[derive(Default)]
struct GifCache {
    entries: HashMap<String, CacheEntry>,
    /// Ready GIFs, least recently shown first.
    recent: VecDeque<String>,
    bytes: usize,
}

impl GifCache {
    fn ready(&mut self, key: String, frames: GifFrames) {
        self.bytes += frames.bytes;
        self.entries.insert(key.clone(), CacheEntry::Ready(Arc::new(frames)));
        self.recent.push_back(key);
        while self.bytes > CACHE_BYTES && self.recent.len() > 1 {
            let Some(old) = self.recent.pop_front() else { break };
            if let Some(CacheEntry::Ready(frames)) = self.entries.remove(&old) { self.bytes -= frames.bytes; }
        }
    }

    fn shown(&mut self, key: &str) {
        if let Some(i) = self.recent.iter().position(|k| k == key) && i + 1 != self.recent.len() {
            let key = self.recent.remove(i).unwrap();
            self.recent.push_back(key);
        }
    }
}

fn cache() -> &'static Mutex<GifCache> {
    static CACHE: OnceLock<Mutex<GifCache>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The GIF at `url`, decoded (and cut to `crop`'s shape, if given), once it's
/// downloaded. The first call starts the download; until it's done, `None`.
pub fn load_gif(url: &str, crop: Option<(u32, u32)>) -> Option<Arc<GifFrames>> {
    let key = match crop {
        Some((w, h)) => format!("{url}#{w}x{h}"),
        None => url.to_string(),
    };

    {
        let mut gifs = cache().lock().ok()?;
        match gifs.entries.get(&key) {
            Some(CacheEntry::Ready(frames)) => {
                let frames = frames.clone();
                gifs.shown(&key);
                return Some(frames);
            }
            Some(CacheEntry::Loading) => return None,
            Some(CacheEntry::Failed(at)) if at.elapsed() < RETRY_AFTER => return None,
            _ => {}
        }
        gifs.entries.insert(key.clone(), CacheEntry::Loading);
    }

    let url = url.to_string();
    std::thread::spawn(move || {
        let frames = download(&url).and_then(|bytes| GifFrames::decode(&bytes, crop));
        if let Ok(mut gifs) = cache().lock() {
            match frames {
                Some(frames) => gifs.ready(key, frames),
                None => { gifs.entries.insert(key, CacheEntry::Failed(Instant::now())); }
            }
        }
    });
    None
}

fn download(url: &str) -> Option<Vec<u8>> {
    let response = agent().get(url).call().ok()?;
    let mut bytes = Vec::new();
    response.into_reader().take(MAX_DOWNLOAD).read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

/// Plays a GIF at a fixed size with rounded corners. Until it's downloaded,
/// a plain box of the same size stands in for it.
///
/// Fields after the layout: the playing frame, the stand-in, where the GIF
/// comes from, its frames once loaded, when it started playing, the number
/// of the frame showing, and whether it's playing (see [`GifPlayer::set_live`]).
#[derive(Debug, Clone, Component)]
pub struct GifPlayer(Stack, Option<Image>, Option<Rectangle>, #[skip] GifSource, #[skip] Option<Arc<GifFrames>>, #[skip] Instant, #[skip] usize, #[skip] bool);

#[derive(Debug, Clone)]
struct GifSource { url: String, crop: Option<(u32, u32)>, size: (f32, f32), radius: f32, placeholder: Color }

impl GifPlayer {
    /// A GIF in a chat.
    pub fn message(theme: &Theme, gif: &Gif) -> Self {
        let (w, h) = (gif.width.max(1) as f32, gif.height.max(1) as f32);
        let scale = (MESSAGE_WIDTH / w).min(MESSAGE_MAX_HEIGHT / h);
        let placeholder = theme.colors().get(ptsd::Background::Secondary);
        Self::new(&gif.url, None, ((w * scale).round(), (h * scale).round()), 16.0, placeholder)
    }

    /// A GIF on the keyboard, filling `size` (cut to fit).
    pub fn thumbnail(gif: &Gif, size: (f32, f32)) -> Self {
        // Cut to the shape, not the size, so a GIF is decoded once for every
        // size of square (whatever the width of the phone).
        let (w, h) = (size.0.round().max(1.0) as u32, size.1.round().max(1.0) as u32);
        let common = gcd(w, h);
        // Much bigger than GIPHY's small preview, the bigger one looks better.
        let url = if size.0 > PREVIEW_WIDTH * 1.25 { &gif.url } else { &gif.preview };
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

    /// Whether it plays (it does to start with). One that doesn't shows its
    /// stand-in and lets go of its frames, so GIFs scrolled well out of sight
    /// don't hold on to memory or keep swapping frames. Their frames stay in
    /// the cache for a while, so playing them again is quick.
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
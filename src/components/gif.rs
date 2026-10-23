use futures::channel::oneshot;
use prism::drawable::{Component, shape::Variant};
use prism::events::{Timer, ChangeSource, OnChange, on_change};
use prism::hardware::{Image, Frame, Player, PlaybackMode};
use prism::Context;
use ptsd::layout::{Stack, Offset, Size, Padding};
use serde::{Serialize, Deserialize};

use std::collections::{BTreeMap, VecDeque};
use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::Task;
use crate::theme::{Theme, Color};
use crate::components::{Rectangle, images::MaskedImage};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Gif {
    pub id: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub struct GifBatch {
    pub gifs: Vec<Gif>,
    pub more: bool,
    pub next_offset: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GifError {
    NoApiKey,
    Unavailable,
}

#[derive(Clone)]
pub struct Gifs(Arc<Mutex<GifCache>>);
impl Gifs {
    pub fn set_api_key(&self, key: &str) {self.0.lock().unwrap().key = Some(key.to_owned());}

    pub fn search(&self, query: Option<String>, offset: usize) -> Task<Result<GifBatch, GifError>> {
        let (key, client) = {
            let cache = self.0.lock().unwrap();
            (cache.key.clone(), cache.client.clone())
        };
        Task::spawn(move || {
            let key = key.filter(|key| !key.is_empty()).ok_or(GifError::NoApiKey)?;
            let endpoint = if query.is_some() {"search"} else {"trending"};
            let mut request = client.get(&format!("https://api.giphy.com/v1/gifs/{endpoint}"))
                .query("api_key", &key).query("limit", "24")
                .query("offset", &offset.to_string()).query("rating", "g");
            if let Some(query) = &query {request = request.query("q", query);}
            let body: serde_json::Value = request.call().ok()
                .and_then(|response| response.into_json().ok()).ok_or(GifError::Unavailable)?;
            let data = body.get("data").and_then(|value| value.as_array())
                .ok_or(GifError::Unavailable)?;
            let total = body.get("pagination").and_then(|value| value.get("total_count"))
                .and_then(|value| value.as_u64());
            let mut batch = GifBatch {
                gifs: Vec::new(), failures: Vec::new(), next_offset: offset + data.len(),
                more: !data.is_empty() && total.is_none_or(|n| ((offset + data.len()) as u64) < n),
            };
            for (index, value) in data.iter().enumerate() {
                let parse = || {
                    let text = |v: &serde_json::Value, key: &str| v.get(key)?.as_str().map(str::to_owned);
                    let number = |v: &serde_json::Value, key: &str| v.get(key)?.as_str()?.parse().ok();
                    let images = value.get("images")?;
                    let full = images.get("fixed_width")?;
                    let small = images.get("fixed_width_small").unwrap_or(full);
                    Some(Gif {
                        id: text(value, "id")?, url: text(full, "url")?,
                        width: number(full, "width")?, height: number(full, "height")?,
                        preview: text(small, "url")?,
                    })
                };
                match parse() {
                    Some(gif) => batch.gifs.push(gif),
                    None => batch.failures.push(format!("GIF {} has invalid metadata", offset + index)),
                }
            }
            Ok(batch)
        })
    }
}
impl Default for Gifs {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(GifCache {
            key: option_env!("GIPHY_API_KEY").map(str::to_owned),
            client: ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build(),
            entries: BTreeMap::new(), recent: VecDeque::new(), bytes: 0,
            downloading: 0, waiting: Vec::new(),
        })))
    }
}
impl std::fmt::Debug for Gifs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {f.write_str("Gifs")}
}

#[derive(Debug, Clone, Component)]
pub struct GifPlayer {
    layout: Stack,
    image: Option<MaskedImage>,
    placeholder: Option<Rectangle>,
    #[skip] url: String,
    #[skip] gifs: Gifs,
    #[skip] loading: Option<GifLoad>,
    #[skip] retry: Timer,
    #[skip] radius: f32,
    #[skip] color: Color,
    #[skip] player: Option<Player>,
    #[skip] active: bool,
}
impl GifPlayer {
    pub fn message(theme: &Theme, gif: &Gif) -> Self {
        let (w, h) = (gif.width.max(1) as f32, gif.height.max(1) as f32);
        let scale = (200.0 / w).min(280.0 / h);
        let color = theme.colors().get(ptsd::Background::Secondary);
        Self::new(theme, &gif.url, ((w * scale).round(), (h * scale).round()), 16.0, color)
    }

    pub fn thumbnail(theme: &Theme, gif: &Gif, size: (f32, f32)) -> Self {
        let url = if size.0 > 125.0 {&gif.url} else {&gif.preview};
        Self::new(theme, url, size, 8.0, Color::from_hex("ffffff", 5))
    }

    pub fn set_active(&mut self, active: bool) {
        if active == self.active {return}
        self.active = active;
        self.loading = active.then(|| GifLoad {url: self.url.clone(), gifs: self.gifs.clone()});
        self.retry.cancel();
        if !active {
            self.image = None;
            self.placeholder = Some(Rectangle::new(self.color, self.radius, None));
            self.player = None;
        }
    }

    fn new(theme: &Theme, url: &str, size: (f32, f32), radius: f32, color: Color) -> Self {
        GifPlayer {
            layout: Stack(
                Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1),
                Padding::default(),
            ),
            image: None, placeholder: Some(Rectangle::new(color, radius, None)),
            url: url.to_string(), gifs: theme.gifs().clone(),
            loading: Some(GifLoad {url: url.to_string(), gifs: theme.gifs().clone()}),
            retry: Timer::default(), radius, color, player: None, active: true,
        }
    }

    fn show_frame(&mut self, frame: Image) {
        self.image = Some(MaskedImage(frame, Variant::RoundedRectangle(self.radius)));
        self.placeholder = None;
    }
}
#[on_change(ChangeSource::changed(&mut self.loading))]
impl OnChange<Result<Arc<Vec<u8>>, Instant>> for GifPlayer {
    fn on_change(&mut self, ctx: &mut Context, result: Result<Arc<Vec<u8>>, Instant>) {
        self.loading = None;
        let bytes = match result {
            Ok(bytes) => bytes,
            Err(at) => {
                self.retry.reset(at.saturating_duration_since(Instant::now()));
                return
            }
        };
        let player = ctx.hardware.read_media(Cursor::new(bytes.as_slice())).and_then(|media| {
            media.dimensions().ok_or("Media has no video track")?;
            Player::new(media)
        });
        match player {
            Ok(player) => {
                player.set_mode(PlaybackMode::Loop);
                if let Some(image) = player.frame().image {self.show_frame(image);}
                player.start();
                self.player = Some(player);
            }
            Err(_) => {
                let at = Instant::now();
                let mut gifs = self.gifs.0.lock().unwrap();
                if let Some(CacheEntry::Ready(bytes)) = gifs.entries.remove(&self.url) {
                    gifs.bytes -= bytes.len();
                }
                gifs.recent.retain(|url| url != &self.url);
                gifs.entries.insert(self.url.clone(), CacheEntry::Failed(at));
                self.retry.reset(Duration::from_secs(30));
            }
        }
    }
}
#[on_change(ChangeSource::changed(&mut self.retry))]
impl OnChange<()> for GifPlayer {
    fn on_change(&mut self, _ctx: &mut Context, _: ()) {
        self.loading = Some(GifLoad {url: self.url.clone(), gifs: self.gifs.clone()});
    }
}
#[on_change(ChangeSource::changed(&mut self.player))]
impl OnChange<Frame> for GifPlayer {
    fn on_change(&mut self, _ctx: &mut Context, frame: Frame) {
        if let Some(image) = frame.image {self.show_frame(image);}
    }
}

enum CacheEntry {
    Loading,
    Ready(Arc<Vec<u8>>),
    Failed(Instant),
}

struct GifCache {
    key: Option<String>,
    client: ureq::Agent,
    downloading: usize,
    entries: BTreeMap<String, CacheEntry>,
    recent: VecDeque<String>,
    bytes: usize,
    waiting: Vec<oneshot::Sender<()>>,
}

#[derive(Debug, Clone)]
struct GifLoad {
    url: String,
    gifs: Gifs,
}
impl ChangeSource for GifLoad {
    type Update = Result<Arc<Vec<u8>>, Instant>;
    async fn next(&mut self) -> Option<Self::Update> {
        loop {
            let waiting = {
                let mut gifs = self.gifs.0.lock().unwrap();
                match gifs.entries.get(&self.url) {
                    Some(CacheEntry::Ready(bytes)) => {
                        let bytes = bytes.clone();
                        gifs.recent.retain(|url| url != &self.url);
                        gifs.recent.push_back(self.url.clone());
                        return Some(Ok(bytes))
                    }
                    Some(CacheEntry::Failed(at)) if at.elapsed() < Duration::from_secs(30) => {
                        return Some(Err(*at + Duration::from_secs(30)))
                    }
                    _ => {},
                }
                if gifs.downloading < 4 && !matches!(gifs.entries.get(&self.url), Some(CacheEntry::Loading)) {
                    gifs.downloading += 1;
                    gifs.entries.insert(self.url.clone(), CacheEntry::Loading);
                    let url = self.url.clone();
                    let owner = self.gifs.clone();
                    let client = gifs.client.clone();
                    std::thread::spawn(move || {
                        let bytes = client.get(&url).call().ok().and_then(|response| {
                            let mut bytes = Vec::new();
                            response.into_reader().take(8 * 1024 * 1024 + 1)
                                .read_to_end(&mut bytes).ok()?;
                            (!bytes.is_empty() && bytes.len() <= 8 * 1024 * 1024).then_some(bytes)
                        });
                        let mut gifs = owner.0.lock().unwrap();
                        gifs.downloading -= 1;
                        match bytes {
                            Some(bytes) => {
                                gifs.bytes += bytes.len();
                                gifs.entries.insert(url.clone(), CacheEntry::Ready(Arc::new(bytes)));
                                gifs.recent.push_back(url.clone());
                            }
                            None => {gifs.entries.insert(url.clone(), CacheEntry::Failed(Instant::now()));}
                        }
                        if gifs.entries.len() > 512 {
                            let failed: Vec<_> = gifs.entries.iter().filter_map(|(key, entry)| {
                                (key != &url && matches!(entry, CacheEntry::Failed(_)))
                                    .then_some(key.clone())
                            }).take(gifs.entries.len() - 512).collect();
                            for url in failed {gifs.entries.remove(&url);}
                        }
                        while (gifs.bytes > 96 * 1024 * 1024 || gifs.entries.len() > 512)
                            && gifs.recent.len() > 1
                        {
                            let old = gifs.recent.pop_front().unwrap();
                            if let Some(CacheEntry::Ready(bytes)) = gifs.entries.remove(&old) {
                                gifs.bytes -= bytes.len();
                            }
                        }
                        let waiting = std::mem::take(&mut gifs.waiting);
                        drop(gifs);
                        for sender in waiting {let _ = sender.send(());}
                    });
                }
                let (sender, receiver) = oneshot::channel();
                gifs.waiting.retain(|sender| !sender.is_canceled());
                gifs.waiting.push(sender);
                receiver
            };
            waiting.await.unwrap();
        }
    }
}

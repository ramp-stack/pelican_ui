use futures::channel::oneshot;
use prism::drawable::{Component, shape::Variant};
use prism::events::{Timer, OnChange, on_change};
use media::{Media, Frame, Image};
use prism::hardware::{Player, PlaybackMode};
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
    #[skip] player: Option<GifPlayback>,
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
            layout: Stack::new(
                Offset::Center, Offset::Center, Size::Static(size.0), Size::Static(size.1),
                Padding::default(),
            ),
            image: None, placeholder: Some(Rectangle::new(color, radius, None)),
            url: url.to_string(), gifs: theme.gifs().clone(),
            loading: Some(GifLoad {url: url.to_string(), gifs: theme.gifs().clone()}),
            retry: Timer::default(), radius, color, player: None, active: true,
        }
    }

    fn show_frame(&mut self, image: Image) {
        self.image = Some(MaskedImage(image, Variant::RoundedRectangle(self.radius)));
        self.placeholder = None;
    }
}
#[on_change(async {
    match &mut self.loading {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<Result<Arc<[u8]>, Instant>> for GifPlayer {
    fn on_change(&mut self, ctx: &mut Context, result: Result<Arc<[u8]>, Instant>) {
        self.loading = None;
        let bytes = match result {
            Ok(bytes) => bytes,
            Err(at) => {
                self.retry.reset(at.saturating_duration_since(Instant::now()));
                return
            }
        };
        let player = Media::read(Cursor::new(bytes)).and_then(|media| {
            media.video.dimensions().ok_or("Media has no video track")?;
            // GIF URLs may resolve to containers with audio. Keep only the
            // video track so previews and messages never create audio output.
            let mut player = media.video.player();
            player.set_volume(0.0);
            ctx.hardware.player(player)
        });
        match player {
            Ok(player) => {
                player.set_mode(PlaybackMode::Loop);
                let frame = player.frame();
                if let Some(frame) = &frame.video {self.show_frame(frame.image.clone());}
                player.start();
                self.player = Some(GifPlayback::new(player, frame.video));
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
#[on_change(self.retry.next())]
impl OnChange<()> for GifPlayer {
    fn on_change(&mut self, _ctx: &mut Context, _: ()) {
        self.loading = Some(GifLoad {url: self.url.clone(), gifs: self.gifs.clone()});
    }
}
#[on_change(async {
    match &mut self.player {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<Frame> for GifPlayer {
    fn on_change(&mut self, _ctx: &mut Context, frame: Frame) {
        if let Some(frame) = frame.video {self.show_frame(frame.image.clone());}
    }
}

// GIF rendering needs the latest image, not every intermediate playback update.
// Sampling snapshots tolerates overflow in the backend's bounded notification queue
// when decoding other GIFs or presenting a frame stalls the UI. Never await
// Player::next here: its lossless subscription can panic after such a stall.
#[derive(Debug, Clone)]
struct GifPlayback {
    player: Player,
    refresh: Timer,
    shown: Option<Arc<media::VideoFrame>>,
}
impl GifPlayback {
    const INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

    fn new(player: Player, shown: Option<Arc<media::VideoFrame>>) -> Self {
        Self {player, refresh: Timer::new(Self::INTERVAL), shown}
    }

    async fn next(&mut self) -> Frame {
        loop {
            self.refresh.next().await;
            // Re-arm from now, so a late poll never tries to catch up on old ticks.
            self.refresh.reset(Self::INTERVAL);
            let frame = self.player.frame();
            if let Some(video) = &frame.video {
                if self.shown.as_ref().is_none_or(|shown| !Arc::ptr_eq(shown, video)) {
                    self.shown = Some(video.clone());
                    return frame;
                }
            }
        }
    }
}

enum CacheEntry {
    Loading,
    Ready(Arc<[u8]>),
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
impl GifLoad {
    async fn next(&mut self) -> Result<Arc<[u8]>, Instant> {
        loop {
            let waiting = {
                let mut gifs = self.gifs.0.lock().unwrap();
                match gifs.entries.get(&self.url) {
                    Some(CacheEntry::Ready(bytes)) => {
                        let bytes = bytes.clone();
                        gifs.recent.retain(|url| url != &self.url);
                        gifs.recent.push_back(self.url.clone());
                        return Ok(bytes)
                    }
                    Some(CacheEntry::Failed(at)) if at.elapsed() < Duration::from_secs(30) => {
                        return Err(*at + Duration::from_secs(30))
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
                                gifs.entries.insert(url.clone(), CacheEntry::Ready(bytes.into()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{FutureExt, executor::block_on};
    use prism::traits;

    // A snapshot-only backend: reading the notification stream is deliberately
    // fatal, as it is for the real backend after its eight-update queue overflows.
    #[derive(Clone, Debug)]
    struct SnapshotPlayer(Arc<Mutex<VecDeque<Frame>>>);
    impl traits::Player for SnapshotPlayer {
        fn has_video(&self) -> bool {true}
        fn set_mode(&self, _: PlaybackMode) {}
        fn mode(&self) -> PlaybackMode {PlaybackMode::Loop}
        fn start(&self) {}
        fn stop(&self) {}
        fn set_speed(&self, _: f64) {}
        fn speed(&self) -> f64 {1.0}
        fn set_volume(&self, _: f64) {}
        fn volume(&self) -> f64 {0.0}
        fn seek(&self, _: Duration) {}
        fn position(&self) -> Duration {Duration::ZERO}
        fn duration(&self) -> Option<Duration> {None}
        fn is_playing(&self) -> bool {true}
        fn error(&self) -> Option<String> {None}
        fn frame(&self) -> Frame {
            let mut frames = self.0.lock().unwrap();
            if frames.len() > 1 {frames.pop_front().unwrap()} else {frames[0].clone()}
        }
        fn next(&mut self) -> traits::BoxFuture<'_, Frame> {
            panic!("GIF rendering must not read the overflowed notification queue")
        }
    }

    fn frame(index: u64) -> Frame {
        Frame {
            video: Some(Arc::new(media::VideoFrame {
                timestamp: Duration::from_millis(index * 20), ..Default::default()
            })),
            position: Duration::from_millis(index * 20), duration: None,
            speed: 1.0, volume: 0.0, playing: true,
        }
    }

    fn playback(frames: Vec<Frame>, shown: &Frame) -> GifPlayback {
        GifPlayback::new(
            Player::new(SnapshotPlayer(Arc::new(Mutex::new(frames.into())))),
            shown.video.clone(),
        )
    }

    fn next_frame(playback: &mut GifPlayback) -> Frame {
        block_on(async {
            let mut timeout = Timer::new(Duration::from_secs(2));
            futures::select! {
                frame = playback.next().fuse() => frame,
                _ = timeout.next().fuse() => panic!("GIF did not resume playback"),
            }
        })
    }

    #[test]
    fn resumes_at_latest_frame_after_missing_more_than_the_queue_capacity() {
        let shown = frame(5);
        let latest = frame(163);
        let mut playback = playback(vec![latest.clone()], &shown);
        let resumed = next_frame(&mut playback);
        assert!(Arc::ptr_eq(resumed.video.as_ref().unwrap(), latest.video.as_ref().unwrap()));
        assert_eq!(resumed.position, latest.position);
        // No burst of stale frames or repeated redraw of this same snapshot.
        assert!(playback.next().now_or_never().is_none());
    }

    #[test]
    fn skips_empty_and_unchanged_snapshots_until_the_image_changes() {
        let shown = frame(5);
        let mut empty = shown.clone();
        empty.video = None;
        let mut progress_only = shown.clone();
        progress_only.position += Duration::from_millis(5);
        let latest = frame(6);
        let mut playback = playback(vec![empty, shown.clone(), progress_only, latest.clone()], &shown);
        assert_eq!(next_frame(&mut playback), latest);
    }

    #[test]
    fn deactivation_drops_playback_and_reactivation_starts_loading() {
        let mut gif = GifPlayer::new(&Theme::default(), "unused", (100.0, 100.0), 8.0, Color::white());
        let shown = frame(5);
        gif.loading = None;
        gif.player = Some(playback(vec![frame(163)], &shown));
        gif.set_active(false);
        assert!(gif.player.is_none());
        assert!(gif.loading.is_none());
        assert!(!gif.retry.is_active());
        gif.set_active(true);
        assert!(gif.player.is_none());
        assert!(gif.loading.is_some());
    }
}

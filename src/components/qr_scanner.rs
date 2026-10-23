use prism::events::{DynEvent, OnChange, on_change, Timer};
use prism::drawable::text::Align;
use prism::hardware::{Image, Camera, Frame, Player, PlaybackMode};
use ptsd::display::{Bin, Opt};
use prism::Context;
use prism::drawable::Component;
use ptsd::layout::{Column, Padding, Size, Offset, Stack};
use ptsd::theme::TextSize;
use ptsd::navigation::NavigationEvent;
use uuid::Uuid;
use image::{ImageBuffer, Rgba};

use crate::Task;
use crate::theme::{Theme, Icons, Color};
use crate::components::text::{TextStyle, Text};
use crate::components::{Icon, Rectangle};
use crate::interface::general::FormEvent;

crate::closure_trait!(QrCodeFound, FnMut(&mut Context, String));

#[derive(Component, Clone)]
#[on_dyn_event]
pub struct QRCodeScanner {
    layout: Stack,
    preview: Option<Bin<Stack, Image>>,
    guide: QRGuide,
    #[skip] decoding: Option<Task<Option<String>>>,
    #[skip] found: Option<String>,
    #[skip] last: Option<String>,
    #[skip] on_find: Box<dyn QrCodeFound>,
    #[skip] player: Option<Player>,
    #[skip] camera: Option<Camera>,
    #[skip] form: Option<Uuid>,
    #[skip] initial: Timer,
}

impl QRCodeScanner {
    pub fn new(theme: &Theme, on_find: Box<dyn QrCodeFound>) -> Self {
        QRCodeScanner {
            layout: Stack::center(),
            preview: None,
            guide: QRGuide::new(theme),
            decoding: None,
            found: None,
            last: None,
            on_find,
            player: None,
            camera: None,
            form: None, initial: Timer::new(std::time::Duration::ZERO),
        }
    }

    pub fn found(&self) -> Option<String> { self.found.clone() }
}

impl std::fmt::Debug for QRCodeScanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QRCodeScanner").field("found", &self.found).finish()
    }
}

impl QRCodeScanner {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(FormEvent::Bind(form)) = event.downcast_ref::<FormEvent>() {
            self.form = Some(*form);
        }
        vec![event]
    }

    fn show(&mut self, image: Image) {
        // bardecoder needs at least one 5x5 block.
        let (width, height) = image.dimensions();
        if width < 5 || height < 5 { return; }
        if self.decoding.is_none() {
            let image = image.clone();
            self.decoding = Some(Task::spawn(move || {
                let ((width, height), pixels) = image.pixels();
                if width < 5 || height < 5 {return None}
                let pixels = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixels).unwrap();
                let decode = |image: &ImageBuffer<Rgba<u8>, &[u8]>| {
                    bardecoder::default_decoder().decode(image)
                        .into_iter().flatten().find(|s| !s.is_empty())
                };
                decode(&pixels).or_else(|| {
                    let flipped = image::imageops::flip_horizontal(&pixels);
                    let pixels = ImageBuffer::from_raw(width, height, flipped.as_raw().as_slice())
                        .unwrap();
                    decode(&pixels)
                })
            }));
        }
        self.guide.1.display(false);
        self.guide.3.display(false);
        let layout = Stack(
            Offset::Center, Offset::Center, Size::Static(300.0), Size::Static(300.0), Padding::default(),
        );
        self.preview = Some(Bin(layout, image));
    }

    fn error(&mut self, error: String) {
        self.decoding = None;
        self.preview = None;
        self.player = None;
        self.camera = None;
        self.guide.1.display(true);
        self.guide.3.inner().2.spans[0].text = error;
        self.guide.3.display(true);
    }
}
#[on_change(self.initial.next())]
impl OnChange<()> for QRCodeScanner {
    fn on_change(&mut self, ctx: &mut Context, _: ()) {
        let capture = ctx.hardware.start_camera().and_then(|camera| {
            Player::new(&camera).map(|player| (camera, player))
        });
        let (camera, player) = match capture {
            Ok(capture) => capture,
            Err(error) => {self.error(error); return},
        };
        player.set_mode(PlaybackMode::Live(std::time::Duration::ZERO));
        player.start();
        let frame = player.frame();
        if let Some(error) = frame.error {self.error(error); return}
        if let Some(image) = frame.image {self.show(image);}
        self.player = Some(player);
        self.camera = Some(camera);
    }
}
#[on_change(async {
    match &mut self.player {
        Some(source) => source.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<Frame> for QRCodeScanner {
    fn on_change(&mut self, _ctx: &mut Context, frame: Frame) {
        if let Some(error) = frame.error {self.error(error); return}
        if let Some(image) = frame.image {self.show(image);}
    }
}
#[on_change(async {
    let Some(task) = &mut self.decoding else {return std::future::pending().await};
    match task.next().await {Some(result) => result, None => std::future::pending().await}
})]
impl OnChange<Option<String>> for QRCodeScanner {
    fn on_change(&mut self, ctx: &mut Context, found: Option<String>) {
        self.decoding = None;
        self.found = found;
        if let Some(code) = self.found.clone() && self.last.as_ref() != Some(&code) {
            self.last = Some(code.clone());
            self.preview = None;
            self.player = None;
            self.camera = None;
            if let Some(form) = self.form {ctx.emit(FormEvent::Refresh(form));}
            let mut on_find = self.on_find.clone();
            ctx.emit(NavigationEvent::reset_with_fn(move |ctx: &mut Context| {
                on_find(ctx, code.clone());
            }));
        }
    }
}

#[derive(Debug, Component, Clone)]
struct QRGuide(Stack, Opt<Rectangle>, Rectangle, Opt<Message>);

impl QRGuide {
    fn new(theme: &Theme) -> Self {
        let background = theme.colors().get(ptsd::Background::Secondary);
        let outline = theme.colors().get(ptsd::Outline::Secondary);
        QRGuide(
            Stack(
                Offset::Center, Offset::Center,
                Size::Static(308.0), Size::Static(308.0), Padding::default(),
            ),
            Opt::new(Rectangle::new(background, 8.0, None), true),
            Rectangle::new(Color::transparent(), 8.0, Some((4.0, outline))),
            Opt::new(Message::new(theme, Icons::Camera, "Accessing device camera."), true),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Message(Column, Icon, Text);

impl Message {
    fn new(theme: &Theme, icon: Icons, msg: &str) -> Self {
        Message(Column::center(4.0),
            Icon::new(theme, icon, Some(theme.colors().get(ptsd::Text::Heading)), 48.0),
            Text::new(theme, msg, TextSize::Sm, TextStyle::Secondary, Align::Left, None)
        )
    }
}

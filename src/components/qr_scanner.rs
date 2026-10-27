use prism::events::{DynEvent, OnEvent, on_event, OnChange, on_change, Timer};
use prism::drawable::text::Align;
use media::{Image, VideoFrame};
use prism::hardware::Camera;
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
pub struct QRCodeScanner {
    layout: Stack,
    preview: Option<Bin<Stack, Image>>,
    guide: QRGuide,
    #[skip] decoding: Option<Task<Option<String>>>,
    #[skip] found: Option<String>,
    #[skip] last: Option<String>,
    #[skip] on_find: Box<dyn QrCodeFound>,
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

#[on_event]
impl OnEvent<FormEvent> for QRCodeScanner {
    fn on(&mut self, _ctx: &mut Context, event: FormEvent) -> Vec<Box<dyn DynEvent>> {
        if let FormEvent::Bind(form) = event {self.form = Some(form);}
        prism::events![event]
    }
}

impl QRCodeScanner {
    fn show(&mut self, image: Image) {
        // bardecoder needs at least one 5x5 block.
        let (width, height) = image.dimensions;
        if width < 5 || height < 5 { return; }
        if self.decoding.is_none() {
            let image = image.clone();
            self.decoding = Some(Task::spawn(move || {
                let (width, height) = image.dimensions;
                let pixels = image.as_ref();
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
        let layout = Stack::new(
            Offset::Center, Offset::Center, Size::Static(300.0), Size::Static(300.0), Padding::default(),
        );
        self.preview = Some(Bin(layout, image));
    }

    fn error(&mut self, error: String) {
        self.decoding = None;
        self.preview = None;
        self.camera = None;
        self.guide.1.display(true);
        self.guide.3.inner().2.spans[0].text = error;
        self.guide.3.display(true);
    }
}
#[on_change(self.initial.next())]
impl OnChange<()> for QRCodeScanner {
    fn on_change(&mut self, ctx: &mut Context, _: ()) {
        match ctx.hardware.start_camera() {
            Ok(camera) => self.camera = Some(camera),
            Err(error) => self.error(error),
        }
    }
}
#[on_change(async {
    match &mut self.camera {
        Some(camera) => camera.next().await,
        None => std::future::pending().await,
    }
})]
impl OnChange<Option<VideoFrame>> for QRCodeScanner {
    fn on_change(&mut self, _ctx: &mut Context, frame: Option<VideoFrame>) {
        match frame {
            Some(frame) => self.show(frame.image),
            None => {
                let error = self.camera.as_ref().and_then(Camera::error)
                    .unwrap_or_else(|| "Camera capture stopped".into());
                self.error(error);
            }
        }
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
            Stack::new(
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
            Text::new(theme, msg, TextSize::Sm, TextStyle::Secondary, Align::Left, None).fit()
        )
    }
}

use prism::event::{OnEvent, Event, TickEvent, CameraFrame};
use prism::canvas::{ShapeType, Image, Align};
use prism::Context;
use prism::drawable::{Component, SizedTree};
use prism::layout::{Column, Padding, Size, Offset, Stack};

use ptsd::theme::TextSize;
use ptsd::navigation::NavigationEvent;

use crate::Task;
use crate::theme::{Theme, Icons, Color};
use crate::components::text::{TextStyle, Text};
use crate::components::{Icon, Rectangle};

use image::DynamicImage;

crate::closure_trait!(QrCodeFound, FnMut(&mut Context, String));

#[derive(Debug, Component, Clone)]
pub struct QRCodeScanner {
    layout: Stack,
    preview: Option<Image>,
    guide: QRGuide,
    #[skip] decoding: Option<Task<Option<String>>>,
    #[skip] found: Option<String>,
    #[skip] last: Option<String>,
    #[skip] on_find: Box<dyn QrCodeFound>,
    #[skip] camera: Option<Box<dyn prism::Camera>>,
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
        }
    }

    pub fn found(&self) -> Option<String> { self.found.clone() }
}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if self.camera.is_none() {
                self.camera = Some(ctx.start_camera());
            }
            if let Some(code) = self.found.take() && self.last.as_ref() != Some(&code) {
                self.last = Some(code.clone());
                self.preview = None;
                self.camera = None;
                let mut on_find = self.on_find.clone();
                ctx.emit(NavigationEvent::reset_with_fn(move |ctx: &mut Context| {
                    on_find(ctx, code.clone());
                    ctx.emit(QRCodeScannedEvent(code.clone()));
                }));
            }
        } else if let Some(CameraFrame(image)) = event.downcast_ref::<CameraFrame>() {
            if let Some(found) = self.decoding.as_ref().and_then(|task| task.take()) {
                self.decoding = None;
                self.found = found;
            }
            if self.decoding.is_none() {
                let image = image.clone();
                self.decoding = Some(Task::spawn(move || {
                    let decoder = bardecoder::default_decoder();
                    let decode = |image: DynamicImage| decoder.decode(&image).into_iter().flatten().find(|s| !s.is_empty());
                    decode(DynamicImage::ImageRgba8(image.clone()))
                        .or_else(|| decode(DynamicImage::ImageRgba8(image::imageops::flip_horizontal(&image))))
                }));
            }

            self.guide.1 = None;
            self.guide.3 = None;
            self.preview = Some(Image {
                shape: ShapeType::Rectangle(0.0, (300.0, 300.0), 0.0),
                image: image.clone().into(),
                color: None,
            });
        }
        vec![event]
    }
}

#[derive(Debug, Component, Clone)]
struct QRGuide(Stack, Option<Rectangle>, Rectangle, Option<Message>);
impl OnEvent for QRGuide {}

impl QRGuide {
    fn new(theme: &Theme) -> Self {
        let background = theme.colors().get(ptsd::Background::Secondary);
        let outline = theme.colors().get(ptsd::Outline::Secondary);
        QRGuide(
            Stack(Offset::Center, Offset::Center, Size::Static(308.0), Size::Static(308.0), Padding::default()),
            Some(Rectangle::new(background, 8.0, None)),
            Rectangle::new(Color::TRANSPARENT, 8.0, Some((4.0, outline))),
            Some(Message::new(theme, Icons::Camera, "Accessing device camera.")),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct Message(Column, Image, Text);
impl OnEvent for Message {}

impl Message {
    fn new(theme: &Theme, icon: Icons, msg: &str) -> Self {
        Message(Column::center(4.0),
            Icon::new(theme, icon, Some(theme.colors().get(ptsd::Text::Heading)), 48.0),
            Text::new(theme, msg, TextSize::Sm, TextStyle::Secondary, Align::Left, None)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QRCodeScannedEvent(pub String);

prism::impl_event_all_children!(QRCodeScannedEvent);

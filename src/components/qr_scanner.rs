use prism::event::{OnEvent, Event};
use prism::canvas::{ShapeType, Image, Align};
use prism::{Context, Request};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Column, Padding, Size, Offset, Stack};

use crate::Theme;
use crate::components::text::{TextStyle, Text, TextSize};
use crate::components::{Icon, Rectangle};

use image::{DynamicImage, GrayImage, RgbaImage};
use std::sync::{Mutex, Arc};

use quircs::Quirc;


/// ## QR Code Scanner
///
/// A camera-based component for scanning and decoding QR codes.
/// Triggers a [`QRCodeScannedEvent`] carrying the data read from the QR code.
///
/// ### Example
/// ```rust
/// let scanner = QRCodeScanner::new(&mut ctx);
/// ```
#[derive(Debug, Component)]
pub struct QRCodeScanner(
    Stack, 
    Option<Image>, 
    QRGuide,
    #[skip] Arc<Mutex<Option<String>>>, 
    #[skip] Arc<Mutex<bool>>
);

impl QRCodeScanner {
    pub fn new(ctx: &mut Context) -> Self {
        QRCodeScanner(
            Stack::center(), 
            None, 
            QRGuide::new(ctx),
            Arc::new(Mutex::new(None)), 
            Arc::new(Mutex::new(false))
        )
    }

    fn find_code(&mut self, img: Arc<RgbaImage>) {
        if *self.4.lock().unwrap() {return;}
        *self.4.lock().unwrap() = true;

        let result_clone = self.3.clone();
        let flag_clone = self.4.clone();

        std::thread::spawn(move || {
            let result = decode_image(Arc::try_unwrap(img).ok().unwrap(), Quirc::default());

            if let Some(r) = result {
                *result_clone.lock().unwrap() = Some(r);
            }

            *flag_clone.lock().unwrap() = false;
        });
    }
}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(CameraEvent::ReceivedFrame(frame)) = event.downcast_ref::<CameraEvent>() {
            if let Some(image) = frame {
                self.find_code(image.clone());
            
                if let Some(data) = &*self.3.lock().unwrap() {
                    ctx.send(Request::Event(Box::new(QRCodeScannedEvent(data.to_string()))));
                }
                
                *self.2.message() = None; 
                *self.2.background() = None;
                self.1 = Some(Image{
                    shape: ShapeType::Rectangle(0.0, (300.0, 300.0), 0.0), 
                    image: image.clone(), 
                    color: None
                });
            } else {
                let background = ctx.state.get_or_default::<Theme>().colors.background.secondary;
                *self.2.background() = Some(Rectangle::new(background, 8.0, None));
                *self.2.message() = Some(Message::new(ctx, "settings", "Camera not available."));
            }
        }
        vec![event]
    }
}

#[derive(Debug, Component)]
struct QRGuide(Stack, Option<Rectangle>, Rectangle, Option<Message>);
impl OnEvent for QRGuide {}

impl QRGuide {
    pub fn new(ctx: &mut Context) -> Self {
        let colors = ctx.state.get_or_default::<Theme>().colors;
        let background = colors.background.secondary;
        let outline = colors.outline.secondary;
        QRGuide(
            Stack(Offset::Center, Offset::Center, Size::Static(308.0), Size::Static(308.0), Padding::default()), 
            Some(Rectangle::new(background, 8.0, None)), 
            Rectangle::new(outline, 8.0, Some((4.0, background))), 
            Some(Message::new(ctx, "camera", "Accessing device camera."))
        )
    }

    pub fn message(&mut self) -> &mut Option<Message> {&mut self.3}
    pub fn background(&mut self) -> &mut Option<Rectangle> {&mut self.1}
}

#[derive(Debug, Component)]
struct Message(Column, Image, Text);
impl OnEvent for Message {}

impl Message {
    pub fn new(ctx: &mut Context, icon: &'static str, msg: &str) -> Self {
        let color = ctx.state.get_or_default::<Theme>().colors.text.heading;

        Message(Column::center(4.0), 
            Icon::new(ctx, icon, Some(color), 48.0),
            Text::new(ctx, msg, TextSize::Sm, TextStyle::Secondary, Align::Left, None)
        )
    }
}

fn decode_image(img_rgba: RgbaImage, mut decoder: Quirc) -> Option<String> {
    let img_gray: GrayImage = DynamicImage::ImageRgba8(img_rgba).to_luma8();

    let codes = decoder.identify(
        img_gray.width() as usize,
        img_gray.height() as usize,
        &img_gray,
    );

    for code in codes {
        match code {
            Ok(c) => match c.decode() {
                Ok(decoded) => {
                    let code = std::str::from_utf8(&decoded.payload).unwrap_or("<invalid utf8>");
                    return Some(code.to_string());
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub struct QRCodeScannedEvent(pub String);

impl Event for QRCodeScannedEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraEvent {
    ReceivedFrame(Option<Arc<RgbaImage>>)
}

impl Event for CameraEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


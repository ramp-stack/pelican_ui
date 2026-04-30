use prism::event::{OnEvent, Event, TickEvent};
use prism::canvas::{ShapeType, Image, Align};
use prism::{Context, Request};
use prism::drawable::{Component, SizedTree};
use prism::layout::{Area, Column, Padding, Size, Offset, Stack};

use ptsd::theme::TextSize;

use crate::theme::{Theme, Icons, Color};
use crate::components::text::{TextStyle, Text};
use crate::components::{Icon, Rectangle};

use image::{DynamicImage, RgbaImage};
use std::sync::{Mutex, Arc};

#[derive(Debug, Component, Clone)]
pub struct QRCodeScanner(
    Stack, 
    Option<Image>, 
    QRGuide,
    #[skip] Arc<Mutex<Option<String>>>, 
    #[skip] Arc<Mutex<bool>>,
    #[skip] Option<String>,
    #[skip] Box<dyn QrCodeFound>,
    #[skip] Option<String>,
);

impl QRCodeScanner {
    pub fn new(theme: &Theme, on_find: Box<dyn QrCodeFound>) -> Self {
        QRCodeScanner(
            Stack::center(), 
            None, 
            QRGuide::new(theme),
            Arc::new(Mutex::new(None)), 
            Arc::new(Mutex::new(false)),
            None,
            on_find,
            None,
        )
    }

    pub fn found(&self) -> Option<String> { self.5.clone() }

    fn find_code(&mut self, img: Arc<RgbaImage>) {
        if *self.4.lock().unwrap() {return;}
        *self.4.lock().unwrap() = true;

        let result_clone = self.3.clone();
        let flag_clone = self.4.clone();

        std::thread::spawn(move || {
            if let Some(r) = decode_image(img) { *result_clone.lock().unwrap() = Some(r); }
            *flag_clone.lock().unwrap() = false;
        });
    }
}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            ctx.start_camera();
            
            if let Some(new_code) = &self.5 {
                if let Some(old_code) = &mut self.7 {
                    if old_code != new_code {
                        (self.6)(ctx, new_code.to_string());
                        *old_code = new_code.to_string();
                        ctx.emit(QRCodeScannedEvent(new_code.to_string()));
                    }
                } else {
                    self.7 = Some(new_code.to_string());
                    (self.6)(ctx, new_code.to_string());
                    ctx.emit(QRCodeScannedEvent(new_code.to_string()));
                }
            }
        }

        // if let Some(HardwareEvent::Camera(image)) = event.downcast_ref::<HardwareEvent>() {
        //     self.find_code(image.clone());
        //     self.5 = self.3.lock().unwrap().clone();
            
        //     *self.2.message() = None; 
        //     *self.2.background() = None;
        //     self.1 = Some(Image{
        //         shape: ShapeType::Rectangle(0.0, (300.0, 300.0), 0.0), 
        //         image: image.clone(), 
        //         color: None
        //     });
        //     //else {
        //         //TODO: fix this 
        //         // let background = ctx.state.get_or_default::<Theme>().colors.background.secondary;
        //         // *self.2.background() = Some(Rectangle::new(background, 8.0, None));
        //         // *self.2.message() = Some(Message::new(ctx, "settings", "Camera not available."));
        //     //}
        // }
        vec![event]
    }
}

#[derive(Debug, Component, Clone)]
struct QRGuide(Stack, Option<Rectangle>, Rectangle, Option<Message>);
impl OnEvent for QRGuide {}

impl QRGuide {
    pub fn new(theme: &Theme) -> Self {
        let background = theme.colors().get(ptsd::Background::Secondary);
        let outline = theme.colors().get(ptsd::Outline::Secondary); 
        QRGuide(
            Stack(Offset::Center, Offset::Center, Size::Static(308.0), Size::Static(308.0), Padding::default()), 
            Some(Rectangle::new(background, 8.0, None)), 
            Rectangle::new(Color::TRANSPARENT, 8.0, Some((4.0, outline))), 
            Some(Message::new(theme, Icons::Camera, "Accessing device camera."))
        )
    }

    pub fn message(&mut self) -> &mut Option<Message> {&mut self.3}
    pub fn background(&mut self) -> &mut Option<Rectangle> {&mut self.1}
}

#[derive(Debug, Component, Clone)]
struct Message(Column, Image, Text);
impl OnEvent for Message {}

impl Message {
    pub fn new(theme: &Theme, icon: Icons, msg: &str) -> Self {
        Message(Column::center(4.0), 
            Icon::new(theme, icon, Some(theme.colors().get(ptsd::Text::Heading)), 48.0),
            Text::new(theme, msg, TextSize::Sm, TextStyle::Secondary, Align::Left, None)
        )
    }
}

// use zxingcpp::BarcodeFormat;
pub fn decode_image(img: Arc<RgbaImage>) -> Option<String> {
    let dyn_img: DynamicImage = DynamicImage::ImageRgba8((*img).clone()); 
    let reader = zxingcpp::read().formats([zxingcpp::BarcodeFormat::QRCode]);
    let barcodes = match reader.from(&dyn_img) {
        Ok(b) => b,
        Err(_) => return None,
    };

    barcodes.into_iter().find(|b| b.format() == zxingcpp::BarcodeFormat::QRCode).and_then(|b| {
        let text = b.text();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    })
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

pub trait QrCodeFound: FnMut(&mut Context, String) + 'static {
    fn clone_box(&self) -> Box<dyn QrCodeFound>;
}

impl PartialEq for dyn QrCodeFound{fn eq(&self, _: &Self) -> bool {true}}

impl<F> QrCodeFound for F where F: FnMut(&mut Context, String) + Clone + 'static {
    fn clone_box(&self) -> Box<dyn QrCodeFound> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn QrCodeFound> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

impl std::fmt::Debug for dyn QrCodeFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QrCodeFound")
    }
}

use roost_ui::events::{OnEvent, Event, TickEvent};
use roost_ui::drawable::{ShapeType, Image, Align};
use roost_ui::{Context, Component};
use roost_ui::layouts::{Column, Padding, Size, Offset, Stack};

use roost_ui::maverick_os::hardware::Camera;

use crate::plugin::PelicanUI;
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
    #[skip] Option<Camera>, 
    #[skip] Arc<Mutex<Option<String>>>, 
    #[skip] Arc<Mutex<bool>>
);

impl QRCodeScanner {
    pub fn new(ctx: &mut Context) -> Self {
        QRCodeScanner(
            Stack::center(), 
            None, 
            QRGuide::new(ctx), 
            Camera::new().ok(), 
            Arc::new(Mutex::new(None)), 
            Arc::new(Mutex::new(false))
        )
    }

    fn find_code(&mut self, img: RgbaImage) {
        if *self.5.lock().unwrap() {return;}
        *self.5.lock().unwrap() = true;

        let result_clone = self.4.clone();
        let flag_clone = self.5.clone();

        std::thread::spawn(move || {
            let result = decode_image(img, Quirc::default());

            if let Some(r) = result {
                *result_clone.lock().unwrap() = Some(r);
            }

            *flag_clone.lock().unwrap() = false;
        });
    }
}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            if let Some(ref mut camera) = self.3 {
                match camera.frame() {
                    Ok(raw_frame) => {
                        self.find_code(raw_frame.clone());
                        
                        if let Some(data) = &*self.4.lock().unwrap() {
                            ctx.trigger_event(QRCodeScannedEvent(data.to_string()));
                        }
                        
                        *self.2.message() = None; 
                        *self.2.background() = None;
                        let image = ctx.assets.add_image(raw_frame);
                        self.1 = Some(Image{
                            shape: ShapeType::Rectangle(0.0, (300.0, 300.0), 0.0), 
                            image, 
                            color: None
                        });
                    },
                    Err(_) => {
                        let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.secondary;
                        *self.2.background() = Some(Rectangle::new(background, 8.0, None));
                        *self.2.message() = Some(Message::new(ctx, "camera", "Waiting for raw camera frame."));
                    }
                }
            } else {
                let background = ctx.get::<PelicanUI>().get().0.theme().colors.background.secondary;
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
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;
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
        let color = ctx.get::<PelicanUI>().get().0.theme().colors.text.heading;

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

#[derive(Debug, Clone)]
pub struct QRCodeScannedEvent(pub String);

impl Event for QRCodeScannedEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


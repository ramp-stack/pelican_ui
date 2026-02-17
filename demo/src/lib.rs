use prism::canvas::{Shape, ShapeType, Color, Align};
use ramp::prism::{self, canvas::{Image, Text as CanvasText}, Context, layout::{Offset, Row, Stack, Column, Size, Padding}, event::{OnEvent, Event, TickEvent}, drawable::{Component, SizedTree}, drawables};

use pelican_ui::{Request, Listener};
use pelican_ui::components::Circle;
use pelican_ui::components::Toggle;
use pelican_ui::components::Slider;
use pelican_ui::components::QRCode;
use pelican_ui::components::Checkbox;
use pelican_ui::components::TextInput;
use pelican_ui::components::RadioSelector;
use pelican_ui::components::NumericalInput;
use pelican_ui::components::avatar::Avatar;
use pelican_ui::components::list_item::ListItem;
use pelican_ui::components::button::SecondaryButton;
use pelican_ui::components::text::{ExpandableText, Text, TextSize, TextStyle, TextEditor};
use pelican_ui::components::button::PrimaryButton;
use pelican_ui::theme::Theme;
use crate::prism::display::{Enum, Opt, EitherOr};

use image::RgbaImage;
use std::sync::Arc;
use pelican_ui::PelicanUI;

use pelican_ui::interface::general::{Interface, Page, Header, Bumper, Content};
use pelican_ui::interface::navigation::{RootInfo, NavigationEvent, AppPage, Flow, FlowContainer};

use pelican_ui::interface::general::Pages;

//     pub fn new(ctx: &mut Context) -> Self {
//         let image: Arc<RgbaImage> = Arc::new(image::open("./seagull.png").unwrap().into());
//         let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
//         let text = ExpandableText::default(ctx, "seagull.png");

//         let image: Arc<RgbaImage> = Arc::new(image::open("./flamingo.png").unwrap().into());
//         let img2 = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 2050.0/6.0), 0.0), image: image.clone(), color: None};
//         let text2 = ExpandableText::default(ctx, "flamingo.png");

//         let avatar = Avatar::default(ctx);
//         let button = SecondaryButton::default(ctx);
//         let checkbox = Checkbox::default(ctx);
//         let listitem = ListItem::default(ctx);
//         let qrcode = QRCode::default(ctx);
//         let radio = RadioSelector::default(ctx);
//         let slider = Slider::default(ctx);
//         let input = TextInput::default(ctx);
//         let toggle = Toggle::default(ctx);
//         let circle = Circle::default();

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct StateTest(String);

#[derive(Debug, Component, Clone)]
pub struct DemoFlow(Stack, Flow);
impl OnEvent for DemoFlow {}
impl FlowContainer for DemoFlow {
    fn flow(&mut self) -> &mut Flow {&mut self.1}
}
impl DemoFlow {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let three = DemoApp3::new(ctx, theme);
        let four = DemoApp4::new(ctx, theme);
        DemoFlow(Stack::default(), Flow::new(vec![Box::new(three), Box::new(four)]))
    }
}

#[derive(Debug, Component, Clone)]
pub struct DemoApp2(Stack, Page);
impl OnEvent for DemoApp2 {}
impl AppPage for DemoApp2 {}
impl DemoApp2 {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./flamingo.png")).unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};

        let img = Listener::new(ctx, theme, img, |ctx: &mut Context, theme: &Theme, img: &mut Image, state: StateTest| {
            let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./{}", state.0.to_string())).unwrap().into());
            *img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        });

        let text = ExpandableText::default(theme, "flamingo.png");
        let content = Content::new(Offset::Start, drawables![img, text], Box::new(|_| true));
        let header = Header::home(theme, "Demo App", None);
        let bumper = Bumper::home(theme, 
            ("Receive".to_string(), Box::new(|ctx: &mut Context, theme: &Theme| {
                let flow = DemoFlow::new(ctx, theme);
                println!("Attempting to navigate flowwwwmwwww");
                ctx.send(Request::event(NavigationEvent::push(flow)));
            })),
            None,
        );
        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}

#[derive(Debug, Component, Clone)]
pub struct DemoApp3(Stack, Page);
impl OnEvent for DemoApp3 {}
impl AppPage for DemoApp3 {}
impl DemoApp3 {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./seagull.png")).unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};

        let img = Listener::new(ctx, theme, img, |ctx: &mut Context, theme: &Theme, img: &mut Image, state: StateTest| {
            let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./{}", state.0.to_string())).unwrap().into());
            *img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        });

        let text = ExpandableText::default(theme, "seagull.png");
        let content = Content::new(Offset::Start, drawables![img, text], Box::new(|_| true));
        let header = Header::stack(theme, "Seagull", None);
        let bumper = Bumper::stack(theme, None, |ctx: &mut Context, theme: &Theme| {
            println!("Pressed");
            ctx.send(Request::event(NavigationEvent::Next));
        }, None);
        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}


#[derive(Debug, Component, Clone)]
pub struct DemoApp4(Stack, Page);
impl OnEvent for DemoApp4 {}
impl AppPage for DemoApp4 {}
impl DemoApp4 {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./turtle.png")).unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};

        let img = Listener::new(ctx, theme, img, |ctx: &mut Context, theme: &Theme, img: &mut Image, state: StateTest| {
            let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./{}", state.0.to_string())).unwrap().into());
            *img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        });

        let text = ExpandableText::default(theme, "turtle.png");
        let content = Content::new(Offset::Start, drawables![img, text], Box::new(|_| true));
        let header = Header::stack(theme, "Turtle", None);
        let bumper = Bumper::stack(theme, None, |ctx: &mut Context, theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }, None);
        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}

ramp::run!{|ctx: &mut Context, assets: Assets| {
    PelicanUI::new(|theme: &Theme| {
        let demo2 = RootInfo::icon("explore", "Demo App 2", Box::new(DemoApp2::new(ctx, theme)));
        Interface::new(theme, vec![demo2], Box::new(|page: &mut Box<dyn Drawable>, ctx: &mut Context, e: Box<dyn Event>| {
            vec![e]
        }))
    })
}}

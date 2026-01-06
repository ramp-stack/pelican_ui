
use prism::canvas::{Shape, ShapeType, Color, Align};
use ramp::prism::{self, canvas::{Image, Text as CanvasText}, Context, layout::{Offset, Row, Stack, Column, Size, Padding}, event::OnEvent, drawable::Component, drawables};

use pelican_ui::components::Circle;
use pelican_ui::components::Toggle;
use pelican_ui::components::Slider;
use pelican_ui::components::QRCode;
use pelican_ui::components::Checkbox;
use pelican_ui::components::TextInput;
use pelican_ui::components::RadioSelector;
use pelican_ui::components::avatar::Avatar;
use pelican_ui::components::list_item::ListItem;
use pelican_ui::components::button::SecondaryButton;
use pelican_ui::components::interface::{AppPage, Interface, Page, Header, Bumper, Content, RootInfo};
use pelican_ui::components::text::{ExpandableText, Text, TextSize, TextStyle, TextEditor};

use crate::prism::display::{Enum, Opt};

use image::RgbaImage;
use std::sync::Arc;
use pelican_ui::PelicanUI;

#[derive(Debug, Component)]
pub struct DemoApp(Stack, Page);
impl OnEvent for DemoApp {}
impl AppPage for DemoApp {}
impl DemoApp {
    pub fn new(ctx: &mut Context) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open("./seagull.png").unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        let text = ExpandableText::default(ctx, "seagull.png");

        let image: Arc<RgbaImage> = Arc::new(image::open("./flamingo.png").unwrap().into());
        let img2 = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 2050.0/6.0), 0.0), image: image.clone(), color: None};
        let text2 = ExpandableText::default(ctx, "flamingo.png");

        let avatar = Avatar::default(ctx);
        let button = SecondaryButton::default(ctx);
        let checkbox = Checkbox::default(ctx);
        let listitem = ListItem::default(ctx);
        let qrcode = QRCode::default(ctx);
        let radio = RadioSelector::default(ctx);
        let slider = Slider::default(ctx);
        let input = TextInput::default(ctx);
        let toggle = Toggle::default(ctx);

        let circle = Circle::default();

        let content = Content::new(Offset::Start, drawables![circle.clone(), circle.clone(), circle.clone(), circle.clone(), circle.clone(), circle.clone(), circle]); //img2, text2, qrcode, radio, slider, input, listitem, toggle, checkbox, button, avatar, img, text]);
        let header = Header::home(ctx, "Demo App", None);
        Self(Stack::default(), Page::new(header, content, Some(Bumper::default(ctx))))
    }
}

#[derive(Debug, Component)]
pub struct DemoApp2(Stack, Page);
impl OnEvent for DemoApp2 {}
impl AppPage for DemoApp2 {}
impl DemoApp2 {
    pub fn new(ctx: &mut Context) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open("./seagull.png").unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        let text = ExpandableText::default(ctx, "seagull.png");

        let image: Arc<RgbaImage> = Arc::new(image::open("./flamingo.png").unwrap().into());
        let img2 = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 2050.0/6.0), 0.0), image: image.clone(), color: None};
        let text2 = ExpandableText::default(ctx, "flamingo.png");

        let avatar = Avatar::default(ctx);
        let button = SecondaryButton::default(ctx);
        let checkbox = Checkbox::default(ctx);
        let listitem = ListItem::default(ctx);
        let qrcode = QRCode::default(ctx);
        let radio = RadioSelector::default(ctx);
        let slider = Slider::default(ctx);
        let input = TextInput::default(ctx);
        let toggle = Toggle::default(ctx);

        let content = Content::new(Offset::Start, drawables![text.clone(), listitem, toggle, checkbox, button, avatar, img, text]);
        let header = Header::home(ctx, "Demo App", None);
        Self(Stack::default(), Page::new(header, content, Some(Bumper::default(ctx))))
    }
}

// #[derive(Debug, Component)]
// pub enum Test {
//     One {stack: Stack, text: CanvasText, circle: Shape, #[skip] img: u8},
//     Image {layout: Stack, img: Image}
// }


#[derive(Debug, Component)]
pub struct Test(Stack, Enum);
impl OnEvent for Test {}
impl Test {
    pub fn new(ctx: &mut Context) -> Self {
        Test(Stack::default(), Enum::new(vec![
            ("circle".to_string(), Box::new(Circle::default())),
            ("text".to_string(), Box::new(Text::default(ctx, "Text Text Text"))),
        ], "text".to_string()))
    }
}



#[derive(Debug, Component)]
pub struct OptTest(Row, Opt<Shape>, Opt<Shape>);
impl OnEvent for OptTest {}
impl OptTest {
    pub fn new(ctx: &mut Context) -> Self {
        OptTest(Row::default(), Opt::new(Circle::default(), true), Opt::new(Circle::default(), false))
    }
}




ramp::run!{|ctx: &mut Context| {
    let demo = RootInfo::icon("home", "Demo App", Box::new(DemoApp::new(ctx)));
    let demo2 = RootInfo::icon("explore", "Demo App 2", Box::new(DemoApp2::new(ctx)));
    Interface::new(ctx, vec![demo2, demo])
    // Text::default(ctx, "test").inner().clone()

    // Circle::default()
    //OneTest{stack: Stack::default(), circle: Circle::default()}
    // Test::new(ctx)
}}

// circle without mini-timers = 31k, 30k, 32k, 29k
// circle with mini-timers = 67k, 83k, 85k, 73k

// text without mini-timers = 63k, 66k, 73k, 61k
// text with mini-timers = 119k, 106k, 108k, 105k

// drawable shape circle = 79,750

// BEFORE: component text = 3,298,750
// AFTER:  component text = 1,855,417
// drawable text = 138,208

// for ever ycomponent we need to keep track of the state objects requeired to build it 

// which means every state object has an ascsoiated changed flag

// we also need to know the difference between state change for the component vs the resize of components around it

// the only time a component will be resized, is if the state for compnent has changed

// the screensize is a state object of the root component 


// we need to know when the state is changed for a particular component

// we need to know when the state changed of other compontes affect this one

// shortterm we could re instantiate and resize very component ever ytick
// half of which we do already.

// in order to reinstantiate every component we. simply need to make it a 
// requirement that the interface only builds 

// first initialization
// re-building of the object based off changed state
// events which have the potentital to change the object and/or state
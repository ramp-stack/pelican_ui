
use prism::canvas::{Shape, ShapeType, Color, Align};
use ramp::prism::{self, canvas::{Image, Text as CanvasText}, Context, layout::{Offset, Row, Stack, Column, Size, Padding}, event::{OnEvent, Event}, drawable::{Component, SizedTree}, drawables};

use pelican_ui::Request;
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
use pelican_ui::components::interface::{AppPage, Interface, Page, Header, Bumper, Content, RootInfo, NavigationEvent};
use pelican_ui::components::text::{ExpandableText, Text, TextSize, TextStyle, TextEditor};
use pelican_ui::components::button::PrimaryButton;

use crate::prism::display::{Enum, Opt, EitherOr};

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

        let content = Content::new(Offset::Start, drawables![circle.clone(), circle.clone(), img2, text2, qrcode, radio, slider, input, listitem, toggle, checkbox, button, avatar, img, text]);
        let header = Header::home(ctx, "Demo App", None);
        Self(Stack::default(), Page::new(header, content, Some(Bumper::default(ctx))))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StateTest(String);

#[derive(Debug, Component)]
pub struct DemoApp2(Stack, Page, #[skip] StateTest);
impl OnEvent for DemoApp2 {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // if event.downcast_ref::<Redraw>
        let state = ctx.state.get::<StateTest>().unwrap();
        if self.2 != *state {
            *self = DemoApp2::new(ctx);
        }
        vec![event]
    }
}
impl AppPage for DemoApp2 {}
impl DemoApp2 {
    pub fn new(ctx: &mut Context) -> Self {
        let path = ctx.state.get::<StateTest>().unwrap().0.to_string();
        let image: Arc<RgbaImage> = Arc::new(image::open(&format!("./{path}")).unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        let text = ExpandableText::default(ctx, &path);
        let content = Content::new(Offset::Start, drawables![img, text]);
        let header = Header::home(ctx, "Demo App", None);
        let bumper = Bumper::home(ctx, 
            ("Receive".to_string(), Box::new(|ctx: &mut Context| {
                if ctx.state.get::<StateTest>().unwrap().0 == "flamingo.png".to_string() {
                    ctx.state.get_mut_or_default::<StateTest>().0 = "seagull.png".to_string();
                } else if ctx.state.get::<StateTest>().unwrap().0 == "seagull.png".to_string() {
                    ctx.state.get_mut_or_default::<StateTest>().0 = "turtle.png".to_string();
                } else {
                    ctx.state.get_mut_or_default::<StateTest>().0 = "flamingo.png".to_string();
                } 
            })),
            None,
            None
        );
        Self(Stack::default(), Page::new(header, content, Some(bumper)), ctx.state.get_or_default::<StateTest>().clone())
    }
}


#[derive(Debug, Component)]
pub struct DemoApp3(Stack, Page);
impl OnEvent for DemoApp3 {}
impl AppPage for DemoApp3 {}
impl DemoApp3 {
    pub fn new(ctx: &mut Context) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open("./seagull.png").unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        let text = ExpandableText::default(ctx, "seagull.png");

        let content = Content::new(Offset::Start, drawables![img, text]);
        let header = Header::stack(ctx, "Seagull", None);
        let bumper = Bumper::stack(ctx, None, |ctx: &mut Context| {
            let page = DemoApp4::new(ctx);
            ctx.send(Request::event(NavigationEvent::push(page)))
        }, None, None);
        Self(Stack::default(), Page::new(header, content, Some(bumper)))
    }
}


#[derive(Debug, Component)]
pub struct DemoApp4(Stack, Page);
impl OnEvent for DemoApp4 {}
impl AppPage for DemoApp4 {}
impl DemoApp4 {
    pub fn new(ctx: &mut Context) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open("./seagull.png").unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        let text = ExpandableText::default(ctx, "seagull.png");

        let content = Content::new(Offset::Start, drawables![img, text]);
        let header = Header::stack_end(ctx, "Seagull received");
        let bumper = Bumper::stack_end(ctx, None);
        Self(Stack::default(), Page::new(header, content, Some(bumper)))
    }
}

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
pub struct OptTest(Row, Opt<Option<Box<dyn Drawable>>>); //EitherOr<PrimaryButton, Option<Box<dyn Drawable>>>);
impl OnEvent for OptTest {}
impl OptTest {
    pub fn new(ctx: &mut Context, item: impl Drawable + 'static) -> Self {
        // let mut eo = EitherOr::new(PrimaryButton::default(ctx), Some(Box::new(item) as Box<dyn Drawable>));
        let mut eo = Opt::new(Some(Box::new(item) as Box<dyn Drawable>), false);
        eo.display(true);
        OptTest(Row::default(), eo)
    }
}

ramp::run!{|ctx: &mut Context| {
    // let b = SecondaryButton::default(ctx);
    // // OptTest::new(ctx, b)
    // let c = Some(b);
    // Some(c)
    ctx.state.insert(StateTest("flamingo.png".to_string()));
    let demo = RootInfo::icon("home", "Demo App", Box::new(DemoApp::new(ctx)));
    let demo2 = RootInfo::icon("explore", "Demo App 2", Box::new(DemoApp2::new(ctx)));
    Interface::new(ctx, vec![demo2, demo], Box::new(|ctx: &mut Context, e: Box<dyn Event>| vec![e]))

    // BumperContent::new(Some(PrimaryButton::default(ctx)), PrimaryButton::default(ctx))
    // OptTest::new(ctx)
}}

// move validity function to the buttons themselves.
// ctx.state.get_mut_or_default::<BitcoinAddress>() = BitcoinAddress::validate(text_input.value())

// let address_page == Page::new(.., |ctx: &mut Context| {
//      ctx.state.get::<BitcoinAddress>().is_some()
// })


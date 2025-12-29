use prism::canvas::{Shape, ShapeType, Color, Align};
use ramp::prism::{self, canvas::Image, Context, layout::{Offset, Stack, Column, Size, Padding}, event::OnEvent, drawable::Component, drawables};
use pelican_ui::PelicanUI;
use pelican_ui::components::TextInput;
use pelican_ui::components::interface::{AppPage, Interface, Page, Header, Bumper, Content, RootInfo};
use pelican_ui::components::text::{ExpandableText, Text, TextSize, TextStyle};
use image::RgbaImage;
use std::sync::Arc;

#[derive(Debug, Component)]
pub struct DemoApp(Stack, Page);
impl OnEvent for DemoApp {}
impl DemoApp {
    pub fn new(ctx: &mut Context) -> Self {
        let image: Arc<RgbaImage> = Arc::new(image::open("./seagull.png").unwrap().into());
        let img = Image{shape: ShapeType::Rectangle(0.0, (1448.0/6.0, 1904.0/6.0), 0.0), image: image.clone(), color: None};
        let text = ExpandableText::default(ctx, "seagull.png");
        let content = Content::new(Offset::Start, drawables![img, text]);
        let header = Header::home(ctx, "Demo App", None);
        Self(Stack::default(), Page::new(header, content, None))
    }
}

ramp::run!{|ctx: &mut Context| DemoApp::new(ctx)}
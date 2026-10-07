use prism::canvas::{self, Align};
use ramp::prism::{self, Context, layout::{Offset, Stack}, event::{OnEvent, Event}, drawable::Component, drawables};
use pelican_ui::{colors};
use pelican_ui::components::QRCode;
use pelican_ui::components::TextInput;
use pelican_ui::components::RadioSelector;
use pelican_ui::components::Icon;
use pelican_ui::components::list_item::{ListItem, ListItemInfoLeft};
use pelican_ui::components::text::{ExpandableText, TextSize, TextStyle};
use pelican_ui::theme::{Theme, Color, Icons};
use pelican_ui::utils::TitleSubtitle;
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use pelican_ui::interface::system::MobileKeyboard;
use pelican_ui::interface::general::{Interface, Page, Header, Bumper, Content};
use pelican_ui::interface::navigation::{NavigatorSelectable, RootInfo, NavigationEvent, AppPage, Flow, FlowContainer};
use pelican_ui::components::list_item::ListItemGroup;
use pelican_ui::components::button::GhostIconButton;

use std::sync::Arc;

#[derive(Debug, Component, Clone)]
pub struct Home(Stack, Page);
impl OnEvent for Home {}
impl AppPage for Home {}
impl Home {
    pub fn new(ctx: &mut Context, theme: &Theme) -> Self {
        let keyboard = MobileKeyboard::new(theme);

        let content = Content::new(Offset::Start, drawables![keyboard], Box::new(|_, _| true));
        let header = Header::home(theme, "My Tickets", None);
        let bumper = Bumper::home(theme, Some(("Buy Ticket".to_string(), Box::new(|_: &mut Context, _: &Theme| {}))), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}

ramp::run!{[], |ctx: &mut Context| {
    let assets = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");
    let theme = Theme::dark(&vec![assets], Color::from_hex("#8efe33", 255));
    let home = RootInfo::icon(Icons::Explore, "My Tickets", Box::new(Home::new(ctx, &theme)));
    Interface::new(ctx, &theme, vec![home], Box::new(|_ctx: &mut Context, e: Box<dyn Event>| {
        vec![e]
    }))
}}

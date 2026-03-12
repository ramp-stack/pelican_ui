use prism::canvas::Align;
use ramp::prism::{self, Context, layout::{Offset, Stack}, event::{OnEvent, Event}, drawable::Component, drawables};
use pelican_ui::{colors, Request};
use pelican_ui::components::QRCode;
use pelican_ui::components::TextInput;
use pelican_ui::components::RadioSelector;
use pelican_ui::components::Icon;
use pelican_ui::components::list_item::{ListItem, ListItemInfoLeft};
use pelican_ui::components::text::{ExpandableText, TextSize, TextStyle};
use pelican_ui::theme::{Theme, Color, Icons};
use pelican_ui::utils::TitleSubtitle;
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use pelican_ui::interface::general::{Interface, Page, Header, Bumper, Content};
use pelican_ui::interface::navigation::{RootInfo, NavigationEvent, AppPage, Flow, FlowContainer};
use pelican_ui::components::list_item::ListItemGroup;

#[derive(Debug, Component, Clone)]
pub struct Home(Stack, Page);
impl OnEvent for Home {}
impl AppPage for Home {}
impl Home {
    pub fn new(_ctx: &mut Context, theme: &Theme) -> Self {
        let tickets = vec![
            Ticket::new("Daniel Vermeer", AgeGroup::Adult, Length::Season),
            Ticket::new("Amanda Vermeer", AgeGroup::Adult, Length::Season),
            Ticket::new("Jimmy Vermeer", AgeGroup::Youth, Length::Season),
            Ticket::new("Annie Vermeer", AgeGroup::Youth, Length::Season),
        ];

        let list = ListItemGroup::new(tickets.into_iter().map(|ticket| 
            ListItem::new(theme, 
                Some(AvatarContent::icon(ticket.age.icon(), AvatarIconStyle::Brand)), 
                ListItemInfoLeft::new(&ticket.name, None, None, None), 
                Some(TitleSubtitle::new(&ticket.age.get(), Some(&ticket.length.get()))),
                None,
                Some(Icons::Forward),
                move |ctx: &mut Context, theme: &Theme| {
                    let flow = ViewTicketFlow::new(theme, ticket.clone());
                    ctx.send(Request::event(NavigationEvent::push(flow)));
                }
            ),
        ).collect::<Vec<_>>());

        let content = Content::new(Offset::Start, drawables![list], Box::new(|_| true));
        let header = Header::home(theme, "My Tickets", None);
        let bumper = Bumper::home(theme, 
            ("Buy Ticket".to_string(), Box::new(|ctx: &mut Context, theme: &Theme| {
                let flow = BuyTicketFlow::new(theme);
                ctx.send(Request::event(NavigationEvent::push(flow)));
            })),
            None,
        );

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}

#[derive(Debug, Clone, Component)]
pub struct BuyTicketFlow(Stack, Flow);
impl OnEvent for BuyTicketFlow {}
impl FlowContainer for BuyTicketFlow {fn flow(&mut self) -> &mut Flow {&mut self.1}}
impl BuyTicketFlow {
    pub fn new(theme: &Theme) -> Self {
        BuyTicketFlow(Stack::default(), Flow::new(vec![
            Box::new(NameOnTicket::new(theme)),
            Box::new(AgeOnTicket::new(theme)),
            Box::new(ExpirationOnTicket::new(theme)),
            Box::new(PurchasedTicket::new(theme))
        ]))
    }
}

#[derive(Debug, Component, Clone)]
pub struct NameOnTicket(Stack, Page);
impl OnEvent for NameOnTicket {}
impl AppPage for NameOnTicket {}
impl NameOnTicket {
    pub fn new(theme: &Theme) -> Self {
        let input = TextInput::new(theme, None, Some("Ticket holder"), Some("Ticket holder's name..."), None, None);
        let content = Content::new(Offset::Start, drawables![input], Box::new(|_| true));
        let header = Header::stack(theme, "Ticket holder", None);
        let bumper = Bumper::stack(theme, None, Box::new(|ctx: &mut Context, _theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}


#[derive(Debug, Component, Clone)]
pub struct AgeOnTicket(Stack, Page);
impl OnEvent for AgeOnTicket {}
impl AppPage for AgeOnTicket {}
impl AgeOnTicket {
    pub fn new(theme: &Theme) -> Self {
        let input = RadioSelector::new(theme, 0, vec![
            ("Youth", "Persons aged 0y - 16y", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Adult", "Persons aged 17y - 65y", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Senior", "Persons 65+", Box::new(|_ctx: &mut Context, _theme: &Theme| {}))
        ]);
        let content = Content::new(Offset::Start, drawables![input], Box::new(|_| true));
        let header = Header::stack(theme, "Holder's age", None);
        let bumper = Bumper::stack(theme, None, Box::new(|ctx: &mut Context, _theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}


#[derive(Debug, Component, Clone)]
pub struct ExpirationOnTicket(Stack, Page);
impl OnEvent for ExpirationOnTicket {}
impl AppPage for ExpirationOnTicket {}
impl ExpirationOnTicket {
    pub fn new(theme: &Theme) -> Self {
        let input = RadioSelector::new(theme, 0, vec![
            ("Day (~04/03/2026)", "Pass valid for the day", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Season (~01/09/2026)", "Pass valid for the entire summer", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Year (~04/03/2027)", "Pass valid for the whole year", Box::new(|_ctx: &mut Context, _theme: &Theme| {}))
        ]);
        let content = Content::new(Offset::Start, drawables![input], Box::new(|_| true));
        let header = Header::stack(theme, "Ticket expiration", None);
        let bumper = Bumper::stack(theme, Some("Complete Purchase"), Box::new(|ctx: &mut Context, _theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}

#[derive(Debug, Component, Clone)]
pub struct PurchasedTicket(Stack, Page);
impl OnEvent for PurchasedTicket {}
impl AppPage for PurchasedTicket {}
impl PurchasedTicket {
    pub fn new(theme: &Theme) -> Self {
        let header = Header::stack_end(theme, "Ticket purchased");
        let bumper = Bumper::stack_end(theme, None);
        let content = Content::new(Offset::Center, drawables![
            Icon::new(theme, Icons::Checkmark, Some(theme.colors().get(colors::Brand)), 128.0),
            ExpandableText::new(theme, "You purchased a ticket", TextSize::H4, TextStyle::Heading, Align::Center, None)
        ], Box::new(|_children| true));

        let page = Page::new(header, content, Some(bumper));
        PurchasedTicket(Stack::default(), page)
    }
}


#[derive(Debug, Clone, Component)]
pub struct ViewTicketFlow(Stack, Flow);
impl OnEvent for ViewTicketFlow {}
impl FlowContainer for ViewTicketFlow {fn flow(&mut self) -> &mut Flow {&mut self.1}}
impl ViewTicketFlow {
    pub fn new(theme: &Theme, ticket: Ticket) -> Self {
        ViewTicketFlow(Stack::default(), Flow::new(vec![
            Box::new(ViewTicket::new(theme, ticket)),
        ]))
    }
}

#[derive(Debug, Component, Clone)]
pub struct ViewTicket(Stack, Page);
impl OnEvent for ViewTicket {}
impl AppPage for ViewTicket {}
impl ViewTicket {
    pub fn new(theme: &Theme, ticket: Ticket) -> Self {
        let header = Header::stack(theme, "View ticket", None);
        let bumper = Bumper::stack(theme, Some("Share"), Box::new(|_ctx: &mut Context, _theme: &Theme| {}), None);
        let content = Content::new(Offset::Center, drawables![
            QRCode::default(theme),
            ExpandableText::new(theme, &format!("Scan {}'s ticket at entry.", ticket.name), TextSize::Sm, TextStyle::Primary, Align::Center, None)
        ], Box::new(|_children| true));

        let page = Page::new(header, content, Some(bumper));
        ViewTicket(Stack::default(), page)
    }
}

ramp::run!{|ctx: &mut Context, assets: Assets| {
    let theme = Theme::dark(assets.all(), Color::from_hex("#8efe33", 255));
    let home = RootInfo::icon(Icons::Explore, "My Tickets", Box::new(Home::new(ctx, &theme)));
    Interface::new(&theme, vec![home], Box::new(|_page: &mut Box<dyn Drawable>, _ctx: &mut Context, e: Box<dyn Event>| {
        vec![e]
    }))
}}

#[derive(Debug, Clone)]
pub struct Ticket {
    name: String,
    age: AgeGroup,
    length: Length,
}

impl Ticket {
    pub fn new(name: &str, age: AgeGroup, length: Length) -> Self {
        Ticket {name: name.to_string(), age, length}
    }
}

#[derive(Debug, Clone)]
pub enum AgeGroup {
    Youth,
    Adult,
    Senior
}

impl AgeGroup {
    pub fn get(&self) -> String {
        match self {
            AgeGroup::Youth => "Youth".to_string(),
            AgeGroup::Adult => "Adult".to_string(),
            AgeGroup::Senior => "Senior".to_string()
        }
    }

    pub fn icon(&self) -> Icons {
        match self {
            AgeGroup::Youth => Icons::Baby,
            AgeGroup::Adult => Icons::Profile,
            AgeGroup::Senior => Icons::Senior
        }
    }
}

#[derive(Debug, Clone)]
pub enum Length {
    Day,
    Season,
    Year,
}

impl Length {
    pub fn get(&self) -> String {
        match self {
            Length::Day => "Day".to_string(),
            Length::Season => "Season".to_string(),
            Length::Year => "Year".to_string()
        }
    }
}

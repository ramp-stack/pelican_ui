use pelican_ui::components::{
    Alert, Checkbox, DataItem, Icon, QRCode, RadioSelector, Rectangle, Slider,
    TextInput, Toggle,
};
use pelican_ui::components::avatar::{Avatar, AvatarContent, AvatarGroup, AvatarIconStyle, AvatarSize};
use pelican_ui::components::button::{
    ActionData, GhostIconButton, PrimaryButton, QuickActions, SecondaryButton,
};
use pelican_ui::components::list_item::{ListItem, ListItemInfoLeft};
use pelican_ui::components::text::{ExpandableText, Text, TextSize, TextStyle};
use pelican_ui::theme::{Color, Icons, Theme};
use prism::drawable::{Component, Drawable, text::Align};
use prism::events::{on_event, ChildEvent, DynEvent, Event, OnEvent, OnChange, on_change, ChangeSource};
use prism::hardware::{Hardware, Picked, Frame, Player, Video};
use prism::{Application, Context};
use ptsd::display::{Bin, Enum, Opt};
use ptsd::emitters::Scrollable;
use ptsd::layout::{Column, Offset, Padding, Row, Size, Stack, Wrap};

#[derive(Clone, Debug)]
enum Action {
    Page(&'static str), Button(&'static str), Slider(f32), Toggle(bool), Checkbox(bool),
    Radio(&'static str), Submit(String), PickVideo, Seek(f32), Preview,
}
impl Event for Action {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Component, Clone)]
struct Demo {
    layout: Stack,
    background: Rectangle,
    shell: Shell,
    #[skip] clicks: usize,
    #[skip] picker: Option<Picked<Video>>,
}
impl Application for Demo {
    fn name() -> String {"Pelican UI · Component Studio".into()}
    fn new(ctx: &mut Context) -> impl Drawable {
        let theme = Theme::dark(&ctx.hardware, &vec![], Color::from_hex("#52d6b5", 255));
        Self {
            layout: Stack::new(Offset::Start, Offset::Start, Size::Fill, Size::Fill, Padding::default()),
            background: Rectangle::new(Color::from_hex("#11161e", 255), 0.0, None),
            shell: Shell::new(&ctx.hardware, &theme), clicks: 0, picker: None,
        }
    }
}
impl Demo {
    fn status(&mut self, message: String) {self.shell.status.0.spans[0].text = message;}
    fn media(&mut self) -> &mut MediaDemo {
        self.shell.pages.1.drawable().inner().downcast_mut::<MediaDemo>().unwrap()
    }
}
impl std::fmt::Debug for Demo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Demo").field("shell", &self.shell).field("clicks", &self.clicks).finish()
    }
}
#[on_event]
impl OnEvent<Action> for Demo {
    fn on(&mut self, ctx: &mut Context, action: Action) -> Vec<Box<dyn DynEvent>> {
        let message = match action {
            Action::Page(name) => {
                if self.shell.pages.1.current() == "Media" {
                    if let Some(player) = &self.media().player.1 {player.stop();}
                }
                self.shell.pages.1.display(name);
                ctx.emit(ptsd::events::Scrollable::ScrollTo(0.0));
                format!("{name} · controls keep their state when you switch pages")
            }
            Action::Button(label) => {
                self.clicks += 1;
                format!("{label} · {} actions handled by the application", self.clicks)
            }
            Action::Slider(value) => format!("Volume · {:.0}%", value * 100.0),
            Action::Toggle(value) => format!("Notifications · {}", if value {"on"} else {"off"}),
            Action::Checkbox(value) => format!("Weekly digest · {}", if value {"subscribed"} else {"unsubscribed"}),
            Action::Radio(value) => format!("Delivery · {value}"),
            Action::Submit(value) => format!("Submitted · {value}"),
            Action::PickVideo => {
                self.picker = Some(ctx.hardware.pick::<Video>());
                "Choose a video to try click-to-play and seeking".into()
            }
            Action::Seek(value) => {
                if let Some(player) = &self.media().player.1 {
                    player.seek(player.duration().mul_f32(value));
                }
                return vec![];
            }
            Action::Preview => {
                let page = self.shell.pages.1.drawable().inner().downcast_mut::<Controls>().unwrap();
                let showing = page.3.is_showing();
                page.3.display(!showing);
                format!("Optional content · {}", if showing {"hidden"} else {"visible"})
            }
        };
        self.status(message);
        vec![]
    }
}
#[on_change(ChangeSource::changed(&mut self.picker))]
impl OnChange<prism::hardware::PickResult<Video>> for Demo {
    fn on_change(&mut self, _ctx: &mut Context, result: prism::hardware::PickResult<Video>) {
        self.picker = None;
        match result {
            Ok(video) => {
                let (width, height) = video.0.dimensions().unwrap();
                self.shell.pages.1.display("Media");
                match Player::new(&video) {
                    Ok(player) => {
                        self.media().player.1 = Some(player);
                        self.status(format!("{width} × {height} · click the video to play or pause"));
                    }
                    Err(error) => self.status(format!("Playback could not start: {error}")),
                }
            }
            Err(_) => self.status("Video selection cancelled".into()),
        }
    }
}

#[derive(Component, Clone, Debug)]
struct Shell {
    layout: Column,
    header: Bin<Row, (Icon, Bin<Column, (Text, ExpandableText)>)>,
    tabs: Bin<Wrap, Vec<SecondaryButton>>,
    status: ExpandableText,
    pages: Scrollable<Enum<Box<dyn Drawable>>>,
}
impl Shell {
    fn new(hardware: &Hardware, theme: &Theme) -> Self {
        let heading = Text::new(theme, "Component Studio", TextSize::H3, TextStyle::Heading, Align::Left, None);
        let subheading = ExpandableText::new(theme, "PELICAN UI  /  Built with PTSD + Prism",
            TextSize::Sm, TextStyle::Secondary, Align::Left, None);
        let header = Bin(Row::start(16.0), (
            Icon::new(theme, Icons::PelicanUI, Some(theme.colors().get(ptsd::Brand)), 44.0),
            Bin(Column::start(4.0), (heading, subheading)),
        ));
        let tabs = [("Controls", Icons::Settings), ("Lists", Icons::Group),
            ("Inputs", Icons::Keyboard), ("Media", Icons::Play)].into_iter().map(|(name, icon)| {
            SecondaryButton::medium(theme, icon, name, None, Stack::default(), move |ctx, _| ctx.emit(Action::Page(name)))
        }).collect();
        let lists = Card::new(theme, "People & activity", "Rich list items, avatars and quick actions", vec![
            Box::new(AvatarGroup::new(hardware, theme, vec![
                AvatarContent::icon(Icons::Profile, AvatarIconStyle::Brand),
                AvatarContent::icon(Icons::Monitor, AvatarIconStyle::Success),
                AvatarContent::icon(Icons::Heart, AvatarIconStyle::Warning),
            ])),
            Box::new(ListItem::new(
                theme, Some(Avatar::new(
                    hardware, theme, AvatarContent::icon(Icons::Profile, AvatarIconStyle::Brand),
                    None, false, AvatarSize::Md, None,
                )),
                ListItemInfoLeft::new(
                    "Alex Rivera", Some("Product designer"), Some("Shared a new collection"), None,
                ), None, None, Some(Icons::Forward), |ctx, _| ctx.emit(Action::Button("Alex Rivera")),
            )),
            Box::new(ListItem::new(
                theme, Some(Avatar::new(
                    hardware, theme, AvatarContent::icon(Icons::Monitor, AvatarIconStyle::Success),
                    None, false, AvatarSize::Md, None,
                )),
                ListItemInfoLeft::new(
                    "Sam Chen", Some("Developer"), Some("Your components are ready to explore"), None,
                ), None, None, Some(Icons::Forward), |ctx, _| ctx.emit(Action::Button("Sam Chen")),
            )),
            Box::new(QuickActions::new(theme, Offset::Start, vec![
                ActionData::new("Save", Some("Saved"), Icons::Heart,
                    Box::new(|ctx, _| ctx.emit(Action::Button("Save")))),
                ActionData::new("Share", None, Icons::Send,
                    Box::new(|ctx, _| ctx.emit(Action::Button("Share")))),
            ])),
            Box::new(DataItem::table(theme, "Project overview", vec![
                ("Components".into(), "Reusable".into()), ("Input".into(), "Mouse + keyboard".into()),
                ("Rendering".into(), "Prism".into()),
            ], None)),
        ]);
        let inputs = Card::new(theme, "Make it yours", "Type, edit and submit — including Unicode", vec![
            Box::new(TextInput::new(theme, uuid::Uuid::new_v4(), None, Some("Display name"), Some("Try your name…"),
                Some("The send button updates the activity line above."), Some((Icons::Send,
                    Box::new(|ctx, value| ctx.emit(Action::Submit(value.clone()))),
                    Box::new(|_: &mut Context, value: &mut String| value.trim().is_empty()))))),
            Box::new(TextInput::new(theme, uuid::Uuid::new_v4(), Some("Hello, world 👋"), Some("A little about you"), None,
                Some("Click to position the cursor; type or backspace to edit."), None)),
            Box::new(Alert::new(theme, "This demo keeps your input in memory.")),
        ]);
        let pages = Enum::new(vec![
            ("Controls".into(), Box::new(Controls::new(theme)) as Box<dyn Drawable>),
            ("Lists".into(), Box::new(lists)), ("Inputs".into(), Box::new(inputs)),
            ("Media".into(), Box::new(MediaDemo::new(theme))),
        ], "Controls".into());
        Self {
            layout: Column::new(20.0, Offset::Start, Size::Fill, Padding::new(28.0), None),
            header, tabs: Bin(Wrap::start(8.0, 8.0), tabs),
            status: ExpandableText::new(theme, "Try a control — its result appears here.",
                TextSize::Sm, TextStyle::Label(theme.colors().get(ptsd::Brand)), Align::Left, Some(2)),
            pages: Scrollable::new(pages),
        }
    }
}

#[derive(Component, Clone, Debug)]
struct Controls(Column, Card, Card, Opt<Alert>);
impl Controls {
    fn new(theme: &Theme) -> Self {
        let buttons = Card::new(theme, "01  /  Actions", "Real controls with hover, press and feedback states", vec![
            Box::new(PrimaryButton::new(theme, "Create something", Stack::default(), |ctx, _| ctx.emit(Action::Button("Create")))),
            Box::new(SecondaryButton::large(theme, "Show / hide a message", Stack::default(), |ctx, _| ctx.emit(Action::Preview))),
            Box::new(Bin(Row::start(8.0), [Icons::Heart, Icons::Copy, Icons::Settings].map(|icon| {
                GhostIconButton::new(
                    theme, icon, Stack::default(), |ctx, _| ctx.emit(Action::Button("Icon action")),
                )
            }))),
        ]);
        let choices = Card::new(theme, "02  /  Preferences", "Sliders, toggles, checkboxes and grouped selection", vec![
            Box::new(Slider::new(theme, 0.5, Some("Volume"), None,
                |ctx, _, value| ctx.emit(Action::Slider(value)))),
            Box::new(Toggle::new(theme, "Notifications", true,
                |ctx, _, value| ctx.emit(Action::Toggle(value)))),
            Box::new(Checkbox::new(theme, "Weekly digest", Some("A short update every Friday".into()), false,
                Box::new(|ctx, _| ctx.emit(Action::Checkbox(true))),
                Box::new(|ctx, _| ctx.emit(Action::Checkbox(false))))),
            Box::new(RadioSelector::new(theme, 0, vec![
                ("Instant", "As soon as it happens", Box::new(|ctx, _| ctx.emit(Action::Radio("Instant")))),
                ("Daily", "One summary each morning", Box::new(|ctx, _| ctx.emit(Action::Radio("Daily")))),
            ])),
        ]);
        Self(Column::start(20.0), buttons, choices,
            Opt::new(Alert::new(theme, "Hidden content keeps its state with PTSD's Opt."), false))
    }
}

#[derive(Component, Clone, Debug)]
struct MediaDemo {
    layout: Column,
    introduction: Card,
    player: Bin<Stack, Option<Player>>,
    seek: Slider,
    time: Text,
    qr: Card,
    #[skip] shown: Option<(u64, u64, bool, Option<String>)>,
}
impl MediaDemo {
    fn new(theme: &Theme) -> Self {
        Self {
            layout: Column::start(16.0),
            introduction: Card::new(theme, "Images & moving pictures", "Parent-sized media with controls composed above Prism", vec![
                Box::new(PrimaryButton::new(
                    theme, "Choose a video", Stack::default(), |ctx, _| ctx.emit(Action::PickVideo),
                )),
            ]),
            player: Bin(Stack::new(Offset::Center, Offset::Center, Size::Fill,
                Size::Static(260.0), Padding::default()), None),
            seek: Slider::new(theme, 0.0, Some("Seek"), Some("Click the picture to play or pause."),
                |ctx, _, value| ctx.emit(Action::Seek(value))),
            time: Text::new(theme, "No video selected", TextSize::Sm, TextStyle::Secondary, Align::Left, None),
            qr: Card::new(theme, "A scannable invitation", "Generated QR code with an embedded brand image", vec![
                Box::new(QRCode::new(theme, "https://github.com/ramp-stack/pelican_ui")),
            ]),
            shown: None,
        }
    }
}
#[on_change(ChangeSource::changed(&mut self.player))]
impl OnChange<Frame> for MediaDemo {
    fn on_change(&mut self, _ctx: &mut Context, event: Frame) {
        *self.seek.value() = if event.duration.is_zero() {0.0} else {
            (event.position.as_secs_f64() / event.duration.as_secs_f64()).clamp(0.0, 1.0) as f32
        };
        let (position, duration) = (event.position.as_secs(), event.duration.as_secs());
        let shown = (position, duration, event.playing, event.error);
        if self.shown.as_ref() != Some(&shown) {
            let status = shown.3.as_deref().unwrap_or(if event.playing {"Playing"} else {"Paused"});
            self.time.spans[0].text = format!("{}:{:02} / {}:{:02}  ·  {}", position / 60, position % 60,
                duration / 60, duration % 60, status);
            self.shown = Some(shown);
        }
    }
}

#[derive(Component, Clone, Debug)]
struct Card(Stack, Rectangle, Bin<Column, (Text, ExpandableText, Vec<Box<dyn Drawable>>)>);
impl Card {
    fn new(theme: &Theme, title: &str, description: &str, children: Vec<Box<dyn Drawable>>) -> Self {
        let layout = Stack::new(Offset::Start, Offset::Start, Size::Fill,
            Size::custom(|sizes| sizes[1]), Padding::default());
        let body = Bin(Column::new(16.0, Offset::Start, Size::Fill, Padding::new(16.0), None), (
            Text::new(theme, title, TextSize::H5, TextStyle::Heading, Align::Left, None),
            ExpandableText::new(theme, description, TextSize::Sm, TextStyle::Secondary, Align::Left, None),
            children,
        ));
        Self(layout, Rectangle::new(Color::from_hex("#1a222d", 255), 16.0,
            Some((1.0, Color::from_hex("#2a3644", 255)))), body)
    }
}

ramp::run!(Demo);

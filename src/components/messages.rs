use prism::event::OnEvent;
use prism::layout::{Stack, Column, Row, Offset, Size, Padding};
use prism::drawable::Component;
use prism::canvas::Align;
use prism::display::Bin;
use ptsd::colors;
use ptsd::utils::Timestamp;

use chrono::{Local, Duration};

use std::sync::Arc;
use image::RgbaImage;

use crate::Theme;
use crate::components::Rectangle;

use crate::components::avatar::{AvatarSize, AvatarContent, AvatarIconStyle, Avatar};
use crate::components::text::{Text, ExpandableText, TextSize, TextStyle};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub pfp: Option<Arc<RgbaImage>>,
}

impl Profile {
    pub fn new(name: &str, _pfp: &str) -> Self {
        Profile {
            name: name.to_string(),
            pfp: None, //Some(Arc::new(image::open(&format!("./{}", pfp.to_string())).unwrap().into()))
        }
    }

    pub fn tests() -> Vec<Profile> {
        vec![
            Profile::daniel(),
        ]
    }

    pub fn more_tests() -> Vec<Profile> {
        vec![
            Profile::daniel(),
            Profile::sofia(),
            Profile::marcus(),
            Profile::chloe(),
            Profile::david(),
            Profile::ethan(),
        ]
    }

    pub fn avatar(&self) -> AvatarContent {
        match &self.pfp {
            Some(img) => AvatarContent::image(img.clone()),
            None => AvatarContent::icon("profile", AvatarIconStyle::Secondary)
        }
    }

    pub fn sofia() -> Profile { Profile::new("Sofia Martinez", "cat.jpeg") }
    pub fn marcus() -> Profile { Profile::new("Marcus Johansson", "deer.jpeg") }
    pub fn chloe() -> Profile { Profile::new("Chloe Bennett", "bird.jpeg") }
    pub fn ethan() -> Profile { Profile::new("Ethan Clarke", "finland.jpeg") }
    pub fn daniel() -> Profile { Profile::new("Daniel Vermeer", "flamingo.png") }
    pub fn david() -> Profile { Profile::new("David Vermeer", "flomango.png") }
    pub fn me() -> Profile { Profile::new("Flamingo", "flamingo.png") }

    pub fn is_me(&self) -> bool {*self == Self::me()}
}



#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Message {
    pub message: String,
    pub timestamp: Timestamp,
    pub author: Profile,
}

impl Message {
    pub fn tests() -> Vec<Message> {
        vec![
            Message::new("Hey, I just found a little dog outside that looks hurt.", Profile::daniel()),
            Message::new("Oh no, is it okay?", Profile::me()),
            Message::new("What happened?", Profile::me()),
            Message::new("I’m not sure, it’s limping and seems really scared.", Profile::daniel()),
            Message::new("Can you see if it has a collar? Maybe we should call a vet.", Profile::me()),
            Message::new("Yeah, I’ll call the nearest animal clinic and stay with it until they arrive.", Profile::daniel()),
        ]
    }

    pub fn new(message: &str, author: Profile) -> Self {
        let yesterday = Local::now() - Duration::days(1);
        Message {
            message: message.to_string(),
            timestamp: Timestamp::new(yesterday),
            author,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageGroups(Column, Vec<MessageGroup>);
impl OnEvent for MessageGroups {}
impl MessageGroups {
    pub fn new(theme: &Theme, messages: Vec<Message>, members: Vec<Profile>, is_room: bool) -> Self {
        if messages.is_empty() || members.is_empty() { return MessageGroups(Column::center(0.0), vec![]); }

        let mut new: Vec<MessageGroup> = vec![];
        let mut collection: Vec<Message> = vec![];
        let mut prev = messages[0].clone();
        let mut prev_auth = prev.author.clone();

        for message in messages {
            let timestamp = message.timestamp.clone();
            let author = message.author.clone();
            let close_enough = prev.timestamp.as_local().signed_duration_since(timestamp.as_local()).num_seconds().abs() <= 60;

            if !collection.is_empty() && (!close_enough || author != prev_auth) {
                let a = collection[0].author.clone();
                let t = collection.last().unwrap().timestamp.clone();
                let msgs = collection.iter().map(|m| m.message.as_str()).collect::<Vec<_>>();
                let direction = if a.is_me() { Direction::Sent } else { Direction::Received };

                let room_type = match is_room {
                    true => Room::Room,
                    false if members.len() > 2 => Room::Group(direction),
                    false => Room::Direct(direction),
                };

                // println!("ROOM TYPE {:?} because {:?}", room_type, members.len());
                new.push(MessageGroup::new(theme, msgs, t, a, room_type));

                collection.clear();
            }

            collection.push(message.clone());

            prev = message;
            prev_auth = prev.author.clone();
        }

        if !collection.is_empty() {
            let a = collection[0].author.clone();
            let t = collection.last().unwrap().timestamp.clone();
            let msgs = collection.iter().map(|m| m.message.as_str()).collect::<Vec<_>>();
            let direction = if a.is_me() { Direction::Sent } else { Direction::Received };

            new.push(MessageGroup::new(theme, msgs, t, a, match is_room {
                true => Room::Room,
                false if members.len() > 2 => Room::Group(direction),
                false => Room::Direct(direction),
            }));
        }

        MessageGroups(Column::center(24.0), new)
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageGroup(Row, Option<Bin<Stack, Avatar>>, _MessageGroup);
impl OnEvent for MessageGroup {}
impl MessageGroup {
    pub fn new(theme: &Theme, messages: Vec<&str>, timestamp: Timestamp, profile: Profile, room: Room) -> Self {
        let avatar = Avatar::new(theme, match profile.pfp {
                Some(ref pfp) => AvatarContent::image(pfp.clone()),
                None => AvatarContent::icon("profile", AvatarIconStyle::Secondary),
            }, None, false, AvatarSize::Xs, None
        );

        let (layout, avatar) = match room {
            Room::Room => (Row::start(12.0), Some(Bin(Stack::default(), avatar))),
            Room::Group(Direction::Received) => {
                let layout = Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding(0.0, 0.0, 0.0, 18.0));
                (Row::end(8.0), Some(Bin(layout, avatar)))
            },
            Room::Direct(Direction::Received)| 
            Room::Direct(Direction::Sent) | 
            Room::Group(Direction::Sent) => (Row::default(), None),
        };

        MessageGroup(layout, avatar, _MessageGroup::new(theme, messages, timestamp, profile, room))
    }
}

#[derive(Debug, Clone, Component)]
pub enum _MessageGroup {
    Group {layout: Column, msg: _TextMessages, info: MessageInfo},
    Direct {layout: Column, msg: _TextMessages, info: MessageInfo},
    Room {layout: Column, info: MessageInfo, msg: _TextMessages},
}

impl OnEvent for _MessageGroup {}
impl _MessageGroup {
    pub fn new(theme: &Theme, messages: Vec<&str>, timestamp: Timestamp, profile: Profile, room: Room) -> Self {
        let info = MessageInfo::new(theme, profile.name, timestamp, room);
        let msg = _TextMessages::new(theme, messages, room);
        match room {
            Room::Room => _MessageGroup::Room {layout: Column::start(8.0), msg, info},
            Room::Direct(Direction::Sent) => _MessageGroup::Direct {layout: Column::end(8.0), msg, info},
            Room::Direct(Direction::Received) => _MessageGroup::Direct {layout: Column::start(8.0), msg, info},
            Room::Group(Direction::Sent) => _MessageGroup::Group {layout: Column::end(8.0), msg, info},
            Room::Group(Direction::Received) => _MessageGroup::Group {layout: Column::start(8.0), msg, info},
        }
    }

    pub fn messages(&mut self) -> &mut _TextMessages {
        match self {
            _MessageGroup::Room{msg, ..} |
            _MessageGroup::Direct{msg, ..} |
            _MessageGroup::Group{msg, ..} => msg,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageInfo(Row, Option<Text>, Option<Text>, Text);
impl OnEvent for MessageInfo {}
impl MessageInfo {
    pub fn new(theme: &Theme, name: String, timestamp: Timestamp, room: Room) -> Self {
        let name = match room {
            Room::Room => Some(Text::new(theme, &name, TextSize::H5, TextStyle::Heading, Align::Left, None)),
            Room::Group(Direction::Received) => Some(Text::new(theme, &name, TextSize::Sm, TextStyle::Secondary, Align::Left, None)),
            Room::Group(Direction::Sent) => Some(Text::new(theme, "You", TextSize::Sm, TextStyle::Secondary, Align::Left, None)),
            Room::Direct(_) => None,
        };

        let timestamp = Text::new(theme, &timestamp.friendly(), TextSize::Sm, TextStyle::Secondary, Align::Left, None);
        let divider = name.is_some().then_some(Text::new(theme, "·", TextSize::Sm, TextStyle::Secondary, Align::Left, None));

        MessageInfo(Row::center(4.0), name, divider, timestamp)
    }
}

#[derive(Debug, Clone, Component)]
pub struct _TextMessages(Column, Vec<_TextMessage>);
impl OnEvent for _TextMessages {}
impl _TextMessages {
    pub fn new(theme: &Theme, messages: Vec<&str>, room: Room) -> Self {
        let layout = match room {
            Room::Group(Direction::Sent) | Room::Direct(Direction::Sent) => Column::end(8.0),
            Room::Group(Direction::Received) | Room::Direct(Direction::Received) => Column::start(8.0),
            Room::Room => Column::start(0.0),
        };

        _TextMessages(layout, messages.iter().map(|msg| _TextMessage::new(theme, msg, room)).collect::<Vec<_>>())
    }
}

#[derive(Debug, Clone, Component)]
pub struct _TextMessage(Stack, Option<Rectangle>, Bin<Stack, ExpandableText>);
impl OnEvent for _TextMessage {}
impl _TextMessage {
    pub fn new(theme: &Theme, message: &str, style: Room) -> Self {
        let text = ExpandableText::new(theme, message, TextSize::Md, TextStyle::Primary, Align::Left, None);
        let background = match style {
            Room::Group(Direction::Sent) | Room::Direct(Direction::Sent) => Some(theme.colors().get(colors::Brand)),
            Room::Group(Direction::Received) | Room::Direct(Direction::Received) => Some(theme.colors().get(colors::Background::Secondary)),
            Room::Room => None,
        };

        let padding = if background.is_some() {12.0} else {0.0};
        let width = Size::custom(move |h: Vec<(f32, f32)>| (h[1].0.max(32.0), h[1].1.max(32.0)));
        let height = Size::custom(move |h: Vec<(f32, f32)>| (h[1].0, h[1].1));
        let layout = Stack::new(Offset::Center, Offset::Center, width, height, Padding::default());
        let width = Size::custom(|w| (w[0].0, w[0].1.min(250.0)));
        let bin = Stack::new(Offset::Start, Offset::Start, width, Size::Fit, Padding::new(padding));
        _TextMessage(layout, background.map(|c| Rectangle::new(c, 16.0, None)), Bin(bin, text))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    Received,
    Sent,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Room {
    Group(Direction),
    Direct(Direction),
    Room,
}
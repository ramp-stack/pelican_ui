#![allow(clippy::large_enum_variant)]
use prism::air::Name;
use prism::events::Event;
use prism::drawable::{Component, Drawable};
use prism::drawable::text::{Align, Font};
use prism::Context;
use ptsd::layout::{Stack, Column, Row, Offset, Size, Padding};
use ptsd::display::Bin;
use ptsd::utils::Timestamp;
use ptsd::{colors, interactions};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::Callback;
use crate::theme::Theme;
use crate::components::{
    Rectangle, Sticker, StickerView, Gif, GifPlayer, Photo, PhotoView, Video, VideoView, NewVideo,
};
use crate::components::avatar::{AvatarSize, AvatarContent, Avatar};
use crate::components::text::{Text, TextSize, TextStyle};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Direction {Received, Sent}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Room {Group(Direction), Direct(Direction), Room}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum MessageContent {Text(String), Sticker(Sticker), Gif(Gif), Photo(Photo), Video(Video), System(SystemMessage)}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum SystemMessage {MemberAdded(Name), MemberLeft(Name)}
impl SystemMessage {
    pub fn member(&self) -> Name {
        match self {Self::MemberAdded(name) | Self::MemberLeft(name) => *name}
    }
    pub fn text(&self, username: &str) -> String {
        match self {
            Self::MemberAdded(_) => format!("{username} was added to the group"),
            Self::MemberLeft(_) => format!("{username} left the group"),
        }
    }
    pub fn text_for(&self, viewer: Name, username: &str) -> String {
        if self.member() != viewer {return self.text(username);}
        match self {
            Self::MemberAdded(_) => "You were added to the group".into(),
            Self::MemberLeft(_) => "You left the group".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SendEvent {Sticker(Sticker), Gif(Gif), Photo(Photo), Video(NewVideo)}

impl Event for SendEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [prism::events::ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub username: String,
    pub name: Name,
    pub pfp: AvatarContent,
}

impl Profile {
    pub fn new(username: &str, pfp: &str, name: Name) -> Self {
        Profile {username: username.to_string(), name, pfp: AvatarContent::from_string(pfp)}
    }

    pub fn is_me(&self, ctx: &mut Context) -> bool {self.name == ctx.air.me()}
}

#[derive(Clone, Debug)]
pub struct Message {
    pub body: MessageContent,
    pub timestamp: Timestamp,
    pub author: Profile,
    pub on_click: Box<dyn Callback>,
}

#[derive(Debug, Clone, Component)]
pub struct MessageGroups(Column, Vec<Box<dyn Drawable>>);
impl MessageGroups {
    pub fn new(ctx: &mut Context, theme: &Theme, messages: Vec<Message>, is_group: bool, is_room: bool) -> Self {
        if messages.is_empty() { return MessageGroups(Column::center(0.0), vec![]); }

        let mut new: Vec<Box<dyn Drawable>> = vec![];
        let mut collection: Vec<Message> = vec![];
        let me = ctx.air.me();
        let flush = |collection: &mut Vec<Message>, new: &mut Vec<Box<dyn Drawable>>| {
            if !collection.is_empty() {
                let a = collection[0].author.clone();
                let t = collection.last().unwrap().timestamp.clone();
                let direction = if a.name == me { Direction::Sent } else { Direction::Received };
                let room_type = match is_room {
                    true => Room::Room,
                    false if is_group => Room::Group(direction),
                    false => Room::Direct(direction),
                };
                new.push(Box::new(MessageGroup::new(theme, std::mem::take(collection), t, a, room_type)));
            }
        };
        for message in messages {
            if let MessageContent::System(notice) = &message.body {
                flush(&mut collection, &mut new);
                // System entries are plain, noninteractive text, separate from
                // sender groups. CHK supplies the affected member's profile.
                new.push(Box::new(SystemMessageView::new(theme, &notice.text_for(me, &message.author.username))));
                continue;
            }
            if collection.last().is_some_and(|prev| {
                prev.author != message.author || prev.timestamp.as_local().unwrap()
                    .signed_duration_since(message.timestamp.as_local().unwrap()).num_seconds().abs() > 60
            }) {flush(&mut collection, &mut new);}
            collection.push(message);
        }
        flush(&mut collection, &mut new);

        MessageGroups(Column::center(24.0), new)
    }
}

#[derive(Debug, Clone, Component)]
pub struct SystemMessageView(Column, Text);
impl SystemMessageView {
    pub fn new(theme: &Theme, text: &str) -> Self {
        Self(Column::center(0.0), Text::new(theme, text, TextSize::Sm, TextStyle::Secondary, Align::Center, None))
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageGroup(Row, Option<Bin<Stack, Avatar>>, _MessageGroup);
impl MessageGroup {
    pub fn new(theme: &Theme, messages: Vec<Message>, timestamp: Timestamp, profile: Profile, room: Room) -> Self {
        let avatar = Avatar::new(theme, profile.pfp.clone(), None, AvatarSize::Xs, None);

        let (layout, avatar) = match room {
            Room::Room => (Row::start(12.0), Some(Bin(Stack::default(), avatar))),
            Room::Direct(Direction::Received) | Room::Direct(Direction::Sent)
                | Room::Group(_) => (Row::default(), None),
        };

        MessageGroup(
            layout, avatar, _MessageGroup::new(theme, messages, timestamp, profile, room),
        )
    }
}

#[derive(Debug, Clone, Component)]
pub enum _MessageGroup {
    Group {layout: Column, msg: _TextMessages, info: MessageInfo},
    ReceivedGroup {layout: Column, msg: _TextMessages, info: Bin<Row, (Avatar, MessageInfo)>},
    Direct {layout: Column, msg: _TextMessages, info: MessageInfo},
    Room {layout: Column, info: MessageInfo, msg: _TextMessages},
}

impl _MessageGroup {
    pub fn new(theme: &Theme, messages: Vec<Message>, timestamp: Timestamp, profile: Profile, room: Room) -> Self {
        let photo = profile.pfp.clone();
        let info = MessageInfo::new(theme, profile.username, timestamp, room);
        let msg = _TextMessages::new(theme, messages, room);
        match room {
            Room::Room => _MessageGroup::Room {layout: Column::start(8.0), msg, info},
            Room::Direct(Direction::Sent) => _MessageGroup::Direct {layout: Column::end(8.0), msg, info},
            Room::Direct(Direction::Received) => {
                _MessageGroup::Direct {layout: Column::start(8.0), msg, info}
            }
            Room::Group(Direction::Sent) => _MessageGroup::Group {layout: Column::end(8.0), msg, info},
            Room::Group(Direction::Received) => {
                let avatar = Avatar::new(theme, photo, None, AvatarSize::Xs, None);
                _MessageGroup::ReceivedGroup {
                    layout: Column::start(8.0), msg,
                    info: Bin(Row::center(8.0), (avatar, info)),
                }
            },
        }
    }

    pub fn messages(&mut self) -> &mut _TextMessages {
        match self {
            _MessageGroup::Room{msg, ..} | _MessageGroup::Direct{msg, ..}
                | _MessageGroup::Group{msg, ..} | _MessageGroup::ReceivedGroup{msg, ..} => msg,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct MessageInfo(Row, Option<Text>, Option<Text>, Text);
impl MessageInfo {
    pub fn new(theme: &Theme, name: String, timestamp: Timestamp, room: Room) -> Self {
        let name = match room {
            Room::Room => Some(Text::new(
                theme, &name, TextSize::H5, TextStyle::Heading, Align::Left, None,
            ).fit()),
            Room::Group(Direction::Received) => Some(Text::new(
                theme, &name, TextSize::Sm, TextStyle::Secondary, Align::Left, None,
            ).fit()),
            Room::Group(Direction::Sent) => Some(Text::new(
                theme, "You", TextSize::Sm, TextStyle::Secondary, Align::Left, None,
            ).fit()),
            Room::Direct(_) => None,
        };

        let timestamp = Text::new(
            theme, &timestamp.precise(), TextSize::Sm, TextStyle::Secondary, Align::Left, None,
        ).fit();
        let divider = name.is_some().then_some(Text::new(
            theme, "·", TextSize::Sm, TextStyle::Secondary, Align::Left, None,
        ).fit());

        MessageInfo(Row::center(4.0), name, divider, timestamp)
    }
}

#[derive(Debug, Clone, Component)]
pub struct _TextMessages(Column, Vec<interactions::Button>);
impl _TextMessages {
    pub fn new(theme: &Theme, messages: Vec<Message>, room: Room) -> Self {
        let layout = match room {
            Room::Group(Direction::Sent) | Room::Direct(Direction::Sent) => Column::end(8.0),
            Room::Group(Direction::Received) => {
                let mut layout = Column::start(8.0);
                layout.padding().0 = AvatarSize::Xs.get() + 8.0;
                layout
            },
            Room::Direct(Direction::Received) => Column::start(8.0),
            Room::Room => Column::start(0.0),
        };

        let bubbles = messages.into_iter().map(|message| {
            let bubble = _TextMessage::new(theme, &message.body, room);
            let theme = theme.clone();
            let mut on_click = message.on_click;
            let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
            let mut button = interactions::Button::new(
                bubble, None::<_TextMessage>, None::<_TextMessage>, None::<_TextMessage>,
                None::<_TextMessage>, callback, false,
            );
            button.set_trigger_on_release();
            button
        }).collect::<Vec<_>>();

        _TextMessages(layout, bubbles)
    }
}

#[derive(Debug, Clone, Component)]
pub enum _TextMessage {
    Text {
        layout: Stack,
        background: Option<Rectangle>,
        text: Bin<Stack, Text>
    },
    Emoji {layout: Stack, text: Text},
    Sticker { layout: Stack, sticker: StickerView },
    Gif { layout: Stack, gif: GifPlayer },
    Photo { layout: Stack, photo: PhotoView },
    Video { layout: Stack, video: VideoView },
    System {layout: Stack, text: SystemMessageView},
}

impl _TextMessage {
    pub fn new(theme: &Theme, message: &MessageContent, style: Room) -> Self {
        match message {
            MessageContent::Text(message) => {
                if message.graphemes(true).count() == 1 && Font::is_emoji_grapheme(message) {
                    let text = Text::new(
                        theme, message, TextSize::H1, TextStyle::Primary, Align::Left, None,
                    ).fit();
                    _TextMessage::Emoji{layout: Stack::center(), text}
                } else {
                    let text = Text::new(
                        theme, message, TextSize::Md, TextStyle::Primary, Align::Left, None,
                    ).fit();
                    let background = match style {
                        Room::Group(Direction::Sent) | Room::Direct(Direction::Sent) => {
                            Some(theme.colors().get(colors::Brand))
                        }
                        Room::Group(Direction::Received) | Room::Direct(Direction::Received) => {
                            Some(theme.colors().get(colors::Background::Secondary))
                        }
                        Room::Room => None,
                    };

                    let width = Size::custom(move |h: Vec<(f32, f32)>| {
                        (h.last().unwrap().0, (h.last().unwrap().1 + 24.0).max(42.0))
                    });
                    let height = Size::custom(move |h: Vec<(f32, f32)>| {
                        (h.last().unwrap().0, h.last().unwrap().1)
                    });
                    let layout = Stack::new(
                        Offset::Center, Offset::Center, width, height, Padding::default(),
                    );
                    let width = Size::custom(|widths| {
                        let width = widths[0].1.min(250.0) + 4.0;
                        (width, width)
                    });
                    let text = Bin(Stack::new(
                        Offset::Center, Offset::Center, width, Size::Fit, Padding(0.0, 8.0, 0.0, 8.0),
                    ), text);
                    _TextMessage::Text {
                        layout, background: background.map(|c| Rectangle::new(c, 18.0, None)), text,
                    }
                }
            },
            MessageContent::Sticker(sticker) => _TextMessage::Sticker {
                layout: Stack::default(), sticker: StickerView::message(sticker),
            },
            MessageContent::Gif(gif) => _TextMessage::Gif {
                layout: Stack::default(), gif: GifPlayer::message(theme, gif),
            },
            MessageContent::Photo(photo) => _TextMessage::Photo {
                layout: Stack::default(), photo: PhotoView::message(photo),
            },
            MessageContent::Video(video) => _TextMessage::Video {
                layout: Stack::default(), video: VideoView::message(theme, video),
            },
            MessageContent::System(notice) => _TextMessage::System {
                layout: Stack::default(), text: SystemMessageView::new(theme, &notice.text(&notice.member().to_string())),
            },
        }
    }
}

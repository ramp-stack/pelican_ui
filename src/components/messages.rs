use prism::event::{OnEvent, Event, TickEvent};
use prism::layout::{Stack, Column, Row, Offset, Size, Padding};
use prism::drawable::{Component, SizedTree};
use prism::canvas::{Align, Image, ShapeType, is_emoji_grapheme};
use prism::display::Bin;
use ptsd::colors;
use ptsd::utils::Timestamp;
use prism::Context;
use serde::{Deserialize, Serialize};

use chrono::{Local, Duration};

use std::sync::Arc;
use image::RgbaImage;

use crate::theme::{Theme, Icons};
use crate::components::Rectangle;
use crate::components::{Sticker, Gif, GifPlayer, Photo, PhotoView, Video, VideoView};

use ptsd::interactions;
use crate::Callback;

use maverick_os::air::Name;

use crate::components::avatar::{AvatarSize, AvatarContent, AvatarIconStyle, Avatar};
use crate::components::text::{Text, ExpandableText, TextSize, TextStyle};

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub username: String,
    pub name: Name,
    pub pfp: AvatarContent,
}

impl Profile {
    pub fn new(username: &str, _pfp: &str, name: Name) -> Self {
        Profile {
            username: username.to_string(),
            name,
            pfp: AvatarContent::default(), //Some(Arc::new(image::open(&format!("./{}", pfp.to_string())).unwrap().into()))
        }
    }

    pub fn is_me(&self, ctx: &mut Context) -> bool {self.name == ctx.me()}
}

#[derive(Clone, Debug)]
pub struct Message {
    pub body: MessageContent,
    pub timestamp: Timestamp,
    pub author: Profile,
    pub on_click: Box<dyn Callback>,
}

#[derive(Debug, Clone, Component)]
pub struct MessageGroups(Column, Vec<MessageGroup>);
impl OnEvent for MessageGroups {}
impl MessageGroups {
    pub fn new(ctx: &mut Context, theme: &Theme, messages: Vec<Message>, is_group: bool, is_room: bool) -> Self {
        if messages.is_empty() { return MessageGroups(Column::center(0.0), vec![]); }

        let mut new: Vec<MessageGroup> = vec![];
        let mut collection: Vec<Message> = vec![];
        let mut prev = messages[0].clone();
        let mut prev_auth = prev.author.clone();

        for message in messages {
            let timestamp = message.timestamp.clone();
            let author = message.author.clone();
            let close_enough = prev.timestamp.as_local().unwrap().signed_duration_since(timestamp.as_local().unwrap()).num_seconds().abs() <= 60;

            if !collection.is_empty() && (!close_enough || author != prev_auth) {
                let a = collection[0].author.clone();
                let t = collection.last().unwrap().timestamp.clone();
                let direction = if a.is_me(ctx) { Direction::Sent } else { Direction::Received };

                let room_type = match is_room {
                    true => Room::Room,
                    false if is_group => Room::Group(direction),
                    false => Room::Direct(direction),
                };

                // println!("ROOM TYPE {:?} because {:?}", room_type, members.len());
                new.push(MessageGroup::new(theme, collection.clone(), t, a, room_type));

                collection.clear();
            }

            collection.push(message.clone());

            prev = message;
            prev_auth = prev.author.clone();
        }

        if !collection.is_empty() {
            let a = collection[0].author.clone();
            let t = collection.last().unwrap().timestamp.clone();
            let direction = if a.is_me(ctx) { Direction::Sent } else { Direction::Received };

            new.push(MessageGroup::new(theme, collection.clone(), t, a, match is_room {
                true => Room::Room,
                false if is_group => Room::Group(direction),
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
    pub fn new(theme: &Theme, messages: Vec<Message>, timestamp: Timestamp, profile: Profile, room: Room) -> Self {
        let avatar = Avatar::new(theme, profile.pfp.clone(), None, false, AvatarSize::Xs, None);

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
    pub fn new(theme: &Theme, messages: Vec<Message>, timestamp: Timestamp, profile: Profile, room: Room) -> Self {
        let info = MessageInfo::new(theme, profile.username, timestamp, room);
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

        let timestamp = Text::new(theme, &timestamp.precise(), TextSize::Sm, TextStyle::Secondary, Align::Left, None);
        let divider = name.is_some().then_some(Text::new(theme, "·", TextSize::Sm, TextStyle::Secondary, Align::Left, None));

        MessageInfo(Row::center(4.0), name, divider, timestamp)
    }
}

#[derive(Debug, Clone, Component)]
pub struct _TextMessages(Column, Vec<interactions::Button>);
impl OnEvent for _TextMessages {}
impl _TextMessages {
    pub fn new(theme: &Theme, messages: Vec<Message>, room: Room) -> Self {
        let layout = match room {
            Room::Group(Direction::Sent) | Room::Direct(Direction::Sent) => Column::end(8.0),
            Room::Group(Direction::Received) | Room::Direct(Direction::Received) => Column::start(8.0),
            Room::Room => Column::start(0.0),
        };

        let bubbles = messages.into_iter().map(|message| {
            let bubble = _TextMessage::new(theme, &message.body, room);
            let theme = theme.clone();
            let mut on_click = message.on_click;
            let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
            let mut button = interactions::Button::new(bubble, None::<_TextMessage>, None::<_TextMessage>, None::<_TextMessage>, None::<_TextMessage>, callback, false);
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
        text: Bin<Stack, ExpandableText>
    },
    Emoji {layout: Stack, text: Text},
    Sticker {
        layout: Stack,
        image: Image,
    },
    Gif {
        layout: Stack,
        gif: GifPlayer,
    },
    Photo { layout: Stack, photo: PhotoView },
    Video { layout: Stack, video: VideoView }
}

impl OnEvent for _TextMessage {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            match self {
                _TextMessage::Text{text, ..} => {
                    let w = text.inner().0.inner().size().0.min(250.0) + 4.0;
                    *text.get_layout() = Stack::new(Offset::Center, Offset::Center, Size::Static(w), Size::Fit, Padding(0.0, 8.0, 0.0, 8.0));
                },
                _ => {}
            }
            
        }
        vec![event]
    }
}

impl _TextMessage {
    pub fn new(theme: &Theme, message: &MessageContent, style: Room) -> Self {
        match message {
            MessageContent::Text(message) => {
                let chars = message.chars().collect::<Vec<char>>();
                if chars.len() == 1 && is_emoji_grapheme(message) {
                    let text = Text::new(theme, message, TextSize::H1, TextStyle::Primary, Align::Left, None);
                    _TextMessage::Emoji{layout: Stack::center(), text}
                } else {
                    let text = ExpandableText::new(theme, message, TextSize::Md, TextStyle::Primary, Align::Left, None);
                    let background = match style {
                        Room::Group(Direction::Sent) | Room::Direct(Direction::Sent) => Some(theme.colors().get(colors::Brand)),
                        Room::Group(Direction::Received) | Room::Direct(Direction::Received) => Some(theme.colors().get(colors::Background::Secondary)),
                        Room::Room => None,
                    };

                    let width = Size::custom(move |h: Vec<(f32, f32)>| (h.last().unwrap().0, (h.last().unwrap().1 + 24.0).max(42.0)));
                    let height = Size::custom(move |h: Vec<(f32, f32)>| (h.last().unwrap().0, h.last().unwrap().1));
                    let layout = Stack::new(Offset::Center, Offset::Center, width, height, Padding::default());
                    _TextMessage::Text{layout, background: background.map(|c| Rectangle::new(c, 18.0, None)), text: Bin(Stack::default(), text)}
                }
            },
            MessageContent::Sticker(sticker) => {
                let sticker_size = 120.0;
                let layout = Stack(Offset::Center, Offset::Center, Size::Static(sticker_size), Size::Static(sticker_size), Padding::default());
                let shape = ShapeType::RoundedRectangle(0.0, (sticker_size, sticker_size), 0.0, 16.0);
                _TextMessage::Sticker{layout, image: Image { shape, image: sticker.image(), color: None }}
            },
            MessageContent::Gif(gif) => _TextMessage::Gif{layout: Stack::default(), gif: GifPlayer::message(theme, gif)},
            MessageContent::Photo(photo) => _TextMessage::Photo{layout: Stack::default(), photo: PhotoView::message(theme, photo)},
            MessageContent::Video(video) => _TextMessage::Video{layout: Stack::default(), video: VideoView::message(theme, video)},
        }
        
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum MessageContent {
    Text(String),
    Sticker(Sticker),
    Gif(Gif),
    Photo(Photo),
    Video(Video),
}
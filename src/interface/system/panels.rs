use prism::drawable::Component;
use ptsd::display::{Bin, Opt};
use ptsd::layout::{Stack, Offset, Size, Padding};

use crate::theme::Theme;
use crate::components::Rectangle;

use super::{EmojiPanel, StickerPanel, GifPanel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {Emoji, Stickers, Gifs}

#[derive(Component, Debug, Clone)]
pub struct Panels {
    layout: Stack,
    background: Rectangle,
    emoji: Opt<Bin<Stack, EmojiPanel>>,
    stickers: Opt<Bin<Stack, StickerPanel>>,
    gifs: Opt<Bin<Stack, GifPanel>>,
}
impl Panels {
    pub fn new(theme: &Theme) -> Self {
        let room = Padding::new(8.0);
        Self {
            layout: Stack(Offset::Start, Offset::Start, Size::Fill,
                Size::custom(|heights| heights.iter().skip(1).fold((0.0_f32, 0.0_f32),
                    |size, h| (size.0.max(h.0), size.1.max(h.1)))), Padding::default()),
            background: Rectangle::new(theme.colors().get(ptsd::Background::Secondary), 12.0, None),
            emoji: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, room), EmojiPanel::new(theme)), true),
            stickers: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, room), StickerPanel::new(theme)), false),
            gifs: Opt::new(Bin(Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, room), GifPanel::new(theme)), false),
        }
    }

    pub fn show(&mut self, page: Panel) {
        self.emoji.display(page == Panel::Emoji);
        self.stickers.display(page == Panel::Stickers);
        self.gifs.display(page == Panel::Gifs);
        if page == Panel::Gifs {self.gifs.inner().retry();}
        else {self.close_search();}
    }

    pub fn searching(&self) -> bool {self.gifs.is_showing() && self.gifs.inner_ref().searching()}
    pub fn close_search(&mut self) {self.gifs.inner().close_search();}
}

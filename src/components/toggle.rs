use prism::event::OnEvent;
use prism::drawable::Component;
use prism::canvas::{Shape, Align};
use prism::layout::{Column, Stack, Size, Offset, Padding};
use prism::display::Bin;
use prism::Context;

use crate::Theme;
use crate::components::text::{TextSize, TextStyle, ExpandableText};
use crate::components::{Rectangle, Circle};

use crate::interactions;

/// Toggle
#[derive(Debug, Component)]
pub struct Toggle(Column, ExpandableText, pub interactions::Toggle);
impl OnEvent for Toggle {}
impl Toggle {
    pub fn new(ctx: &mut Context, label: &str, is_selected: bool, on_click: impl FnMut(&mut Context, bool) + 'static) -> Self {
        let label = ExpandableText::new(ctx, label, TextSize::H5, TextStyle::Heading, Align::Left, None);

        let on = _Toggle::new(ctx, true);
        let off = _Toggle::new(ctx, false); 

        Toggle(Column::start(16.0), label, interactions::Toggle::new(on, off, is_selected, on_click))
    }
}

#[derive(Debug, Component)]
pub struct _Toggle(Stack, Rectangle, Bin<Stack, Shape>);
impl OnEvent for _Toggle {}
impl _Toggle {
    pub fn new(ctx: &mut Context, is_selected: bool) -> Self {
        let height = 32.0;
        let offset = if is_selected {Offset::End} else {Offset::Start};
        let colors = ctx.state.get_or_default::<Theme>().colors;

        let (hc, bc) = match is_selected {
            true => (colors.background.primary, colors.brand),
            false => (colors.background.primary, colors.text.secondary)
        };

        let background = Rectangle::new(bc, height / 2.0, None);
        let handle = Stack(Offset::default(), Offset::default(), Size::default(), Size::default(), Padding::new(4.0));
        let handle = Bin(handle, Circle::new(height*0.75, hc, false));
        let layout = Stack(offset, Offset::Center, Size::Static(height*2.0), Size::Static(height), Padding::default());
    
        _Toggle(layout, background, handle)
    }
}


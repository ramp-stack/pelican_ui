use roost_ui::events::OnEvent;
use roost_ui::{Context, Component};
use roost_ui::drawable::{Shape, Align};
use roost_ui::layouts::{Bin, Column, Stack, Size, Offset, Padding};
use crate::components::{TextSize, TextStyle, ExpandableText, Rectangle, Circle};
use crate::plugin::PelicanUI;

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
        let colors = ctx.get::<PelicanUI>().get().0.theme().colors;

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


use pelican_ui::events::{MouseState, MouseEvent, OnEvent, Event};
use pelican_ui::drawable::{Drawable, Component, Image, Color};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use crate::utils::Callback;
use crate::components::{Icon, Rectangle};
use crate::layout::{Offset, Padding, Size, Stack, Opt};
use super::{ButtonSize, ButtonState, ButtonStyle};

/// ## Icon Button
///
/// A clickable icon component.  
///  
/// See various examples below.
#[derive(Debug, Component)]
pub struct IconButton(Stack, IconButtonContent, Option<Opt<Image>>, #[skip] bool);
impl IconButton {
    pub fn new(
        ctx: &mut Context,
        icon: &'static str,
        size: ButtonSize,
        style: ButtonStyle,
        state: ButtonState,
        on_click: Box<dyn FnMut(&mut Context)>,
        flair: Option<(&'static str, Color, bool)>, // icon name, color, is shown
    ) -> Self {
        let content = IconButtonContent::new(ctx, icon, size, style, state, on_click);
        let s = if size == ButtonSize::Large {52.0} else {36.0};
        let icon = flair.map(|(i, c, h)| Opt::new(Icon::new(ctx, i, c, s / 1.8), h));
        IconButton(Stack(Offset::End, Offset::Start, Size::Fit, Size::Fit, Padding::default()), content, icon, true)
    }

    pub fn color(&mut self, ctx: &mut Context, state: ButtonState) {
        let colors = state.color(ctx, self.1.3);
        *self.1.1.background() = colors.background;
        self.1.2.color = Some(colors.label);
        if let Some(c) = self.1.1.outline() { *c = colors.outline; }
    }

    pub fn show_flair(&mut self, hide: bool) {if let Some(i) = self.2.as_mut() {i.display(hide);}}
    pub fn status(&mut self) -> &mut ButtonState {&mut self.1.4}

    /// Sets the trigger of the on_click to either `On Press` or `On Release`
    pub fn set_trigger_on_press(&mut self, on_press: bool) {self.3 = on_press;}
}

impl OnEvent for IconButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(event) = event.downcast_ref::<MouseEvent>() {
            if let Some(state) = self.1.4.handle(ctx, *event) {
                self.color(ctx, state);
            }
            
            if (matches!(event, MouseEvent { state: MouseState::Pressed, position: Some(_) }) && self.3
            || matches!(event, MouseEvent { state: MouseState::Released, position: Some(_)}) && !self.3)
            && matches!(self.1.4, ButtonState::Default | ButtonState::Hover | ButtonState::Pressed) {
                ctx.hardware.haptic();
                (self.1.5)(ctx);
            }

            false
        } else {true}
    }
}

#[derive(Component)]
pub struct IconButtonContent(Stack, Rectangle, Image, #[skip] ButtonStyle, #[skip] ButtonState, #[skip] pub Box<dyn FnMut(&mut Context)>);
impl OnEvent for IconButtonContent {}
impl IconButtonContent {
    pub fn new(
        ctx: &mut Context,
        icon: &'static str,
        size: ButtonSize,
        style: ButtonStyle,
        state: ButtonState,
        on_click: Box<dyn FnMut(&mut Context)>,
    ) -> Self {
        let colors = state.color(ctx, style);
        let (size, icon_size, radius) = match (style, size) {
            (ButtonStyle::Secondary, ButtonSize::Large) => (52.0, 32.0, 12.0),
            (ButtonStyle::Secondary, ButtonSize::Medium) => (36.0, 20.0, 8.0),
            (ButtonStyle::Ghost, ButtonSize::Large) => (52.0, 48.0, 12.0),
            (ButtonStyle::Ghost, ButtonSize::Medium) => (36.0, 32.0, 8.0),
            _ => panic!("{style:?} is not a valid IconButton style")
        };

        let icon = Icon::new(ctx, icon, colors.label, icon_size);
        let background = Rectangle::new(colors.background, radius, Some((1.0, colors.outline)));


        IconButtonContent(
            Stack(Offset::Center, Offset::Center, Size::Static(size), Size::Static(size), Padding::default()),
            background, icon, style, state, on_click
        )
    }
}

impl std::fmt::Debug for IconButtonContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IconButtonContent(...)")
    }
}

impl IconButton {
    /// ## Secondary Icon Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/secondary_icons.png"
    ///      alt="Secondary Icon Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = IconButton::secondary(ctx, "info", |ctx: &mut Context| println!("This button has been clicked!"));
    /// ```
    pub fn secondary(
        ctx: &mut Context, 
        icon: &'static str, 
        on_click: Callback
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Large,
            ButtonStyle::Secondary,
            ButtonState::Default,
            on_click,
            None,
        )
    }

    /// ## Secondary Medium Icon Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/secondary_icons.png"
    ///      alt="Secondary Icon Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = IconButton::secondary_medium(ctx, "info", |ctx: &mut Context| println!("This button has been clicked!"));
    /// ```
    pub fn secondary_medium(
        ctx: &mut Context, 
        icon: &'static str, 
        on_click: Callback
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Medium,
            ButtonStyle::Secondary,
            ButtonState::Default,
            on_click,
            None,
        )
    }

    /// ## Ghost Icon Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/ghost_icons.png"
    ///      alt="Ghost Icons Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = IconButton::ghost(ctx, "explore", |ctx: &mut Context| println!("This button has been clicked!"));
    /// ```
    pub fn ghost(
        ctx: &mut Context, 
        icon: &'static str, 
        on_click: Callback
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Large,
            ButtonStyle::Ghost,
            ButtonState::Default,
            on_click,
            None,
        )
    }


    /// ## Medium Ghost Icon Button
    ///
    /// <img src="https://raw.githubusercontent.com/ramp-stack/pelican_ui_std/main/src/examples/ghost_icons.png"
    ///      alt="Ghost Icons Example"
    ///      width="250">
    ///
    /// ### Example
    /// ```rust
    /// let button = IconButton::ghost_medium(ctx, "explore", |ctx: &mut Context| println!("This button has been clicked!"));
    /// ```
    pub fn ghost_medium(
        ctx: &mut Context, 
        icon: &'static str, 
        on_click: Callback
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Medium,
            ButtonStyle::Ghost,
            ButtonState::Default,
            on_click,
            None,
        )
    }

    /// Icon Button designed for text inputs.
    pub fn input(
        ctx: &mut Context, 
        icon: &'static str, 
        on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Medium,
            ButtonStyle::Secondary,
            ButtonState::Default,
            Box::new(on_click),
            None,
        )
    }

    /// Icon Button designed for keyboards.
    pub fn keyboard(
        ctx: &mut Context, 
        icon: &'static str,
        on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Medium,
            ButtonStyle::Ghost,
            ButtonState::Default,
            Box::new(on_click),
            None,
        )
    }
    
    /// Icon Button designed for page navigation.
    pub fn navigation(
        ctx: &mut Context, 
        icon: &'static str, 
        on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Medium,
            ButtonStyle::Ghost,
            ButtonState::Default,
            Box::new(on_click),
            None
        )
    }

    /// Creates a button designed for the ending or closing page of a flow.
    pub fn close(
        ctx: &mut Context, 
        on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        IconButton::new(
            ctx,
            "close",
            ButtonSize::Medium,
            ButtonStyle::Ghost,
            ButtonState::Default,
            Box::new(on_click),
            None,
        )
    }

    /// Icon Button designed for interface navigators.
    pub fn tab_nav(
        ctx: &mut Context, 
        icon: &'static str, 
        selected: bool,
        on_click: impl FnMut(&mut Context) + 'static,
    ) -> Self {
        let color = ctx.theme.colors.brand;
        let state = if selected {ButtonState::Selected} else {ButtonState::UnSelected};
        IconButton::new(
            ctx,
            icon,
            ButtonSize::Medium,
            ButtonStyle::Ghost,
            state,
            Box::new(on_click),
            Some(("notification", color, false))
        )
    }
}


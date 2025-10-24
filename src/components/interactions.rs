use mustache::events::{MouseState, MouseEvent, OnEvent, Event, TickEvent, KeyboardState, KeyboardEvent};
use mustache::drawable::{Drawable};
use mustache::{Context, Component};
use mustache::layouts::{Stack, Bin, Opt, Offset, Size, Padding};

// use crate::components::avatar::{Avatar, AvatarContent};
use crate::utils::{ElementID};
use crate::components::interface::mobile::ShowKeyboard;

// #[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
// pub enum ButtonState {Default, Disabled, Selected, Pressed, Hover}


// #[derive(Component)]
// pub struct Button {
//     layout: Stack,
//     default: Opt<Box<dyn Drawable>>,
//     hover: Opt<Box<dyn Drawable>>,
//     pressed: Opt<Box<dyn Drawable>>,
//     disabled: Opt<Box<dyn Drawable>>,
//     #[skip] state: ButtonState,
//     #[skip] on_click: Callback,
// }

// impl Button{
//     pub fn new(
//         on_click: Callback,
//         default: impl Drawable + 'static,
//         hover: impl Drawable + 'static,
//         pressed: impl Drawable + 'static,
//         disabled: impl Drawable + 'static,
//         state: ButtonState
//     ) -> Self {
//         Button {
//             layout: Stack::default(),
//             on_click,
//             default: Opt::new(Box::new(default), state == ButtonState::Default),
//             disabled: Opt::new(Box::new(disabled), state == ButtonState::Disabled),
//             pressed: Opt::new(Box::new(pressed), state == ButtonState::Pressed),
//             hover: Opt::new(Box::new(hover), state == ButtonState::Hover),
//             state,
//         }
//     }

//     pub fn disabled(&mut self, is_disabled: bool) {
//         self.state = if is_disabled {ButtonState::Disabled} else {ButtonState::Default};
//     }
// }

// impl OnEvent for Button {
//     fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
//         if event.downcast_ref::<TickEvent>().is_some() {
//             self.default.display(self.state == ButtonState::Default);
//             self.hover.display(self.state == ButtonState::Hover);
//             self.pressed.display(self.state == ButtonState::Pressed);
//             self.disabled.display(self.state == ButtonState::Disabled);
//         } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
//             let state = match self.state {
//                 ButtonState::Default if event.position.is_some() => {
//                     match event.state {
//                         MouseState::Pressed => {
//                             Some(ButtonState::Pressed)
//                         },
//                         MouseState::Moved | MouseState::Scroll(..) => Some(if mustache::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
//                         _ => None
//                     }
//                 },
//                 ButtonState::Pressed => {
//                     match event.state {
//                         MouseState::Released if event.position.is_some() => Some(if mustache::IS_MOBILE {ButtonState::Default} else {ButtonState::Hover}),
//                         MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
//                         _ => None
//                     }
//                 },
//                 ButtonState::Hover => {
//                     match event.state {
//                         MouseState::Pressed if event.position.is_some() => Some(ButtonState::Pressed),
//                         MouseState::Moved | MouseState::Scroll(..) if event.position.is_none() => Some(ButtonState::Default),
//                         _ => None
//                     }
//                 }
//                 _ => None
//             };

//             if let Some(state) = state { self.state = state; }
//             if let MouseEvent { state: MouseState::Pressed, position: Some(_) } = event {
//                 if matches!(self.state, ButtonState::Default | ButtonState::Hover | ButtonState::Pressed) {
//                     ctx.hardware.haptic();
//                     (self.on_click)(ctx);
//                 }
//             }
//         }
//         true
//     }
// }

// impl std::fmt::Debug for Button {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Button")
//     }
// }

// #[derive(Component)]
// pub struct Selectable {
//     layout: Stack,
//     default: Opt<Box<dyn Drawable>>,
//     selected: Opt<Box<dyn Drawable>>,
//     #[skip] is_selected: bool,
//     #[skip] on_click: Callback,
//     #[skip] id: ElementID,
//     #[skip] group_id: ElementID,
// }

// impl std::fmt::Debug for Selectable {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Selectable")
//     }
// }

// impl Selectable {
//     pub fn new(
//         on_click: impl FnMut(&mut Context) + 'static,
//         default: impl Drawable + 'static,
//         selected: impl Drawable + 'static,
//         is_selected: bool,
//         group_id: ElementID,
//     ) -> Self {
//         Selectable {
//             layout: Stack::default(),
//             on_click: Box::new(on_click),
//             default: Opt::new(Box::new(default), !is_selected),
//             selected: Opt::new(Box::new(selected), is_selected),
//             is_selected,
//             group_id,
//             id: ElementID::new(),
//         }
//     }

//     pub fn selected(&mut self, is_selected: bool) { self.is_selected = is_selected; }
// }

// impl OnEvent for Selectable {
//     fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
//         if event.downcast_ref::<TickEvent>().is_some() {
//             self.default.display(!self.is_selected);
//             self.selected.display(self.is_selected);
//         } else if let Some(MouseEvent { state: MouseState::Pressed, position: Some(_) }) = event.downcast_ref::<MouseEvent>() {
//             ctx.hardware.haptic();
//             (self.on_click)(ctx);
//             ctx.trigger_event(SelectableEvent(self.id, self.group_id))
//         } else if let Some(SelectableEvent(id, group_id)) = event.downcast_ref::<SelectableEvent>() {
//             if *group_id == self.group_id {self.is_selected = *id == self.id; }
//         }
//         false
//     }
// }

// #[derive(Debug, Copy, Clone)]
// pub struct SelectableEvent(pub ElementID, pub ElementID);
// impl Event for SelectableEvent {
//     fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
//         children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
//     }
// }

// type SliderClosure = Box<dyn FnMut(&mut Context, f32)>;

// impl std::fmt::Debug for Slider {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Slider")
//     }
// }

// #[derive(Component)]
// pub struct Slider {
//     layout: Stack,
//     pub background: Bin<Stack, Box<dyn Drawable>>,
//     pub foreground: Bin<Stack, Box<dyn Drawable>>,
//     pub knob: Bin<Stack, Box<dyn Drawable>>,
//     #[skip] pub value: f32,
//     #[skip] closure: SliderClosure,
//     #[skip] dragging: bool,
// }

// impl Slider {
//     pub fn new(
//         start: f32, 
//         background: impl Drawable + 'static,
//         foreground: impl Drawable + 'static,
//         knob: impl Drawable + 'static,
//         on_change: impl FnMut(&mut Context, f32) + 'static
//     ) -> Self {
//         let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(300.0), f32::MAX));
//         let b_layout = Stack(Offset::Start, Offset::Center, width, Size::Static(6.0), Padding::default());
//         let f_layout = Stack(Offset::Start, Offset::Start, Size::Static(30.0), Size::Static(6.0), Padding::default());
//         let k_layout = Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default());
//         let layout = Stack(Offset::Start, Offset::Center, Size::Fit, Size::Fit, Padding::default());

//         Slider {
//             layout,
//             background: Bin(b_layout, Box::new(background)),
//             foreground: Bin(f_layout, Box::new(foreground)),
//             knob: Bin(k_layout, Box::new(knob)),
//             value: start, 
//             closure: Box::new(on_change),
//             dragging: false,
//         }
//     }

//     fn clamp(&mut self, ctx: &mut Context, x: f32) {
//         let full_width = (**self.background.inner()).request_size(ctx).max_width();
//         self.value = x.clamp(0.0, full_width);
//     }
// }

// impl OnEvent for Slider {
//     fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
//         if event.downcast_ref::<TickEvent>().is_some() {
//             let full_width = (**self.background.inner()).request_size(ctx).max_width();
//             let knob_size = (**self.knob.inner()).request_size(ctx).min_width() / 2.0;

//             let clamped_x = self.value.clamp(0.0, full_width);
//             self.knob.layout().0 = Offset::Static((clamped_x - knob_size).max(0.0));
//             self.foreground.layout().2 = Size::Static(clamped_x);
//         } else if let Some(MouseEvent { state, position, }) = event.downcast_ref::<MouseEvent>() {
//             match state {
//                 MouseState::Pressed => {
//                     if let Some((x, _)) = position {
//                         ctx.hardware.haptic();
//                         self.dragging = true;
//                         self.clamp(ctx, *x);
//                     }
//                 },
//                 MouseState::Released if self.dragging => {
//                     self.dragging = false;
//                     (self.closure)(ctx, self.value);
//                 }
//                 MouseState::Scroll(..) | MouseState::Moved 
//                 if self.dragging => {
//                     (self.closure)(ctx, self.value);
//                     if let Some((x, _)) = position { self.clamp(ctx, *x); }
//                 }
//                 _ => {}
//             }
//         }

//         true
//     }
// }

// #[derive(Debug, Component)]
// pub struct InputField {
//     layout: Stack,
//     default: Opt<Box<dyn Drawable>>,
//     focus: Opt<Box<dyn Drawable>>,
//     hover: Option<Opt<Box<dyn Drawable>>>,
//     error: Option<Opt<Box<dyn Drawable>>>,
//     content: Box<dyn Drawable>,
//     #[skip] state: InputState,
//     #[skip] pub id: ElementID,
//     #[skip] pub has_error: bool
// }

// impl InputField {
//     pub fn new(
//         default: impl Drawable + 'static,
//         focus: impl Drawable + 'static,
//         hover: Option<impl Drawable + 'static>,
//         error: Option<impl Drawable + 'static>,
//         content: impl Drawable + 'static,
//         height: f32,
//         id: ElementID,
//     ) -> Self {
//         let height = Size::custom(move |h: Vec<(f32, f32)>| (h[4].0.max(height), h[4].1.max(height)));
//         InputField {
//             layout: Stack(Offset::Start, Offset::Start, Size::Fit, height, Padding::default()),
//             default: Opt::new(Box::new(default), true),
//             focus: Opt::new(Box::new(focus), false),
//             hover: hover.map(|h| Opt::new(Box::new(h) as Box<dyn Drawable>, false)),
//             error: error.map(|e| Opt::new(Box::new(e) as Box<dyn Drawable>, false)),
//             content: Box::new(content),
//             state: InputState::Default,
//             id,
//             has_error: false,
//         }
//     }
    
//     pub fn error(&mut self, is_errored: bool) {
//         self.has_error = is_errored;
//         // self.state = if is_errored {InputState::Error} else {InputState::Default};
//     }
// }

// impl OnEvent for InputField {
//     fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
//         if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
//             self.default.display(self.state == InputState::Default);
//             self.focus.display(self.state == InputState::Focus);

//             if let Some(h) = &mut self.hover {
//                 h.display(self.state == InputState::Hover);
//             } else if self.state == InputState::Hover {
//                 self.default.display(true);
//             }

//             if let Some(e) = &mut self.error {
//                 e.display(self.state == InputState::Error);
//             } else if self.state == InputState::Error {
//                 self.default.display(true);
//             }

//         } else if let Some(event) = event.downcast_ref::<MouseEvent>() {
//             self.state = match self.state {
//                 InputState::Default => {
//                     match event {
//                         MouseEvent{state: MouseState::Pressed, position: Some(_)} => {
//                             ctx.hardware.haptic();
//                             ctx.trigger_event(ShowKeyboard(true)); 
//                             ctx.trigger_event(InputFieldEvent::Select(self.id));
//                             Some(InputState::Focus)
//                         },
//                         MouseEvent{state: MouseState::Moved, position: Some(_)} => Some(InputState::Hover),
//                         _ => None
//                     }
//                 },
//                 InputState::Hover => {
//                     match event {
//                         MouseEvent{state: MouseState::Pressed, position: Some(_)} => {
//                             ctx.hardware.haptic();
//                             ctx.trigger_event(ShowKeyboard(true)); 
//                             ctx.trigger_event(InputFieldEvent::Select(self.id));
//                             Some(InputState::Focus)
//                         },
//                         MouseEvent{state: MouseState::Moved, position: None} if self.has_error => Some(InputState::Error),
//                         MouseEvent{state: MouseState::Moved, position: None} => Some(InputState::Default),
//                         _ => None
//                     }
//                 },
//                 InputState::Focus => {
//                     match event {
//                         MouseEvent{state: MouseState::Pressed, position: None} if self.has_error && !mustache::IS_MOBILE => {
//                             ctx.trigger_event(InputFieldEvent::Deselect(self.id));
//                             Some(InputState::Error)
//                         },
//                         MouseEvent{state: MouseState::Pressed, position: None} if !mustache::IS_MOBILE => {
//                             ctx.trigger_event(InputFieldEvent::Deselect(self.id));
//                             Some(InputState::Default)
//                         },
//                         _ => None
//                     }
//                 },
//                 InputState::Error => {
//                     match event {
//                         MouseEvent{state: MouseState::Pressed, position: Some(_)} => Some(InputState::Focus),
//                         MouseEvent{state: MouseState::Moved, position: Some(_)} => Some(InputState::Hover),
//                         _ => None
//                     }
//                 }
//             }.unwrap_or(self.state);
//         } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key: _}) = event.downcast_ref() {
//             return self.state == InputState::Focus;
//         }
//         true
//     }
// }

// #[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
// pub enum InputState {
//     Default,
//     Hover,
//     Focus,
//     Error
// }

// /// Event used to focus active input field on mobile and enable editing of the text input content.
// #[derive(Debug, Clone)]
// pub enum InputFieldEvent {
//     Select(ElementID),
//     Deselect(ElementID)
// }

// impl Event for InputFieldEvent {
//     fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
//         children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
//     }
// }
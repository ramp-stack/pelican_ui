use roost_ui::events::{self, OnEvent, TickEvent, Event};
use roost_ui::drawable::Align;
use roost_ui::{Context, Component, emitters};
use roost_ui::layouts::{Stack, Row, Opt};

use crate::plugin::PelicanUI;
use crate::components::text::{TextSize, TextStyle, Text};

#[derive(Debug, Clone, PartialEq)]
pub enum SlotType {
    // A permanent visible character (e.g. "$")
    FixedChar(char),

    // A ghost placeholder that becomes primary when the user inputs a digit.
    // Example: 'D', 'M', 'Y', or ghost cents '0'.
    // also the max amount of characters that can replace this
    Ghost(char, usize), 

    // A primary placeholder that stays primary when the user inputs a digit.
    // Example: '0' after the '$'
    // also the max amount of characters that can replace this
    Primary(char, usize),

    // A slot that is created only when triggered by input.
    // Example: currency fractional digits ("00" ghost cents)
    // first char is the display usual and the trigger.
    Triggered(char),

    GhostInput(char), // this only shows up when it is 'next' and is replaced by primary texto n input 
}

#[derive(Debug, Component)]
pub struct InputSegment {
    layout: Stack,
    inner: Opt<Text>,
    replacement: Option<Opt<Text>>,
    #[skip] slot: SlotType,
}

impl InputSegment {
    pub fn new(ctx: &mut Context, slot: SlotType) -> Self {
        let ghost = ctx.get::<PelicanUI>().get().0.theme().colors.text.secondary;
        let (inner, replacement) = match slot.clone() {
            // A permanent visible character (e.g. "$", "/", ":")
            SlotType::FixedChar(c) => {
                let text = Text::new(ctx, &c.to_string(), TextSize::Title, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, true), None)
            },

            // A ghost placeholder that becomes primary when the user inputs a digit.
            // Example: 'D', 'M', 'Y', or ghost cents '0'.
            SlotType::Ghost(c, _max) => {
                let text = Text::new(ctx, &c.to_string(), TextSize::Title, TextStyle::Label(ghost), Align::Left, None);
                let rep = Text::new(ctx, "", TextSize::Title, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, true), Some(Opt::new(rep, false)))
            },

            // A primary placeholder that stays primary when the user inputs a digit.
            // Example: '0' after the '$'
            SlotType::Primary(c, _max) => {
                let text = Text::new(ctx, &c.to_string(), TextSize::Title, TextStyle::Heading, Align::Left, None);
                let rep = Text::new(ctx, "", TextSize::Title, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, true), Some(Opt::new(rep, false)))
            },

            // A slot that is created only when triggered by input.
            // Example: currency fractional digits ("00" ghost cents).
            SlotType::Triggered(c) => {
                let text = Text::new(ctx, &c.to_string(), TextSize::Title, TextStyle::Heading, Align::Left, None);
                let rep = Text::new(ctx, "", TextSize::Title, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, false), Some(Opt::new(rep, false)))
            }

            // this only shows up when it is 'next' and is replaced by primary texto n input 
            SlotType::GhostInput(c) => {
                let text = Text::new(ctx, &c.to_string(), TextSize::Title, TextStyle::Label(ghost), Align::Left, None);
                let rep = Text::new(ctx, "", TextSize::Title, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, false), Some(Opt::new(rep, false)))
            },
        };

        InputSegment { layout: Stack::default(), inner, replacement, slot }
    }
}

impl OnEvent for InputSegment {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<InputEvent>() {
            match (self.slot.clone(), event) {
                (SlotType::Primary(_c, _max) | SlotType::Ghost(_c, _max), InputEvent::Delete) => {
                    // if the replacement text is less than or equal to 1
                    // then hide the replacement text and show the defaul text
                    // if it is greater than 1, then just remove one character from the replacement text.
                    let rep = self.replacement.as_mut().unwrap();
                    match rep.inner().spans[0].len() <= 1 {
                        true => {
                            rep.display(false);
                            self.inner.display(true);
                            ctx.trigger_event(InputEvent::MoveBack);
                        },
                        false => { 
                            rep.inner().spans[0].pop(); 
                        }
                    }
                },
                (SlotType::Triggered(_c), InputEvent::Delete) => {
                    self.inner.display(false);
                    ctx.trigger_event(InputEvent::MoveBack);
                    ctx.trigger_event(InputEvent::Triggered(true));
                },
                (SlotType::GhostInput(_c), InputEvent::Delete) =>  {
                    // if the replacement text is visible, then hide it and show the default text.
                    // if is not visible, then hide the default text too
                    let rep = self.replacement.as_mut().unwrap();
                            ctx.trigger_event(InputEvent::MoveBack);
                    match rep.is_showing() {
                        true => {
                            rep.display(false);
                            self.inner.display(true);
                        },
                        false => {
                            // self.inner.display(false);
                            // ctx.trigger_event(InputEvent::MoveBack);
                        }
                    }
                },

                (SlotType::Ghost(_c, max) | SlotType::Primary(_c, max), InputEvent::InputDigit(n)) => {
                    let rep = self.replacement.as_mut().unwrap();
                    if rep.inner().spans[0].len() >= max { ctx.trigger_event(InputEvent::MoveForward); }
                    self.inner.display(false);
                    // if replacement is hiden, then show it and replace its value with n
                    if !rep.is_showing() {
                        rep.display(true);
                        rep.inner().spans[0] = n.to_string();
                    } else if rep.inner().spans[0].len() < max {
                        let is_zero = rep.inner().spans[0].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<f32>().ok().map(|t| t <= 0.0).unwrap_or(false);
                        if !is_zero {
                            rep.inner().spans[0].push(*n);
                        }
                    }
                }

                (SlotType::Triggered(c), InputEvent::InputChar(n)) => {
                    if c == *n {
                        self.inner.display(true);
                        ctx.trigger_event(InputEvent::MoveForward);
                        ctx.trigger_event(InputEvent::Triggered(false));
                    }
                },

                (SlotType::GhostInput(_), InputEvent::InputDigit(n)) => {
                    // if replacement is hiden, then show it and replace its value with n
                    
                    let rep = self.replacement.as_mut().unwrap();
                    self.inner.display(false);
                    // if replacement is hiden, then show it and replace its value with n
                    if !rep.is_showing() {
                        rep.display(true);
                        rep.inner().spans[0] = n.to_string();
                    }
                    if !rep.inner().spans[0].is_empty() { ctx.trigger_event(InputEvent::MoveForward); }
                }
                _ => {}
            }
        }

        vec![event]
    }
}

#[derive(Debug, Component)]
pub struct NumericalInput(Stack, emitters::NumericalInput<_NumericalInput>);
impl OnEvent for NumericalInput {}


impl NumericalInput {
    pub fn new(ctx: &mut Context, items: Vec<SlotType>) -> Self {
        NumericalInput(Stack::default(), emitters::NumericalInput::new(_NumericalInput::new(ctx, items)))
    }

    pub fn value(&mut self) -> String { self.1.1.value() }
}

#[derive(Debug, Component)]
pub struct _NumericalInput {
    layout: Row,
    segments: Vec<InputSegment>,
    #[skip] cursor: usize
}

impl _NumericalInput {
    pub fn new(ctx: &mut Context, items: Vec<SlotType>) -> Self {
        let segments = items.into_iter().map(|i| InputSegment::new(ctx, i)).collect();

        _NumericalInput { layout: Row::center(0.0), segments, cursor: 0, }
    }

    pub fn value(&mut self) -> String {
        let mut out = String::new();

        for seg in &mut self.segments {
            // If replacement is visible, it always overrides inner.
            if let Some(rep) = &mut seg.replacement {
                if rep.is_showing() {
                    let text = rep.inner().spans[0].clone();
                    out.push_str(&text);
                    continue;
                }
            }

            // Fallback to inner text if visible.
            if seg.inner.is_showing() {
                let text = &seg.inner.inner().spans[0];
                out.push_str(text);
            }
        }

        out
    }
}

impl OnEvent for _NumericalInput {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            let text_size = match self.value().len() {
                0..=3 => TextSize::Title,
                _ => TextSize::H1,
            };

            for seg in &mut self.segments {
                seg.inner.inner().size = text_size;
                if let Some(rep) = &mut seg.replacement {
                    rep.inner().size = text_size;
                }
            }
        }

        if let Some(InputEvent::MoveForward) = event.downcast_ref::<InputEvent>() {
            if self.cursor < self.segments.len() - 1 {
                self.cursor += 1;
                if let Some(seg) = self.segments.get(self.cursor) {
                    if let SlotType::FixedChar(_) = seg.slot { 
                        self.cursor += 1; 
                    }
                }
            }
        } else if let Some(InputEvent::MoveBack) = event.downcast_ref::<InputEvent>() {
            self.cursor = self.cursor.saturating_sub(1);
            if let Some(seg) = self.segments.get(self.cursor) {
                if let SlotType::FixedChar(_) = seg.slot {
                    self.cursor = self.cursor.saturating_sub(1);
                }
            }
        } else if let Some(event) = event.downcast_ref::<events::NumericalInput>() {
            match event {
                events::NumericalInput::Delete => {
                    if let Some(seg) = self.segments.get_mut(self.cursor) {
                        seg.on_event(ctx, Box::new(InputEvent::Delete));
                    }
                },
                events::NumericalInput::Digit(c) => {
                    if let Some(seg) = self.segments.get(self.cursor) {
                        if let SlotType::FixedChar(_) = seg.slot { self.cursor += 1; }
                    }

                    if let Some(seg) = self.segments.get_mut(self.cursor) {
                        seg.on_event(ctx, Box::new(InputEvent::InputDigit(*c)));
                    }
                },
                events::NumericalInput::Char(c) => {
                    if let Some(seg) = self.segments.get(self.cursor) {
                        if let SlotType::FixedChar(_) = seg.slot { self.cursor += 1; }
                    }
                    for (i, segment) in self.segments.iter_mut().enumerate() {
                        if let SlotType::Triggered(trigger) = segment.slot {
                            if i > self.cursor && *c == trigger { self.cursor = i; }
                        }
                    }

                    if let Some(seg) = self.segments.get_mut(self.cursor) {
                        seg.on_event(ctx, Box::new(InputEvent::InputChar(*c)));
                    }
                }
            }
        } else if let Some(InputEvent::Triggered(is_delete)) = event.downcast_ref::<InputEvent>() {
            for seg in &mut self.segments {
                if let SlotType::GhostInput(_) = seg.slot {
                    seg.inner.display(!*is_delete);
                }
            }
        }

        vec![event]
    }
}

impl Event for InputEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Debug, Clone)]
enum InputEvent {
    Delete,
    InputDigit(char),
    InputChar(char),

    MoveBack,
    MoveForward,
    Triggered(bool)
}
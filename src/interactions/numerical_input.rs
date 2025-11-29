use roost_ui::events::{OnEvent, Event, NamedKey, Key, KeyboardEvent, KeyboardState};
use roost_ui::drawable::Align;
use roost_ui::{Context, Component};
use roost_ui::layouts::{Stack, Row, Opt};

use crate::plugin::PelicanUI;
use crate::components::text::{TextSize, TextStyle, Text};

#[derive(Debug, Clone, PartialEq)]
pub enum SlotType {
    // A permanent visible character (e.g. "$", "/", ":", ".")
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
    #[skip] id: usize,
}

impl InputSegment {
    pub fn new(ctx: &mut Context, slot: SlotType, id: usize, text_size: TextSize) -> Self {
        let ghost = ctx.get::<PelicanUI>().get().0.theme().colors.text.secondary;
        let (inner, replacement) = match slot.clone() {
            // A permanent visible character (e.g. "$", "/", ":")
            SlotType::FixedChar(c) => {
                let text = Text::new(ctx, &c.to_string(), text_size, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, true), None)
            },

            // A ghost placeholder that becomes primary when the user inputs a digit.
            // Example: 'D', 'M', 'Y', or ghost cents '0'.
            SlotType::Ghost(c, _max) => {
                let text = Text::new(ctx, &c.to_string(), text_size, TextStyle::Label(ghost), Align::Left, None);
                let rep = Text::new(ctx, "", text_size, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, true), Some(Opt::new(rep, false)))
            },

            // A primary placeholder that stays primary when the user inputs a digit.
            // Example: '0' after the '$'
            SlotType::Primary(c, _max) => {
                let text = Text::new(ctx, &c.to_string(), text_size, TextStyle::Heading, Align::Left, None);
                let rep = Text::new(ctx, "", text_size, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, true), Some(Opt::new(rep, false)))
            },

            // A slot that is created only when triggered by input.
            // Example: currency fractional digits ("00" ghost cents).
            SlotType::Triggered(c) => {
                let text = Text::new(ctx, &c.to_string(), text_size, TextStyle::Label(ghost), Align::Left, None);
                let rep = Text::new(ctx, "", text_size, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, false), Some(Opt::new(rep, false)))
            }

            // this only shows up when it is 'next' and is replaced by primary texto n input 
            SlotType::GhostInput(c) => {
                let text = Text::new(ctx, &c.to_string(), text_size, TextStyle::Label(ghost), Align::Left, None);
                let rep = Text::new(ctx, "", text_size, TextStyle::Heading, Align::Left, None);
                (Opt::new(text, false), Some(Opt::new(rep, false)))
            },
        };

        InputSegment { layout: Stack::default(), inner, replacement, slot, id }
    }
}

impl OnEvent for InputSegment {
    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(InputEvent::NumericalInputEvent(id, event)) = event.downcast_ref::<InputEvent>() {
            if *id == self.id {
                match (self.slot.clone(), event) {
                    (SlotType::Primary(_c, _max) | SlotType::Ghost(_c, _max), NumericalInputEvent::Delete) => {
                        // if the replacement text is less than or equal to 1
                        // then hide the replacement text and show the defaul text
                        // if it is greater than 1, then just remove one character from the replacement text.

                        match self.replacement.as_mut().unwrap().inner().spans[0].len() <= 1 {
                            true => {
                                self.replacement.as_mut().unwrap().display(false);
                                self.inner.display(true);
                                ctx.trigger_event(InputEvent::MoveCursorBack);
                            },
                            false => { 
                                self.replacement.as_mut().unwrap().inner().spans[0].pop(); 
                            }
                        }
                    },
                    (SlotType::Triggered(_c), NumericalInputEvent::Delete) => {
                        self.inner.display(false);
                        ctx.trigger_event(InputEvent::MoveCursorBack);
                    },
                    (SlotType::GhostInput(_c), NumericalInputEvent::Delete) =>  {
                        // if the replacement text is visible, then hide it and show the default text.
                        // if is not visible, then hide the default text too
                        match self.replacement.as_mut().unwrap().is_showing() {
                            true => {
                                self.replacement.as_mut().unwrap().display(false);
                                self.inner.display(true);
                            },
                            false => {
                                self.inner.display(false);
                                ctx.trigger_event(InputEvent::MoveCursorBack);
                            }
                        }
                    },

                    (SlotType::Ghost(c, max) | SlotType::Primary(c, max), NumericalInputEvent::Digit(n)) => {
                        if self.replacement.as_mut().unwrap().inner().spans[0].len() >= max { ctx.trigger_event(InputEvent::MoveCursorForward); }
                        self.inner.display(false);
                        // if replacement is hiden, then show it and replace its value with n
                        if !self.replacement.as_mut().unwrap().is_showing() {
                            self.replacement.as_mut().unwrap().display(true);
                            self.replacement.as_mut().unwrap().inner().spans[0] = n.to_string();
                        } else if self.replacement.as_mut().unwrap().inner().spans[0].len() < max {
                            self.replacement.as_mut().unwrap().inner().spans[0].push(*n);
                        }
                    }

                    (SlotType::Triggered(c), NumericalInputEvent::Digit(n) | NumericalInputEvent::Special(n)) => {
                        if c == *n {
                            self.inner.display(true);
                            ctx.trigger_event(InputEvent::MoveCursorForward);
                            ctx.trigger_event(InputEvent::Triggered);
                        }
                    },

                    (SlotType::GhostInput(_), NumericalInputEvent::Digit(n)) => {
                        // if replacement is hiden, then show it and replace its value with n
                        ctx.trigger_event(InputEvent::MoveCursorForward);
                        if self.replacement.as_mut().unwrap().is_showing() {
                            self.replacement.as_mut().unwrap().display(true);
                            self.replacement.as_mut().unwrap().inner().spans[0] = n.to_string();
                        }
                    }
                    _ => {}
                }
            }
        }

        vec![event]
    }
}

#[derive(Debug, Component)]
pub struct NumericalInput {
    layout: Row,
    segments: Vec<InputSegment>,
    #[skip] cursor: usize
}

impl NumericalInput {
    pub fn new(ctx: &mut Context, items: Vec<SlotType>) -> Self {
        let text_size = match items.len() {
            0..=5 => TextSize::Title,
            6 | 7 => TextSize::H1,
            _ => TextSize::H2
        };

        let segments = items.into_iter().enumerate().map(|(x, i)| InputSegment::new(ctx, i, x, text_size)).collect();

        NumericalInput { layout: Row::center(0.0), segments, cursor: 0, }
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

impl OnEvent for NumericalInput {
    fn on_event(&mut self, _ctx: &mut Context, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(InputEvent::MoveCursorForward) = event.downcast_ref::<InputEvent>() {
            self.cursor += 1;
            if let Some(seg) = self.segments.get(self.cursor) {
                if let SlotType::FixedChar(_) = seg.slot { 
                    self.cursor += 1; 
                }
            }
        } else if let Some(InputEvent::MoveCursorBack) = event.downcast_ref::<InputEvent>() {
            self.cursor = self.cursor.saturating_sub(1);
            if let Some(seg) = self.segments.get(self.cursor) {
                if let SlotType::FixedChar(_) = seg.slot {
                    self.cursor = self.cursor.saturating_sub(1);
                }
            }
        } else if let Some(KeyboardEvent {state: KeyboardState::Pressed, key }) = event.downcast_ref() {
            match key {
                Key::Named(NamedKey::Delete | NamedKey::Backspace) => {
                    return vec![Box::new(InputEvent::NumericalInputEvent(self.cursor, NumericalInputEvent::Delete))];
                },
                Key::Character(c) => {
                    let c = c.to_string().chars().next().unwrap();
                    if let Some(seg) = self.segments.get(self.cursor) {
                        if let SlotType::FixedChar(_) = seg.slot { self.cursor += 1; }
                    }
                    match c {
                        '.' | '/' | ':' => {
                            for (i, segment) in self.segments.iter_mut().enumerate() {
                                if let SlotType::Triggered(trigger) = segment.slot {
                                    if i > self.cursor && c == trigger { self.cursor = i; }
                                }
                            }
                            return vec![Box::new(InputEvent::NumericalInputEvent(self.cursor, NumericalInputEvent::Special(c)))];
                        }
                        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                            return vec![Box::new(InputEvent::NumericalInputEvent(self.cursor, NumericalInputEvent::Digit(c)))];
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        } else if let Some(InputEvent::Triggered) = event.downcast_ref::<InputEvent>() {
            for seg in &mut self.segments {
                if let SlotType::GhostInput(_) = seg.slot {
                    seg.inner.display(true);
                }
            }
        }

        vec![event]
    }
}


#[derive(Debug, Clone)]
pub enum InputEvent {
    NumericalInputEvent(usize, NumericalInputEvent),
    MoveCursorForward,
    MoveCursorBack,
    Triggered,
}

impl Event for InputEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}


#[derive(Debug, Clone)]
pub enum NumericalInputEvent {
    Digit(char),
    Special(char),
    Delete,
}

impl Event for NumericalInputEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}
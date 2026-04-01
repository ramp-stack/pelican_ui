use prism::{event, event::{Event, OnEvent, Key, NamedKey, TickEvent}, display::{Opt, EitherOr}, Context, canvas::{Align, Image}, layout::{Stack, Row, Padding, Size, Offset, Column}, drawable::{Component, SizedTree}, emitters};
use crate::components::text::{Text, ExpandableText, TextStyle};
use crate::components::Icon;
use crate::theme::{Theme, Icons};
use ptsd::{colors, TextSize};

#[derive(Debug, Clone, Component)]
pub struct NumericalInput(Stack, _NumericalInput);
impl OnEvent for NumericalInput {}
impl NumericalInput {
    pub fn numerical(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::numerical(theme))
    }

    pub fn date(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::date(theme))
    }

    pub fn time(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::time(theme))
    }

    pub fn display(theme: &Theme, amount: f32, instructions: &str) -> Self {
        let input = _NumericalInput::new(theme, instructions, SlotDisplay::display(theme, amount));
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fit, Padding::default());
        NumericalInput(layout, input)
    }

    pub fn value(&self) -> String {
        let mut out = String::new();

        for slot in &self.1.inner.1.1 {
            let s = match slot.2.get_visual() {
                SlotVisual::Primary(s) | SlotVisual::Ghost(s) => s,
                SlotVisual::None => continue,
            };

            for ch in s.chars() {
                match ch.is_ascii_digit() || matches!(ch, '$' | '.' | ',' | '-' | ':' | '/') {
                    true => out.push(ch),
                    false => out.push('0'),
                }
            }
        }

        out
    }

    fn new(theme: &Theme, instructions: &str, input: SlotDisplay) -> Self {
        let input = _NumericalInput::new(theme, instructions, input);
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default());
        NumericalInput(layout, input)
    }

    pub fn error(&mut self, error: Result<String, String>) {
        self.1.error(error);
    }
}

#[derive(Clone, Debug, Component)]
pub struct _NumericalInput {
    layout: Column,
    inner: emitters::TextInput<SlotDisplay>, 
    _subtext: EitherOr<ExpandableText, NumericalInputError>, 
    #[skip] pub error: Option<String>,
    #[skip] pub subtext: (Option<String>, String),
}

impl OnEvent for _NumericalInput {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some(e) = &self.error {
                if !e.is_empty() {
                    self._subtext.right().2.spans[0] = e.to_string();
                    self._subtext.display_left(false); 
                } else {
                    self._subtext.left().0.spans[0] = self.subtext.1.to_string(); 
                    self._subtext.display_left(true); 
                }
            } else {
                if let Some(h) = &self.subtext.0 {
                    self._subtext.left().0.spans[0] = h.to_string(); 
                } else {
                    self._subtext.left().0.spans[0] = self.subtext.1.to_string(); 
                }

                self._subtext.display_left(true); 
            }
        }
        vec![event]
    }
}

impl _NumericalInput {
    fn new(theme: &Theme, instructions: &str, input: SlotDisplay) -> Self {
        let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::new(64.0), None);
        let help = ExpandableText::new(theme, instructions, TextSize::Lg, TextStyle::Secondary, Align::Center, None);
        let error = NumericalInputError::new(theme);
        _NumericalInput{ layout, inner: emitters::TextInput::new(input, false).1, _subtext: EitherOr::new(help, error), error: None, subtext: (None, instructions.to_string()) }
    }

    pub fn error(&mut self, error: Result<String, String>) {
        match error {
            Ok(help) => {self.error = None; self.subtext.0 = (!help.is_empty()).then_some(help); },
            Err(e) => self.error = Some(e),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct NumericalInputError(Row, Image, Text);
impl OnEvent for NumericalInputError {}
impl NumericalInputError {
    pub fn new(theme: &Theme) -> Self {
        let error = theme.colors().get(ptsd::Status::Danger);
        let icon = Icon::new(theme, Icons::Error, Some(error), 18.0);
        let text = Text::new(theme, "", TextSize::Lg, TextStyle::Error, Align::Center, None);
        NumericalInputError(Row::center(8.0), icon, text)
    }
}

#[derive(Clone, Debug, Component)]
pub struct SlotDisplay(Row, Vec<Slot>, #[skip] bool);
impl SlotDisplay {
    pub fn numerical(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::Fixed('$')),
            Slot::new(theme, SlotType::InputWithDefault(String::new(), 6, '0', InputFormat::Numerical)), 
            Slot::new(theme, SlotType::TriggersGhost('.', false)), 
            Slot::new(theme, SlotType::TriggeredGhostInputWithDefault(String::new(), 1, '0', false)),
            Slot::new(theme, SlotType::TriggeredGhostInputWithDefault(String::new(), 1, '0', false)),
        ];

        SlotDisplay(Row::center(0.0), slots, true)
    }

    pub fn display(theme: &Theme, amount: f32) -> Self {
        let chars = format!("{:.2}", amount).chars().collect::<Vec<char>>();
        let mut slots = vec![Slot::new(theme, SlotType::Fixed('$'))];
        chars.into_iter().for_each(|c| slots.push(Slot::new(theme, SlotType::Fixed(c))));
        SlotDisplay(Row::center(0.0), slots, false)
    }

    pub fn date(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'D')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'D')),
            Slot::new(theme, SlotType::Fixed('-')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'M')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'M')),
        ];

        SlotDisplay(Row::center(0.0), slots, true)
    }

    pub fn time(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
            Slot::new(theme, SlotType::Fixed(':')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
        ];

        SlotDisplay(Row::center(0.0), slots, true)
    }
}

impl OnEvent for SlotDisplay { 
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> { 
        if let Some(e) = event.downcast_ref::<event::TextInput>() && self.2 && let event::TextInput::Edited(key) = e {
            let mut reversed = false;
            let mut slots: Vec<Slot> = vec![];
            let mut start = 0;

            match key {
                Key::Named(NamedKey::Delete) => {
                    reversed = true;
                    slots = self.1.clone().into_iter().rev().collect::<Vec<_>>();
                },
                _ => {
                    slots = self.1.clone();
                    for (i, slot) in slots.iter_mut().enumerate() {
                        if let SlotType::TriggersGhost(_, is_on) = slot.2 && is_on {
                            start = i;
                        }
                    }
                }
            }

            for (i, slot) in slots.iter_mut().enumerate() {
                if start != 0 && i < start {continue;}
                let mut edited = false;
                match key {
                    Key::Named(NamedKey::Delete) => {
                        match &mut slot.2 {
                            SlotType::TriggersGhost(_, is_on) if *is_on => {
                                *is_on = false;
                                edited = true;
                            },
                            SlotType::InputWithDefault(inputs, _, _, _) => {if inputs.pop().is_some() { edited = true; }},
                            SlotType::GhostInputWithDefault(inputs, _, _) => {if inputs.pop().is_some() { edited = true; }},
                            SlotType::TriggeredGhostInputWithDefault(inputs, _, _, _) => {if inputs.pop().is_some() { edited = true; }}
                            _ => {} // later...
                        }
                    },
                    Key::Character(character) => {
                        let character = character.chars().next().unwrap();
                        if matches!(character, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') {
                            match &mut slot.2 {
                                SlotType::InputWithDefault(inputs, limit, default, _) => {
                                    if inputs.len() < *limit && (!inputs.is_empty() || *default != character) {
                                        inputs.push(character);
                                        edited = true;
                                    }
                                },
                                SlotType::GhostInputWithDefault(inputs, limit, _) => {
                                    if inputs.len() < *limit {
                                        inputs.push(character);
                                        edited = true;
                                    }
                                },
                                SlotType::TriggeredGhostInputWithDefault(inputs, limit, default, is_on) if *is_on => {
                                    if inputs.len() < *limit && (!inputs.is_empty() || *default != character) {
                                        inputs.push(character);
                                        edited = true;
                                    }
                                },
                                _ => {} // later...
                            }
                        } else if let SlotType::TriggersGhost(trigger, is_on) = &mut slot.2 && *trigger == character {
                            *is_on = true;
                            edited = true;
                        }
                    },
                    _ => {} // don't care
                }

                if edited {
                    match slot.2.get_visual() {
                        SlotVisual::Primary(v) => {
                            slot.1.inner().left().spans[0] = v;
                            slot.1.inner().display_left(true);
                            slot.1.display(true);
                        },
                        SlotVisual::Ghost(v) => {
                            slot.1.inner().right().spans[0] = v;
                            slot.1.inner().display_left(false);
                            slot.1.display(true);
                        },
                        SlotVisual::None => slot.1.display(false),
                    }
                }

                if edited {break;}
            }

            if reversed {slots = slots.into_iter().rev().collect::<Vec<_>>();}
            self.1 = slots;

            let mut triggered = false;
            for slot in &mut self.1 {
                if let SlotType::TriggersGhost(_, is_on) = slot.2 {
                    triggered = is_on;
                } else if let SlotType::TriggeredGhostInputWithDefault(_, _, _, is_on) = &mut slot.2 {
                    *is_on = triggered;
                    match slot.2.get_visual() {
                        SlotVisual::Primary(v) => {
                            slot.1.inner().left().spans[0] = v;
                            slot.1.inner().display_left(true);
                            slot.1.display(true);
                        },
                        SlotVisual::Ghost(v) => {
                            slot.1.inner().right().spans[0] = v;
                            slot.1.inner().display_left(false);
                            slot.1.display(true);
                        },
                        SlotVisual::None => {
                            slot.1.display(false);
                        }
                    }
                }
            }
        }
        vec![event]
    }
}

#[derive(Clone, Debug, Component)]
pub struct Slot(Stack, Opt<EitherOr<Text, Text>>, #[skip] SlotType); // text<primary, ghost>
impl OnEvent for Slot {}
impl Slot {
    pub fn new(theme: &Theme, ty: SlotType) -> Self {
        let ghost = theme.colors().get(colors::Text::Secondary);
        let (display_left, show_opt, text) = match ty.get_visual() {
            SlotVisual::Primary(s) => (true, true, s.to_string()),
            SlotVisual::Ghost(s) => (false, true, s.to_string()),
            SlotVisual::None => (true, false, String::new()),
        };

        let mut eo = EitherOr::new(
            Text::new(theme, &text, TextSize::H1, TextStyle::Heading, Align::Left, None),
            Text::new(theme, &text, TextSize::H1, TextStyle::Label(ghost), Align::Left, None),
        );

        eo.display_left(display_left);
        Slot(Stack::default(), Opt::new(eo, show_opt), ty)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SlotType {
    Fixed(char), // $ (cannot be deleted)
    TriggersGhost(char, bool), // .
    GhostInputWithDefault(String, usize, char), // 0
    TriggeredGhostInputWithDefault(String, usize, char, bool), // 0
    ConditionalVisual(char), // , / - (add conditional callback here)
    InputWithDefault(String, usize, char, InputFormat), // 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    ConditionalInput(Option<char>), // (add contditianal callback here)
}

impl SlotType {
    pub fn get_visual(&self) -> SlotVisual {
        match self {
            SlotType::Fixed(c) => SlotVisual::Primary(c.to_string()),
            SlotType::TriggersGhost(c, i) => match i {
                true => SlotVisual::Primary(c.to_string()),
                false => SlotVisual::None,
            },
            SlotType::TriggeredGhostInputWithDefault(chars, _, default, is_on) => {
                match is_on {
                    true => match chars.to_string().is_empty() {
                        true => SlotVisual::Ghost(default.to_string()),
                        false => SlotVisual::Primary(chars.to_string()),
                    },
                    false => SlotVisual::None,
                }
            },
            SlotType::GhostInputWithDefault(chars, _, default) => {
                match chars.to_string().is_empty() {
                    true => SlotVisual::Ghost(default.to_string()),
                    false => SlotVisual::Primary(chars.to_string()),
                }
            },
            SlotType::ConditionalVisual(c) => SlotVisual::Primary(c.to_string()), // check cond
            SlotType::InputWithDefault(chars, _, default, format) => {
                SlotVisual::Primary(match chars.to_string().is_empty() {
                    true => default.to_string(),
                    false => {
                        match format {
                            InputFormat::Numerical => chars.chars().rev().collect::<Vec<char>>().chunks(3).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join(",").chars().rev().collect::<String>(),
                            InputFormat::Date => chars.chars().rev().collect::<Vec<char>>().chunks(2).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join("/").chars().rev().collect::<String>(),
                            _ => chars.to_string()
                        }
                    }
                })
            },
            SlotType::ConditionalInput(c) => match c {
                Some(character) => SlotVisual::Primary(character.to_string()),
                None => SlotVisual::None,
            },
        }
    }
}

pub enum SlotVisual {
    Ghost(String),
    Primary(String),
    None,
}

#[derive(Eq, Clone, Debug, PartialEq)]
pub enum InputFormat {
    Numerical,
    Date,
    Time,
}
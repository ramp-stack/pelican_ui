use prism::events::{DynEvent, Key, ChangeSource, OnChange, on_change};
use prism::drawable::{Component, text::Align};
use prism::Context;
use ptsd::{events as event, emitters};
use ptsd::display::{Opt, EitherOr};
use ptsd::layout::{Stack, Row, Padding, Size, Offset, Column};
use ptsd::{colors, TextSize};
use uuid::Uuid;

use std::rc::Rc;
use std::pin::Pin;

use crate::components::text::{Text, ExpandableText, TextStyle};
use crate::components::Icon;
use crate::theme::{Theme, Icons};
use crate::interface::general::FormEvent;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SlotType {
    Fixed(char),
    TriggersGhost(char, bool),
    GhostInputWithDefault(String, usize, char),
    TriggeredGhostInputWithDefault(String, usize, char, bool),
    ConditionalVisual(char),
    InputWithDefault(String, usize, char, InputFormat),
    ConditionalInput(Option<char>),
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
            SlotType::ConditionalVisual(c) => SlotVisual::Primary(c.to_string()),
            SlotType::InputWithDefault(chars, _, default, format) => {
                SlotVisual::Primary(match chars.to_string().is_empty() {
                    true => default.to_string(),
                    false => {
                        match format {
                            InputFormat::Numerical => chars.chars().rev().collect::<Vec<char>>()
                                .chunks(3).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>()
                                .join(",").chars().rev().collect::<String>(),
                            InputFormat::Date => chars.chars().rev().collect::<Vec<char>>()
                                .chunks(2).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>()
                                .join("/").chars().rev().collect::<String>(),
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

pub enum SlotVisual {Ghost(String), Primary(String), None}

#[derive(Eq, Clone, Debug, PartialEq)]
pub enum InputFormat {Numerical, Date, Time}

/// A fixed number or a number calculated from a changing source.
#[derive(Clone)]
pub struct Number {
    value: f64,
    source: Option<Box<dyn NumberSource>>,
}
impl Number {
    /// The callback runs initially and after notifications. `latest` is initially `None`.
    pub fn watch<S: ChangeSource + Clone, F: FnMut(&S, Option<&S::Update>) -> f64 + Clone + 'static>(source: S, value: F) -> Self {
        let mut source = WatchedNumber {source, latest: None, value};
        Self {value: source.value(), source: Some(Box::new(source))}
    }

    pub fn value(&mut self) -> f64 {
        if let Some(source) = &mut self.source {self.value = source.value();}
        self.value
    }
}
impl From<f64> for Number {
    fn from(value: f64) -> Self {Self {value, source: None}}
}
impl std::fmt::Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Number").field("value", &self.value).finish_non_exhaustive()
    }
}
impl ChangeSource for Number {
    type Update = f64;
    async fn next(&mut self) -> Option<f64> {
        self.source.as_mut()?.next().await?;
        Some(self.value())
    }
}

#[derive(Debug, Clone, Component)]
pub struct NumericalInput(Stack, _NumericalInput);
impl NumericalInput {
    pub fn numerical(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::currency(theme))
    }

    pub fn date(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::date(theme))
    }

    pub fn time(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::time(theme))
    }

    pub fn number(theme: &Theme, instructions: &str) -> Self {
        NumericalInput::new(theme, instructions, SlotDisplay::number(theme))
    }

    pub fn display_currency(theme: &Theme, amount: impl Into<Number>, instructions: &str) -> Self {
        let input = _NumericalInput::new(theme, instructions, SlotDisplay::display_currency(theme, amount));
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fit, Padding::default());
        NumericalInput(layout, input)
    }

    pub fn display_number(theme: &Theme, amount: impl Into<Number>, instructions: &str) -> Self {
        let input = _NumericalInput::new(theme, instructions, SlotDisplay::display_number(theme, amount));
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fit, Padding::default());
        NumericalInput(layout, input)
    }

    pub fn value(&self) -> String {
        let mut out = String::new();

        for slot in &self.1.inner.1.1 {
            if let SlotType::GhostInputWithDefault(inputs, limit, _) = &slot.2
                && inputs.chars().count() < *limit {
                return String::new();
            }

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

    pub fn error(&mut self, error: Result<String, String>) {self.1.error(error);}

    fn new(theme: &Theme, instructions: &str, input: SlotDisplay) -> Self {
        let input = _NumericalInput::new(theme, instructions, input);
        let layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default());
        NumericalInput(layout, input)
    }
}

#[derive(Clone, Debug, Component)]
pub struct _NumericalInput {
    layout: Column,
    inner: emitters::TextInput<SlotDisplay>,
    _subtext: EitherOr<ExpandableText, NumericalInputError>,
    #[skip] error: Option<String>,
    #[skip] subtext: (Option<String>, String),
}

impl _NumericalInput {
    pub fn error(&mut self, error: Result<String, String>) {
        match error {
            Ok(help) => {
                let help = (!help.is_empty()).then_some(help);
                if self.error.is_none() && self.subtext.0 == help {return}
                self.error = None;
                self.subtext.0 = help;
            },
            Err(error) => {
                if self.error.as_ref() == Some(&error) {return}
                self.error = Some(error);
            },
        }
        if let Some(error) = self.error.as_ref().filter(|error| !error.is_empty()) {
            self._subtext.right().2.spans[0].text.clone_from(error);
            self._subtext.display_left(false);
        } else {
            let help = if self.error.is_none() {self.subtext.0.as_ref()} else {None};
            self._subtext.left().0.spans[0].text.clone_from(help.unwrap_or(&self.subtext.1));
            self._subtext.display_left(true);
        }
    }

    fn new(theme: &Theme, instructions: &str, input: SlotDisplay) -> Self {
        let layout = Column::new(24.0, Offset::Center, Size::Fit, Padding::new(64.0), None);
        let help = ExpandableText::new(
            theme, instructions, TextSize::Lg, TextStyle::Secondary, Align::Center, None,
        );
        let error = NumericalInputError::new(theme);
        _NumericalInput {
            layout, inner: emitters::TextInput::new(input, false).1,
            _subtext: EitherOr::new(help, error), error: None, subtext: (None, instructions.to_string()),
        }
    }
}

#[derive(Clone, Debug, Component)]
pub struct NumericalInputError(Row, Icon, Text);
impl NumericalInputError {
    pub fn new(theme: &Theme) -> Self {
        let error = theme.colors().get(ptsd::Status::Danger);
        let icon = Icon::new(theme, Icons::Error, Some(error), 18.0);
        let text = Text::new(theme, "", TextSize::Lg, TextStyle::Error, Align::Center, None);
        NumericalInputError(Row::center(8.0), icon, text)
    }
}

#[derive(Clone, Debug, Component)]
#[on_dyn_event]
pub struct SlotDisplay(
    Row, Vec<Slot>, #[skip] bool, #[skip] Option<Number>,
    #[skip] bool, #[skip] Theme, #[skip] Option<Uuid>,
);
impl SlotDisplay {
    pub fn currency(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::Fixed('$')),
            Slot::new(theme, SlotType::InputWithDefault(String::new(), 6, '0', InputFormat::Numerical)),
            Slot::new(theme, SlotType::TriggersGhost('.', false)),
            Slot::new(theme, SlotType::TriggeredGhostInputWithDefault(String::new(), 1, '0', false)),
            Slot::new(theme, SlotType::TriggeredGhostInputWithDefault(String::new(), 1, '0', false)),
        ];

        SlotDisplay(Row::center(0.0), slots, true, None, false, theme.clone(), None)
    }

    pub fn number(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::InputWithDefault(String::new(), 6, '0', InputFormat::Numerical)),
            Slot::new(theme, SlotType::TriggersGhost('.', false)),
            Slot::new(theme, SlotType::TriggeredGhostInputWithDefault(String::new(), 1, '0', false)),
            Slot::new(theme, SlotType::TriggeredGhostInputWithDefault(String::new(), 1, '0', false)),
        ];

        SlotDisplay(Row::center(0.0), slots, true, None, false, theme.clone(), None)
    }

    pub fn display_currency(theme: &Theme, amount: impl Into<Number>) -> Self {
        Self::display(theme, amount.into(), true)
    }

    pub fn display_number(theme: &Theme, number: impl Into<Number>) -> Self {
        Self::display(theme, number.into(), false)
    }

    pub fn date(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'D')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'D')),
            Slot::new(theme, SlotType::Fixed('-')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'M')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, 'M')),
        ];

        SlotDisplay(Row::center(0.0), slots, true, None, false, theme.clone(), None)
    }

    pub fn time(theme: &Theme) -> Self {
        let slots = vec![
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
            Slot::new(theme, SlotType::Fixed(':')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
            Slot::new(theme, SlotType::GhostInputWithDefault(String::new(), 1, '0')),
        ];

        SlotDisplay(Row::center(0.0), slots, true, None, false, theme.clone(), None)
    }
}

#[on_change(ChangeSource::changed(&mut self.3))]
impl OnChange<f64> for SlotDisplay {
    fn on_change(&mut self, ctx: &mut Context, value: f64) {
        if self.show(value) && let Some(form) = self.6 {ctx.emit(FormEvent::Refresh(form));}
    }
}

impl SlotDisplay {
    fn display(theme: &Theme, mut number: Number, currency: bool) -> Self {
        let value = number.value();
        let mut display = Self(Row::center(0.0), vec![], false, Some(number), currency, theme.clone(), None);
        display.show(value);
        display
    }

    fn show(&mut self, value: f64) -> bool {
        let text = if self.4 {format!("${value:.2}")} else {value.to_string()};
        if self.1.iter().map(|slot| slot.2.clone()).eq(text.chars().map(SlotType::Fixed)) {
            return false;
        }
        self.1 = text.chars().map(|c| Slot::new(&self.5, SlotType::Fixed(c))).collect();
        true
    }

    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(FormEvent::Bind(form)) = event.downcast_ref::<FormEvent>() {
            self.6 = Some(*form);
        } else if let Some(e) = event.downcast_ref::<event::TextInput>() && self.2
            && let event::TextInput::Edited(key) = e
        {
            let mut changed = false;
            let mut reversed = false;
            let mut slots: Vec<Slot> = vec![];
            let mut start = 0;

            match key {
                Key::Delete | Key::Backspace => {
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
                    Key::Delete | Key::Backspace => {
                        match &mut slot.2 {
                            SlotType::TriggersGhost(_, is_on) if *is_on => {
                                *is_on = false;
                                edited = true;
                            },
                            SlotType::InputWithDefault(inputs, _, _, _)
                                | SlotType::GhostInputWithDefault(inputs, _, _)
                                | SlotType::TriggeredGhostInputWithDefault(inputs, _, _, _) => {
                                if inputs.pop().is_some() {edited = true;}
                            }
                            _ => {}
                        }
                    },
                    Key::Character(character) => {
                        let character = character.clone();
                        if matches!(character.as_str(),
                            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                        {
                            match &mut slot.2 {
                                SlotType::InputWithDefault(inputs, limit, default, _) => {
                                    if inputs.len() < *limit
                                        && (!inputs.is_empty() || *default.to_string() != character)
                                    {
                                        inputs.push(character.chars().next().unwrap());
                                        edited = true;
                                    }
                                },
                                SlotType::GhostInputWithDefault(inputs, limit, _) => {
                                    if inputs.len() < *limit {
                                        inputs.push(character.chars().next().unwrap());
                                        edited = true;
                                    }
                                },
                                SlotType::TriggeredGhostInputWithDefault(inputs, limit, _, is_on)
                                    if *is_on && inputs.len() < *limit =>
                                {
                                    inputs.push(character.chars().next().unwrap());
                                    edited = true;
                                },
                                _ => {}
                            }
                        } else if let SlotType::TriggersGhost(trigger, is_on) = &mut slot.2
                            && *trigger.to_string() == character
                        {
                            *is_on = true;
                            edited = true;
                        }
                    },
                    _ => {}
                }

                if edited {
                    match slot.2.get_visual() {
                        SlotVisual::Primary(v) => {
                            slot.1.inner().left().spans[0].text = v;
                            slot.1.inner().display_left(true);
                            slot.1.display(true);
                        },
                        SlotVisual::Ghost(v) => {
                            slot.1.inner().right().spans[0].text = v;
                            slot.1.inner().display_left(false);
                            slot.1.display(true);
                        },
                        SlotVisual::None => slot.1.display(false),
                    }
                }

                if edited {changed = true; break;}
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
                            slot.1.inner().left().spans[0].text = v;
                            slot.1.inner().display_left(true);
                            slot.1.display(true);
                        },
                        SlotVisual::Ghost(v) => {
                            slot.1.inner().right().spans[0].text = v;
                            slot.1.inner().display_left(false);
                            slot.1.display(true);
                        },
                        SlotVisual::None => {
                            slot.1.display(false);
                        }
                    }
                }
            }
            if changed && let Some(form) = self.6 {ctx.emit(FormEvent::Refresh(form));}
        }
        vec![event]
    }
}

#[derive(Clone, Debug, Component)]
pub struct Slot(Stack, Opt<EitherOr<Text, Text>>, #[skip] SlotType);
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

trait NumberSource {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + '_>>;
    fn value(&mut self) -> f64;
    fn clone_box(&self) -> Box<dyn NumberSource>;
}
impl<S: ChangeSource + Clone, F: FnMut(&S, Option<&S::Update>) -> f64 + Clone + 'static>
    NumberSource for WatchedNumber<S, F> {
    fn next(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + '_>> {
        Box::pin(ChangeSource::next(self))
    }
    fn value(&mut self) -> f64 {(self.value)(&self.source, self.latest.as_deref())}
    fn clone_box(&self) -> Box<dyn NumberSource> {Box::new(self.clone())}
}
impl Clone for Box<dyn NumberSource> {
    fn clone(&self) -> Self {self.as_ref().clone_box()}
}

struct WatchedNumber<S: ChangeSource, F> {
    source: S,
    latest: Option<Rc<S::Update>>,
    value: F,
}
impl<S: ChangeSource + Clone, F: Clone> Clone for WatchedNumber<S, F> {
    fn clone(&self) -> Self {
        Self {source: self.source.clone(), latest: self.latest.clone(), value: self.value.clone()}
    }
}
impl<S: ChangeSource, F: 'static> ChangeSource for WatchedNumber<S, F> {
    type Update = ();
    async fn next(&mut self) -> Option<()> {
        self.latest = Some(Rc::new(self.source.next().await?));
        Some(())
    }
}

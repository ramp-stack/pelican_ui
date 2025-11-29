use roost_ui::{Context, Component, IS_MOBILE};
use roost_ui::events::OnEvent;
use roost_ui::drawable::Align;
use roost_ui::layouts::{Bin, Stack, Column};

use crate::components::Keypad;
use crate::components::text::{Text, TextStyle, TextSize};
use crate::interactions::{SlotType, self};

#[derive(Debug, Component)]
pub struct NumericalInput(Column, Bin<Stack, _NumericalInput>, Option<Keypad>);
impl OnEvent for NumericalInput {}

impl NumericalInput {
    pub fn display(ctx: &mut Context, amount: f32, instructions: &str) -> Self {
        let keypad = IS_MOBILE.then_some(Keypad::new(ctx, '.'));
        NumericalInput(Column::center(0.0), Bin(Stack::fill(), _NumericalInput::Currency(CurrencyInput::new(ctx, '$', instructions))), keypad)
    }

    pub fn currency(ctx: &mut Context, instructions: &str) -> Self {
        let keypad = IS_MOBILE.then_some(Keypad::new(ctx, '.'));
        NumericalInput(Column::center(0.0), Bin(Stack::fill(), _NumericalInput::Currency(CurrencyInput::new(ctx, '$', instructions))), keypad)
    }

    pub fn date(ctx: &mut Context, instructions: &str) -> Self {
        let keypad = IS_MOBILE.then_some(Keypad::new(ctx, '/'));
        NumericalInput(Column::center(0.0), Bin(Stack::fill(), _NumericalInput::Date(DateInput::new(ctx, instructions))), keypad)
    }

    pub fn time(ctx: &mut Context, instructions: &str) -> Self {
        let keypad = IS_MOBILE.then_some(Keypad::new(ctx, ':'));
        NumericalInput(Column::center(0.0), Bin(Stack::fill(), _NumericalInput::Time(TimeInput::new(ctx, instructions))), keypad)
    }

    pub fn value(&mut self) -> String {
        match &mut self.1.inner() {
            _NumericalInput::Currency(v) => v.1.value(),
            _NumericalInput::Date(v) => v.1.value(),
            _NumericalInput::Time(v) => v.1.value(),
        }
    }
}

#[derive(Debug, Component)]
pub enum _NumericalInput {
    Currency(CurrencyInput),
    Date(DateInput),
    Time(TimeInput),
}

impl OnEvent for _NumericalInput {}


#[derive(Debug, Component)]
pub struct CurrencyInput(Column, interactions::NumericalInput, pub Text);
impl OnEvent for CurrencyInput {}
impl CurrencyInput {
    pub fn new(ctx: &mut Context, currency: char, instructions: &str) -> Self {
        let input = interactions::NumericalInput::new(ctx, vec![
            SlotType::FixedChar(currency), // This always shows and cannot be delted
            SlotType::Primary('0', 6), // this always shows, but can be replaced by another digit, when deleted on, it goes back to the char
            SlotType::Triggered('.'), // This shows up as primary only when the user types the decimal
            SlotType::GhostInput('0'), // this only shows up when it is 'next' and is replaced by primary texto n input 
            SlotType::GhostInput('0'), 
        ]);

        
        let text = Text::new(ctx, instructions, TextSize::Md, TextStyle::Secondary, Align::Left, None);
        CurrencyInput(Column::center(8.0), input, text)
    }
}

#[derive(Debug, Component)]
pub struct DateInput(Column, interactions::NumericalInput, pub Text);
impl OnEvent for DateInput {}
impl DateInput {
    pub fn new(ctx: &mut Context, instructions: &str) -> Self {
        let input = interactions::NumericalInput::new(ctx, vec![
            SlotType::Ghost('d', 1),
            SlotType::Ghost('d', 1),
            SlotType::FixedChar('/'),
            SlotType::Ghost('m', 1),
            SlotType::Ghost('m', 1),
            SlotType::FixedChar('/'),
            SlotType::Ghost('y', 1),
            SlotType::Ghost('y', 1),
            SlotType::Ghost('y', 1),
            SlotType::Ghost('y', 1),
        ]);

        
        let text = Text::new(ctx, instructions, TextSize::Md, TextStyle::Secondary, Align::Left, None);
        DateInput(Column::center(8.0), input, text)
    }
}

#[derive(Debug, Component)]
pub struct TimeInput(Column, interactions::NumericalInput, pub Text);
impl OnEvent for TimeInput {}
impl TimeInput {
    pub fn new(ctx: &mut Context, instructions: &str) -> Self {
        let input = interactions::NumericalInput::new(ctx, vec![
            SlotType::Ghost('0', 1),
            SlotType::Ghost('0', 1),
            SlotType::FixedChar(':'),
            SlotType::Ghost('0', 1),
            SlotType::Ghost('0', 1),
        ]);

        let text = Text::new(ctx, instructions, TextSize::Md, TextStyle::Secondary, Align::Left, None);
        TimeInput(Column::center(8.0), input, text)
    }
}
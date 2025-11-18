use roost_ui::{Context, Component};
use roost_ui::events::OnEvent;
use roost_ui::drawable::Align;
use roost_ui::layouts::{Stack, Column};

use crate::components::text::{Text, TextStyle, TextSize};
use crate::interactions::{SlotType, self};

#[derive(Debug, Component)]
pub struct NumericalInput(Stack, _NumericalInput);
impl OnEvent for NumericalInput {}

impl NumericalInput {
    pub fn currency(ctx: &mut Context, instructions: &str) -> Self {
        NumericalInput(Stack::fill(), _NumericalInput::Currency(CurrencyInput::new(ctx, '$', instructions)))
    }

    pub fn date(ctx: &mut Context, instructions: &str) -> Self {
        NumericalInput(Stack::fill(), _NumericalInput::Date(DateInput::new(ctx, instructions)))
    }

    pub fn time(ctx: &mut Context, instructions: &str) -> Self {
        NumericalInput(Stack::fill(), _NumericalInput::Time(TimeInput::new(ctx, instructions)))
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
            SlotType::Primary('0', 8), // this always shows, but can be replaced by another digit, when deleted on, it goes back to the char
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
            SlotType::Primary('0', 1),
            SlotType::Primary('0', 1),
            SlotType::FixedChar(':'),
            SlotType::Primary('0', 1),
            SlotType::Primary('0', 1),
        ]);

        let text = Text::new(ctx, instructions, TextSize::Md, TextStyle::Secondary, Align::Left, None);
        TimeInput(Column::center(8.0), input, text)
    }
}
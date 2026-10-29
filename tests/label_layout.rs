use pelican_ui::components::button::{ActionData, PrimaryButton, QuickActions};
use pelican_ui::components::DataItem;
use pelican_ui::components::text::{Text, TextSize, TextStyle};
use pelican_ui::theme::{Icons, Theme};
use prism::drawable::{Drawable, Instruction, Item};
use prism::drawable::text::Align;
use ptsd::layout::{Offset, Stack};

fn draw(component: &mut impl Drawable, available: f32) -> Vec<Instruction> {
    let (min, max) = component.request_width();
    let width = available.clamp(min, max);
    let (_, height) = component.request_height(width);
    let layout = component.layout((width, height));
    component.draw(&layout, (0.0, 0.0), (0.0, 0.0, width, height))
}

fn assert_single_line(instructions: &[Instruction], label: &str) {
    let text = instructions.iter().find_map(|instruction| match &instruction.item {
        Item::Text(text) if text.spans.iter().map(|span| span.text.as_str()).collect::<String>() == label => Some(text),
        _ => None,
    }).unwrap_or_else(|| panic!("{label:?} is missing from the rendered component"));
    assert_eq!(text.len(), 1, "{label:?} wraps at width {:?}, measured {:?}", text.width, text.size());
}

#[test]
fn save_button_label_fits_on_one_line() {
    let theme = Theme::default();
    let mut button = PrimaryButton::new(&theme, "Save", Stack::default(), |_, _| {});
    for width in [280.0, 327.0, 375.0] {
        assert_single_line(&draw(&mut button, width), "Save");
    }
}

#[test]
fn about_me_heading_fits_on_one_line() {
    let theme = Theme::default();
    let mut item = DataItem::text(&theme, "About me", None, "No bio yet.", None);
    for width in [280.0, 327.0, 375.0] {
        assert_single_line(&draw(&mut item, width), "About me");
    }
}

#[test]
fn quick_actions_render_their_labels_and_wrap_whole_buttons() {
    let theme = Theme::default();
    let mut actions = QuickActions::new(&theme, Offset::Start, vec![
        ActionData::new("Paste clipboard", Some("Pasted"), Icons::Paste, Box::new(|_, _| {})),
        ActionData::new("Scan QR code", None, Icons::QrCode, Box::new(|_, _| {})),
    ]);
    for width in [200.0, 327.0, 375.0] {
        let instructions = draw(&mut actions, width);
        assert_single_line(&instructions, "Paste clipboard");
        assert_single_line(&instructions, "Scan QR code");
        let positions: Vec<_> = instructions.iter().filter_map(|instruction| {
            matches!(&instruction.item, Item::Text(_)).then_some(instruction.offset.1)
        }).collect();
        if width == 200.0 {
            assert!(positions[1] > positions[0], "whole buttons should move to separate rows");
        } else if width == 375.0 {
            assert_eq!(positions[0], positions[1], "both buttons should fit on one row");
        }
    }
}

#[test]
fn fitted_text_still_wraps_when_space_is_actually_limited() {
    let theme = Theme::default();
    let mut text = Text::new(
        &theme, "A longer heading that needs another line", TextSize::H5,
        TextStyle::Heading, Align::Left, None,
    ).fit();
    let instructions = draw(&mut text, 100.0);
    let Item::Text(text) = &instructions[0].item else {panic!("text must render")};
    assert!(text.len() > 1);
    assert_eq!(text.width, Some(100.0));
}

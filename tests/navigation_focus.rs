use std::marker::PhantomData;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use pelican_ui::components::{TextInput, TextInputEvent};
use pelican_ui::theme::Theme;
use prism::drawable::{Component, Drawable};
use prism::events::{Event, Key, KeyboardEvent, KeyboardState, Modifiers};
use prism::{Context, Window, hardware, traits};
use ptsd::interfaces::{Body, Interface, ShowKeyboard};
use ptsd::layout::Stack;
use ptsd::navigation::{AppPage, Flow, FlowContainer, NavigationEvent, Pages};
use uuid::Uuid;

#[test]
fn mobile_navigation_centers_icons_and_keeps_the_entire_row_selectable() {
    use pelican_ui::interface::navigation::{Navigator, RootInfo};
    use pelican_ui::theme::Icons;
    use prism::drawable::Item;
    use prism::events::{MouseButton, MouseEvent, MouseState};

    let mut app = App::new(Platform::Mobile);
    for count in 1..=5 {
        let roots = (0..count).map(|index| RootInfo::icon(
            Icons::Profile, &index.to_string(), Box::new(InputPage::new(&app.theme)),
        )).collect();
        let mut navigator = Navigator::mobile(&app.theme, roots);
        for width in [320.0, 375.0, 430.0, 768.0] {
            if width < count as f32 * 100.0 {continue;}
            let (min, max) = navigator.request_width();
            assert!(min <= width && max >= width);
            let (_, height) = navigator.request_height(width);
            let layout = Drawable::layout(&mut navigator, (width, height));
            let instructions = navigator.draw(&layout, (0.0, 0.0), (0.0, 0.0, width, height));
            let centers: Vec<_> = instructions.iter().filter_map(|instruction| {
                if let Item::Svg(shape, ..) = &instruction.item {
                    Some(instruction.offset.0 + shape.width / 2.0)
                } else {None}
            }).collect();
            assert_eq!(centers.len(), count);
            let first_center = (width - (count - 1) as f32 * 100.0) / 2.0;
            for (index, center) in centers.iter().enumerate() {
                assert!((center - (first_center + index as f32 * 100.0)).abs() < 0.01);
            }

            // Check actual event routing across the whole row, including the
            // vertical padding and the expanded areas outside the icon group.
            for y in [0.5, height / 2.0, height - 0.5] {
                for x in 0..width as usize {
                    let x = x as f32 + 0.25;
                    app.context.emit(MouseEvent {
                        position: Some((x, y)), state: MouseState::Pressed(MouseButton::Left),
                    });
                    let mut pending = app.context.events();
                    let mut selected = Vec::new();
                    while let Some(event) = pending.pop_front() {
                        if let Some(NavigationEvent::Root(Some(root))) = event.downcast_ref() {
                            selected.push(root.clone());
                        } else {
                            navigator.event(&mut app.context, &layout, event);
                        }
                        pending.extend(app.context.events());
                    }
                    let expected = ((x - first_center) / 100.0).round()
                        .clamp(0.0, (count - 1) as f32) as usize;
                    assert_eq!(selected, vec![expected.to_string()], "tap at ({x}, {y}), width {width}");
                }
            }
        }
    }
}

#[derive(Clone, Default)]
struct TestKeyboard(Arc<AtomicBool>);

impl traits::Keyboard for TestKeyboard {
    fn show(&self, _: f32) {self.0.store(true, Ordering::Relaxed);}
    fn hide(&self) {self.0.store(false, Ordering::Relaxed);}
    fn visible(&self) -> bool {self.0.load(Ordering::Relaxed)}
}

#[derive(Clone, Default)]
struct TestHardware(TestKeyboard);

struct CancelledPick<T>(PhantomData<T>);
impl<T: Clone + 'static> traits::Picked<T> for CancelledPick<T> {
    async fn get(&mut self) -> hardware::PickResult<T> {Err(hardware::Cancelled)}
}
impl<T: Clone + 'static> traits::Picker<T> for TestHardware {
    type Picked = CancelledPick<T>;
    fn pick(&self) -> Self::Picked {CancelledPick(PhantomData)}
}

impl traits::Hardware for TestHardware {
    fn keyboard(&self) -> hardware::Keyboard {hardware::Keyboard::new(self.0.clone())}
    fn player(&self, _: media::Player) -> Result<hardware::Player, String> {
        panic!("navigation tests must not start media playback")
    }
    fn start_camera(&self) -> Result<hardware::Camera, String> {panic!("unexpected camera")}
    fn start_microphone(&self) -> Result<hardware::Microphone, String> {panic!("unexpected microphone")}
    fn start_capture(&self) -> Result<hardware::Capture, String> {panic!("unexpected capture")}
    fn share(&self, _: &hardware::Data) {panic!("unexpected share")}
    fn set_clipboard(&self, _: &hardware::Data) {panic!("unexpected clipboard write")}
    fn get_clipboard(&self) -> Option<hardware::Data> {None}
    fn set_cache(&self, _: &str, _: &hardware::Data) -> Result<(), String> {Ok(())}
    fn get_cache(&self, _: &str) -> Option<hardware::Data> {None}
    fn trigger_haptic(&self) {}
}

#[derive(Clone)]
struct TestAir;
impl traits::Air for TestAir {
    fn me(&self) -> prism::air::Name {panic!("navigation tests must not access Air")}
    fn instances(&self, _: prism::air::ContractType) -> Box<dyn traits::DynInstances> {
        panic!("navigation tests must not access Air")
    }
    fn start(&self, _: prism::service::ServiceInit) {panic!("unexpected service")}
    fn shutdown(&self, _: prism::service::Key) {panic!("unexpected service")}
}

#[derive(Clone, Debug, Component)]
struct InputPage(Stack, TextInput);
impl InputPage {
    fn new(theme: &Theme) -> Self {
        Self(Stack::default(), TextInput::new(
            theme, Uuid::new_v4(), None, None, None, None, None,
        ))
    }
}
impl AppPage for InputPage {}

#[derive(Clone, Debug, Component)]
struct TestBody(Stack, Pages);
impl Body for TestBody {
    fn pages(&mut self) -> &mut Pages {&mut self.1}
}

#[derive(Clone, Debug, Component)]
struct TestFlow(Stack, Flow);
impl FlowContainer for TestFlow {
    fn flow(&mut self) -> &mut Flow {&mut self.1}
}

#[derive(Clone, Copy, Debug)]
enum Platform {Mobile, Desktop, Web}

struct App {
    interface: Interface,
    context: Context,
    theme: Theme,
}

impl App {
    fn new(platform: Platform) -> Self {
        let theme = Theme::default();
        let pages = Pages::new(vec![
            ("first".into(), Box::new(InputPage::new(&theme))),
            ("second".into(), Box::new(InputPage::new(&theme))),
        ]);
        let body = TestBody(Stack::default(), pages);
        let interface = match platform {
            Platform::Mobile => Interface::mobile(None, body, None),
            Platform::Desktop => Interface::desktop(None, body),
            Platform::Web => Interface::web(None, body),
        };
        let context = Context::new(
            TestHardware::default(), prism::air::Air::new(TestAir),
            Window {size: (400.0, 800.0), is_mobile: matches!(platform, Platform::Mobile), ..Window::default()},
        );
        Self {interface, context, theme}
    }

    // Exercise normal Drawable event routing and queued follow-up events, including
    // navigation storing pages that no longer receive events while hidden.
    fn send(&mut self, event: impl Event) {
        self.context.emit(event);
        let mut pending = self.context.events();
        while let Some(event) = pending.pop_front() {
            let (min, max) = self.interface.request_width();
            let width = self.context.window.size.0.clamp(min, max);
            let (min, max) = self.interface.request_height(width);
            let height = self.context.window.size.1.clamp(min, max);
            let layout = Drawable::layout(&mut self.interface, (width, height));
            self.interface.event(&mut self.context, &layout, event);
            pending.extend(self.context.events());
        }
    }

    fn input(&mut self) -> &mut TextInput {
        &mut self.interface.pages().current().as_mut()
            .downcast_mut::<InputPage>().unwrap().1
    }

    fn focus(&mut self) {
        let id = self.input().id();
        self.send(TextInputEvent::Focus {id});
        assert!(self.input().has_focus());
        assert!(self.input().inner.is_focused());
    }

    fn type_text(&mut self, text: &str) {
        self.send(KeyboardEvent {
            key: Key::Character(text.to_owned()), state: KeyboardState::Pressed,
            modifiers: Modifiers::default(),
        });
    }

    fn assert_blurred_draft(&mut self, draft: &str) {
        assert!(!self.input().has_focus(), "input retained logical focus");
        assert!(!self.input().inner.is_focused(), "input retained its focus decoration");
        assert!(!self.input().focused, "input retained its text cursor");
        assert_eq!(self.input().value(), draft);
        self.type_text("unexpected");
        assert_eq!(self.input().value(), draft, "unfocused input accepted keyboard events");
    }

    fn flow(&self) -> TestFlow {
        TestFlow(Stack::default(), Flow::new(vec![
            Box::new(InputPage::new(&self.theme)), Box::new(InputPage::new(&self.theme)),
        ]))
    }

    #[cfg(target_os = "macos")]
    fn caret_x(&mut self) -> Option<f32> {
        let input = self.input();
        input.request_width();
        let (_, height) = input.request_height(300.0);
        let layout = Drawable::layout(input, (300.0, height));
        input.draw(&layout, (0.0, 0.0), (0.0, 0.0, 300.0, height))
            .into_iter().find_map(|instruction| match instruction.item {
                prism::drawable::Item::Shape(shape) if shape.width == 2.0 => Some(instruction.offset.0),
                _ => None,
            })
    }
}

#[cfg(target_os = "macos")]
#[test]
fn mac_cursor_is_drawn_when_focused_and_returns_after_navigation() {
    let mut app = App::new(Platform::Desktop);
    assert!(app.caret_x().is_none());
    app.focus();
    let start = app.caret_x().expect("focused empty input must draw a cursor");
    app.type_text("draft");
    assert!(app.caret_x().expect("typing must keep the cursor visible") > start);

    let id = app.input().id();
    app.input().anchor = Some(0);
    app.send(TextInputEvent::Refresh {id});
    assert!(app.caret_x().is_none(), "selected text must not show an insertion cursor");
    app.input().anchor = None;
    app.send(TextInputEvent::Refresh {id});
    assert!(app.caret_x().is_some());

    app.send(NavigationEvent::root("second"));
    app.send(NavigationEvent::root("first"));
    app.assert_blurred_draft("draft");
    assert!(app.caret_x().is_none());
    app.focus();
    assert!(app.caret_x().is_some(), "refocusing must restore the visible cursor");
}

#[cfg(target_os = "macos")]
#[test]
fn cursor_blinks_resets_when_typing_and_stops_when_blurred() {
    use futures::{executor::block_on, FutureExt};
    use pelican_ui::components::text::TextEditor;
    use prism::events::{Timer, WindowEvent};
    use std::time::Duration;
    let mut app = App::new(Platform::Desktop);
    let mut editor = TextEditor::default(&app.theme);
    fn visible(editor: &mut TextEditor) -> bool {
        editor.request_width();
        let (_, h) = editor.request_height(300.0);
        let layout = editor.layout((300.0, h));
        editor.draw(&layout, (0.0, 0.0), (0.0, 0.0, 300.0, h)).iter().any(|i| {
            matches!(&i.item, prism::drawable::Item::Shape(shape) if shape.width == 2.0)
        })
    }
    let tick = |editor: &mut TextEditor, ctx: &mut Context| {
        let callback = block_on(async {
            let mut timeout = Timer::new(Duration::from_secs(2));
            futures::select! {
                callback = editor.changed().fuse() => callback,
                _ = timeout.next().fuse() => panic!("caret did not blink"),
            }
        });
        callback(editor, ctx);
    };
    editor.display_cursor(true);
    assert!(visible(&mut editor));
    tick(&mut editor, &mut app.context);
    assert!(!visible(&mut editor), "layout must not reset the blink timer");
    tick(&mut editor, &mut app.context);
    assert!(visible(&mut editor));
    tick(&mut editor, &mut app.context);
    editor.insert(&mut app.context, "typing");
    assert!(visible(&mut editor), "typing must immediately reveal the caret");
    let layout = editor.layout((300.0, 40.0));
    editor.event(&mut app.context, &layout, Box::new(WindowEvent::Focused(false)));
    assert!(!visible(&mut editor));
    assert!(editor.changed().now_or_never().is_none());
    editor.event(&mut app.context, &layout, Box::new(WindowEvent::Focused(true)));
    assert!(visible(&mut editor));
    editor.display_cursor(false);
    assert!(!visible(&mut editor));
    assert!(editor.changed().now_or_never().is_none());
}

#[test]
fn emoji_scroll_moves_continuously_horizontally_without_inserting_text() {
    use futures::FutureExt;
    use pelican_ui::interface::system::EmojiPanel;
    use prism::drawable::{Item, LayoutTree};
    use prism::events::{MouseButton, MouseEvent, MouseState};
    for platform in [Platform::Desktop, Platform::Mobile] {
    let mut app = App::new(platform);
    let mut panel = EmojiPanel::new(&app.theme);
    fn settle(panel: &mut EmojiPanel, ctx: &mut Context) -> LayoutTree {
        for _ in 0..10 {
            panel.request_width();
            let (_, height) = panel.request_height(375.0);
            let layout = Drawable::layout(panel, (375.0, height));
            if let Some(callback) = panel.changed().now_or_never() {callback(panel, ctx);}
            else {return layout;}
        }
        panic!("emoji viewport did not settle")
    }
    let first = settle(&mut panel, &mut app.context);
    let first_draw = panel.draw(&first, (0.0, 0.0), (0.0, 0.0, 375.0, 300.0));
    let emoji = first_draw.iter().find(|i| matches!(&i.item, Item::Text(_)) && i.offset.1 < 250.0 && i.offset.0 > 100.0).unwrap();
    let Item::Text(text) = &emoji.item else {unreachable!()};
    let id = text.id;
    let origin = emoji.offset;
    let delta = if matches!(platform, Platform::Mobile) {
        panel.event(&mut app.context, &first, Box::new(MouseEvent {
            position: Some((150.0, 100.0)), state: MouseState::Pressed(MouseButton::Left),
        }));
        20.5
    } else {7.5};
    panel.event(&mut app.context, &first, Box::new(MouseEvent {
        position: Some((150.0 - delta, 100.0)), state: MouseState::Scroll(delta, 0.0),
    }));
    let after = settle(&mut panel, &mut app.context);
    let drawing = panel.draw(&after, (0.0, 0.0), (0.0, 0.0, 375.0, 300.0));
    let moved = drawing.iter().find(|i| matches!(&i.item, Item::Text(text) if text.id == id)).unwrap();
    assert!((moved.offset.0 - (origin.0 - delta)).abs() < 0.01);
    assert_eq!(moved.offset.1, origin.1);
    panel.event(&mut app.context, &after, Box::new(MouseEvent {
        position: Some((150.0 - delta, 100.0)), state: MouseState::Released(MouseButton::Left),
    }));
    let mut events = app.context.events();
    while let Some(event) = events.pop_front() {
        assert!(!event.is::<KeyboardEvent>(), "scrolling must not select an emoji");
        panel.event(&mut app.context, &after, event);
        events.extend(app.context.events());
    }
    }
}

#[test]
fn returning_to_a_root_preserves_its_draft_without_restoring_focus() {
    for platform in [Platform::Mobile, Platform::Desktop, Platform::Web] {
        let mut app = App::new(platform);
        app.focus();
        app.type_text("root draft");
        app.send(NavigationEvent::root("second"));
        app.focus();
        app.type_text("other draft");
        app.send(NavigationEvent::root("first"));
        app.assert_blurred_draft("root draft");
        app.send(NavigationEvent::root("second"));
        app.assert_blurred_draft("other draft");
    }
}

#[test]
fn flow_next_back_and_history_restore_blurred_inputs() {
    let mut app = App::new(Platform::Mobile);
    app.focus();
    app.type_text("root draft");
    app.send(NavigationEvent::push(app.flow()));
    app.focus();
    app.type_text("first step");
    app.send(NavigationEvent::Next);
    app.focus();
    app.type_text("second step");
    app.send(NavigationEvent::Pop);
    app.assert_blurred_draft("first step");
    app.send(NavigationEvent::Next);
    app.assert_blurred_draft("second step");

    app.focus();
    app.send(NavigationEvent::push(app.flow()));
    app.focus();
    app.type_text("nested draft");
    app.send(NavigationEvent::Reset);
    app.assert_blurred_draft("second step");
    app.send(NavigationEvent::Root(None));
    app.assert_blurred_draft("root draft");
}

#[test]
fn hiding_the_keyboard_alone_preserves_focus_and_the_draft() {
    let mut app = App::new(Platform::Mobile);
    app.focus();
    app.type_text("draft");
    assert!(app.context.hardware.keyboard().visible());
    app.send(ShowKeyboard::Hide);
    assert!(!app.context.hardware.keyboard().visible());
    assert!(app.input().has_focus());
    assert!(app.input().inner.is_focused());
    assert!(app.input().focused);
    assert_eq!(app.input().value(), "draft");
    app.type_text(" continued");
    assert_eq!(app.input().value(), "draft continued");
}

#[test]
fn mobile_focus_adjusts_scroll_without_adding_blank_content() {
    focus_adjusts_scroll_without_adding_blank_content(Platform::Mobile);
}

#[test]
fn desktop_focus_adjusts_scroll_without_adding_blank_content() {
    focus_adjusts_scroll_without_adding_blank_content(Platform::Desktop);
}

#[test]
fn web_focus_adjusts_scroll_without_adding_blank_content() {
    focus_adjusts_scroll_without_adding_blank_content(Platform::Web);
}

fn focus_adjusts_scroll_without_adding_blank_content(platform: Platform) {
    use futures::FutureExt;
    use pelican_ui::interface::general::{AdjustScrollEvent, Content};
    use pelican_ui::components::Rectangle;
    use pelican_ui::theme::Color;
    use prism::drawable::{Instruction, Item};
    use ptsd::display::Bin;
    use ptsd::layout::{Offset, Padding, Size};

    fn valid(_: &mut Context, _: Vec<&mut Box<dyn Drawable>>) -> bool {true}
    fn settle(content: &mut Content, ctx: &mut Context, height: f32) -> Vec<Instruction> {
        for _ in 0..100 {
            content.request_width();
            content.request_height(375.0);
            let layout = Drawable::layout(content, (375.0, height));
            let instructions = content.draw(&layout, (0.0, 80.0), (0.0, 80.0, 375.0, height));
            let mut events = ctx.events();
            if !events.is_empty() {
                while let Some(event) = events.pop_front() {
                    content.request_width();
                    content.request_height(375.0);
                    let layout = Drawable::layout(content, (375.0, height));
                    content.event(ctx, &layout, event);
                    events.extend(ctx.events());
                }
                continue;
            }
            if let Some(callback) = content.changed().now_or_never() {
                callback(content, ctx);
                continue;
            }
            return instructions;
        }
        panic!("focus adjustment did not settle");
    }
    fn focused_y(instructions: &[Instruction], id: Uuid) -> f32 {
        instructions.iter().find_map(|instruction| match &instruction.item {
            Item::Text(text) if text.id == id && text.focused => Some(instruction.offset.1),
            _ => None,
        }).expect("focused input must be visible")
    }
    fn spacer(height: f32) -> Box<dyn Drawable> {
        Box::new(Bin(Stack::new(Offset::Start, Offset::Start, Size::Static(1.0),
            Size::Static(height), Padding::default()), Rectangle::new(Color::transparent(), 0.0, None)))
    }

    let mut app = App::new(platform);
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let mut content = Content::new(Offset::Start, vec![
        Box::new(TextInput::new(&app.theme, first, Some("First"), Some("Username"), None, None, None)),
        spacer(250.0),
        Box::new(TextInput::new(&app.theme, second, Some("Second"), Some("About me"), None, None, None)),
    ], Box::new(valid));
    settle(&mut content, &mut app.context, 600.0);
    let natural_height = content.children.1.request_height(327.0);
    app.context.emit(TextInputEvent::Focus {id: first});
    let first_frame = settle(&mut content, &mut app.context, 600.0);
    let y = focused_y(&first_frame, first);
    assert!((128.0..160.0).contains(&y), "first field should stay at its natural position: {y}");
    assert_eq!(content.children.scroll(), 0.0);
    app.context.emit(TextInputEvent::Focus {id: second});
    settle(&mut content, &mut app.context, 600.0);
    assert_eq!(content.children.scroll(), 0.0, "a fitting form must not acquire blank scroll space");

    for height in [450.0, 300.0] {
        let instructions = settle(&mut content, &mut app.context, height);
        let y = focused_y(&instructions, second);
        assert!(y >= 128.0 && y < 80.0 + height - 16.0, "focused field must remain visible: {y}");
        assert_eq!(content.children.1.request_height(327.0), natural_height);
        assert_eq!(content.children.1.column().padding().3, 0.0);
        let max_scroll = (natural_height.1 - height).max(0.0);
        assert!((content.children.scroll() - max_scroll).abs() < 0.1,
            "last input must stop at the natural bottom, not scroll into whitespace");
    }
    assert!(!content.children()[0].downcast_ref::<TextInput>().unwrap().has_focus());
    assert!(content.children()[2].downcast_ref::<TextInput>().unwrap().has_focus());

    // The historical event adjusts the real scroll offset in either direction.
    app.context.emit(AdjustScrollEvent::Vertical(-40.0));
    let previous = content.children.scroll();
    settle(&mut content, &mut app.context, 300.0);
    assert!((content.children.scroll() - (previous - 40.0)).abs() < 0.1);
    app.context.emit(AdjustScrollEvent::Vertical(-1e6));
    settle(&mut content, &mut app.context, 300.0);
    assert_eq!(content.children.scroll(), 0.0);

    // Editing must not snap a manually scrolled form back to the focused field.
    app.context.emit(TextInputEvent::Set {id: second, value: "More text".into()});
    settle(&mut content, &mut app.context, 300.0);
    assert_eq!(content.children.scroll(), 0.0);
    app.context.emit(TextInputEvent::Focus {id: first});
    settle(&mut content, &mut app.context, 300.0);
    app.context.emit(TextInputEvent::Focus {id: second});
    let instructions = settle(&mut content, &mut app.context, 300.0);
    assert!(focused_y(&instructions, second) < 80.0 + 300.0 - 16.0,
        "programmatic focus must reveal an off-screen input");

    // With real content after the input, the preferred 128px position is reachable.
    content.children_mut().push(spacer(400.0));
    app.context.emit(TextInputEvent::Focus {id: first});
    settle(&mut content, &mut app.context, 300.0);
    app.context.emit(TextInputEvent::Focus {id: second});
    assert!((focused_y(&settle(&mut content, &mut app.context, 300.0), second) - 128.0).abs() < 0.1);
    app.context.emit(ptsd::events::TextInput::Focused(false));
    settle(&mut content, &mut app.context, 600.0);
    assert_eq!(content.children.1.column().padding().3, 0.0);
    assert!(content.children.scroll() <= (content.children.1.request_height(327.0).1 - 600.0).max(0.0));
}

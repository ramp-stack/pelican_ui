# Pelican UI Overview

## Frontend development setup

Clone `orange`, `chk`, `pelican_ui`, and `ptsd` into sibling directories and update
their default branches together. Frontend dependencies use these local checkouts;
backend dependencies use published crates. Follow the
[Orange setup instructions](https://github.com/ramp-stack/orange#frontend-setup)
for Cargo Piston, the shared team identity, and build troubleshooting.

Pelican UI is a Rust UI crate designed to enable developers to build applications using a predefined set of components and layout rules. Rather than serving as a general-purpose UI framework, Pelican UI provides a structured component library and layout system intended for building consistent application interfaces quickly and predictably.

The crate is built on top of several underlying systems:

- **Prism** — provides drawing, events, and platform capability interfaces
- **PTSD** — provides layouts, navigation, interactions, and theming
- **AIR** — provides data storage and backend integration through Prism

Together, these systems form the foundation on which Pelican UI defines its higher-level application structure and UI components.

## Design Goals

Pelican UI prioritizes **consistency and structure** over flexibility. The crate is designed to guide developers toward a specific application layout pattern rather than acting as a toolkit for building arbitrary UI systems.

Applications built with Pelican UI are typically composed of pages structured using a small set of layout primitives, including:

- **Headers**
- **Bumpers**
- **Content**

Within these sections, developers use the predefined components provided by the crate. Pelican UI also includes built-in mechanisms for navigating between pages.

This opinionated approach allows applications to maintain a consistent design and behavior while reducing the complexity of UI composition.

The root `Interface` sets Prism's full-window background to the theme's `Background::Primary`.
The fill extends behind the status bar and home indicator on iOS, while buttons, navigation,
and page content stay inside the safe area. CHK applications receive this behavior automatically.
In portrait, the custom keyboard paints its own background through the bottom safe-area inset
while visible. Its keys keep their safe-area layout and touch bounds; hiding the keyboard restores
the theme background, and landscape keeps the normal safe-area clipping.

Drag downward from the content above the keyboard into its top edge, or start in its empty
top padding, to dismiss it interactively. The keyboard follows the pointer and gives the page
back space as it moves. Release after a sufficiently long drag or downward flick to close it;
a short drag or upward reversal returns it to its original position. Keys continue activating
on touch and cannot start dismissal drags. Mouse dragging and active touchscreen scrolling
use the same keyboard behavior; hover, wheel input without a press, and trailing touch momentum
do not start a dismissal.

The reusable gesture and moving layout live in PTSD's `display::Dismissible`, configured
with `DismissEdge::Bottom`. Pelican owns the keyboard content, safe-area paint, and the
focus and visibility events emitted after dismissal. Other popovers can use the same
container with `DismissEdge::Top` to dismiss toward the top of the screen.

## Keyboards and picker panels

`Interface::new(ctx, theme, roots, keyboard, on_event)` accepts an optional text keyboard.
Pass `None` to use the native keyboard on iOS and Android. To use Pelican's typing keyboard,
construct `CustomKeyboard::new(ctx, theme)` and pass it as `Some(Box::new(keyboard))`.
CHK's `App::keyboard` defaults to `None`; override that method to select a custom keyboard.

`CustomKeyboard`, `EmojiPanel`, `StickerPanel`, and `GifPanel` are separate exports from
`interface::system`. The custom keyboard contains only typing controls. Emoji selection and
backspace emit standard Prism keyboard events; sticker and GIF selection retain their existing
`SendEvent` messages. GIF search uses a normal text input and independent `GifSearchEvent`
commands (`Open`, `Submit`, `Close`). `TextInput::new` no longer takes `KeyboardPages`.

`MessageBar` owns its toolbar and a dismissible `Panels` tray on every platform. Native keyboard
mode shows stickers, GIFs, and the photo picker; custom keyboard mode also shows keyboard and
emoji buttons. Desktop keeps emoji, stickers, GIFs, and photos. Each newly created message bar
requests the interface's keyboard mode, including message bars created after navigation.

Selecting a picker dismisses the text keyboard. The tray waits until native keyboard visibility
in `ctx.window.keyboard_visible` reports that it has closed, avoiding two stacked input areas
during the hide animation. Tapping the message input closes the tray and resumes typing.
GIF search opens the configured text keyboard and allows the results area to shrink into the available space;
submitting or closing search dismisses typing and returns to browsing without changing the
message draft. Keyboard and picker heights remain content-sized, not matched to the native
keyboard height.

GIF credentials, HTTP requests, and bounded download caches belong to `Theme::gifs()`.
Configure a theme with `theme.gifs().set_api_key(key)`. Cloning a theme shares that service;
constructing another theme creates an independent service.

Photos retain their existing JPEG, width, and height wire representation. Deserialization
validates the encoded image before exposing it to views; malformed incoming photo messages
are discarded by AIR without a UI error. No format migration or snapshot replay is required.

## Scope

Pelican UI provides:

- A predefined component library
- Structured page layout primitives
- Integration with Prism’s layout, event, and rendering systems
- Integration with PTSD’s navigation, interaction and theming systems
- Data connectivity through AIR servers

Pelican UI intentionally **does not attempt to provide a flexible UI toolkit** for building custom design systems or component libraries.

Developers who want to create their own design systems, custom components, or alternative UI architectures should consider:

- Forking **Pelican UI** and adapting it to their needs, or
- Building directly on top of **Prism** and **PTSD** to implement their own UI system.

## Platform Support

Pelican UI is designed to be cross-platform and can be used to build applications targeting:

- Linux
- Windows
- macOS
- Android
- iOS

## Demo

Run the component studio from this crate:

```sh
cargo run --features demo --bin pelican_ui
```

The demo in `src/main.rs` includes buttons, sliders, toggles, checkboxes, radio groups,
list items, avatars, text inputs, QR codes, and video playback with seeking. Controls
emit events to the application, and pages retain their state when switching tabs.

The library uses PTSD and Prism. RAMP is optional and is enabled only by the `demo`
feature to run the native application; it is not required by library consumers.

Applications using sticker packs or shared video clips must register their contracts:

```rust,ignore
fn contracts<A: prism::traits::Air>(contracts: prism::air::Contracts<A>) -> prism::air::Contracts<A> {
    contracts.add::<pelican_ui::components::StickerPack>()
        .add::<pelican_ui::components::VideoClip>()
}
```

## Discord

https://discord.gg/53ERRpS4S4

Join the Discord server to ask questions, discuss development, or share projects.

## Opacity values

`Color::from_hex` takes a linear alpha byte. Keyboard backgrounds, pressed states, selection
highlights, GIF placeholders, and media badges use adjusted values to retain their appearance
from the former `wgpu_canvas` renderer, which incorrectly color-converted alpha:

| Former alpha | Linear alpha |
| --- | --- |
| 40 | 5 |
| 110 | 40 |
| 130 | 57 |
| 140 | 67 |

These are authored design values, not a conversion performed by the color API. New colors use
ordinary linear opacity; fully transparent and fully opaque colors remain 0 and 255.

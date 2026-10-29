# Pelican UI Overview

## Frontend development setup

This development checkout uses the sibling `ptsd` 0.2.0 repository to retain the local
scrolling and focus changes. Cargo retrieves backend dependencies from crates.io.
Keep that sibling checkout available while using this local dependency configuration.
Follow the [Orange setup instructions](https://github.com/ramp-stack/orange#frontend-setup)
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

Drag downward from the keyboard's empty top edge to dismiss it interactively. The keyboard
follows the pointer and gives the page back space as it moves. Release after a sufficiently
long drag or downward flick to close it; a short drag or upward reversal returns it to its
original position. The interface observes
pointer events for dismissal and forwards them unchanged. Keys continue activating on touch,
including during dismissal; a press on a key does not start dismissal. Mouse dragging and
active touchscreen scrolling use the same keyboard behavior; hover, wheel input without a
press, and trailing touch momentum do not start a dismissal.

The reusable gesture and moving layout live in PTSD's `display::Dismissible`, configured
with `DismissEdge::Bottom`. Pelican owns the keyboard content, safe-area paint, and the
focus and visibility events emitted after dismissal. Other popovers can use the same
container with `DismissEdge::Top` to dismiss toward the top of the screen.

## Keyboards and picker panels

`Interface::new(ctx, theme, roots, keyboard)` accepts an optional text keyboard.
Pass `None` to use the native keyboard on iOS and Android. To use Pelican's typing keyboard,
construct `CustomKeyboard::new(ctx, theme)` and pass it as `Some(Box::new(keyboard))`.
CHK's `App::keyboard` defaults to `None`; override that method to select a custom keyboard.
Handle application events with typed `#[on_event] impl OnEvent<E>` implementations on the
page or root component that owns the behavior.

Form focus on mobile, desktop, and web uses `AdjustScrollEvent::Vertical` to move the content's existing
scroll offset. The preferred input position is 128 logical pixels from the screen top,
limited by the natural scroll range; focusing an input never adds bottom padding.
Focus and viewport changes request an adjustment after layout, scoped to the owning form.
Editing text does not reset the user's scroll position.

`CustomKeyboard`, `EmojiPanel`, `StickerPanel`, and `GifPanel` are separate exports from
`interface::system`. The custom keyboard contains only typing controls. Emoji selection and
backspace emit standard Prism keyboard events; sticker and GIF selection retain their existing
`SendEvent` messages. GIF search uses a normal text input and independent `GifSearchEvent`
commands (`Open`, `Submit`, `Close`). `TextInput::new` no longer takes `KeyboardPages`.

Emoji use a continuous horizontal strip with four rows and category shortcuts, without
page snapping. Only nearby emoji text is built as the viewport moves. The custom macOS
caret blinks every 500 ms and becomes visible immediately when typing or refocusing;
mobile keeps its native caret.

`MessageBar` owns its toolbar and a dismissible `Panels` tray on every platform. It uses
`ctx.window.is_mobile` to assume a native keyboard on mobile and show stickers, GIFs, and
the photo picker. Desktop also shows emoji. There is no keyboard-mode query or keyboard
selection button, including when an application supplies a custom typing keyboard.

On mobile, the toolbar appears while a native or custom keyboard is visible or a picker is active.
Native visibility from `ctx.hardware.keyboard().visible()` includes opening, closing, and partial swipe
dismissal while any part of the keyboard remains visible. The native keyboard request includes
the measured toolbar height and its bottom padding as dismissal padding. The toolbar uses
`Dismissible`: window movement absorbs the drag first, and further finger movement slides the
toolbar away. It remains present until its drag or settling animation finishes. The Down button
requests native dismissal and starts the same toolbar dismissal; its remaining slide waits while
the window edge moves. Toolbar taps remain active, and captured drags cancel the pending tap.
Selecting a picker dismisses the text keyboard and shows the tray once the native keyboard is
fully hidden. The toolbar then handles dragging the picker tray. Tapping the message input closes
the tray and resumes typing.
GIF search opens the configured text keyboard and allows the results area to shrink into the available space;
submitting or closing search dismisses typing and returns to browsing without changing the
message draft. Keyboard and picker heights remain content-sized, not matched to the native
keyboard height.

GIF credentials, HTTP requests, and bounded download caches belong to `Theme::gifs()`.
Configure a theme with `theme.gifs().set_api_key(key)`. Cloning a theme shares that service;
constructing another theme creates an independent service.
GIF playback samples the latest frame at up to 60 Hz instead of consuming every backend
notification. Delayed rendering skips stale frames, unchanged images do not request a redraw,
and deactivating a GIF drops its player and refresh timer.

Photos retain their existing JPEG, width, and height wire representation. Deserialization
validates the encoded image before exposing it to views; malformed incoming photo messages
are discarded by AIR without a UI error. No format migration or snapshot replay is required.

Still-image views and video/GIF previews hold `media::Image` directly. Cloning an image shares
its pixels; editing a clone copies the buffer on the first mutation. Playback uses
`ctx.hardware.player(media.player())`; the QR scanner consumes camera frames directly. GIF and video
downloads retain shared owned bytes for lazy decoding. Video attachments preserve both tracks, reuse compatible MP4 source settings, and
use an 8 Mbps MPEG-4 video target when conversion is needed.

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

Sticker packs and shared video clips register their contracts when their components access
`ctx.air.instances::<C>()`. Applications do not need a separate contract-registration method.

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

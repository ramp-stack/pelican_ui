# Pelican UI Overview

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
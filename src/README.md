# Pelican UI Overview

Pelican UI is a Rust UI crate designed to enable developers to build applications using a predefined set of components and layout rules. Rather than serving as a general-purpose UI framework, Pelican UI provides a structured component library and layout system intended for building consistent application interfaces quickly and predictably.

The crate is built on top of several underlying systems:

- **Prism** — provides event handling, layout management, and rendering via `wgpu`
- **PTSD** — provides navigation, interaction handling and theming
- **AIR servers** — provide data storage and backend integration

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
- Navigation between application pages
- Integration with Prism’s layout, event, and rendering systems
- Integration with PTSD’s interaction and theming systems
- Data connectivity through AIR servers

Pelican UI intentionally **does not attempt to provide a flexible UI toolkit** for building custom design systems or component libraries.

## Non-Goals

Pelican UI is **not intended to be a general-purpose UI framework**.

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

## Examples

Several demo projects are provided in the Pelican UI GitHub repository. These examples demonstrate common usage patterns and serve as a reference for building applications with the framework.

Developers are encouraged to explore these demos to understand how Pelican UI applications are structured and how the provided components and layout primitives are intended to be used.
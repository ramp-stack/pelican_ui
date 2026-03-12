# Recreating the Pelican UI Ticket Demo

This guide walks through the demo **step by step**, including why each type exists and how each page is built.

It is written for this dependency setup:

```toml
[dependencies]
ramp = { package = "ramp2", version = "0.1.3" }
pelican_ui = { package = "pelican_ui", version = "2.1.1" }
```

---

## What you are building

You are building a small ticket app with:

- a **home page** that lists tickets
- a **buy ticket flow** with multiple pages
- a **view ticket flow** for showing a QR code
- a **theme** and a single root interface

The finished app has this shape:

```text
Interface
└── Root page: Home
    ├── Ticket list
    ├── Tap a ticket -> ViewTicketFlow
    └── Tap "Buy Ticket" -> BuyTicketFlow
         ├── NameOnTicket
         ├── AgeOnTicket
         ├── ExpirationOnTicket
         └── PurchasedTicket
```

---

## Step 1: Create a new Rust project

Create a binary crate:

```bash
cargo new pelican-ticket-demo
cd pelican-ticket-demo
```

This gives you a `src/main.rs` file.

---

## Step 2: Add the dependencies

Open `Cargo.toml` and replace the dependency section with:

```toml
[dependencies]
ramp = { package = "ramp2", version = "0.1.3" }
pelican_ui = { package = "pelican_ui", version = "2.1.1" }
```

These are the two crates your demo directly depends on.

- `ramp2` gives you the runtime entrypoint and access to Prism types through `ramp::prism`
- `pelican_ui` gives you the pages, headers, bumpers, flows, and components

---

## Step 3: Replace `src/main.rs`

Delete everything in `src/main.rs` and paste in:
fn main() {
    demo::maverick_main()
}

Create a `src/lib.rs` and paste in the demo code.

After that, this guide will explain the file from top to bottom.

---

## Step 4: Start with the imports

At the top of the file, import the types you need:

```rust
use prism::canvas::Align;
use ramp::prism::{self, Context, layout::{Offset, Stack}, event::{OnEvent, Event}, drawable::Component, drawables};
use pelican_ui::{colors, Request};
use pelican_ui::components::QRCode;
use pelican_ui::components::TextInput;
use pelican_ui::components::RadioSelector;
use pelican_ui::components::Icon;
use pelican_ui::components::list_item::{ListItem, ListItemInfoLeft};
use pelican_ui::components::text::{ExpandableText, TextSize, TextStyle};
use pelican_ui::theme::{Theme, Color};
use pelican_ui::utils::TitleSubtitle;
use pelican_ui::components::avatar::{AvatarContent, AvatarIconStyle};
use pelican_ui::interface::general::{Interface, Page, Header, Bumper, Content};
use pelican_ui::interface::navigation::{RootInfo, NavigationEvent, AppPage, Flow, FlowContainer};
use pelican_ui::components::list_item::ListItemGroup;
```

---

## Step 5: Understand the basic page pattern

Most pages in this demo follow the same structure:

```rust
#[derive(Debug, Component, Clone)]
pub struct SomePage(Stack, Page);

impl OnEvent for SomePage {}
impl AppPage for SomePage {}

impl SomePage {
    pub fn new(theme: &Theme) -> Self {
        let content = Content::new(...);
        let header = Header::stack(theme, "Title", None);
        let bumper = Bumper::stack(theme, None, Box::new(|ctx, _theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}
```

### What this means

Every page is made from three main parts:

1. **Header** – the top area
2. **Content** – the main body
3. **Bumper** – the bottom action area

Then those are wrapped into a `Page`:

```rust
let page = Page::new(header, content, Some(bumper));
```

And then returned inside a struct shaped like this:

```rust
Self(Stack::default(), page)
```

That pattern appears over and over in the demo.

---

## Step 6: Create the home page struct

The first major UI type is `Home`:

```rust
#[derive(Debug, Component, Clone)]
pub struct Home(Stack, Page);
impl OnEvent for Home {}
impl AppPage for Home {}
```

### What this does

- `Home` is a page
- It contains a `Stack` because all `Components` must have a layout and a `Page`
- It implements `OnEvent` so it can participate in the event system
- It implements `AppPage` so Pelican navigation knows it is a page

---

## Step 7: Add a `new()` constructor for `Home`

Inside `impl Home`, define a constructor:

```rust
impl Home {
    pub fn new(theme: &Theme) -> Self {
        // build data
        // build list
        // build content/header/bumper
        // return page
    }
}
```

This constructor is where the full home page gets built.

---

## Step 8: Create the demo data shown on the home page

Inside `Home::new`, create some hard-coded tickets:

```rust
let tickets = vec![
    Ticket::new("Daniel Vermeer", AgeGroup::Adult, Length::Season),
    Ticket::new("Amanda Vermeer", AgeGroup::Adult, Length::Season),
    Ticket::new("Jimmy Vermeer", AgeGroup::Youth, Length::Season),
    Ticket::new("Annie Vermeer", AgeGroup::Youth, Length::Season),
];
```

This is the data used to populate the ticket list.

Each ticket contains:

- a name
- an age group
- a pass length

---

## Step 9: Turn the tickets into list items

Now convert the `tickets` vector into a Pelican UI `ListItemGroup`:

```rust
let list = ListItemGroup::new(tickets.into_iter().map(|ticket| ListItem::new(
    theme,
    Some(AvatarContent::icon(&ticket.age.get().to_lowercase(), AvatarIconStyle::Brand)),
    ListItemInfoLeft::new(&ticket.name, None, None, None),
    Some(TitleSubtitle::new(&ticket.age.get(), Some(&ticket.length.get()))),
    None,
    Some("forward"),
    move |ctx: &mut Context, theme: &Theme| {
        let flow = ViewTicketFlow::new(theme, ticket.clone());
        ctx.send(Request::event(NavigationEvent::push(flow)));
    }
)).collect::<Vec<_>>());
```

### What each part of `ListItem::new(...)` means

For each ticket, you create one row in the list.

#### 1. Theme

This object provides colors and styling. Almost every Pelican UI component will require this object.

#### 2. Leading content

```rust
Some(AvatarContent::icon(&ticket.age.get().to_lowercase(), AvatarIconStyle::Brand))
```

This creates an avatar icon based on the age group.

#### 3. Left-side text info

```rust
ListItemInfoLeft::new(&ticket.name, None, None, None)
```

This makes the main title of the row the ticket holder name.

#### 4. Right-side subtitle block

```rust
Some(TitleSubtitle::new(&ticket.age.get(), Some(&ticket.length.get())))
```

This displays age group and pass length together.

#### 5. Optional left-side icon

```rust
None
```

You are not adding a left-side icon here.

#### 6. Trailing icon

```rust
Some("forward")
```

This shows a forward arrow icon. Pelican UI reccomends using this icon when clicking on the `ListItem` triggers a `NavigationEvent.`

#### 7. Tap handler

```rust
move |ctx: &mut Context, theme: &Theme| {
    let flow = ViewTicketFlow::new(theme, ticket.clone());
    ctx.send(Request::event(NavigationEvent::push(flow)));
}
```

When the user taps the ticket row:

- create a `ViewTicketFlow`
- push it onto the navigation stack

That is how tapping a ticket opens the QR-code page.

---

## Step 10: Put the list inside page content

Now wrap the list in a `Content` object:

```rust
let content = Content::new(Offset::Start, drawables![list], Box::new(|_| true));
```

### What this does

- `Offset::Start` aligns the content near the top/start of the content area
- `drawables![list]` puts the list into the page body
- `Box::new(|children: &Vec<Box<dyn Drawable>>| true)` is the closure used to enable/disable the next button on the screen.

---

## Step 11: Create the home header

Now create the top bar for the page:

```rust
let header = Header::home(theme, "My Tickets", None);
```

This gives the home page its title.
There are many variations of the Header, but on `Root` pages, we always use Header::home(..)

---

## Step 12: Create the home bumper button

Now create the bottom action area:

```rust
let bumper = Bumper::home(
    theme,
    (
        "Buy Ticket".to_string(),
        Box::new(|ctx: &mut Context, theme: &Theme| {
            let flow = BuyTicketFlow::new(theme);
            ctx.send(Request::event(NavigationEvent::push(flow)));
        })
    ),
    None,
);
```

### What this does

This adds a single bottom button labeled **Buy Ticket**.

When pressed:

1. create a new `BuyTicketFlow`
2. push that flow onto navigation

This is how the purchase flow begins.

There are many variations of the Bumper, but on `Root` pages, we always use Bumper::home(..);

---

## Step 13: Finish the `Home` page

Now combine header, content, and bumper into a page:

```rust
let page = Page::new(header, content, Some(bumper));
Self(Stack::default(), page)
```

That completes the home page.

---

## Step 14: Create a flow container for buying tickets

Now define the multi-step buy flow:

```rust
#[derive(Debug, Clone, Component)]
pub struct BuyTicketFlow(Stack, Flow);
impl OnEvent for BuyTicketFlow {}
impl FlowContainer for BuyTicketFlow {
    fn flow(&mut self) -> &mut Flow { &mut self.1 }
}
```

### Why this exists

A `Flow` is not just a single page. It is a stack of pages that move step by step.

`BuyTicketFlow` is the container for the purchase process.

By using a trait, we can utilize the wrapper object to get/move data inputed by user in the flow.

---

## Step 15: Add the pages to `BuyTicketFlow`

Inside its constructor:

```rust
impl BuyTicketFlow {
    pub fn new(theme: &Theme) -> Self {
        BuyTicketFlow(
            Stack::default(),
            Flow::new(vec![
                Box::new(NameOnTicket::new(theme)),
                Box::new(AgeOnTicket::new(theme)),
                Box::new(ExpirationOnTicket::new(theme)),
                Box::new(PurchasedTicket::new(theme))
            ])
        )
    }
}
```

### What this does

It defines the exact page order for the purchase flow:

1. enter the ticket holder name
2. choose the age group
3. choose the expiration type
4. show success

---

## Step 16: Build the first flow page, `NameOnTicket`

Create the page struct:

```rust
#[derive(Debug, Component, Clone)]
pub struct NameOnTicket(Stack, Page);
impl OnEvent for NameOnTicket {}
impl AppPage for NameOnTicket {}
```

Then build the page in `new()`:

```rust
impl NameOnTicket {
    pub fn new(theme: &Theme) -> Self {
        let input = TextInput::new(
            theme,
            None,
            Some("Ticket holder"),
            Some("Ticket holder's name..."),
            None,
            None
        );

        let content = Content::new(Offset::Start, drawables![input], Box::new(|_| true));
        let header = Header::stack(theme, "Ticket holder", None);
        let bumper = Bumper::stack(theme, None, Box::new(|ctx: &mut Context, _theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}
```

### What happens here

- `TextInput` creates the field for the ticket holder name
- `Header::stack(...)` creates a stacked flow-style header
- `Bumper::stack(...)` creates the bottom action area for advancing through the flow
- `NavigationEvent::Next` moves to the next page in the flow

This page does not yet save input into a model. It just demonstrates the UI structure.

---

## Step 17: Build the second flow page, `AgeOnTicket`

Create the struct:

```rust
#[derive(Debug, Component, Clone)]
pub struct AgeOnTicket(Stack, Page);
impl OnEvent for AgeOnTicket {}
impl AppPage for AgeOnTicket {}
```

Now build the page:

```rust
impl AgeOnTicket {
    pub fn new(theme: &Theme) -> Self {
        let input = RadioSelector::new(theme, 0, vec![
            ("Youth", "Persons aged 0y - 16y", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Adult", "Persons aged 17y - 65y", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Senior", "Persons 65+", Box::new(|_ctx: &mut Context, _theme: &Theme| {}))
        ]);

        let content = Content::new(Offset::Start, drawables![input], Box::new(|_| true));
        let header = Header::stack(theme, "Holder's age", None);
        let bumper = Bumper::stack(theme, None, Box::new(|ctx: &mut Context, _theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}
```

### What happens here

- `RadioSelector` builds a selectable list of age options
- the first argument after theme, `0`, is the selected index by default
- each tuple is one radio option
- the page advances with `NavigationEvent::Next`

Again, this demo focuses on page composition and flow navigation more than form persistence.

---

## Step 18: Build the third flow page, `ExpirationOnTicket`

Create the struct:

```rust
#[derive(Debug, Component, Clone)]
pub struct ExpirationOnTicket(Stack, Page);
impl OnEvent for ExpirationOnTicket {}
impl AppPage for ExpirationOnTicket {}
```

Build the page:

```rust
impl ExpirationOnTicket {
    pub fn new(theme: &Theme) -> Self {
        let input = RadioSelector::new(theme, 0, vec![
            ("Day (~04/03/2026)", "Pass valid for the day", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Season (~01/09/2026)", "Pass valid for the entire summer", Box::new(|_ctx: &mut Context, _theme: &Theme| {})),
            ("Year (~04/03/2027)", "Pass valid for the whole year", Box::new(|_ctx: &mut Context, _theme: &Theme| {}))
        ]);

        let content = Content::new(Offset::Start, drawables![input], Box::new(|_| true));
        let header = Header::stack(theme, "Ticket expiration", None);
        let bumper = Bumper::stack(theme, Some("Complete Purchase"), Box::new(|ctx: &mut Context, _theme: &Theme| {
            ctx.send(Request::event(NavigationEvent::Next));
        }), None);

        let page = Page::new(header, content, Some(bumper));
        Self(Stack::default(), page)
    }
}
```

### What is different here

This page is very similar to `AgeOnTicket`, but the bottom button text is customized:

```rust
Some("Complete Purchase")
```

That makes the final action clearer to the user.

---

## Step 19: Build the confirmation page, `PurchasedTicket`

Create the struct:

```rust
#[derive(Debug, Component, Clone)]
pub struct PurchasedTicket(Stack, Page);
impl OnEvent for PurchasedTicket {}
impl AppPage for PurchasedTicket {}
```

Build the page:

```rust
impl PurchasedTicket {
    pub fn new(theme: &Theme) -> Self {
        let header = Header::stack_end(theme, "Ticket purchased");
        let bumper = Bumper::stack_end(theme, None);
        let content = Content::new(
            Offset::Center,
            drawables![
                Icon::new(theme, "checkmark", Some(theme.colors().get(colors::Brand)), 128.0),
                ExpandableText::new(
                    theme,
                    "You purchased a ticket",
                    TextSize::H4,
                    TextStyle::Heading,
                    Align::Center,
                    None
                )
            ],
            Box::new(|_children| true)
        );

        let page = Page::new(header, content, Some(bumper));
        PurchasedTicket(Stack::default(), page)
    }
}
```

### What happens here

This is the final success screen.

- `Header::stack_end(...)` is used for the final step header style
- `Bumper::stack_end(...)` is the closing bottom area
- `Offset::Center` centers the content vertically
- `Icon::new(...)` shows a large checkmark
- `ExpandableText::new(...)` shows the purchase confirmation text

---

## Step 20: Create a flow for viewing an existing ticket

Now define a second flow type:

```rust
#[derive(Debug, Clone, Component)]
pub struct ViewTicketFlow(Stack, Flow);
impl OnEvent for ViewTicketFlow {}
impl FlowContainer for ViewTicketFlow {
    fn flow(&mut self) -> &mut Flow { &mut self.1 }
}
```

This works just like `BuyTicketFlow`, but it only contains one page.

---

## Step 21: Add the page to `ViewTicketFlow`

```rust
impl ViewTicketFlow {
    pub fn new(theme: &Theme, ticket: Ticket) -> Self {
        ViewTicketFlow(Stack::default(), Flow::new(vec![
            Box::new(ViewTicket::new(theme, ticket)),
        ]))
    }
}
```

This flow exists so that when a ticket row is tapped, you can push a dedicated view flow.

---

## Step 22: Build the `ViewTicket` page

Create the struct:

```rust
#[derive(Debug, Component, Clone)]
pub struct ViewTicket(Stack, Page);
impl OnEvent for ViewTicket {}
impl AppPage for ViewTicket {}
```

Build the page:

```rust
impl ViewTicket {
    pub fn new(theme: &Theme, ticket: Ticket) -> Self {
        let header = Header::stack(theme, "View ticket", None);
        let bumper = Bumper::stack(theme, Some("Share"), Box::new(|_ctx: &mut Context, _theme: &Theme| {}), None);
        let content = Content::new(
            Offset::Center,
            drawables![
                QRCode::default(theme),
                ExpandableText::new(
                    theme,
                    &format!("Scan {}'s ticket at entry.", ticket.name),
                    TextSize::Sm,
                    TextStyle::Primary,
                    Align::Center,
                    None
                )
            ],
            Box::new(|_children| true)
        );

        let page = Page::new(header, content, Some(bumper));
        ViewTicket(Stack::default(), page)
    }
}
```

### What happens here

This page shows:

- a QR code
- a short label telling whose ticket is being shown
- a `Share` bumper action stub

The text uses the selected ticket's name:

```rust
&format!("Scan {}'s ticket at entry.", ticket.name)
```

---

## Step 23: Create the application entrypoint with `ramp::run!`

Near the bottom of the file, add:

```rust
ramp::run!{|ctx: &mut Context, assets: Assets| {
    let theme = Theme::dark(assets.all(), Color::from_hex("#8efe33", 255));
    let home = RootInfo::icon("explore", "My Tickets", Box::new(Home::new(ctx, &theme)));
    Interface::new(&theme, vec![home], Box::new(|_page: &mut Box<dyn Drawable>, _ctx: &mut Context, e: Box<dyn Event>| {
        vec![e]
    }))
}}
```

### What each line does

#### 1. Start the app

```rust
ramp::run!{|ctx: &mut Context, assets: Assets| {
```

This is the runtime entrypoint.

#### 2. Build the theme

```rust
let theme = Theme::dark(assets.all(), Color::from_hex("#8efe33", 255));
```

This creates a dark theme with a bright green brand color.

#### 3. Create the root page

```rust
let home = RootInfo::icon("explore", "My Tickets", Box::new(Home::new(ctx, &theme)));
```

This registers `Home` as the root entry in the interface.

#### 4. Build the interface

```rust
Interface::new(&theme, vec![home], Box::new(|_page: &mut Box<dyn Drawable>, _ctx: &mut Context, e: Box<dyn Event>| {
    vec![e]
}))
```

This creates the app interface with:

- your theme
- a list of root pages
- an event pass-through closure

---

## Step 24: Add the ticket data model

Now define the model used by the UI list:

```rust
#[derive(Debug, Clone)]
pub struct Ticket {
    name: String,
    age: AgeGroup,
    length: Length,
}
```

Add a constructor:

```rust
impl Ticket {
    pub fn new(name: &str, age: AgeGroup, length: Length) -> Self {
        Ticket { name: name.to_string(), age, length }
    }
}
```

This is just helper data for the demo.

---

## Step 25: Add the `AgeGroup` enum

```rust
#[derive(Debug, Clone)]
pub enum AgeGroup {
    Youth,
    Adult,
    Senior
}
```

Add a display helper:

```rust
impl AgeGroup {
    pub fn get(&self) -> String {
        match self {
            AgeGroup::Youth => "Youth".to_string(),
            AgeGroup::Adult => "Adult".to_string(),
            AgeGroup::Senior => "Senior".to_string()
        }
    }
}
```

The UI needs user-facing strings, so `get()` converts enum values into display text.

---

## Step 26: Add the `Length` enum

```rust
#[derive(Debug, Clone)]
pub enum Length {
    Day,
    Season,
    Year,
}

impl Length {
    pub fn get(&self) -> String {
        match self {
            Length::Day => "Day".to_string(),
            Length::Season => "Season".to_string(),
            Length::Year => "Year".to_string()
        }
    }
}
```

This is used when displaying ticket details in the list.

---

## Step 27: Run the demo

Now compile and run it:

```bash
cargo run
```

If your environment is set up correctly, the app should open with the **My Tickets** screen.

---

## Step 28: Click through the full demo

After running:

1. Open the app
2. View the **My Tickets** home page
3. Click a ticket row
4. Confirm that a **View ticket** page opens with a QR code
5. Go back
6. Press **Buy Ticket**
7. Step through:
   - Ticket holder
   - Holder's age
   - Ticket expiration
   - Ticket purchased

That confirms both flows are wired correctly.

---

## Step 29: Understand the reusable Pelican UI mental model

This demo teaches a simple pattern you can reuse:

### For a normal page

1. create a struct shaped like `pub struct MyPage(Stack, Page);`
2. derive `Component`
3. implement `OnEvent`
4. implement `AppPage`
5. build:
   - `Header`
   - `Content`
   - `Bumper`
6. combine them with `Page::new(...)`

### For a multi-step flow

1. create a struct shaped like `pub struct MyFlow(Stack, Flow);`
2. derive `Component`
3. implement `OnEvent`
4. implement `FlowContainer`
5. build a `Flow::new(vec![ ... ])`
6. push it with `NavigationEvent::push(flow)`

### For moving to the next flow screen

Use:

```rust
ctx.send(Request::event(NavigationEvent::Next));
```

### For opening a new flow

Use:

```rust
ctx.send(Request::event(NavigationEvent::push(flow)));
```

---

## Step 30: Where to customize this demo next

Once the basic version works, good next upgrades are:

- store the `TextInput` value instead of discarding it
- store selected `RadioSelector` values
- generate the purchased ticket dynamically
- replace hard-coded ticket data with AIR-backed data
- add more root pages to the `Interface`

---

## Summary

To recreate this demo, you:

1. created a new Rust app
2. added `ramp2` and `pelican_ui`
3. built a `Home` page
4. created a `BuyTicketFlow` with four pages
5. created a `ViewTicketFlow` for ticket details
6. wired everything together with `NavigationEvent`
7. launched the UI through `ramp::run!`

The most important thing to remember is that Pelican UI is built around **structured pages and flows**.

A page is usually:

- `Header`
- `Content`
- `Bumper`

A flow is just multiple pages connected together.

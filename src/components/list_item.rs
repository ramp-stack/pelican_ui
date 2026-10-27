use prism::drawable::text::Align;
use ptsd::layout::{Column, Stack, Row, Padding, Offset, Size};
use prism::drawable::Component;
use prism::Context;
use ptsd::interactions;
use ptsd::utils::{TitleSubtitle};

use crate::theme::{Theme, Color, Icons};
use crate::components::text::{Text, TextSize, TextStyle};
use crate::components::Icon;
use crate::components::avatar::Avatar;

#[derive(Clone, Debug)]
pub struct ListItemInfoLeft {
    pub title: TitleSubtitle,
    pub flair: Option<(Icons, Color)>,
    pub description: Option<String>,
}

impl ListItemInfoLeft {
    pub fn new(title: &str, subtitle: Option<&str>, description: Option<&str>, flair: Option<(Icons, Color)>) -> Self {
        ListItemInfoLeft {
            title: TitleSubtitle::new(title, subtitle),
            description: description.map(|text| text.to_string()),
            flair,
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct ListItem(Stack, interactions::Button, #[skip] String, #[skip] String);

impl ListItem {
    pub fn new(
        theme: &Theme, avatar: Option<Avatar>, left: ListItemInfoLeft,
        right: Option<TitleSubtitle>, icon_l: Option<Icons>, icon_r: Option<Icons>,
        mut on_click: impl FnMut(&mut Context, &Theme) + Clone + 'static,
    ) -> Self {
        let label = left.title.title.to_string();
        let subtitle = left.title.subtitle.clone().unwrap_or_default().to_string();
        let list_item = ListItemContent::new(theme, avatar, left, right, icon_l, icon_r);

        let theme = theme.clone();
        let callback = Box::new(move |ctx: &mut Context| (on_click)(ctx, &theme));
        let button = interactions::Button::new(
            list_item, None::<ListItemContent>, None::<ListItemContent>,
            None::<ListItemContent>, None::<ListItemContent>, callback, false,
        );
        ListItem(Stack::default(), button, label, subtitle)
    }

    pub fn default(theme: &Theme, title: String) -> Self {
        Self::new(theme, None, ListItemInfoLeft::new(&title, None, None, None),
            None, None, Some(Icons::Forward), |_: &mut Context, _: &Theme| println!("Pressed..."))
    }

    pub fn title(&self) -> &String {&self.2}
    pub fn subtitle(&self) -> &String {&self.3}
}

#[derive(Debug, Component, Clone)]
pub struct Group(Column, Vec<ListItem>);

impl Group {
    pub fn new(items: Vec<ListItem>) -> Self {Group(Column::start(16.0), items)}
}

#[derive(Debug, Component, Clone)]
pub struct ListItemGroup(Stack, Option<Text>, Option<Group>);

impl ListItemGroup {
    pub fn new(theme: &Theme, instructions: Option<String>, mut items: Vec<ListItem>, has_label: bool) -> Self {
        items.iter_mut().for_each(|i| i.1.set_trigger_on_release());

        let layout = match items.is_empty() {
            true if has_label => Stack::new(
                Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default(),
            ),
            true => Stack::new(
                Offset::Center, Offset::Center, Size::Fill, Size::Fill, Padding::default(),
            ),
            false => Stack::new(
                Offset::Start, Offset::Start, Size::Fill, Size::Fill, Padding::default(),
            ),
        };

        let align = if has_label {Align::Left} else {Align::Center};
        let instructions = items.is_empty().then_some(Text::new(
            theme, &instructions.unwrap_or_default(), TextSize::Md, TextStyle::Secondary, align, None,
        ));
        let group = (!items.is_empty()).then_some(Group::new(items));
        ListItemGroup(layout, instructions, group)
    }
}

#[derive(Debug, Component, Clone)]
pub struct ListItemSection(Column, Option<Text>, ListItemGroup);

impl ListItemSection {
    pub fn new(theme: &Theme, label: Option<String>, instructions: Option<String>, items: Vec<ListItem>) -> Self {
        let text = label.as_ref().map(|label| {
            Text::new(theme, label, TextSize::H5, TextStyle::Heading, Align::Left, None)
        });
        ListItemSection(
            Column::start(16.0), text, ListItemGroup::new(theme, instructions, items, label.is_some()),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct ListItemContent(Row, Option<Icon>, Option<Avatar>, ListItemData, Option<Icon>);

impl ListItemContent {
    #[allow(clippy::too_many_arguments)]
    fn new(
        theme: &Theme, avatar: Option<Avatar>, left: ListItemInfoLeft,
        right: Option<TitleSubtitle>, icon_l: Option<Icons>, icon_r: Option<Icons>,
    ) -> Self {
        let c = theme.colors().get(ptsd::Text::Primary);
        let content = ListItemData::new(theme, left, right);
        let icon_l = icon_l.map(|i| Icon::new(theme, i, Some(c), 24.0));
        let icon_r = icon_r.map(|i| Icon::new(theme, i, Some(c), 16.0));
        let layout = Row::new(16.0, Offset::Center, Size::Fit, Padding(0.0, 16.0, 0.0, 16.0));
        ListItemContent(layout, icon_l, avatar, content, icon_r)
    }
}

#[derive(Debug, Component, Clone)]
struct ListItemData(Row, LeftData, Option<RightData>);

impl ListItemData {
    fn new(theme: &Theme, left: ListItemInfoLeft, right: Option<TitleSubtitle>) -> Self {
        ListItemData(
            Row::start(8.0), LeftData::new(theme, left), right.map(|info| RightData::new(theme, info)),
        )
    }
}

#[derive(Debug, Component, Clone)]
struct LeftData(Column, TitleRow, Option<Text>, Option<Text>);

impl LeftData {
    fn new(theme: &Theme, info: ListItemInfoLeft) -> Self {
        let layout = Column::new(4.0, Offset::Start, Size::Fill, Padding::default(), None);
        let subtitle = info.title.subtitle.map(|text| {
            Text::new(theme, &text, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2))
        });
        let description = info.description.map(|text| {
            Text::new(theme, &text, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2))
        });
        LeftData(layout, TitleRow::new(theme, &info.title.title, info.flair), subtitle, description)
    }
}

#[derive(Debug, Component, Clone)]
struct TitleRow(Row, Text, Option<Icon>);

impl TitleRow {
    fn new(theme: &Theme, title: &str, flair: Option<(Icons, Color)>) -> Self {
        let layout = Row::new(4.0, Offset::Center, Size::Fit, Padding::default());
        let text = Text::new(theme, title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1));
        let flair = flair.map(|(name, color)| Icon::new(theme, name, Some(color), 16.0));
        TitleRow(layout, text, flair)
    }
}

#[derive(Debug, Component, Clone)]
struct RightData(Column, Text, Option<Text>);

impl RightData {
    fn new(theme: &Theme, info: TitleSubtitle) -> Self {
        let title = Text::new(
            theme, &info.title, TextSize::H5, TextStyle::Heading, Align::Left, Some(1),
        ).fit();
        let subtitle = info.subtitle.map(|text| {
            Text::new(theme, &text, TextSize::Xs, TextStyle::Secondary, Align::Left, Some(2)).fit()
        });
        RightData(Column::end(4.0), title, subtitle)
    }
}

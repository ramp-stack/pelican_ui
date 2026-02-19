
#[derive(Debug, Clone, Component)]
pub struct Message(Column, Vec<_TextMessage>);

#[derive(Debug, Clone, Component)]
pub struct _TextMessages(Column, Vec<_TextMessage>);

#[derive(Debug, Clone, Component)]
pub struct _TextMessage(Stack, Rectangle, ExpandableText);


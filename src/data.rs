use crate::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::text::Text as UiText;
use bevy::utils::HashMap;

#[derive(Debug)]
pub enum NodeType {
    Node,
    Image,
    Text,
    Button,
    Include,
    Slot,
    Template,
    Custom(String),
}

#[derive(Debug)]
pub struct XNode {
    pub src: Option<String>,
    pub styles: Vec<StyleAttr>,
    pub target: Option<String>,
    pub watch: Option<String>,
    pub id: Option<String>,
    pub uncompiled: Vec<AttrTokens>,
    pub tags: Vec<(String, String)>,
    pub defs: Vec<(String, String)>,
    pub event_listener: Vec<Action>,
    pub content: Option<String>,
    pub node_type: NodeType,
    pub children: Vec<XNode>,
}

#[derive(Debug, Asset, TypePath)]
pub struct Template {
    pub name: Option<String>,
    pub properties: HashMap<String, String>,
    pub root: Vec<XNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Attribute {
    Style(StyleAttr),
    PropertyDefinition(String, String), // to remove
    Uncompiled(AttrTokens),
    Action(Action),
    Path(String),
    Target(String),
    Id(String),
    Watch(String),
    Tag(String, String),
}

#[derive(Debug, Reflect, PartialEq, Clone)]
#[reflect]
pub struct AttrTokens {
    pub prefix: Option<String>,
    pub ident: String,
    pub key: String,
}

impl AttrTokens {
    pub fn compile(&self, props: &TemplateState) -> Option<Attribute> {
        let Some(prop_val) = props.get_prop(&self.key) else {
            warn!("failed to parse property, key not found `{}`", self.key);
            return None;
        };

        let Ok((_, attr)) = crate::parse::attribute_from_parts(
            self.prefix.as_ref().map(|s| s.as_bytes()),
            self.ident.as_bytes(),
            prop_val.as_bytes(),
        ) else {
            warn!(
                "failed to parse property key: `{}` val:`{}`",
                self.key, prop_val
            );
            return None;
        };

        // recurive compile, what could go wrong
        if let Attribute::Uncompiled(prop) = attr {
            return prop.compile(props);
        };

        Some(attr)
    }
}

#[derive(Debug,Reflect, PartialEq, Clone)]
#[reflect]
pub enum Action {
    OnPress(Vec<String>),
    OnEnter(Vec<String>),
    OnExit(Vec<String>),
    OnSpawn(Vec<String>),
}
impl Action {
    pub fn self_insert(self, mut cmd: EntityCommands) {
        match self {
            Action::OnPress(fn_id) => {
                cmd.insert(crate::prelude::OnPress(fn_id));
            }
            Action::OnEnter(fn_id) => {
                cmd.insert(crate::prelude::OnEnter(fn_id));
            }
            Action::OnExit(fn_id) => {
                cmd.insert(crate::prelude::OnExit(fn_id));
            }
            Action::OnSpawn(fn_id) => {
                cmd.insert(crate::prelude::OnSpawn(fn_id));
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StyleAttr {
    Display(Display),
    Position(PositionType),
    Overflow(Overflow),
    Left(Val),
    Right(Val),
    Top(Val),
    Bottom(Val),
    Width(Val),
    Height(Val),
    MinWidth(Val),
    MinHeight(Val),
    MaxWidth(Val),
    MaxHeight(Val),
    AspectRatio(f32),
    AlignItems(AlignItems),
    JustifyItems(JustifyItems),
    AlignSelf(AlignSelf),
    JustifySelf(JustifySelf),
    AlignContent(AlignContent),
    JustifyContent(JustifyContent),
    Margin(UiRect),
    Padding(UiRect),

    // ------------
    // border
    Border(UiRect),
    BorderColor(Color),
    BorderRadius(UiRect),

    // ------------
    // flex
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexBasis(Val),
    RowGap(Val),
    ColumnGap(Val),

    // -----------
    // grid
    GridAutoFlow(GridAutoFlow),
    GridTemplateRows(Vec<RepeatedGridTrack>),
    GridTemplateColumns(Vec<RepeatedGridTrack>),
    GridAutoRows(Vec<GridTrack>),
    GridAutoColumns(Vec<GridTrack>),
    GridRow(GridPlacement),
    GridColumn(GridPlacement),

    // -----
    // font
    Direction(Direction),
    FontSize(f32),
    Font(String),
    FontColor(Color),

    // -----
    // color
    Background(Color),

    // -----
    Hover(Box<StyleAttr>),
    Pressed(Box<StyleAttr>),

    // -----
    // animations
    Delay(f32),
    Easing(interpolation::EaseFunction),
}

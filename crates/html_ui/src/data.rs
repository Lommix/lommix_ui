use crate::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
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
    Property,
    Custom(String),
}

#[derive(Debug)]
pub struct XNode {
    pub src: Option<String>,
    pub styles: Vec<StyleAttr>,
    pub target: Option<String>,
    pub watch: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub uncompiled: Vec<AttrTokens>,
    pub tags: Vec<(String, String)>,
    pub defs: Vec<(String, String)>,
    pub event_listener: Vec<Action>,
    pub content: Option<String>,
    pub node_type: NodeType,
    pub children: Vec<XNode>,
}

impl XNode {
    pub fn find_node(&self, hash: u64) -> &XNode {
        Self::find(&self, hash)
    }

    fn find(curent: &XNode, hash: u64) -> &XNode {
        todo!()
    }
}

#[derive(Debug, Asset, TypePath)]
pub struct HtmlTemplate {
    pub name: Option<String>,
    pub properties: HashMap<String, String>,
    pub root: Vec<XNode>,
}

#[derive(Debug, Clone)]
pub enum Attribute {
    Style(StyleAttr),
    PropertyDefinition(String, String),
    Name(String),
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
    pub fn compile(&self, props: &TemplateProperties) -> Option<Attribute> {
        let Some(prop_val) = props.get_prop(&self.key) else {
            warn!("failed to parse property, key not found `{}`", self.key);
            return None;
        };

        let (_, attr) = match crate::parse::attribute_from_parts::<nom::error::VerboseError<&[u8]>>(
            self.prefix.as_ref().map(|s| s.as_bytes()),
            self.ident.as_bytes(),
            prop_val.as_bytes(),
        ) {
            Ok(val) => val,
            Err(_) => {
                warn!(
                    "failed to parse property key: `{}` val:`{}`",
                    self.key, prop_val
                );
                return None;
            }
        };

        // recursive compile, what could go wrong
        if let Attribute::Uncompiled(prop) = attr {
            return prop.compile(props);
        };

        Some(attr)
    }
}

#[derive(Debug, Reflect, PartialEq, Clone)]
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
                cmd.insert(crate::prelude::OnUiPress(fn_id));
            }
            Action::OnEnter(fn_id) => {
                cmd.insert(crate::prelude::OnUiEnter(fn_id));
            }
            Action::OnExit(fn_id) => {
                cmd.insert(crate::prelude::OnUiExit(fn_id));
            }
            Action::OnSpawn(fn_id) => {
                cmd.insert(crate::prelude::OnUiSpawn(fn_id));
            }
        }
    }
}

#[derive(Debug, Clone)]
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
    FontSize(f32),
    Font(String),
    FontColor(Color),

    // -----
    // color
    Background(Color),

    // -----
    // @todo: blocks reflect, rethink
    Hover(Box<StyleAttr>),
    Pressed(Box<StyleAttr>),
    Active(Box<StyleAttr>),

    // -----
    // animations
    Delay(f32),
    Easing(interpolation::EaseFunction),
    // -----
    // image
    // scale mode
    // atlas
    ImageScaleMode(NodeImageMode),
    // ImageAtlas(TextureAtlas),
}

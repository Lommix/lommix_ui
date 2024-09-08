use crate::prelude::*;
use bevy::prelude::*;


#[derive(Debug, Asset, TypePath)]
pub enum XNode {
    Div(Div),
    Image(Image),
    Text(Text),
    Button(Button),
    Include(Include),
    Unkown,
}

// -------------------------------
#[derive(Debug, Default)]
pub struct Div {
    pub styles: Vec<StyleAttr>,
    pub children: Vec<XNode>,
}

#[derive(Debug, Default)]
pub struct Image {
    pub path: String,
    pub styles: Vec<StyleAttr>,
}

#[derive(Debug, Default)]
pub struct Text {
    pub content: String,
    pub styles: Vec<StyleAttr>,
}

#[derive(Debug, Default)]
pub struct Button {
    pub action: String,
    pub styles: Vec<StyleAttr>,
    pub children: Vec<XNode>,
}

#[derive(Debug, Default)]
pub struct Include {
    pub path: String,
    pub styles: Vec<StyleAttr>,
    pub children: Vec<XNode>,
}


#[derive(Debug, PartialEq, Clone)]
pub enum Attribute {
    Style(StyleAttr),
    Path(String),
    Click(String),
    Compontent(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StyleAttr {
    Display(Display),
    Position(PositionType),
    Overflow(Overflow),
    Direction(Direction),
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
    FontColor(LinearRgba),

    // -----
    // color
    Background(Color),
    BorderColor(Color),
    BorderRadius(UiRect),

    // -----
    Hover(Box<StyleAttr>),
    Active(Box<StyleAttr>),
    // -----
    // animations
    Duration(f32),
}

impl From<StyleAttr> for Attribute {
    fn from(value: StyleAttr) -> Self {
        Attribute::Style(value)
    }
}

impl StyleAttr {
    pub fn apply(&self, entity: Entity, cmd: &mut Commands, style: &mut Style) {
        match self {
            StyleAttr::Display(display) => style.display = *display,
            StyleAttr::Position(position) => style.position_type = *position,
            StyleAttr::Overflow(overflow) => style.overflow = *overflow,
            StyleAttr::Direction(dir) => style.direction = *dir,
            StyleAttr::Left(left) => style.left = *left,
            StyleAttr::Right(right) => style.right = *right,
            StyleAttr::Top(top) => style.top = *top,
            StyleAttr::Bottom(bottom) => style.bottom = *bottom,
            StyleAttr::Width(width) => style.width = *width,
            StyleAttr::Height(height) => style.height = *height,
            StyleAttr::FlexDirection(dir) => style.flex_direction = *dir,
            StyleAttr::Margin(rect) => style.margin = *rect,
            StyleAttr::Padding(rect) => style.padding = *rect,
            StyleAttr::Border(rect) => style.border = *rect,
            StyleAttr::AlignItems(val) => style.align_items = *val,
            StyleAttr::JustifyItems(val) => style.justify_items = *val,
            StyleAttr::AlignSelf(val) => style.align_self = *val,
            StyleAttr::JustifySelf(val) => style.justify_self = *val,
            StyleAttr::AlignContent(val) => style.align_content = *val,
            StyleAttr::JustifyContent(val) => style.justify_content = *val,
            StyleAttr::RowGap(val) => style.row_gap = *val,
            StyleAttr::ColumnGap(val) => style.column_gap = *val,
            StyleAttr::MinWidth(val) => style.min_width = *val,
            StyleAttr::MinHeight(val) => style.min_height = *val,
            StyleAttr::MaxWidth(val) => style.max_width = *val,
            StyleAttr::MaxHeight(val) => style.max_height = *val,
            StyleAttr::Background(color) => {
                cmd.entity(entity).insert(BackgroundColor(*color));
            }
            StyleAttr::BorderRadius(val) => {
                cmd.entity(entity).insert(BorderRadius {
                    top_left: val.top,
                    top_right: val.right,
                    bottom_right: val.bottom,
                    bottom_left: val.left,
                });
            }
            StyleAttr::BorderColor(color) => {
                cmd.entity(entity).insert(BorderColor(*color));
            }
            _ => (),
            // StyleAttr::Clicked(attrs) => {
            //     // cmd.entity(entity).insert(ClickStyle(attrs.clone()));
            // }
            // StyleAttr::Hover(attrs) => {
            //     // cmd.entity(entity).insert(HoverStyle(attrs.clone()));
            // }
            // StyleAttr::AspectRatio(_) => todo!(),

            // StyleAttr::FlexWrap(_) => todo!(),
            // StyleAttr::FlexGrow(_) => todo!(),
            // StyleAttr::FlexShrink(_) => todo!(),
            // StyleAttr::FlexBasis(_) => todo!(),
            // StyleAttr::GridAutoFlow(_) => todo!(),
            // StyleAttr::GridTemplateRows(_) => todo!(),
            // StyleAttr::GridTemplateColumns(_) => todo!(),
            // StyleAttr::GridAutoRows(_) => todo!(),
            // StyleAttr::GridAutoColumns(_) => todo!(),
            // StyleAttr::GridRow(_) => todo!(),
            // StyleAttr::GridColumn(_) => todo!(),
            // StyleAttr::FontSize(_) => todo!(),
            // StyleAttr::Font(_) => todo!(),
            // StyleAttr::FontColor(_) => todo!(),
        }
    }
}

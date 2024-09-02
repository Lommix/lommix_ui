use bevy::prelude::*;
use quick_xml::events::attributes::Attribute;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ParseError, StyleParserError},
    parse::{parse_color, parse_ui_rect, parse_val},
};

// use crate::build::{ClickStyle, HoverStyle};

#[derive(Serialize, Debug, PartialEq, Deserialize, Clone)]
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
    BorderRadius(Val),

    // -----
    // conditions
    Hover(Box<StyleAttr>),
    Active(Box<StyleAttr>),

    // -----
    // animations
    Duration(f32),
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
            StyleAttr::Background(color) => {
                cmd.entity(entity).insert(BackgroundColor(*color));
            }
            StyleAttr::BorderRadius(val) => {
                cmd.entity(entity).insert(BorderRadius::all(*val));
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
            // StyleAttr::MinWidth(_) => todo!(),
            // StyleAttr::MinHeight(_) => todo!(),
            // StyleAttr::MaxWidth(_) => todo!(),
            // StyleAttr::MaxHeight(_) => todo!(),
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

        cmd.entity(entity).add(|mut ent: EntityWorldMut| {
            ent.get::<Style>().map(|mut style| {});
        });
    }
}

impl<'a> TryFrom<&'a Attribute<'a>> for StyleAttr {
    type Error = nom::Err<nom::error::Error<&'a [u8]>>;

    fn try_from(value: &'a Attribute<'a>) -> Result<Self, Self::Error> {
        let style = match value.key.local_name().as_ref() {
            b"height" => StyleAttr::Height(parse_val(&value.value).map(|(_, val)| val)?),
            b"width" => StyleAttr::Width(parse_val(&value.value).map(|(_, val)| val)?),
            b"padding" => StyleAttr::Padding(parse_ui_rect(&value.value).map(|(_, val)| val)?),
            b"margin" => StyleAttr::Margin(parse_ui_rect(&value.value).map(|(_, val)| val)?),
            b"border" => StyleAttr::Border(parse_ui_rect(&value.value).map(|(_, val)| val)?),
            b"background" => StyleAttr::Background(parse_color(&value.value).map(|(_, val)| val)?),
            _ => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    value.key.as_ref(),
                    nom::error::ErrorKind::Tag,
                )));
            }
        };

        // wrap into
        match value.key.prefix() {
            Some(prefix) => match prefix.as_ref() {
                b"hover" => Ok(StyleAttr::Hover(Box::new(style))),
                b"active" => Ok(StyleAttr::Active(Box::new(style))),
                _ => todo!(),
            },
            None => Ok(style),
        }
    }
}

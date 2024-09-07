use crate::{
    error::AttributeError,
    parse::{parse_color, parse_ui_rect, parse_val},
};
use bevy::prelude::*;
use quick_xml::events::attributes::Attribute;
use serde::{Deserialize, Serialize};

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
    BorderRadius(UiRect),

    // -----
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
    }
}

impl<'a> TryFrom<&'a Attribute<'a>> for StyleAttr {
    type Error = crate::error::AttributeError;

    #[rustfmt::skip]
    fn try_from(value: &'a Attribute<'a>) -> Result<Self, Self::Error> {
        let style = match value.key.local_name().as_ref() {
            b"height" => StyleAttr::Height(to_value(&value.value)?),
            b"width" => StyleAttr::Width(to_value(&value.value)?),
            b"padding" => StyleAttr::Padding(to_rect(&value.value)?),
            b"margin" => StyleAttr::Margin(to_rect(&value.value)?),
            b"border" => StyleAttr::Border(to_rect(&value.value)?),
            b"background" => StyleAttr::Background(to_color(&value.value)?),
            b"border_color" => StyleAttr::Background(to_color(&value.value)?),
            b"border_radius" => StyleAttr::BorderRadius(to_rect(&value.value)?),
            _ => {
                let token = String::from_utf8(value.value.to_vec()).unwrap_or_default();
                return Err(AttributeError::UnkownToken(token));
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

fn to_value(input: &[u8]) -> Result<Val, AttributeError> {
    parse_val(input).map(|(_, val)| val).map_err(|_| {
        let code = std::str::from_utf8(input)
            .map(|c| c.to_string())
            .unwrap_or_default();
        AttributeError::FailedToParseVal(code)
    })
}

fn to_rect(input: &[u8]) -> Result<UiRect, AttributeError> {
    parse_ui_rect(input).map(|(_, val)| val).map_err(|_| {
        let code = std::str::from_utf8(input)
            .map(|c| c.to_string())
            .unwrap_or_default();
        AttributeError::FailedToParseRect(code)
    })
}

fn to_color(input: &[u8]) -> Result<Color, AttributeError> {
    parse_color(input).map(|(_, color)| color).map_err(|_| {
        let code = std::str::from_utf8(input)
            .map(|c| c.to_string())
            .unwrap_or_default();
        AttributeError::FailedToParseColor(code)
    })
}

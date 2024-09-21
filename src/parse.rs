use crate::data::{Action, AttrTokens, Attribute, StyleAttr, Template, XNode};
use crate::error::ParseError;
use crate::prelude::NodeType;
use bevy::ui::{
    AlignContent, AlignItems, AlignSelf, Direction, Display, FlexDirection, FlexWrap, GridAutoFlow,
    GridPlacement, GridTrack, JustifyContent, JustifyItems, JustifySelf, Overflow, OverflowAxis,
    PositionType, RepeatedGridTrack,
};
use bevy::utils::HashMap;
use bevy::{
    color::Color,
    ui::{UiRect, Val},
};
use nom::bytes::complete::{is_a, is_not, take_until, take_while1};
use nom::combinator::{flat_map, map_parser, not, peek, rest};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::terminated;
use nom::Parser;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n},
    character::complete::multispace0,
    combinator::{complete, map, map_res},
    number::complete::float,
    sequence::{delimited, preceded, tuple, Tuple},
    IResult,
};

struct XmlAttr<'a> {
    prefix: Option<&'a [u8]>,
    key: &'a [u8],
    value: &'a [u8],
}

impl std::fmt::Debug for XmlAttr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(prefix:{} key:{} value:{})",
            std::str::from_utf8(self.prefix.unwrap_or_default()).unwrap_or_default(),
            std::str::from_utf8(self.key).unwrap_or_default(),
            std::str::from_utf8(self.value).unwrap_or_default(),
        )
    }
}

pub(crate) fn parse_template(input: &[u8]) -> Result<Template, ParseError> {
    let (input, _) = trim_comments(input)?;
    let (input, _xml_header) = alt((
        delimited(tag("<?"), take_until("?>"), tag("?>")).map(Some),
        |i| Ok((i, None)),
    ))(input)?;
    let (_, xml) = parse_xml_node(input)?;

    let mut name = None;
    let mut properties = HashMap::default();
    let mut root = vec![];

    for child in xml.children.iter() {
        match child.name {
            b"property" => {
                if let (Some(key), Some(value)) = (
                    child
                        .attributes
                        .iter()
                        .find_map(|attr| (attr.key == b"name").then_some(attr.value)),
                    child.value,
                ) {
                    let str_key = String::from_utf8_lossy(key).to_string();
                    let str_val = String::from_utf8_lossy(value).to_string();
                    properties.insert(str_key, str_val);
                };
            }
            b"name" => {
                if let Some(content) = child.value {
                    let str_name = String::from_utf8_lossy(content).to_string();
                    name = Some(str_name);
                };
            }
            _ => root.push(XNode::try_from(child)?),
        }
    }

    Ok(Template {
        name,
        properties,
        root,
    })
}

// try from
impl TryFrom<&Xml<'_>> for XNode {
    type Error = ParseError;

    fn try_from(xml: &Xml<'_>) -> Result<Self, Self::Error> {
        let (_, node_type) = parse_node_type(xml.name)?;

        let content = xml
            .value
            .map(|bytes| String::from_utf8_lossy(bytes).to_string());

        let mut styles = vec![];
        let mut defs = vec![];
        let mut event_listener = vec![];
        let mut uncompiled = vec![];
        let mut tags = vec![];
        let mut src = None;
        let mut id = None;
        let mut target = None;
        let mut watch = None;

        for attr in xml.attributes.iter() {
            match attribute_from_parts(attr.prefix, attr.key, attr.value).map(|(_, a)| a)? {
                Attribute::Style(style_attr) => styles.push(style_attr),
                Attribute::PropertyDefinition(key, val) => defs.push((key, val)),
                Attribute::Uncompiled(attr_tokens) => uncompiled.push(attr_tokens),
                Attribute::Action(action) => event_listener.push(action),
                Attribute::Path(path) => src = Some(path),
                Attribute::Target(tar) => target = Some(tar),
                Attribute::Id(i) => id = Some(i),
                Attribute::Tag(key, val) => tags.push((key, val)),
                Attribute::Watch(watch_id) => watch = Some(watch_id),
            }
        }

        // dbg!(&src);

        let mut children = vec![];
        for child in xml.children.iter() {
            let node: XNode = child.try_into()?;
            children.push(node);
        }

        Ok(Self {
            src,
            styles,
            target,
            watch,
            uncompiled,
            id,
            tags,
            defs,
            event_listener,
            content,
            node_type,
            children,
        })
    }
}

struct Xml<'a> {
    prefix: Option<&'a [u8]>,
    name: &'a [u8],
    value: Option<&'a [u8]>,
    attributes: Vec<XmlAttr<'a>>,
    children: Vec<Xml<'a>>,
}

impl std::fmt::Debug for Xml<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n prefix:{} \n name:{} \n value:{} \n attributes:{:?} \n children:{:?}",
            std::str::from_utf8(self.prefix.unwrap_or_default()).unwrap_or_default(),
            std::str::from_utf8(self.name).unwrap_or_default(),
            std::str::from_utf8(self.value.unwrap_or_default()).unwrap_or_default(),
            self.attributes,
            self.children,
        )
    }
}

fn parse_xml_node(input: &[u8]) -> IResult<&[u8], Xml> {
    let (input, _) = trim_comments(input)?;

    not(tag("</"))(input)?;

    let (input, (prefix, start_name)) =
        preceded(tag("<"), tuple((parse_prefix0, take_snake)))(input)?;

    let (input, attributes) = parse_xml_attr(input)?;

    let (input, is_empty) = alt((
        preceded(multispace0, tag("/>")).map(|_| true),
        preceded(multispace0, tag(">")).map(|_| false),
    ))(input)?;

    if is_empty {
        return Ok((
            input,
            Xml {
                prefix,
                name: start_name,
                attributes,
                value: None,
                children: vec![],
            },
        ));
    }

    let (input, children) = many0(parse_xml_node)(input)?;
    let (input, _) = trim_comments(input)?;
    let (input, value) = map(take_while(|b: u8| b != b'<'), |c: &[u8]| {
        (c.len() > 0).then_some(c)
    })(input)?;
    let (input, (end_prefix, end_name)) = parse_xml_end(input)?;
    if start_name != end_name || prefix != end_prefix {
        return Err(nom::Err::Failure(nom::error::make_error(
            end_name,
            nom::error::ErrorKind::TagClosure,
        )));
    }
    Ok((
        input,
        Xml {
            prefix,
            name: start_name,
            attributes,
            value,
            children,
        },
    ))
}

fn parse_xml_end(input: &[u8]) -> IResult<&[u8], (Option<&[u8]>, &[u8])> {
    let (input, (_, prefix, end_tag, _)) =
        tuple((tag("</"), parse_prefix0, take_snake, tag(">")))(input)?;

    Ok((input, (prefix, end_tag)))
}

fn parse_xml_attr(input: &[u8]) -> IResult<&[u8], Vec<XmlAttr>> {
    many0(map(
        tuple((
            preceded(trim_comments, parse_prefix0),
            terminated(take_snake, tag("=")),
            delimited(tag("\""), is_not("\""), tag("\"")),
        )),
        |(prefix, key, value)| XmlAttr { prefix, key, value },
    ))(input)
}

fn trim_comments(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, trimmed) = nom::character::complete::multispace0(input)?;
    let o: Result<(&[u8], Vec<&[u8]>), nom::Err<nom::error::Error<&[u8]>>> = many0(terminated(
        delimited(tag("<!--"), take_until("-->"), tag("-->")),
        multispace0,
    ))(input);

    o.map(|(r, _)| (r, "".as_bytes()))
        .or_else(|_| Ok((input, trimmed)))
}

fn parse_node_type(input: &[u8]) -> IResult<&[u8], NodeType> {
    alt((
        map(tag("node"), |_| NodeType::Node),
        map(tag("img"), |_| NodeType::Image),
        map(tag("include"), |_| NodeType::Include),
        map(tag("button"), |_| NodeType::Button),
        map(tag("text"), |_| NodeType::Text),
        map(tag("slot"), |_| NodeType::Slot),
        map(tag("template"), |_| NodeType::Template),
        map(take_while1(|u: u8| u.is_ascii_alphabetic()), |val| {
            let custom = String::from_utf8_lossy(val).to_string();
            NodeType::Custom(custom)
        }),
    ))(input)
}

fn parse_prop_var<'a>(
    prefix: Option<&'a [u8]>,
    key: &'a [u8],
    value: &'a [u8],
) -> Option<Attribute> {
    let result: IResult<&[u8], &[u8]> = delimited(tag("{"), is_not("}"), tag("}"))(value);
    match result {
        Ok((_, prop)) => {
            return Some(Attribute::Uncompiled(AttrTokens {
                prefix: prefix.map(|p| String::from_utf8_lossy(p).to_string()),
                ident: String::from_utf8_lossy(key).to_string(),
                key: String::from_utf8_lossy(prop).to_string(),
            }));
        }
        Err(_) => None,
    }
}

pub(crate) fn attribute_from_parts<'a>(
    prefix: Option<&'a [u8]>,
    key: &'a [u8],
    value: &'a [u8],
) -> IResult<&'a [u8], Attribute> {
    if let Some(attr) = parse_prop_var(prefix, key, value) {
        return Ok((b"", attr));
    }

    match prefix {
        Some(b"tag") => {
            let (_, prop_ident) = as_string(key)?;
            let (_, prop_value) = as_string(value)?;
            return Ok((b"", Attribute::Tag(prop_ident, prop_value)));
        }
        Some(b"prop") => {
            let (_, prop_ident) = as_string(key)?;
            let (_, prop_value) = as_string(value)?;
            return Ok((b"", Attribute::PropertyDefinition(prop_ident, prop_value)));
        }
        _ => (),
    };

    let attribute = match key {
        b"watch" => Attribute::Watch(as_string(value).map(|(_, hash)| hash)?),
        b"id" => Attribute::Id(as_string(value).map(|(_, hash)| hash)?),
        b"target" => Attribute::Target(as_string(value).map(|(_, hash)| hash)?),
        b"src" => Attribute::Path(as_string(value).map(|(_, string)| string)?),
        b"onexit" => Attribute::Action(Action::OnExit(
            as_string_list(value).map(|(_, string)| string)?,
        )),
        b"onenter" => Attribute::Action(Action::OnEnter(
            as_string_list(value).map(|(_, string)| string)?,
        )),
        b"onpress" => Attribute::Action(Action::OnPress(
            as_string_list(value).map(|(_, string)| string)?,
        )),
        b"onspawn" => Attribute::Action(Action::OnSpawn(
            as_string_list(value).map(|(_, string)| string)?,
        )),
        any => match parse_style(prefix, any, value) {
            Ok((_, style)) => Attribute::Style(style),
            Err(_) => {
                let (_, value) = as_string(value)?;
                let (_, key) = as_string(any)?;
                Attribute::PropertyDefinition(key, value)
            }
        },
    };

    Ok((b"", attribute))
}

#[rustfmt::skip]
fn parse_style<'a>(
    prefix: Option<&'a [u8]>,
    ident: &'a [u8],
    value: &'a [u8],
) -> IResult<&'a [u8], StyleAttr> {
    let (input, style) = match ident {
        b"position" => map(parse_position_type, StyleAttr::Position)(value)?,
        b"display" => map(parse_display, StyleAttr::Display)(value)?,
        b"overflow" => map(parse_overflow, StyleAttr::Overflow)(value)?,
        b"direction" => map(parse_direction, StyleAttr::Direction)(value)?,
        // align & justify
        b"align_self" => map(parse_align_self, StyleAttr::AlignSelf)(value)?,
        b"align_items" => map(parse_align_items, StyleAttr::AlignItems)(value)?,
        b"align_content" => map(parse_align_content, StyleAttr::AlignContent)(value)?,
        b"justify_self" => map(parse_justify_self, StyleAttr::JustifySelf)(value)?,
        b"justify_items" => map(parse_justify_items, StyleAttr::JustifyItems)(value)?,
        b"justify_content" => map(parse_justify_content, StyleAttr::JustifyContent)(value)?,
        // flex
        b"flex_direction" => map(parse_flex_direction, StyleAttr::FlexDirection)(value)?,
        b"flex_wrap" => map(parse_flex_wrap, StyleAttr::FlexWrap)(value)?,
        b"flex_grow" => map(float, StyleAttr::FlexGrow)(value)?,
        b"flex_shrink" => map(float, StyleAttr::FlexShrink)(value)?,
        b"flex_basis" => map(parse_val, StyleAttr::FlexBasis)(value)?,
        b"row_gap" => map(parse_val, StyleAttr::RowGap)(value)?,
        b"column_gap" => map(parse_val, StyleAttr::ColumnGap)(value)?,

        // grid
        b"grid_auto_flow" => map(parse_auto_flow, |v| StyleAttr::GridAutoFlow(v))(value)?,
        b"grid_auto_rows" => map(many0(parse_grid_track), |v| StyleAttr::GridAutoRows(v))(value)?,
        b"grid_auto_columns" => map(many0(parse_grid_track), |v| StyleAttr::GridAutoColumns(v))(value)?,
        b"grid_template_rows" => map(many0(parse_grid_track_repeated), |v| StyleAttr::GridTemplateRows(v))(value)?,
        b"grid_template_columns" => map(many0(parse_grid_track_repeated), |v| StyleAttr::GridTemplateColumns(v))(value)?,
        b"grid_row" => map(parse_grid_placement, |v| StyleAttr::GridRow(v))(value)?,
        b"grid_column" => map(parse_grid_placement, |v| StyleAttr::GridColumn(v))(value)?,

        // values
        b"font" => map(as_string, StyleAttr::Font)(value)?,
        b"font_color" => map(parse_color, StyleAttr::FontColor)(value)?,
        b"font_size" => map(parse_float, StyleAttr::FontSize)(value)?,
        b"delay" => map(parse_float, StyleAttr::Delay)(value)?,
        b"max_height" => map(parse_val, StyleAttr::MaxHeight)(value)?,
        b"max_width" => map(parse_val, StyleAttr::MaxWidth)(value)?,
        b"min_height" => map(parse_val, StyleAttr::MinHeight)(value)?,
        b"min_width" => map(parse_val, StyleAttr::MinWidth)(value)?,
        b"bottom" => map(parse_val, StyleAttr::Bottom)(value)?,
        b"top" => map(parse_val, StyleAttr::Top)(value)?,
        b"right" => map(parse_val, StyleAttr::Right)(value)?,
        b"left" => map(parse_val, StyleAttr::Left)(value)?,
        b"height" => map(parse_val, StyleAttr::Height)(value)?,
        b"width" => map(parse_val, StyleAttr::Width)(value)?,
        b"padding" => map(parse_ui_rect, StyleAttr::Padding)(value)?,
        b"margin" => map(parse_ui_rect, StyleAttr::Margin)(value)?,
        b"border" => map(parse_ui_rect, StyleAttr::Border)(value)?,
        b"border_radius" => map(parse_ui_rect, StyleAttr::BorderRadius)(value)?,
        b"background" => map(parse_color, StyleAttr::Background)(value)?,
        b"border_color" => map(parse_color, StyleAttr::BorderColor)(value)?,
        _ => {
            return Err(nom::Err::Error(nom::error::make_error(
                ident,
                nom::error::ErrorKind::NoneOf,
            )))
        }
    };

    match prefix {
        Some(b"pressed") => Ok((input, StyleAttr::Pressed(Box::new(style)))),
        Some(b"hover") => Ok((input, StyleAttr::Hover(Box::new(style)))),
        _ => Ok((input, style)),
    }
}

fn parse_float(input: &[u8]) -> IResult<&[u8], f32> {
    nom::number::complete::float(input)
}

fn parse_position_type(input: &[u8]) -> IResult<&[u8], PositionType> {
    alt((
        map(tag("absolute"), |_| PositionType::Absolute),
        map(tag("relative"), |_| PositionType::Relative),
    ))(input)
}

fn parse_display(input: &[u8]) -> IResult<&[u8], Display> {
    alt((
        map(tag("none"), |_| Display::None),
        map(tag("flex"), |_| Display::Flex),
        map(tag("block"), |_| Display::Block),
        map(tag("grid"), |_| Display::Grid),
    ))(input)
}

fn parse_direction(input: &[u8]) -> IResult<&[u8], Direction> {
    alt((
        map(tag("inherit"), |_| Direction::Inherit),
        map(tag("left_to_right"), |_| Direction::LeftToRight),
        map(tag("right_to_left"), |_| Direction::RightToLeft),
    ))(input)
}

fn parse_overflow(input: &[u8]) -> IResult<&[u8], Overflow> {
    let (input, (x, _, y)) = tuple((parse_overflow_axis, multispace0, parse_overflow_axis))(input)?;
    Ok((input, Overflow { x, y }))
}

fn parse_overflow_axis(input: &[u8]) -> IResult<&[u8], OverflowAxis> {
    alt((
        map(tag("visible"), |_| OverflowAxis::Visible),
        map(tag("hidden"), |_| OverflowAxis::Hidden),
        map(tag("clip"), |_| OverflowAxis::Clip),
    ))(input)
}

fn parse_align_items(input: &[u8]) -> IResult<&[u8], AlignItems> {
    alt((
        map(tag("default"), |_| AlignItems::Default),
        map(tag("center"), |_| AlignItems::Center),
        map(tag("start"), |_| AlignItems::Start),
        map(tag("flex_end"), |_| AlignItems::FlexEnd),
        map(tag("stretch"), |_| AlignItems::Stretch),
        map(tag("end"), |_| AlignItems::End),
        map(tag("baseline"), |_| AlignItems::Baseline),
        map(tag("flex_start"), |_| AlignItems::FlexStart),
    ))(input)
}

fn parse_align_content(input: &[u8]) -> IResult<&[u8], AlignContent> {
    alt((
        map(tag("center"), |_| AlignContent::Center),
        map(tag("start"), |_| AlignContent::Start),
        map(tag("flex_end"), |_| AlignContent::FlexEnd),
        map(tag("stretch"), |_| AlignContent::Stretch),
        map(tag("end"), |_| AlignContent::End),
        map(tag("space_evenly"), |_| AlignContent::SpaceEvenly),
        map(tag("space_around"), |_| AlignContent::SpaceAround),
        map(tag("space_between"), |_| AlignContent::SpaceBetween),
        map(tag("flex_start"), |_| AlignContent::FlexStart),
    ))(input)
}

fn parse_align_self(input: &[u8]) -> IResult<&[u8], AlignSelf> {
    alt((
        map(tag("auto"), |_| AlignSelf::Auto),
        map(tag("center"), |_| AlignSelf::Center),
        map(tag("start"), |_| AlignSelf::Start),
        map(tag("flex_end"), |_| AlignSelf::FlexEnd),
        map(tag("stretch"), |_| AlignSelf::Stretch),
        map(tag("end"), |_| AlignSelf::End),
        map(tag("flex_start"), |_| AlignSelf::FlexStart),
    ))(input)
}

fn parse_justify_items(input: &[u8]) -> IResult<&[u8], JustifyItems> {
    alt((
        map(tag("default"), |_| JustifyItems::Default),
        map(tag("center"), |_| JustifyItems::Center),
        map(tag("start"), |_| JustifyItems::Start),
        map(tag("stretch"), |_| JustifyItems::Stretch),
        map(tag("end"), |_| JustifyItems::End),
        map(tag("baseline"), |_| JustifyItems::Baseline),
    ))(input)
}

fn parse_justify_content(input: &[u8]) -> IResult<&[u8], JustifyContent> {
    alt((
        map(tag("center"), |_| JustifyContent::Center),
        map(tag("start"), |_| JustifyContent::Start),
        map(tag("flex_end"), |_| JustifyContent::FlexEnd),
        map(tag("stretch"), |_| JustifyContent::Stretch),
        map(tag("end"), |_| JustifyContent::End),
        map(tag("space_evenly"), |_| JustifyContent::SpaceEvenly),
        map(tag("space_around"), |_| JustifyContent::SpaceAround),
        map(tag("space_between"), |_| JustifyContent::SpaceBetween),
        map(tag("flex_start"), |_| JustifyContent::FlexStart),
    ))(input)
}

fn parse_justify_self(input: &[u8]) -> IResult<&[u8], JustifySelf> {
    alt((
        map(tag("auto"), |_| JustifySelf::Auto),
        map(tag("center"), |_| JustifySelf::Center),
        map(tag("start"), |_| JustifySelf::Start),
        map(tag("stretch"), |_| JustifySelf::Stretch),
        map(tag("end"), |_| JustifySelf::End),
        map(tag("baseline"), |_| JustifySelf::Baseline),
    ))(input)
}

fn parse_flex_direction(input: &[u8]) -> IResult<&[u8], FlexDirection> {
    alt((
        map(tag("row"), |_| FlexDirection::Row),
        map(tag("column"), |_| FlexDirection::Column),
        map(tag("column_reverse"), |_| FlexDirection::ColumnReverse),
        map(tag("row_reverse"), |_| FlexDirection::RowReverse),
        map(tag("default"), |_| FlexDirection::DEFAULT),
    ))(input)
}

fn parse_flex_wrap(input: &[u8]) -> IResult<&[u8], FlexWrap> {
    alt((
        map(tag("wrap"), |_| FlexWrap::Wrap),
        map(tag("no_wrap"), |_| FlexWrap::NoWrap),
        map(tag("wrap_reverse"), |_| FlexWrap::WrapReverse),
    ))(input)
}

fn as_string(input: &[u8]) -> IResult<&[u8], String> {
    map(rest, |v| String::from_utf8_lossy(v).to_string())(input)
}

fn as_string_list(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    map(
        separated_list1(tag(","), take_while1(|b: u8| b != b',' && b != b'"')),
        |bytes: Vec<&[u8]>| {
            bytes
                .iter()
                .map(|b| String::from_utf8_lossy(b).to_string())
                .collect::<Vec<_>>()
        },
    )(input)
}

#[rustfmt::skip]
fn parse_prefix0(input: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    alt((
        terminated(take_snake,tag(":")).map(Some),
        |i| Ok((i, None)),
    ))(input)
}

fn take_snake(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(|b: u8| b.is_ascii_alphabetic() || b == b'_')(input)
}

/// convert string values to uirect
/// 20px/% single
/// 10px/% 10px axis
/// 10px 10px 10px 10px rect
fn parse_ui_rect(input: &[u8]) -> IResult<&[u8], UiRect> {
    alt((
        // 10px 10px 10px 10px
        complete(map(
            tuple((
                preceded(multispace0, parse_val),
                preceded(multispace0, parse_val),
                preceded(multispace0, parse_val),
                preceded(multispace0, parse_val),
            )),
            |(top, right, bottom, left)| UiRect {
                left,
                right,
                top,
                bottom,
            },
        )),
        // 10px 10px
        complete(map(
            tuple((
                preceded(multispace0, parse_val),
                preceded(multispace0, parse_val),
            )),
            |(x, y)| UiRect::axes(x, y),
        )),
        // 10px
        complete(map(preceded(multispace0, parse_val), |all| {
            UiRect::all(all)
        })),
    ))(input)
}

// grid_template_rows="auto 10% 10%"
fn parse_grid_track(input: &[u8]) -> IResult<&[u8], GridTrack> {
    let (input, track) = delimited(
        multispace0,
        alt((
            map(tag("auto"), |_| GridTrack::auto()),
            map(tag("min"), |_| GridTrack::min_content()),
            map(tag("max"), |_| GridTrack::max_content()),
            map(tuple((float, tag("px"))), |(val, _)| GridTrack::px(val)),
            map(tuple((float, tag("%"))), |(val, _)| GridTrack::percent(val)),
            map(tuple((float, tag("fr"))), |(val, _)| GridTrack::fr(val)),
            map(tuple((float, tag("flex"))), |(val, _)| GridTrack::flex(val)),
            map(tuple((float, tag("vh"))), |(val, _)| GridTrack::vh(val)),
            map(tuple((float, tag("vw"))), |(val, _)| GridTrack::vw(val)),
            map(tuple((float, tag("vmin"))), |(val, _)| GridTrack::vmin(val)),
            map(tuple((float, tag("vmax"))), |(val, _)| GridTrack::vmax(val)),
        )),
        multispace0,
    )(input)?;

    Ok((input, track))
}

// (5, auto)
// (2, 150px)
#[rustfmt::skip]
fn parse_grid_track_repeated(input: &[u8]) -> IResult<&[u8], RepeatedGridTrack> {

    let (input, (_,repeat,_, value ,_)) = tuple((
        preceded(multispace0, tag("(")),
        preceded(multispace0, map_res(integer,|i:i64| u16::try_from(i))),
        preceded(multispace0, tag(",")),
        preceded(multispace0, take_while1(|b: u8| b.is_ascii_alphanumeric())),
        preceded(multispace0, tag(")")),
    ))(input)?;

    let (_, track) : (&[u8], RepeatedGridTrack) = alt((
            map(tag("auto"), |_| RepeatedGridTrack::auto::<RepeatedGridTrack>(repeat)),
            map(tag("min"), |_| RepeatedGridTrack::min_content(repeat)),
            map(tag("max"), |_| RepeatedGridTrack::max_content(repeat)),
            map(tuple((float, tag("px"))), |(val, _)| RepeatedGridTrack::px(repeat,val)),
            map(tuple((float, tag("%"))), |(val, _)| RepeatedGridTrack::percent(repeat,val)),
            map(tuple((float, tag("fr"))), |(val, _)| RepeatedGridTrack::fr(repeat,val)),
            map(tuple((float, tag("flex"))), |(val, _)| RepeatedGridTrack::flex(repeat,val)),
            map(tuple((float, tag("vh"))), |(val, _)| RepeatedGridTrack::vh(repeat,val)),
            map(tuple((float, tag("vw"))), |(val, _)| RepeatedGridTrack::vw(repeat,val)),
            map(tuple((float, tag("vmin"))), |(val, _)| RepeatedGridTrack::vmin(repeat,val)),
            map(tuple((float, tag("vmax"))), |(val, _)| RepeatedGridTrack::vmax(repeat,val)),
    ))(value)?;

    Ok((input, track))
}

fn parse_auto_flow(input: &[u8]) -> IResult<&[u8], GridAutoFlow> {
    delimited(
        multispace0,
        alt((
            map(tag("row"), |_| GridAutoFlow::Row),
            map(tag("column"), |_| GridAutoFlow::Column),
            map(tag("row_dense"), |_| GridAutoFlow::RowDense),
            map(tag("column_dense"), |_| GridAutoFlow::ColumnDense),
        )),
        multispace0,
    )(input)
}

fn integer(input: &[u8]) -> IResult<&[u8], i64> {
    let (input, integer) = map_res(
        map_res(take_while(|u: u8| u.is_ascii_digit() || u == b'-'), |d| {
            std::str::from_utf8(d)
        }),
        |str| str.parse::<i64>(),
    )(input)?;

    Ok((input, integer))
}

// auto
// start_span(5,5)
// end_span(5,5)
fn parse_grid_placement(input: &[u8]) -> IResult<&[u8], GridPlacement> {
    let (input, _) = multispace0(input)?;
    let (input, ident) = take_while1(|b: u8| b != b'(' && b != b'"')(input)?;
    match ident {
        b"auto" => Ok((input, GridPlacement::auto())),
        // span(5)
        b"span" => map(
            delimited(
                tag("("),
                delimited(
                    multispace0,
                    map(integer, |i| u16::try_from(i).unwrap_or_default()),
                    multispace0,
                ),
                tag(")"),
            ),
            |v| GridPlacement::span(v),
        )(input),
        // end(5)
        b"end" => map(
            delimited(
                tag("("),
                delimited(
                    multispace0,
                    map(integer, |i| i16::try_from(i).unwrap_or_default()),
                    multispace0,
                ),
                tag(")"),
            ),
            |v| GridPlacement::end(v),
        )(input),
        // start(5)
        b"start" => map(
            delimited(
                tag("("),
                delimited(
                    multispace0,
                    map(integer, |i| i16::try_from(i).unwrap_or_default()),
                    multispace0,
                ),
                tag(")"),
            ),
            |v| GridPlacement::start(v),
        )(input),
        // start_span(5,5)
        b"start_span" => map(
            delimited(
                tag("("),
                tuple((
                    delimited(
                        multispace0,
                        map(integer, |i| i16::try_from(i).unwrap_or_default()),
                        multispace0,
                    ),
                    tag(","),
                    delimited(
                        multispace0,
                        map(integer, |i| u16::try_from(i).unwrap_or_default()),
                        multispace0,
                    ),
                )),
                tag(")"),
            ),
            |(a, _, b)| GridPlacement::start_span(a, b),
        )(input),
        // end_span(5,5)
        b"end_span" => map(
            delimited(
                tag("("),
                tuple((
                    delimited(
                        multispace0,
                        map(integer, |i| i16::try_from(i).unwrap_or_default()),
                        multispace0,
                    ),
                    tag(","),
                    delimited(
                        multispace0,
                        map(integer, |i| u16::try_from(i).unwrap_or_default()),
                        multispace0,
                    ),
                )),
                tag(")"),
            ),
            |(a, _, b)| GridPlacement::end_span(a, b),
        )(input),
        _ => Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::LengthValue,
        ))),
    }
}

/// 10px 10%
fn parse_val(input: &[u8]) -> IResult<&[u8], Val> {
    delimited(
        multispace0,
        alt((
            map(tag("auto"), |_| Val::Auto),
            map(tag("0"), |_| Val::Px(0.)),
            map(tuple((float, tag("px"))), |(val, _)| Val::Px(val)),
            map(tuple((float, tag("%"))), |(val, _)| Val::Percent(val)),
            map(tuple((float, tag("vw"))), |(val, _)| Val::Vw(val)),
            map(tuple((float, tag("vh"))), |(val, _)| Val::Vh(val)),
            map(tuple((float, tag("vmin"))), |(val, _)| Val::VMin(val)),
            map(tuple((float, tag("vmax"))), |(val, _)| Val::VMax(val)),
        )),
        multispace0,
    )(input)
}

// rgb(1,1,1)
// rgba(1,1,1,1)
// #000000
// #FFF
#[rustfmt::skip]
fn parse_color(input: &[u8]) -> IResult<&[u8], Color> {
    delimited(
        multispace0,
        alt((
            parse_rgba_color,
            parse_rgb_color,
            color_hex8_parser,
            color_hex6_parser,
            color_hex4_parser,
            color_hex3_parser,
        )),
        multispace0,
    )(input)
}

// rgba(1,1,1,1)
fn parse_rgba_color(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("rgba")(input)?;

    let (input, (r, _, g, _, b, _, a)) = delimited(
        tag("("),
        tuple((float, tag(","), float, tag(","), float, tag(","), float)),
        tag(")"),
    )(input)?;

    Ok((input, Color::linear_rgba(r, g, b, a)))
}

// rgb(1,1,1)
fn parse_rgb_color(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("rgb")(input)?;

    let (input, (r, _, g, _, b)) = delimited(
        tag("("),
        tuple((float, tag(","), float, tag(","), float)),
        tag(")"),
    )(input)?;

    Ok((input, Color::linear_rgb(r, g, b)))
}

// #FFFFFFFF (with alpha)
fn color_hex8_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("#")(input)?;

    if input.len() != 8 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::LengthValue,
        )));
    }

    let (input, (r, g, b, a)) = tuple((hex_byte, hex_byte, hex_byte, hex_byte))(input)?;
    Ok((
        input,
        Color::LinearRgba(Color::srgba_u8(r, g, b, a).to_linear()),
    ))
}

// #FFFFFF
fn color_hex6_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("#")(input)?;

    if input.len() != 6 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::LengthValue,
        )));
    }

    let (input, (r, g, b)) = tuple((hex_byte, hex_byte, hex_byte))(input)?;
    Ok((
        input,
        Color::LinearRgba(Color::srgb_u8(r, g, b).to_linear()),
    ))
}

// #FFFF (with alpha)
fn color_hex4_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("#")(input)?;

    if input.len() != 4 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::LengthValue,
        )));
    }

    let (input, (r, g, b, a)) = tuple((hex_nib, hex_nib, hex_nib, hex_nib))(input)?;
    Ok((
        input,
        Color::LinearRgba(Color::srgba_u8(r, g, b, a).to_linear()),
    ))
}

// short
// #FFF
fn color_hex3_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("#")(input)?;

    if input.len() != 3 {
        return Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::LengthValue,
        )));
    }

    let (input, (r, g, b)) = tuple((hex_nib, hex_nib, hex_nib))(input)?;
    Ok((
        input,
        Color::LinearRgba(Color::srgb_u8(r, g, b).to_linear()),
    ))
}

/// FF
fn hex_byte(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, val) = map_res(take_while_m_n(2, 2, is_hex_digit), from_hex_byte)(input)?;
    Ok((input, val))
    // map_res(take_while_m_n(2, 2, is_hex_digit), from_hex_byte)(input)
}

/// F
fn hex_nib(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(take_while_m_n(1, 1, is_hex_digit), from_hex_nib)(input)
}

fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// FF -> u8
fn from_hex_byte(input: &[u8]) -> Result<u8, std::num::ParseIntError> {
    let str = std::str::from_utf8(input).expect("fix later");
    u8::from_str_radix(format!("{}", str).as_str(), 16)
}

/// F -> u8
fn from_hex_nib(input: &[u8]) -> Result<u8, std::num::ParseIntError> {
    let str = std::str::from_utf8(input).expect("fix later");
    u8::from_str_radix(format!("{}{}", str, str).as_str(), 16)
}

#[cfg(test)]
mod tests {
    use std::string;

    use super::*;
    use test_case::test_case;

    #[test_case("#FFFFFFFF", Color::WHITE)]
    #[test_case("#FFFFFF", Color::WHITE)]
    #[test_case("#FFFF", Color::WHITE)]
    #[test_case("#FFF", Color::WHITE)]
    #[test_case("rgb(1,1,1)", Color::WHITE)]
    #[test_case("rgba(1,1,1,1)", Color::WHITE)]
    fn test_color(input: &str, expected: Color) {
        let result = parse_color(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    // #[test_case(r#"font_size="20""#, Attribute::Style(StyleAttr::FontSize(20.)))]
    // #[test_case(r#"prop:myvar="test""#, Attribute::PropertyDefinition("myvar".into(), "test".into()))]
    // #[test_case(r#"onenter="test_enter""#, Attribute::Action(Action::OnEnter(vec!["test_enter".to_string()])))]
    // #[test_case(r#"onspawn="init_inventory""#, Attribute::Action(Action::OnSpawn(vec!["init_inventory".to_string()])))]
    // #[test_case(r#"onpress="test,test_50,test o""#, Attribute::Action(Action::OnPress(vec!["test".to_string(),"test_50".to_string(), "test o".to_string()])))]
    // #[test_case(r#"width="10px""#, Attribute::Style(StyleAttr::Width(Val::Px(10.))))]
    // #[test_case(r#"height="{my_var}""#, Attribute::Uncompiled( AttrTokens{ key: "my_var".into(),prefix: None, ident: "height".into() }))]
    // #[test_case(r#"font_size="{test_that}""#, Attribute::Uncompiled( AttrTokens{ key: "test_that".into(),prefix: None, ident: "font_size".into() }))]
    // fn test_parse_attribute(input: &str, expected: Attribute) {
    //     let result = parse_attribute(input.as_bytes());
    //     assert_eq!(Ok(("".as_bytes(), expected)), result);
    // }

    #[test_case("0", Val::Px(0.))]
    #[test_case("20vw", Val::Vw(20.))]
    #[test_case("20%", Val::Percent(20.))]
    #[test_case("20vh", Val::Vh(20.))]
    #[test_case("20px", Val::Px(20.))]
    #[test_case("20vmin", Val::VMin(20.))]
    #[test_case("20vmax", Val::VMax(20.))]
    fn test_value(input: &str, expected: Val) {
        let result = parse_val(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("auto", GridPlacement::auto())]
    #[test_case("end_span(5,50)", GridPlacement::end_span(5, 50))]
    #[test_case("start_span(-5, 5)", GridPlacement::start_span(-5,5))]
    #[test_case("span( 55  )", GridPlacement::span(55))]
    #[test_case("span(5)", GridPlacement::span(5))]
    fn test_grid_placement(input: &str, expected: GridPlacement) {
        let result = parse_grid_placement(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("min max auto", vec![GridTrack::min_content(), GridTrack::max_content(), GridTrack::auto()])]
    #[test_case("50% auto   8fr   ", vec![GridTrack::percent(50.), GridTrack::auto(), GridTrack::fr(8.)])]
    #[test_case("50px       ", vec![GridTrack::px(50.)])]
    fn test_tracks(input: &str, expected: Vec<GridTrack>) {
        let result = many0(parse_grid_track)(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("(4, 8flex)(1, 50px)", vec![RepeatedGridTrack::flex(4, 8.), RepeatedGridTrack::px(1,50.)])]
    #[test_case("(1, auto)(5, 50fr)", vec![RepeatedGridTrack::auto(1), RepeatedGridTrack::fr(5,50.)])]
    #[test_case("(1, auto)", vec![RepeatedGridTrack::auto(1)])]
    fn test_repeat_tracks(input: &str, expected: Vec<RepeatedGridTrack>) {
        let result = many0(parse_grid_track_repeated)(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("20px", UiRect::all(Val::Px(20.)))]
    #[test_case("20px 10px", UiRect::axes(Val::Px(20.), Val::Px(10.)))]
    #[test_case(
        "5px 10px 5% 6px",
        UiRect{ top:Val::Px(5.), right: Val::Px(10.), bottom: Val::Percent(5.), left: Val::Px(6.)}
    )]
    fn test_rect(input: &str, expected: UiRect) {
        let result = parse_ui_rect(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case(
        "   \n<!-- hello world <button> test thah </button> fdsfsd-->\nok",
        "ok"
    )]
    #[test_case(r#"  <!-- hello <tag/> <""/>world -->    ok"#, "ok")]
    #[test_case("   <!-- hello world -->    ok", "ok")]
    fn test_comments(input: &str, expected: &str) {
        let (remaining, _trimmed) = trim_comments(&input.as_bytes()).unwrap();
        assert_eq!(std::str::from_utf8(remaining).unwrap(), expected);
    }

    #[test_case(r#"    pressed:background="fsdfsf"  pressed:background="fsdfsf"  <!-- test -->    pressed:background="fsdfsf" \n"#)]
    #[test_case(r#"pressed:background="fsdfsf"#)]
    fn test_parse_xml_attr(input: &str) {
        let (_, _attr) = parse_xml_attr(input.as_bytes())
            .map_err(|err| err.map_input(|i| std::str::from_utf8(i).unwrap()))
            .unwrap();

        // dbg!(&attr);
    }

    #[test_case(r#"<node pressed:background="fsdfsf" active="hello"><text p:hello="sdf">hello</text></node>"#)]
    #[test_case(r#"<slot/>"#)]
    #[test_case(r#"<node pressed:background="fsdfsf" active="hello" />"#)]
    #[test_case(r#"<property name="press"><property name="press"></property></property>"#)]
    #[test_case(
        r#"
    <my_template>
        <name>test</name>
        <property this="press">test</property>
        <property this="press">test</property>
        <node></node>
    </my_template>
    "#
    )]
    fn test_parse_xml_node(input: &str) {
        let (_, _xml) = parse_xml_node(input.as_bytes())
            .map_err(|err| err.map_input(|i| std::str::from_utf8(i).unwrap()))
            .unwrap();

        // dbg!(&xml);
    }

    #[test_case("./assets/menu.xml")]
    #[test_case("./assets/panel.xml")]
    #[test_case("./assets/button.xml")]
    #[test_case("./assets/card.xml")]
    fn test_parse_template_full(file_path: &str) {
        let input = std::fs::read_to_string(file_path).unwrap();
        let _template = parse_template(input.as_bytes()).unwrap();
        // dbg!(&template);
    }

    #[test_case(r#"hover:background="{color}""#)]
    fn parse_attribute_parts(input: &str) {
        let (_, attr) = parse_xml_attr(input.as_bytes()).unwrap();
        let first = &attr[0];
        let (_, attribute) = attribute_from_parts(first.prefix, first.key, first.value).unwrap();
        dbg!(attribute);
    }
}

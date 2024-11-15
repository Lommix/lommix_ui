use crate::data::{Action, AttrTokens, Attribute, HtmlTemplate, StyleAttr, XNode};
use crate::prelude::NodeType;
use bevy::sprite::{BorderRect, ImageScaleMode, SliceScaleMode, TextureSlicer};
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
use nom::bytes::complete::{is_not, take_until, take_while1};
use nom::combinator::{map_parser, not, rest};
use nom::multi::{many0, separated_list1};
use nom::sequence::terminated;
use nom::Parser;

#[allow(unused)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while_m_n},
    character::complete::multispace0,
    combinator::{complete, map},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    number::complete::float,
    sequence::{delimited, preceded, tuple},
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

pub(crate) fn parse_template<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], HtmlTemplate, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    trim_comments0(input)?;
    let (input, _xml_header) = alt((
        delimited(tag("<?"), take_until("?>"), tag("?>")).map(Some),
        |i| Ok((i, None)),
    ))(input)?;

    let (_, mut xml) = parse_xml_node(input)?;

    let mut name = None;
    let mut properties = HashMap::default();
    let mut root = vec![];

    for child in xml.children.drain(..) {
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
            _ => {
                let (_, node) = from_raw_xml::<E>(child)?;
                root.push(node);
            }
        }
    }

    Ok((
        "".as_bytes(),
        HtmlTemplate {
            name,
            properties,
            root,
        },
    ))
}

fn trim_comments0<'a, E>(input: &'a [u8]) -> IResult<&[u8], Vec<&[u8]>, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    many0(parse_comment::<E>)(input)
}

fn parse_comment<'a, E>(input: &'a [u8]) -> IResult<&[u8], &[u8], E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    preceded(
        multispace0,
        delimited(tag("<!--"), take_until("-->"), tag("-->")),
    )(input)
}


// try from
fn from_raw_xml<'a, E>(mut xml: Xml<'a>) -> IResult<&[u8], XNode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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
    let mut name = None;

    for attr in xml.attributes.iter() {
        let (_input, compiled_attr) = attribute_from_parts(attr.prefix, attr.key, attr.value)?;
        match compiled_attr {
            Attribute::Style(style_attr) => styles.push(style_attr),
            Attribute::PropertyDefinition(key, val) => {
                match node_type {
                    NodeType::Include | NodeType::Custom(_) | NodeType::Property => {
                        defs.push((key, val))
                    }
                    _ => {
                        // prop defs are not allowed, unless include or custom
                        let err = E::from_error_kind(attr.key, ErrorKind::Verify);
                        return Err(nom::Err::Error(E::add_context(attr.key, "Is not a valid attribute, custom values (properties) are only allowed on includes/custom nodes. Did you misspell a style?", err)));
                    }
                }
            }
            Attribute::Name(s) => name = Some(s),
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
    for child in xml.children.drain(..) {
        let (_, node) = from_raw_xml(child)?;
        children.push(node);
    }

    Ok((
        "".as_bytes(),
        XNode {
            src,
            styles,
            target,
            watch,
            name,
            uncompiled,
            id,
            tags,
            defs,
            event_listener,
            content,
            node_type,
            children,
        },
    ))
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

fn parse_xml_node<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Xml<'a>, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, _) = trim_comments0(input)?;
    let (input, _) = multispace0(input)?;

    not(tag("</"))(input)?;

    let (input, (prefix, start_name)) = preceded(
        tag("<"),
        preceded(multispace0, tuple((parse_prefix0, take_snake))),
    )(input)?;

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

    let (input, _) = trim_comments0(input)?;

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

fn parse_xml_end<'a, E>(input: &'a [u8]) -> IResult<&[u8], (Option<&[u8]>, &[u8]), E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, (_, prefix, end_tag, _)) =
        tuple((tag("</"), parse_prefix0, take_snake, tag(">")))(input)?;

    Ok((input, (prefix, end_tag)))
}

fn parse_xml_attr<'a, E>(input: &'a [u8]) -> IResult<&[u8], Vec<XmlAttr>, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    many0(map(
        tuple((
            preceded(multispace0, parse_prefix0),
            terminated(take_snake, tag("=")),
            delimited(tag("\""), is_not("\""), tag("\"")),
        )),
        |(prefix, key, value)| XmlAttr { prefix, key, value },
    ))(input)
}

fn parse_node_type<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], NodeType, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

pub(crate) fn attribute_from_parts<'a, E>(
    prefix: Option<&'a [u8]>,
    key: &'a [u8],
    value: &'a [u8],
) -> IResult<&'a [u8], Attribute, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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
        any => match parse_style::<E>(prefix, any, value) {
            Ok((_, style)) => Attribute::Style(style),
            Err(_) => {
                let (_, value) = as_string(value)?;
                let (_, key) = as_string(any)?;
                // @todo: this is bad an prevents miss spelling
                Attribute::PropertyDefinition(key, value)
            }
        },
    };

    Ok((b"", attribute))
}

#[rustfmt::skip]
fn parse_style<'a, E>(
    prefix: Option<&'a [u8]>,
    ident: &'a [u8],
    value: &'a [u8],
) -> IResult<&'a [u8], StyleAttr,E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

        //slices
        b"image_scale_mode" => map(parse_image_scale_mode, |v| StyleAttr::ImageScaleMode(v))(value)?,

        // values
        b"font" => map(as_string, StyleAttr::Font)(value)?,
        b"font_color" => map(parse_color, StyleAttr::FontColor)(value)?,
        b"font_size" => map(parse_float, StyleAttr::FontSize)(value)?,
        b"delay" => map(parse_float, StyleAttr::Delay)(value)?,
        b"ease" => map(parse_easing, StyleAttr::Easing)(value)?,
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
            let err = E::from_error_kind(
                ident,
                ErrorKind::NoneOf,
            );
            return Err(nom::Err::Error(E::add_context(ident, "not a valid style or attribute", err)));
        }
    };

    match prefix {
        Some(b"pressed") => Ok((input, StyleAttr::Pressed(Box::new(style)))),
        Some(b"hover") => Ok((input, StyleAttr::Hover(Box::new(style)))),
        Some(b"active") => Ok((input, StyleAttr::Active(Box::new(style)))),
        _ => Ok((input, style)),
    }
}

fn parse_float<'a, E>(input: &'a [u8]) -> IResult<&[u8], f32, E>
where
    E: nom::error::ParseError<&'a [u8]>,
{
    nom::number::complete::float(input)
}

fn parse_easing<'a, E>(input: &'a [u8]) -> IResult<&[u8], interpolation::EaseFunction, E>
where
    E: nom::error::ParseError<&'a [u8]>,
{
    match input {
        b"cubic" => Ok((input, interpolation::EaseFunction::CubicInOut)),
        b"bounce" => Ok((input, interpolation::EaseFunction::BounceOut)),
        b"sine" => Ok((input, interpolation::EaseFunction::SineInOut)),
        b"quad" => Ok((input, interpolation::EaseFunction::QuadraticInOut)),

        b"quadratic_in" => Ok((input, interpolation::EaseFunction::QuadraticIn)),
        b"quadratic_out" => Ok((input, interpolation::EaseFunction::QuadraticOut)),
        b"quadratic_in_out" => Ok((input, interpolation::EaseFunction::QuadraticInOut)),
        b"cubic_in" => Ok((input, interpolation::EaseFunction::CubicIn)),
        b"cubic_out" => Ok((input, interpolation::EaseFunction::CubicOut)),
        b"cubic_in_out" => Ok((input, interpolation::EaseFunction::CubicInOut)),
        b"quartic_in" => Ok((input, interpolation::EaseFunction::QuarticIn)),
        b"quartic_out" => Ok((input, interpolation::EaseFunction::QuarticOut)),
        b"quartic_in_out" => Ok((input, interpolation::EaseFunction::QuarticInOut)),
        b"quintic_in" => Ok((input, interpolation::EaseFunction::QuinticIn)),
        b"quintic_out" => Ok((input, interpolation::EaseFunction::QuinticOut)),
        b"quintic_in_out" => Ok((input, interpolation::EaseFunction::QuinticInOut)),
        b"sine_in" => Ok((input, interpolation::EaseFunction::SineIn)),
        b"sine_out" => Ok((input, interpolation::EaseFunction::SineOut)),
        b"sine_in_out" => Ok((input, interpolation::EaseFunction::SineInOut)),
        b"circular_in" => Ok((input, interpolation::EaseFunction::CircularIn)),
        b"circular_out" => Ok((input, interpolation::EaseFunction::CircularOut)),
        b"circular_in_out" => Ok((input, interpolation::EaseFunction::CircularInOut)),
        b"exponential_in" => Ok((input, interpolation::EaseFunction::ExponentialIn)),
        b"exponential_out" => Ok((input, interpolation::EaseFunction::ExponentialOut)),
        b"exponential_in_out" => Ok((input, interpolation::EaseFunction::ExponentialInOut)),
        b"elastic_in" => Ok((input, interpolation::EaseFunction::ElasticIn)),
        b"elastic_out" => Ok((input, interpolation::EaseFunction::ElasticOut)),
        b"elastic_in_out" => Ok((input, interpolation::EaseFunction::ElasticInOut)),
        b"back_in" => Ok((input, interpolation::EaseFunction::BackIn)),
        b"back_out" => Ok((input, interpolation::EaseFunction::BackOut)),
        b"back_in_out" => Ok((input, interpolation::EaseFunction::BackInOut)),
        b"bounce_in" => Ok((input, interpolation::EaseFunction::BounceIn)),
        b"bounce_out" => Ok((input, interpolation::EaseFunction::BounceOut)),
        b"bounce_in_out" => Ok((input, interpolation::EaseFunction::BounceInOut)),
        _ => Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn parse_position_type<'a, E>(input: &'a [u8]) -> IResult<&[u8], PositionType, E>
where
    E: nom::error::ParseError<&'a [u8]>,
{
    alt((
        map(tag("absolute"), |_| PositionType::Absolute),
        map(tag("relative"), |_| PositionType::Relative),
    ))(input)
}

fn parse_display<'a, E>(input: &'a [u8]) -> IResult<&[u8], Display, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("none"), |_| Display::None),
        map(tag("flex"), |_| Display::Flex),
        map(tag("block"), |_| Display::Block),
        map(tag("grid"), |_| Display::Grid),
    ))(input)
}

fn parse_direction<'a, E>(input: &'a [u8]) -> IResult<&[u8], Direction, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("inherit"), |_| Direction::Inherit),
        map(tag("left_to_right"), |_| Direction::LeftToRight),
        map(tag("right_to_left"), |_| Direction::RightToLeft),
    ))(input)
}

fn parse_overflow<'a, E>(input: &'a [u8]) -> IResult<&[u8], Overflow, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, (x, _, y)) = tuple((parse_overflow_axis, multispace0, parse_overflow_axis))(input)?;
    Ok((input, Overflow { x, y }))
}

fn parse_overflow_axis<'a, E>(input: &'a [u8]) -> IResult<&[u8], OverflowAxis, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("visible"), |_| OverflowAxis::Visible),
        map(tag("hidden"), |_| OverflowAxis::Hidden),
        map(tag("clip"), |_| OverflowAxis::Clip),
    ))(input)
}

fn parse_align_items<'a, E>(input: &'a [u8]) -> IResult<&[u8], AlignItems, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

fn parse_align_content<'a, E>(input: &'a [u8]) -> IResult<&[u8], AlignContent, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

fn parse_align_self<'a, E>(input: &'a [u8]) -> IResult<&[u8], AlignSelf, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

fn parse_justify_items<'a, E>(input: &'a [u8]) -> IResult<&[u8], JustifyItems, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("default"), |_| JustifyItems::Default),
        map(tag("center"), |_| JustifyItems::Center),
        map(tag("start"), |_| JustifyItems::Start),
        map(tag("stretch"), |_| JustifyItems::Stretch),
        map(tag("end"), |_| JustifyItems::End),
        map(tag("baseline"), |_| JustifyItems::Baseline),
    ))(input)
}

fn parse_justify_content<'a, E>(input: &'a [u8]) -> IResult<&[u8], JustifyContent, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

fn parse_justify_self<'a, E>(input: &'a [u8]) -> IResult<&[u8], JustifySelf, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("auto"), |_| JustifySelf::Auto),
        map(tag("center"), |_| JustifySelf::Center),
        map(tag("start"), |_| JustifySelf::Start),
        map(tag("stretch"), |_| JustifySelf::Stretch),
        map(tag("end"), |_| JustifySelf::End),
        map(tag("baseline"), |_| JustifySelf::Baseline),
    ))(input)
}

fn parse_flex_direction<'a, E>(input: &'a [u8]) -> IResult<&[u8], FlexDirection, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("row"), |_| FlexDirection::Row),
        map(tag("column"), |_| FlexDirection::Column),
        map(tag("column_reverse"), |_| FlexDirection::ColumnReverse),
        map(tag("row_reverse"), |_| FlexDirection::RowReverse),
        map(tag("default"), |_| FlexDirection::DEFAULT),
    ))(input)
}

fn parse_flex_wrap<'a, E>(input: &'a [u8]) -> IResult<&[u8], FlexWrap, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((
        map(tag("wrap"), |_| FlexWrap::Wrap),
        map(tag("no_wrap"), |_| FlexWrap::NoWrap),
        map(tag("wrap_reverse"), |_| FlexWrap::WrapReverse),
    ))(input)
}

fn as_string<'a, E>(input: &'a [u8]) -> IResult<&[u8], String, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    map(rest, |v| String::from_utf8_lossy(v).to_string())(input)
}

fn as_string_list<'a, E>(input: &'a [u8]) -> IResult<&[u8], Vec<String>, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

// parse xml prefix
fn parse_prefix0<'a, E>(input: &'a [u8]) -> IResult<&[u8], Option<&[u8]>, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    match terminated(take_snake::<E>, tag(":"))(input) {
        Ok((input, prefix)) => Ok((input, Some(prefix))),
        Err(_) => Ok((input, None)),
    }
}

// parses snake case identifier
fn take_snake<'a, E>(input: &'a [u8]) -> IResult<&[u8], &[u8], E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    take_while(|b: u8| b.is_ascii_alphabetic() || b == b'_')(input)
}

fn parse_image_scale_mode<'a, E>(input: &'a [u8]) -> IResult<&[u8], ImageScaleMode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((parse_image_tile, parse_image_slice))(input)
}

// 10px tiled tiled 1
fn parse_image_slice<'a, E>(input: &'a [u8]) -> IResult<&[u8], ImageScaleMode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, (val, x, y, s)) = tuple((
        preceded(multispace0, parse_px),
        preceded(multispace0, parse_slice_scale),
        preceded(multispace0, parse_slice_scale),
        preceded(multispace0, parse_float),
    ))(input)?;

    Ok((
        input,
        ImageScaleMode::Sliced(TextureSlicer {
            border: BorderRect::square(val),
            center_scale_mode: x,
            sides_scale_mode: y,
            max_corner_scale: s,
        }),
    ))
}

fn parse_image_tile<'a, E>(input: &'a [u8]) -> IResult<&[u8], ImageScaleMode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, (x, y, s)) = tuple((
        preceded(multispace0, parse_bool),
        preceded(multispace0, parse_bool),
        preceded(multispace0, parse_float),
    ))(input)?;

    Ok((
        input,
        ImageScaleMode::Tiled {
            tile_x: x,
            tile_y: y,
            stretch_value: s,
        },
    ))
}

fn parse_bool<'a, E>(input: &'a [u8]) -> IResult<&[u8], bool, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

// stretch
// tile(1)
fn parse_slice_scale<'a, E>(input: &'a [u8]) -> IResult<&[u8], SliceScaleMode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    alt((parse_stretch, parse_tile))(input)
}

fn parse_stretch<'a, E>(input: &'a [u8]) -> IResult<&[u8], SliceScaleMode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    map(tag("stretch"), |_| SliceScaleMode::Stretch)(input)
}

fn parse_tile<'a, E>(input: &'a [u8]) -> IResult<&[u8], SliceScaleMode, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    map(
        tuple((tag("tile"), delimited(tag("("), parse_float, tag(")")))),
        |(_, v)| SliceScaleMode::Tile { stretch_value: v },
    )(input)
}

/// convert string values to uirect
/// 20px/% single
/// 10px/% 10px axis
/// 10px 10px 10px 10px rect
fn parse_ui_rect<'a, E>(input: &'a [u8]) -> IResult<&[u8], UiRect, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "is not a valid `UiRect`",
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
        )),
    )(input)
}

// grid_template_rows="auto 10% 10%"
fn parse_grid_track<'a, E>(input: &'a [u8]) -> IResult<&[u8], GridTrack, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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
fn parse_grid_track_repeated<'a,E>(input: &'a [u8]) -> IResult<&[u8], RepeatedGridTrack,E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{

    let (input, (_,repeat,_, value ,_)) = tuple((
        preceded(multispace0, tag("(")),
        preceded(multispace0, map(parse_number,|n| u16::try_from(n).unwrap_or_default())),
        preceded(multispace0, tag(",")),
        preceded(multispace0, take_until(")")),
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

fn parse_auto_flow<'a, E>(input: &'a [u8]) -> IResult<&[u8], GridAutoFlow, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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

fn parse_str<'a, E>(input: &'a [u8]) -> IResult<&[u8], &str, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    match std::str::from_utf8(input) {
        Ok(str) => Ok(("".as_bytes(), str)),
        Err(_) => Err(nom::Err::Error(E::from_error_kind(
            input,
            nom::error::ErrorKind::MapRes,
        ))),
    }
}

fn parse_number<'a, E>(input: &'a [u8]) -> IResult<&[u8], i64, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, num_bytes) =
        take_while(|u: u8| u.is_ascii_alphanumeric() || u == b'-' || u == b'+')(input)?;

    let (_, str) = parse_str(num_bytes)?;

    match str.parse::<i64>() {
        Ok(num) => Ok((input, num)),
        Err(_) => Err(nom::Err::Error(E::from_error_kind(
            input,
            nom::error::ErrorKind::MapRes,
        ))),
    }
}

// auto
// start_span(5,5)
// end_span(5,5)
fn parse_grid_placement<'a, E>(input: &'a [u8]) -> IResult<&[u8], GridPlacement, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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
                    map(parse_number, |i| u16::try_from(i).unwrap_or_default()),
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
                    map(parse_number, |i| i16::try_from(i).unwrap_or_default()),
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
                    map(parse_number, |i| i16::try_from(i).unwrap_or_default()),
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
                        map(parse_number, |i| i16::try_from(i).unwrap_or_default()),
                        multispace0,
                    ),
                    tag(","),
                    delimited(
                        multispace0,
                        map(parse_number, |i| u16::try_from(i).unwrap_or_default()),
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
                        map(parse_number, |i| i16::try_from(i).unwrap_or_default()),
                        multispace0,
                    ),
                    tag(","),
                    delimited(
                        multispace0,
                        map(parse_number, |i| u16::try_from(i).unwrap_or_default()),
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
fn parse_val<'a, E>(input: &'a [u8]) -> IResult<&[u8], Val, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "cannot be parsed as `Val`, expected number + `px`/`%`/`vw`/`vh`/`vmin`/`vmax`",
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
        ),
    )(input)
}

fn parse_px<'a, E>(input: &'a [u8]) -> IResult<&[u8], f32, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    terminated(parse_float, tag("px"))(input)
}

// rgb(1,1,1)
// rgba(1,1,1,1)
// #000000
// #FFF
#[rustfmt::skip]
fn parse_color<'a,E>(input: &'a [u8]) -> IResult<&[u8], Color,E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("is not a valid color",
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
    ))(input)
}

// rgba(1,1,1,1)
fn parse_rgba_color<'a, E>(input: &'a [u8]) -> IResult<&[u8], Color, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, _) = tag("rgba")(input)?;

    let (input, (r, _, g, _, b, _, a)) = delimited(
        tag("("),
        tuple((float, tag(","), float, tag(","), float, tag(","), float)),
        tag(")"),
    )(input)?;

    Ok((input, Color::linear_rgba(r, g, b, a)))
}

// rgb(1,1,1)
fn parse_rgb_color<'a, E>(input: &'a [u8]) -> IResult<&[u8], Color, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, _) = tag("rgb")(input)?;

    let (input, (r, _, g, _, b)) = delimited(
        tag("("),
        tuple((float, tag(","), float, tag(","), float)),
        tag(")"),
    )(input)?;

    Ok((input, Color::linear_rgb(r, g, b)))
}

// #FFFFFFFF (with alpha)
fn color_hex8_parser<'a, E>(input: &'a [u8]) -> IResult<&[u8], Color, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, _) = tag("#")(input)?;

    if input.len() != 8 {
        return Err(nom::Err::Error(E::from_error_kind(
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
fn color_hex6_parser<'a, E>(input: &'a [u8]) -> IResult<&[u8], Color, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, _) = tag("#")(input)?;

    if input.len() != 6 {
        return Err(nom::Err::Error(E::from_error_kind(
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
fn color_hex4_parser<'a, E>(input: &'a [u8]) -> IResult<&[u8], Color, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, _) = tag("#")(input)?;

    if input.len() != 4 {
        return Err(nom::Err::Error(E::from_error_kind(
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
fn color_hex3_parser<'a, E>(input: &'a [u8]) -> IResult<&[u8], Color, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
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
fn hex_byte<'a, E>(input: &'a [u8]) -> IResult<&[u8], u8, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    map_parser(take_while_m_n(2, 2, is_hex_digit), from_hex_byte)(input)
}

/// F
fn hex_nib<'a, E>(input: &'a [u8]) -> IResult<&[u8], u8, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    map_parser(take_while_m_n(1, 1, is_hex_digit), from_hex_nib)(input)
}

fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_hexdigit()
}

/// FF -> u8
fn from_hex_byte<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], u8, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (_, str) = parse_str(input)?;
    match u8::from_str_radix(format!("{}", str).as_str(), 16) {
        Ok(byte) => Ok(("".as_bytes(), byte)),
        Err(_) => Err(nom::Err::Error(E::from_error_kind(
            input,
            nom::error::ErrorKind::MapRes,
        ))),
    }
}

/// F -> u8
fn from_hex_nib<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], u8, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let str = std::str::from_utf8(input).expect("fix later");
    match u8::from_str_radix(format!("{}{}", str, str).as_str(), 16) {
        Ok(byte) => Ok(("".as_bytes(), byte)),
        Err(_) => Err(nom::Err::Error(E::from_error_kind(
            input,
            nom::error::ErrorKind::MapRes,
        ))),
    }
}

pub fn convert_verbose_error(
    input: &[u8],
    err: nom::Err<nom::error::VerboseError<&[u8]>>,
) -> String {
    use nom::error::VerboseErrorKind;
    use std::fmt::Write;

    let mut result = String::new();

    match err {
        nom::Err::Failure(e) => {
            for (i, (_sub, kind)) in e.errors.iter().enumerate() {
                match kind {
                    VerboseErrorKind::Context(ctx) => {
                        write!(&mut result, "[FAILURE]:{}: {}\n", i, ctx).unwrap();
                    }
                    VerboseErrorKind::Char(c) => {
                        write!(
                            &mut result,
                            "[FAILURE]:{}: expected `{}` got empty input \n",
                            i, c
                        )
                        .unwrap();
                    }
                    VerboseErrorKind::Nom(error_kind) => {
                        write!(
                            &mut result,
                            "{}: in `{:?}` got empty input \n",
                            i, error_kind
                        )
                        .unwrap();
                    }
                }

                let input_str = std::str::from_utf8(input).unwrap_or("[!not a string!]");
                write!(&mut result, "in code: {}", input_str).unwrap();
            }
        }
        nom::Err::Incomplete(needed) => {
            write!(&mut result, "incomplete, not enough data {:?}\n", needed).unwrap();
        }
        nom::Err::Error(e) => {
            for (i, (_sub, kind)) in e.errors.iter().enumerate() {
                match kind {
                    VerboseErrorKind::Context(ctx) => {
                        write!(&mut result, "{}: {}\n", i, ctx).unwrap();
                    }
                    VerboseErrorKind::Char(c) => {
                        write!(&mut result, "{}: expected `{}` got empty input \n", i, c).unwrap();
                    }
                    VerboseErrorKind::Nom(error_kind) => {
                        write!(
                            &mut result,
                            "{}: in `{:?}` got empty input \n",
                            i, error_kind
                        )
                        .unwrap();
                    }
                }

                let input_str = std::str::from_utf8(input).unwrap_or("[!not a string!]");
                write!(&mut result, "in code: `{}`", input_str).unwrap();
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::VerboseError;
    use test_case::test_case;

    #[test_case("#FFFFFFFF", Color::WHITE)]
    #[test_case("#FFFFFF", Color::WHITE)]
    #[test_case("#FFFF", Color::WHITE)]
    #[test_case("#FFF", Color::WHITE)]
    #[test_case("rgb(1,1,1)", Color::WHITE)]
    #[test_case("rgba(1,1,1,1)", Color::WHITE)]
    fn test_color(input: &str, expected: Color) {
        let result = parse_color::<nom::error::Error<_>>(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("0", Val::Px(0.))]
    #[test_case("20vw", Val::Vw(20.))]
    #[test_case("20%", Val::Percent(20.))]
    #[test_case("20vh", Val::Vh(20.))]
    #[test_case("20px", Val::Px(20.))]
    #[test_case("20vmin", Val::VMin(20.))]
    #[test_case("20vmax", Val::VMax(20.))]
    fn test_value(input: &str, expected: Val) {
        let result = parse_val::<nom::error::Error<_>>(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("auto", GridPlacement::auto())]
    #[test_case("end_span(5,50)", GridPlacement::end_span(5, 50))]
    #[test_case("start_span(-5, 5)", GridPlacement::start_span(-5,5))]
    #[test_case("span( 55  )", GridPlacement::span(55))]
    #[test_case("span(5)", GridPlacement::span(5))]
    fn test_grid_placement(input: &str, expected: GridPlacement) {
        match parse_grid_placement::<VerboseError<_>>(&input.as_bytes()) {
            Ok((rem, grid)) => {
                assert_eq!(expected, grid);
                assert_eq!(rem.len(), 0);
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        };
    }

    #[test_case("min max auto", vec![GridTrack::min_content(), GridTrack::max_content(), GridTrack::auto()])]
    #[test_case("50% auto   8fr   ", vec![GridTrack::percent(50.), GridTrack::auto(), GridTrack::fr(8.)])]
    #[test_case("50px       ", vec![GridTrack::px(50.)])]
    fn test_tracks(input: &str, expected: Vec<GridTrack>) {
        let result = many0(parse_grid_track::<nom::error::Error<_>>)(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case("(4, 8flex)(1, 50px)", vec![RepeatedGridTrack::flex(4, 8.), RepeatedGridTrack::px(1,50.)])]
    #[test_case("(1, auto)(5, 50fr)", vec![RepeatedGridTrack::auto(1), RepeatedGridTrack::fr(5,50.)])]
    #[test_case("(1, auto)", vec![RepeatedGridTrack::auto(1)])]
    fn test_repeat_tracks(input: &str, expected: Vec<RepeatedGridTrack>) {
        match many0(parse_grid_track_repeated::<nom::error::VerboseError<_>>)(input.as_bytes()) {
            Ok((rem, grid)) => {
                assert_eq!(expected, grid);
                assert_eq!(rem.len(), 0);
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        }
    }

    #[test_case("20px", UiRect::all(Val::Px(20.)))]
    #[test_case("20px 10px", UiRect::axes(Val::Px(20.), Val::Px(10.)))]
    #[test_case(
        "5px 10px 5% 6px",
        UiRect{ top:Val::Px(5.), right: Val::Px(10.), bottom: Val::Percent(5.), left: Val::Px(6.)}
    )]
    fn test_rect(input: &str, expected: UiRect) {
        let result = parse_ui_rect::<nom::error::Error<_>>(input.as_bytes());
        assert_eq!(Ok(("".as_bytes(), expected)), result);
    }

    #[test_case(
        "   \n<!-- hello world <button> test thah </button> fdsfsd-->\nok",
        "\nok"
    )]
    #[test_case(r#"  <!-- hello <tag/> <""/>world -->ok"#, "ok")]
    #[test_case("   <!-- hello world -->ok", "ok")]
    fn test_comments(input: &str, expected: &str) {
        match trim_comments0::<VerboseError<_>>(&input.as_bytes()) {
            Ok((rem, _)) => {
                assert_eq!(expected, std::str::from_utf8(rem).unwrap());
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        };
    }

    #[test_case(r#"    pressed:background="fsdfsf"  pressed:background="fsdfsf"  <!-- test -->    pressed:background="fsdfsf" \n"#)]
    #[test_case(r#"pressed:background="fsdfsf"#)]
    fn test_parse_xml_attr(input: &str) {
        let (_, _attr) = parse_xml_attr::<nom::error::Error<_>>(input.as_bytes())
            .map_err(|err| err.map_input(|i| std::str::from_utf8(i).unwrap()))
            .unwrap();

        match parse_xml_attr::<VerboseError<_>>(&input.as_bytes()) {
            Ok((_rem, attributes)) => {
                dbg!(attributes);
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        };

        // dbg!(&attr);
    }

    #[test_case(r#"<node pressed:background="rgb(1,1,1)" active="hello"><text p:hello="sdf">hello</text></node>"#)]
    #[test_case(r#"<slot/>"#)]
    #[test_case(r#"<node pressed:background="rgba(1,1,1,0)" active="hello" />"#)]
    #[test_case(r#"<property name="press"><property name="press"></property></property>"#)]
    #[test_case(
        r#"
    <node>
        <property this="press">test</property>
        <property this="press">test</property>
        <node></node>
    </node>
    "#
    )]
    fn test_parse_xml_node(input: &str) {
        match parse_xml_node::<VerboseError<_>>(&input.as_bytes()) {
            Ok((_rem, node)) => {
                dbg!(node);
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        };
    }

    #[test_case("../../example/assets/demo/menu.xml")]
    #[test_case("../../example/assets/demo/panel.xml")]
    #[test_case("../../example/assets/demo/button.xml")]
    #[test_case("../../example/assets/demo/card.xml")]
    fn test_parse_template_full(file_path: &str) {
        let input = std::fs::read_to_string(file_path).unwrap();
        match parse_template::<nom::error::VerboseError<_>>(input.as_bytes()) {
            Ok((_, node)) => {
                dbg!(node);
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        };
    }

    #[test_case(r#"hover:background="{color}""#)]
    #[test_case(r#"pressed:width="10%""#)]
    #[test_case(r#"active:height="10vw""#)]
    fn parse_attribute_parts(input: &str) {
        match parse_xml_attr::<nom::error::VerboseError<_>>(input.as_bytes()) {
            Ok((rem, attrs)) => {
                assert_eq!(rem, "".as_bytes());
                dbg!(attrs);
            }
            Err(err) => {
                println!("{}", super::convert_verbose_error(input.as_bytes(), err));
                assert!(false, "");
            }
        }
    }

    #[test_case(r#"10px stretch stretch 1"#)]
    fn test_parse_nine_slice(input: &str) {
        let (_, slice) = parse_image_slice::<nom::error::Error<_>>(input.as_bytes()).unwrap();
        // dbg!(slice);
        // let t = TextureSlicer{
        //     border: BorderRect::(Val::Px(10.)),
        //     center_scale_mode: todo!(),
        //     sides_scale_mode: todo!(),
        //     max_corner_scale: todo!(),
        // };
    }
}

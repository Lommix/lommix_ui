use crate::data::{Attribute, StyleAttr};
use crate::error::ParseError;
use crate::{
    data::{Button, Div, Image, Include, Text},
    prelude::XNode,
};
use bevy::{
    color::Color,
    ui::{UiRect, Val},
};
use nom::bytes::complete::take_while1;
use nom::combinator::{flat_map, map_parser, rest};
use nom::error::context;
use nom::multi::{many0, many1};
use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take_while},
        streaming::take_while_m_n,
    },
    character::complete::multispace0,
    combinator::{complete, map, map_res},
    number::streaming::float,
    sequence::{delimited, preceded, tuple, Tuple},
    IResult, Parser,
};

/// --------------------------------------------------
/// try parsing a ui xml bytes
pub fn parse_bytes(input: &[u8]) -> Result<XNode, ParseError> {
    let (_, node) = parse_element(input)?;
    Ok(node)
}

#[rustfmt::skip]
fn parse_element(input: &[u8]) -> IResult<&[u8], XNode> {
    let (input, _) = multispace0(input)?;
    let (input, (start_tag, attributes, is_empty)) = parse_start_tag(input)?;
    let (input, _) = multispace0(input)?;

    let (input, content, children ) = if !is_empty {

        let (input, children) = many0(parse_element)(input)?;
        let (input, content) = parse_content(input)?;
        let (input, end_tag) = parse_end_tag(input)?;

        if start_tag != end_tag {
            return Err(nom::Err::Failure(nom::error::make_error(end_tag, nom::error::ErrorKind::TagClosure)));
        }

        ( input, content, children )

    } else {( input, "", vec![] )};


    let styles = attributes.iter().flat_map(|attr|
        match  attr { Attribute::Style(style) => Some(style),
            _ => None }).cloned().collect::<Vec<_>>();

    // @todo: move
    let path = attributes.iter().flat_map(|attr|
        match attr {
            Attribute::Path(path) => Some(path.clone()),
            _ => None
        }
    ).next().ok_or(nom::Err::Failure(nom::error::make_error(start_tag, nom::error::ErrorKind::Tag)));

    // @todo: move
    let click = attributes.iter().flat_map(|attr|
        match attr {
            Attribute::Click(path) => Some(path.clone()),
            _ => None
        }
    ).next().ok_or(nom::Err::Failure(nom::error::make_error(start_tag, nom::error::ErrorKind::Tag)));

    match start_tag {
        b"div" => Ok((input, XNode::Div(Div { styles, children }))),
        b"img" => Ok((input, XNode::Image(Image { styles, path: path? }))),
        b"include" => Ok((input, XNode::Include(Include { styles, children, path: path? }))),
        b"button" => Ok((input, XNode::Button(Button { styles, children, action: click? }))),
        b"text" => Ok((input, XNode::Text(Text { styles, content: content.to_string() }))),
        unkown => Err(nom::Err::Failure(nom::error::make_error(unkown, nom::error::ErrorKind::Tag)))
    }
}

fn parse_start_tag(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<Attribute>, bool)> {
    let (input, (_, element_tag, attributes, _, is_empty)) = tuple((
        tag("<"),
        take_while1(|c: u8| c.is_ascii_alphabetic()),
        many0(parse_attribute),
        multispace0,
        alt((map(tag("/>"), |_| true), map(tag(">"), |_| false))),
    ))(input)?;

    Ok((input, (element_tag, attributes, is_empty)))
}

fn parse_end_tag(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, (_, end_tag, _)) = tuple((
        tag("</"),
        take_while1(|c: u8| c.is_ascii_alphabetic()),
        tag(">"),
    ))(input)?;

    Ok((input, end_tag))
}

fn parse_content(input: &[u8]) -> IResult<&[u8], &str> {
    let (input, content) = map_res(take_while(|c: u8| c != b'>' && c != b'<'), |c| {
        std::str::from_utf8(c)
    })(input)?;
    Ok((input, content.trim().trim_end()))
}

fn parse_attribute(input: &[u8]) -> IResult<&[u8], Attribute> {
    let (input, (_, prefix, ident, _, value)) = tuple((
        multispace0,
        parse_prefix0,
        take_while_m_n(1, 32, |c: u8| c != b'='),
        tag("="),
        delimited(tag("\""), take_while(|b: u8| b != b'"'), tag("\"")),
    ))(input)?;

    let attribute = match ident {
        b"comp" => Attribute::Compontent(as_string(value).map(|(_, string)| string)?),
        b"path" => Attribute::Path(as_string(value).map(|(_, string)| string)?),
        b"click" => Attribute::Click(as_string(value).map(|(_, string)| string)?),
        ident => {
            let (_, style) = parse_style(prefix, ident, value)?;
            Attribute::Style(style)
        }
    };

    Ok((input, attribute))
}

fn parse_style<'a>(
    prefix: Option<&'a [u8]>,
    ident: &'a [u8],
    value: &'a [u8],
) -> IResult<&'a [u8], StyleAttr> {
    let (input, style) = match ident {
        // b"duration" => map(nom::number::complete::f32, |val| StyleAttr::Duration(val))(value)?,
        b"column_gap" => map(parse_val, |val| StyleAttr::ColumnGap(val))(value)?,
        b"row_gap" => map(parse_val, |val| StyleAttr::RowGap(val))(value)?,
        b"max_height" => map(parse_val, |val| StyleAttr::MaxHeight(val))(value)?,
        b"max_width" => map(parse_val, |val| StyleAttr::MaxWidth(val))(value)?,
        b"min_height" => map(parse_val, |val| StyleAttr::MinHeight(val))(value)?,
        b"min_width" => map(parse_val, |val| StyleAttr::MinWidth(val))(value)?,
        b"bottom" => map(parse_val, |val| StyleAttr::Bottom(val))(value)?,
        b"top" => map(parse_val, |val| StyleAttr::Top(val))(value)?,
        b"right" => map(parse_val, |val| StyleAttr::Right(val))(value)?,
        b"left" => map(parse_val, |val| StyleAttr::Left(val))(value)?,
        b"height" => map(parse_val, |val| StyleAttr::Height(val))(value)?,
        b"width" => map(parse_val, |val| StyleAttr::Width(val))(value)?,
        b"padding" => map(parse_ui_rect, |val| StyleAttr::Padding(val))(value)?,
        b"margin" => map(parse_ui_rect, |val| StyleAttr::Margin(val))(value)?,
        b"border" => map(parse_ui_rect, |val| StyleAttr::Border(val))(value)?,
        b"border_radius" => map(parse_ui_rect, |val| StyleAttr::BorderRadius(val))(value)?,
        b"background" => map(parse_color, |val| StyleAttr::Background(val))(value)?,
        b"border_color" => map(parse_color, |val| StyleAttr::BorderColor(val))(value)?,
        _ => {
            return Err(nom::Err::Error(nom::error::make_error(
                ident,
                nom::error::ErrorKind::NoneOf,
            )))
        }
    };

    match prefix {
        Some(b"active") => Ok((input, StyleAttr::Hover(Box::new(style)))),
        Some(b"hover") => Ok((input, StyleAttr::Hover(Box::new(style)))),
        _ => Ok((input, style)),
    }
}

fn as_string(input: &[u8]) -> IResult<&[u8], String> {
    map(rest, |v| String::from_utf8_lossy(v).to_string())(input)
}

#[rustfmt::skip]
fn parse_prefix0(input: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    let res : IResult<&[u8], (&[u8], &[u8])>= tuple((
        take_while1(|b: u8| b.is_ascii_alphabetic()),
        tag(":"),
    ))(input);

    match res {
        Ok((input, (prefix,_))) => Ok((input, Some(prefix))),
        Err(_) => Ok((input, None)),
    }
}

/// convert string values to uirect
/// 20px/% single
/// 10px/% 10px axis
/// 10px 10px 10px 10px rect
pub(crate) fn parse_ui_rect(input: &[u8]) -> IResult<&[u8], UiRect> {
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

/// 10px 10%
fn parse_val(input: &[u8]) -> IResult<&[u8], Val> {
    delimited(
        multispace0,
        alt((
            map(tag("auto"), |_| Val::Auto),
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
    let str = std::str::from_utf8(input).expect("fuck");
    u8::from_str_radix(format!("{}", str).as_str(), 16)
}

/// F -> u8
fn from_hex_nib(input: &[u8]) -> Result<u8, std::num::ParseIntError> {
    let str = std::str::from_utf8(input).expect("fuck");
    u8::from_str_radix(format!("{}{}", str, str).as_str(), 16)
}

#[cfg(test)]
mod tests {
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
        // assert_eq!(color.to_linear(), Color::WHITE.to_linear());
    }

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

    #[test]
    fn test_parse_element() {
        let input = std::fs::read_to_string("test.xml").unwrap();
        let result = parse_element(input.as_bytes());

        match result {
            Ok((_, _)) => (),
            Err(err) => {
                let err = err.map_input(|i| std::str::from_utf8(i));
                dbg!(err);
            }
        };
    }
}

use crate::error::{AttributeError, ParseError};
use crate::style::StyleAttr;
use crate::{
    node::{Button, Div, Image, Include, Text},
    prelude::NNode,
};
use bevy::{
    color::Color,
    ui::{UiRect, Val},
};
use nom::bytes::complete::take_while1;
use nom::combinator::{flat_map, map_parser};
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

pub fn parse_xml_bytes(input: &[u8]) -> Result<(NNode, Vec<AttributeError>), ParseError> {
    let mut reader = quick_xml::reader::Reader::from_reader(input);
    reader.config_mut().trim_text(true);
    reader.config_mut().check_end_names = true;

    parse_next_node(None, &mut reader)
}

fn parse_next_node(
    current: Option<NNode>,
    reader: &mut quick_xml::reader::Reader<&[u8]>,
) -> Result<(NNode, Vec<AttributeError>), ParseError> {
    loop {
        let next_event = match reader.read_event() {
            Ok(ev) => ev,
            //@todo: handle, clean this
            Err(err) => {
                return Err(ParseError::Failed(format!(
                    "error when?: {}",
                    err.to_string()
                )))
            }
        };

        match next_event {
            quick_xml::events::Event::Start(start) => {
                let (_, mut next_node) = parse_enum_node_type(start.name().as_ref())?;

                let mut styles = Vec::new();
                for attr in start.attributes().flatten() {
                    let style = match StyleAttr::try_from(&attr) {
                        Ok(attr) => attr,
                        Err(err) => {
                            dbg!(err);
                            continue;
                        }
                    };
                    styles.push(style);
                }

                let (styles, errors) = start
                    .attributes()
                    .map(|res| res.map_err(|err| AttributeError::FailedToParse(err.to_string())))
                    .map(|res| res.map(|attr| StyleAttr::try_from(&attr)))
                    .flatten()
                    .fold((Vec::new(), Vec::new()), |(mut attrs, mut errs), res| {
                        match res {
                            Ok(attr) => attrs.push(attr),
                            Err(err) => errs.push(err),
                        };
                        (attrs, errs)
                    });

                next_node.add_styles(styles);

                match current {
                    Some(mut node) => {
                        let (child, attr_errors) = parse_next_node(Some(next_node), reader)?;
                        node.add_child(child);
                        return parse_next_node(Some(node), reader).map(|(n, mut e)| {
                            e.extend(attr_errors);
                            (n, e)
                        });
                    }
                    None => {
                        return parse_next_node(Some(next_node), reader);
                    }
                }
            }
            quick_xml::events::Event::Empty(start) => {
                let (_, mut new_node) = parse_enum_node_type(start.name().as_ref())?;

                let mut styles = Vec::new();
                for attr in start.attributes().flatten() {
                    let style = match StyleAttr::try_from(&attr) {
                        Ok(attr) => attr,
                        Err(err) => {
                            dbg!(err);
                            continue;
                        }
                    };
                    styles.push(style);
                }

                new_node.add_styles(styles);

                match current {
                    Some(mut node) => {
                        node.add_child(new_node);
                        return parse_next_node(Some(node), reader);
                    }
                    None => {
                        todo!()
                        // return Ok(new_node),
                    }
                }
            }
            quick_xml::events::Event::End(end) => {
                //@todo: unkown node
                let (_, node) = parse_enum_node_type(end.name().as_ref())?;

                if let Some(n) = current.filter(|n| *n == node).take() {
                    // return Ok(n);
                    todo!()
                }
                return Err(ParseError::Unclosed("".into()));
            }
            quick_xml::events::Event::Eof => {
                panic!("end of file?");
            }
            quick_xml::events::Event::Text(text) => {}
            _ => (),
            // quick_xml::events::Event::CData(_) => todo!(),
            // quick_xml::events::Event::Comment(_) => todo!(),
            // quick_xml::events::Event::Decl(_) => todo!(),
            // quick_xml::events::Event::PI(_) => todo!(),
            // quick_xml::events::Event::DocType(_) => todo!(),
        }
    }
}

#[rustfmt::skip]
fn parse_enum_node_type(input: &[u8]) -> IResult<&[u8], NNode> {
    let (remaining, word) = take_while(|c| c != b' ')(input)?;
    match word {
        b"div" => Ok((remaining, NNode::Div(Div::default()))),
        b"img" => Ok((remaining, NNode::Image(Image::default()) )),
        b"include" => Ok((remaining, NNode::Include(Include::default()) )),
        b"button" => Ok((remaining, NNode::Button(Button::default()) )),
        b"text" => Ok((remaining, NNode::Text(Text::default()) )),
        _ => Ok((remaining, NNode::Unkown)),
    }
}

#[rustfmt::skip]
fn parse_element(input: &[u8]) -> IResult<&[u8], NNode> {
    let (input, _) = multispace0(input)?;
    let (input, (start_tag, attributes, is_empty)) = parse_start_tag(input)?;
    let (input, _) = multispace0(input)?;

    // children?
    let (input, content, children ) = if !is_empty {

        let (input, children) = many0(parse_element)(input)?;
        let (input, content) = parse_content(input)?;
        let (input, end_tag) = parse_end_tag(input)?;

        if start_tag != end_tag {
            return Err(nom::Err::Failure(nom::error::make_error(end_tag, nom::error::ErrorKind::TagClosure)));
        }

        ( input, content, children )

    } else {( input, "", vec![] )};

    match start_tag {
        b"div" => Ok((input, NNode::Div(Div { styles: attributes, children }))),
        b"img" => Ok((input, NNode::Image(Image { styles: attributes, children, path: String::new() }))),
        b"include" => Ok((input, NNode::Include(Include { styles: attributes, children, path: String::new(), slot: None }))),
        b"button" => Ok((input, NNode::Button(Button { styles: attributes, children, action: String::new() }))),
        b"text" => Ok((input, NNode::Text(Text { styles: attributes, content: content.to_string() }))),
        unkown => Err(nom::Err::Failure(nom::error::make_error(unkown, nom::error::ErrorKind::Tag)))
    }
}

fn parse_start_tag(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<StyleAttr>, bool)> {
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

pub enum Attribute {
    Style(StyleAttr),
    Path(String),
    Action(String),
}

fn parse_attribute(input: &[u8]) -> IResult<&[u8], StyleAttr> {
    // add an optional prefix like hover:`ident`
    let (input, (_, prefix, ident, _, value)) = tuple((
        multispace0,
        parse_prefix0,
        take_while(|c: u8| c.is_ascii_alphabetic()),
        tag("="),
        delimited(tag("\""), take_while(|b: u8| b != b'"'), tag("\"")),
    ))(input)?;

    let attribute = match ident {
        b"height" => {
            let (_, val) = parse_val(value)?;
            Ok((input, StyleAttr::Height(val)))
        }
        b"width" => {
            let (_, val) = parse_val(value)?;
            Ok((input, StyleAttr::Width(val)))
        }
        b"padding" => {
            let (_, val) = parse_ui_rect(value)?;
            Ok((input, StyleAttr::Padding(val)))
        }
        b"margin" => {
            let (_, val) = parse_ui_rect(value)?;
            Ok((input, StyleAttr::Margin(val)))
        }
        _ => Err(nom::Err::Error(nom::error::make_error(
            ident,
            nom::error::ErrorKind::Tag,
        ))),
    };

    match prefix {
        Some(prefix) => match prefix {
            b"hover" => attribute.map(|(input, attr)| (input, StyleAttr::Hover(Box::new(attr)))),
            b"active" => attribute.map(|(input, attr)| (input, StyleAttr::Active(Box::new(attr)))),
            _ => attribute,
        },
        None => attribute,
    }
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

#[test]
fn test_parse_element() {
    let input = std::fs::read_to_string("test.xml").unwrap();
    match parse_element(input.as_bytes()) {
        Ok((input, node)) => {
            dbg!(node);
        }
        Err(err) => {
            let err = err.map_input(|i| std::str::from_utf8(i));
            dbg!(err);
        }
    };
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
            |(top, left, right, bottom)| UiRect {
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
pub(crate) fn parse_val(input: &[u8]) -> IResult<&[u8], Val> {
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
pub(crate) fn parse_color(input: &[u8]) -> IResult<&[u8], Color> {
    delimited(
        multispace0,
        alt((color_hex6_parser, color_hex3_parser)),
        multispace0,
    )(input)
}

// rgb(1,1,1)
// rgba(1,1,1,1)
fn parse_rgb_color(input: &[u8]) -> IResult<&[u8], Color> {
    todo!()
}

// #FFF
fn color_hex6_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (r, g, b)) = (hex_primary, hex_primary, hex_primary).parse(input)?;
    Ok((input, Color::srgb_u8(r, g, b)))
}

// #FFF
fn color_hex3_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (r, g, b)) = (hex_half, hex_half, hex_half).parse(input)?;
    Ok((input, Color::srgb_u8(r, g, b)))
}

/// Parses a byte hex string, like "FF"
fn from_hex(input: &[u8]) -> Result<u8, std::num::ParseIntError> {
    let str = std::str::from_utf8(input).expect("fuck");
    u8::from_str_radix(format!("{}", str).as_str(), 16)
}

/// Parses a hex string character interpreted as a byte, like "F" -> "FF" -> 255
fn from_half_hex(input: &[u8]) -> Result<u8, std::num::ParseIntError> {
    let str = std::str::from_utf8(input).expect("fuck");
    u8::from_str_radix(format!("{}{}", str, str).as_str(), 16)
}

/// Returns true if the character is a valid hexadecimal character
fn is_hex_digit(c: u8) -> bool {
    let c = char::from(c);
    c.is_ascii_hexdigit()
}

/// Takes a two letter hexadecimal from the input and return it as a byte
fn hex_primary(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex).parse(input)
}

/// Takes a single letter hexadecimal from the input and return it as a byte
fn hex_half(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(take_while_m_n(1, 1, is_hex_digit), from_half_hex).parse(input)
}

#[test]
fn test_color() {
    let str = "#FFFFFF";
    let (_, color) = parse_color(str.as_bytes()).unwrap();
    assert_eq!(color.to_linear(), Color::WHITE.to_linear());
}

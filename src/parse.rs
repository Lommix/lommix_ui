use bevy::{
    color::Color,
    ui::{UiRect, Val},
};
use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take_while},
        streaming::{take, take_while_m_n},
    },
    character::complete::multispace0,
    combinator::{complete, map, map_res},
    number::streaming::float,
    sequence::{delimited, preceded, tuple, Tuple},
    IResult, Parser,
};
use quick_xml::events::attributes::Attribute;

use crate::style::StyleAttr;

#[derive(Debug)]
pub struct XNode {
    pub ty: NodeToken,
    pub attributes: Vec<XAttribute>,
    pub styles: Vec<StyleAttr>,
    pub children: Vec<XNode>,
}

#[derive(Debug)]
pub enum NNode {
    Div {
        styles: Vec<StyleAttr>,
        children: Vec<NNode>,
    },
    Image {
        path: String,
        styles: Vec<StyleAttr>,
        children: Vec<NNode>,
    },
    Text {
        styles: Vec<StyleAttr>,
        content: String,
    },
    Button {
        styles: Vec<StyleAttr>,
        children: Vec<NNode>,
        click: String,
    },
    Include {
        styles: Vec<StyleAttr>,
        path: String,
    },
    Unkown,
}

impl PartialEq for NNode {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl NNode {
    #[rustfmt::skip]
    #[allow(unused)]
    fn add_child(&mut self, child: NNode) -> Result<(), ()> {
        match self {
            NNode::Div { styles, children } => children.push(child),
            NNode::Image { styles, path, children } => children.push(child),
            NNode::Button { styles, children, click } => children.push(child),
            _=> return Err(()),
        }

        Ok(())
    }

    #[allow(unused)]
    fn add_text(&mut self, text: &str) -> Result<(), ()> {
        match self {
            NNode::Text { styles, content } => {
                *content = text.to_string();
            }
            _ => return Err(()),
        }
        Ok(())
    }

    #[rustfmt::skip]
    #[allow(unused)]
    fn add_styles(&mut self, style: Vec<StyleAttr>) -> Result<(), ()> {
        match self {
            NNode::Div { styles, children } => styles.extend(style),
            NNode::Image { styles, path, children } => styles.extend(style),
            NNode::Text { styles, content } => styles.extend(style),
            NNode::Button { styles, children, click } => styles.extend(style),
            NNode::Include { styles, path } => styles.extend(style),
            _ => return Err(()),
        }
        Ok(())
    }

    #[rustfmt::skip]
    #[allow(unused)]
    fn set_path(&mut self, path: &str) -> Result<(), ()> {
        match self {
            NNode::Include { styles, path } => *path = path.to_string(),
            NNode::Image { styles, path, children } => *path = path.to_string(),
            _ => return Err(()),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum XAttribute {
    Path(String),
    Click(String),
}

#[derive(Debug, PartialEq)]
pub enum NodeToken {
    Div,
    Image,
    Button,
    Include,
    Input,
    Text,
    Unkown,
}

pub(crate) fn parse_xml_new(input: &[u8]) -> Result<NNode, ()> {
    let mut reader = quick_xml::reader::Reader::from_reader(input);
    reader.config_mut().trim_text(true);
    reader.config_mut().check_end_names = true;
    parse_next_enum_node(None, &mut reader)
}

pub(crate) fn parse_xml(input: &[u8]) -> Result<XNode, ()> {
    let mut reader = quick_xml::reader::Reader::from_reader(input);
    reader.config_mut().trim_text(true);
    reader.config_mut().check_end_names = true;

    parse_next_node(None, &mut reader)
}

fn parse_next_node(
    current: Option<XNode>,
    reader: &mut quick_xml::reader::Reader<&[u8]>,
) -> Result<XNode, ()> {
    loop {
        let next_event = match reader.read_event() {
            Ok(ev) => ev,
            //@todo: handle
            Err(err) => panic!("{err}"),
        };
        match next_event {
            quick_xml::events::Event::Start(start) => {
                let (_, ty) = parse_node_type(start.name().as_ref()).map_err(|err| ())?;

                let mut styles = Vec::new();
                for attr in start.attributes().flatten() {
                    let Ok(style) = StyleAttr::try_from(&attr) else {
                        // handle error
                        continue;
                    };
                    styles.push(style);
                }

                match current {
                    Some(mut node) => {
                        let child = parse_next_node(
                            Some(XNode {
                                ty,
                                attributes: vec![],
                                styles,
                                children: vec![],
                            }),
                            reader,
                        )?;

                        node.children.push(child);
                        return parse_next_node(Some(node), reader);
                    }
                    None => {
                        return parse_next_node(
                            Some(XNode {
                                ty,
                                attributes: vec![],
                                styles,
                                children: vec![],
                            }),
                            reader,
                        );
                    }
                }
            }
            quick_xml::events::Event::Empty(start) => {
                let (_, ty) = parse_node_type(start.name().as_ref()).map_err(|err| ())?;

                let mut styles = Vec::new();
                for attr in start.attributes().flatten() {
                    let Ok(style) = StyleAttr::try_from(&attr) else {
                        // handle error
                        continue;
                    };

                    if let Ok(style) = StyleAttr::try_from(&attr) {
                        styles.push(style);
                    }
                }

                match current {
                    Some(mut node) => {
                        node.children.push(XNode {
                            ty,
                            styles,
                            attributes: vec![],
                            children: vec![],
                        });
                        return parse_next_node(Some(node), reader);
                    }
                    None => {
                        return Ok(XNode {
                            ty,
                            styles,
                            attributes: vec![],
                            children: vec![],
                        });
                    }
                }
            }
            quick_xml::events::Event::End(end) => {
                //@todo: unkown node

                let (_, ty) = parse_node_type(end.name().as_ref()).map_err(|err| ())?;

                if let Some(n) = current.filter(|n| n.ty == ty).take() {
                    return Ok(n);
                }

                // @todo: mismatching tags
                return Err(());
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
    Err(())
}

fn parse_next_enum_node(
    current: Option<NNode>,
    reader: &mut quick_xml::reader::Reader<&[u8]>,
) -> Result<NNode, ()> {
    loop {
        let next_event = match reader.read_event() {
            Ok(ev) => ev,
            //@todo: handle
            Err(err) => panic!("{err}"),
        };
        match next_event {
            quick_xml::events::Event::Start(start) => {
                let (_, mut next_node) =
                    parse_enum_node_type(start.name().as_ref()).map_err(|err| ())?;

                let mut styles = Vec::new();
                for attr in start.attributes().flatten() {
                    let Ok(style) = StyleAttr::try_from(&attr) else {
                        // handle error
                        continue;
                    };
                    styles.push(style);
                }

                next_node.add_styles(styles);

                match current {
                    Some(mut node) => {
                        let child = parse_next_enum_node(Some(next_node), reader)?;
                        node.add_child(child);
                        return parse_next_enum_node(Some(node), reader);
                    }
                    None => {
                        return parse_next_enum_node(Some(next_node), reader);
                    }
                }
            }
            quick_xml::events::Event::Empty(start) => {
                let (_, mut new_node) =
                    parse_enum_node_type(start.name().as_ref()).map_err(|err| ())?;

                let mut styles = Vec::new();
                for attr in start.attributes().flatten() {
                    let Ok(style) = StyleAttr::try_from(&attr) else {
                        // handle error
                        continue;
                    };

                    if let Ok(style) = StyleAttr::try_from(&attr) {
                        styles.push(style);
                    }
                }

                new_node.add_styles(styles);

                match current {
                    Some(mut node) => {
                        node.add_child(new_node);
                        return parse_next_enum_node(Some(node), reader);
                    }
                    None => return Ok(new_node),
                }
            }
            quick_xml::events::Event::End(end) => {
                //@todo: unkown node
                let (_, node) = parse_enum_node_type(end.name().as_ref()).map_err(|err| ())?;

                if let Some(n) = current.filter(|n| *n == node).take() {
                    return Ok(n);
                }

                println!("faiiiiilll!");

                // @todo: mismatching tags
                return Err(());
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
    Err(())
}

#[rustfmt::skip]
fn parse_enum_node_type(input: &[u8]) -> IResult<&[u8], NNode> {
    let (remaining, word) = take_while(|c| c != b' ')(input)?;
    match word {
        b"div" => Ok((remaining, NNode::Div{styles: vec![], children: vec![]})),
        b"img" => Ok((remaining, NNode::Image{styles:vec![], children: vec![], path: String::new()})),
        b"include" => Ok((remaining, NNode::Include{styles: vec![], path: String::new()})),
        b"button" => Ok((remaining, NNode::Button{styles:vec![], children: vec![], click:String::new()})),
        b"text" => Ok((remaining, NNode::Text{styles:vec![], content: String::new()})),
        _ => Ok((remaining, NNode::Unkown)),
    }
}

fn parse_node_type(input: &[u8]) -> IResult<&[u8], NodeToken> {
    let (remaining, word) = take_while(|c| c != b' ')(input)?;
    match word {
        b"div" => Ok((remaining, NodeToken::Div)),
        b"img" => Ok((remaining, NodeToken::Image)),
        b"include" => Ok((remaining, NodeToken::Include)),
        b"button" => Ok((remaining, NodeToken::Button)),
        b"input" => Ok((remaining, NodeToken::Input)),
        b"text" => Ok((remaining, NodeToken::Text)),
        _ => Ok((remaining, NodeToken::Unkown)),
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

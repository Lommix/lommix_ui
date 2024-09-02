use crate::prelude::*;
use xml::{reader::XmlEvent, ParserConfig};

#[derive(Debug, PartialEq)]
pub enum XmlNode {
    Img(Image),
    Div(Div),
}

#[derive(Debug, PartialEq)]
pub struct Div {
    pub children: Vec<XmlNode>,
    pub style: Vec<StyleAttr>,
}

#[derive(Debug, PartialEq)]
pub struct Image {
    pub style: Vec<StyleAttr>,
    pub children: Vec<XmlNode>,
    pub path: String,
}

pub fn deserialize(str: &str) -> Result<XmlNode, ParseError> {

    let config = ParserConfig::new()
        .ignore_comments(true)
        .add_entity("hover", "http://www.example.com/hover")
        .ignore_invalid_encoding_declarations(true);

    let mut parser = xml::EventReader::new_with_config(str.as_bytes(), config);

    loop {
        let next = parser
            .next()
            .map_err(|err| ParseError::Failed(err.to_string()))?;

        match next {
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => {
                return next_node(
                    XmlEvent::StartElement {
                        name,
                        attributes,
                        namespace,
                    },
                    &mut parser,
                );
            }
            _ => (),
        }
    }
}

fn next_node(
    start: XmlEvent,
    parser: &mut xml::EventReader<&[u8]>
) -> Result<XmlNode, ParseError> {

    let XmlEvent::StartElement {
        name: start_name,
        attributes: start_attributes,
        namespace: start_namespace,
    } = start
    else {
        return Err(ParseError::Failed("Not a start".into()));
    };

    let mut children = Vec::new();

    loop {
        let next = parser
            .next()
            .map_err(|err| ParseError::Failed(err.to_string()))?;

        match next {
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => {
                let child_node = next_node(
                    XmlEvent::StartElement {
                        name,
                        attributes,
                        namespace,
                    },
                    parser,
                )?;

                children.push(child_node);
            }
            XmlEvent::EndElement { name } => {
                if name.local_name != start_name.local_name {
                    return Err(ParseError::Failed("start and end does not match".into()));
                }

                // gen styles
                let style = start_attributes
                    .iter()
                    .flat_map(|attr| StyleAttr::try_from(attr))
                    .collect();

                match start_name.local_name.to_lowercase().as_str() {
                    "div" => {
                        return Ok(XmlNode::Div(Div { children, style }));
                    }
                    "img" => {
                        let path = start_attributes
                            .iter()
                            .find(|attr| attr.name.local_name == "path")
                            .map(|attr| attr.value.clone())
                            .unwrap_or_default();

                        return Ok(XmlNode::Img(Image {
                            children,
                            path,
                            style,
                        }));
                    }
                    unknown => {
                        return Err(ParseError::UnknownToken(
                            format!("what is this {unknown}").into(),
                        ));
                    }
                };
            }
            _ => (),
            // XmlEvent::StartDocument { version, encoding, standalone } => todo!(),
            // XmlEvent::EndDocument => todo!(),
            // XmlEvent::ProcessingInstruction { name, data } => todo!(),
            // XmlEvent::CData(_) => todo!(),
            // XmlEvent::Comment(_) => todo!(),
            // XmlEvent::Characters(_) => todo!(),
            // XmlEvent::Whitespace(_) => todo!(),
        }
    }
}

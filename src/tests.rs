use crate::prelude::*;
use bevy::ui::Val;
use nom::{bytes::complete::take_while, IResult};
use std::io::Read;

// #[test]
fn test_transform() {
    // let ron_string = std::fs::read_to_string("assets/demo_ui.ron").unwrap();
    // let asset = ron::de::from_str::<UiNode>(&ron_string).unwrap();

    // let asset = XmlNode::Div(Div {
    //     children: vec![XmlNode::Img(Image {
    //         styles: Style {
    //             width: Some(20.),
    //             height: None,
    //         },
    //         path: "test".to_string(),
    //         children: vec![],
    //     })],
    // });
    //
    // let s = quick_xml::se::to_string(&asset).unwrap();
    // std::fs::write("test.xml", s).unwrap();
}

// #[test]
// fn test_read() {
//     let str = std::fs::read_to_string("test.xml").unwrap();
//     let asset: XmlNode = quick_xml::de::from_str(&str).unwrap();
//     dbg!(asset);
// }
//

// #[test]
// fn test_read() {
//     let str = std::fs::read_to_string("test.xml").unwrap();
//     let out = deserialize(&str).unwrap();
//     dbg!(out);
// }

#[test]
fn test_new() {
    let bytes = std::fs::read("assets/demo_ui.xml").unwrap();
    let output = crate::parse::parse_xml_new(&bytes).unwrap();
    dbg!(output);
}

//"div width=\"20px\" height=\"10px\" hover:padding=\"15px 20% 10px 10px\" "
// fn container(input &str) -> IResult<&str, u8>{
// }

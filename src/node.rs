use crate::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Asset, TypePath)]
pub enum NNode {
    Div(Div),
    Image(Image),
    Text(Text),
    Button(Button),
    Include(Include),
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
    pub fn add_child(&mut self, child: NNode) -> Result<(), ()> {
        match self {
            NNode::Div(div) => div.children.push(child),
            NNode::Image(img) => img.children.push(child),
            NNode::Button(btn) => btn.children.push(child),
            NNode::Include(inc) => inc.children.push(child),
            _=> return Err(()),
        }
        Ok(())
    }

    #[allow(unused)]
    pub fn add_text(&mut self, txt: &str) -> Result<(), ()> {
        match self {
            NNode::Text(text) => {
                text.content = txt.to_string();
            }
            _ => return Err(()),
        }
        Ok(())
    }

    #[rustfmt::skip]
    #[allow(unused)]
    pub fn add_styles(&mut self, style: Vec<StyleAttr>) -> Result<(), ()> {
        match self {
            NNode::Div(div) => div.styles.extend(style),
            NNode::Image(img) => img.styles.extend(style),
            NNode::Text(text) => text.styles.extend(style),
            NNode::Button(btn) => btn.styles.extend(style),
            NNode::Include(inc) => inc.styles.extend(style),
            _ => return Err(()),
        }
        Ok(())
    }

    #[rustfmt::skip]
    #[allow(unused)]
    pub fn set_path(&mut self, path: &str) -> Result<(), ()> {
        match self {
            NNode::Include(inc) => inc.path = path.to_string(),
            NNode::Image(img) => img.path = path.to_string(),
            _ => return Err(()),
        }

        Ok(())
    }

    #[rustfmt::skip]
    pub fn spawn(&self, cmd: &mut Commands, server: &AssetServer, assets: &Assets<NNode>){
        todo!()
    }
}

// -------------------------------
#[derive(Debug, Default)]
pub struct Div {
    pub styles: Vec<StyleAttr>,
    pub children: Vec<NNode>,
}

#[derive(Debug, Default)]
pub struct Image {
    pub path: String,
    pub styles: Vec<StyleAttr>,
    pub children: Vec<NNode>,
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
    pub children: Vec<NNode>,
}

#[derive(Debug, Default)]
pub struct Include {
    pub path: String,
    pub styles: Vec<StyleAttr>,
    pub children: Vec<NNode>,
    pub slot: Option<Entity>,
}

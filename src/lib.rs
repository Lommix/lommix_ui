use bevy::prelude::*;

mod build;
mod error;
mod load;
mod node;
mod parse;
mod style;
mod tests;

pub mod prelude {
    pub use crate::build::{RonUiBundle, StyleAttributes};
    pub use crate::error::{ParseError, AttributeError};
    pub use crate::node::{Button, Div, Image, Include, NNode, Text};
    pub use crate::parse::parse_xml_bytes;
    pub use crate::style::StyleAttr;
    pub use crate::LommixUiPlugin;
}

pub struct LommixUiPlugin;
impl Plugin for LommixUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((load::LoaderPlugin, build::BuildPlugin));
    }
}

use bevy::prelude::*;

mod build;
mod error;
mod load;
mod data;
mod parse;

pub mod prelude {
    pub use crate::build::{RonUiBundle, StyleAttributes};
    pub use crate::error::ParseError;
    pub use crate::data::{Button, Div, Image, Include, Text, XNode, Attribute, StyleAttr};
    pub use crate::LommixUiPlugin;
}

pub struct LommixUiPlugin;
impl Plugin for LommixUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((load::LoaderPlugin, build::BuildPlugin));
    }
}

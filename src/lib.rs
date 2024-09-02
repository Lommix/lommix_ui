use bevy::prelude::*;

mod build;
mod error;
mod load;
mod style;
mod tests;
mod parse;
// mod node;

pub mod prelude {
    pub use crate::build::{RonUiBundle, StyleAttributes};
    pub use crate::error::{ UiError, ParseError };
    pub use crate::load::XmlUi;
    pub use crate::style::StyleAttr;
    // pub use crate::node::XmlNode;
    pub use crate::LommixUiPlugin;
}

pub struct LommixUiPlugin;
impl Plugin for LommixUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((load::LoaderPlugin, build::BuildPlugin));
    }
}

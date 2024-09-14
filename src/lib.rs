use bevy::app::{App, Plugin};

mod bindings;
mod build;
mod data;
mod error;
mod load;
mod parse;
mod properties;
// mod lexer;

pub mod prelude {
    pub use crate::bindings::{ComponentBindings, FunctionBindings};
    pub use crate::build::{
        OnEnter, OnExit, OnPress, OnSpawn, StyleAttributes, Tag, Tags, HtmlBundle, UiId, UiTarget,
    };
    pub use crate::data::{Action, Attribute, NodeType, StyleAttr, XNode};
    pub use crate::error::ParseError;
    pub use crate::properties::PropertyDefintions;
    pub use crate::LommixUiPlugin;
}

pub struct LommixUiPlugin;
impl Plugin for LommixUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            load::LoaderPlugin,
            build::BuildPlugin,
            bindings::BindingPlugin,
            properties::PropertyPlugin,
        ));
    }
}

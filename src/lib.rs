use bevy::prelude::*;

mod bindings;
mod build;
mod data;
mod error;
mod load;
mod parse;

pub mod prelude {
    pub use crate::bindings::{FunctionBindings, SpawnBindings};
    pub use crate::build::{StyleAttributes, UiBundle};
    pub use crate::data::{Action, Attribute, NodeType, StyleAttr, XNode};
    pub use crate::error::ParseError;
    pub use crate::LommixUiPlugin;
}

pub struct LommixUiPlugin;
impl Plugin for LommixUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            load::LoaderPlugin,
            build::BuildPlugin,
            bindings::BindingPlugin,
        ));
    }
}

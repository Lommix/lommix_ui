use bevy::app::{App, Plugin};

mod auto;
mod bindings;
mod build;
mod compile;
mod data;
mod error;
mod load;
mod parse;
mod styles;

pub mod prelude {
    pub use crate::auto::{AutoLoadState, HtmlAutoLoadPlugin};
    pub use crate::bindings::{ComponentBindings, FunctionBindings, HtmlComponents, HtmlFunctions};
    pub use crate::build::{
        HtmlBundle, HtmlNode, OnUiEnter, OnUiExit, OnUiPress, OnUiSpawn, ScopeEntity, Tag, Tags,
        TemplateState, UiId, UiTarget, UiWatch, UnbuildTag,
    };
    pub use crate::compile::{CompileContextEvent, CompileNodeEvent};
    pub use crate::data::{Action, Attribute, HtmlTemplate, NodeType, StyleAttr};
    pub use crate::error::ParseError;
    pub use crate::styles::{HoverTimer, InteractionTimer, NodeStyle, PressedTimer, UiActive};
    pub use crate::HtmlUiPlugin;
}

#[derive(Default)]
pub struct HtmlUiPlugin;

impl Plugin for HtmlUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            load::LoaderPlugin,
            build::BuildPlugin,
            bindings::BindingPlugin,
            styles::TransitionPlugin,
            compile::CompilePlugin,
        ));
    }
}

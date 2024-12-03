#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../../../README.md")]

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
mod util;

pub mod prelude {
    pub use crate::auto::{AutoLoadState, HuiAutoLoadPlugin};
    pub use crate::bindings::{
        ComponentBindings, FunctionBindings, HtmlComponents, HtmlFunctions, UiChangedEvent,
    };
    pub use crate::build::{
        HtmlNode, OnUiChange, OnUiEnter, OnUiExit, OnUiPress, OnUiSpawn, Tags, TemplateProperties,
        TemplateScope, UiId, UiTarget, UiWatch,
    };
    pub use crate::compile::{CompileContextEvent, CompileNodeEvent};
    pub use crate::data::{Action, Attribute, HtmlTemplate, NodeType, StyleAttr};
    pub use crate::error::ParseError;
    pub use crate::error::VerboseHtmlError;
    pub use crate::parse::parse_template;
    pub use crate::styles::{HoverTimer, HtmlStyle, InteractionTimer, PressedTimer, UiActive};
    pub use crate::HuiPlugin;
}

pub struct HuiPlugin;
impl Plugin for HuiPlugin {
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

use bevy::prelude::*;
use build::HtmlBundle;
use data::Template;
use prelude::ComponentBindings;

mod bindings;
mod build;
mod data;
mod error;
mod load;
mod parse;
mod state;
// mod lexer;

pub mod prelude {
    pub use crate::bindings::{ComponentBindings, FunctionBindings};
    pub use crate::build::{
        HtmlBundle, OnEnter, OnExit, OnPress, OnSpawn, StyleAttributes, Tag, Tags, TemplateState,
        UiId, UiTarget,
    };
    pub use crate::data::{Action, Attribute, NodeType, StyleAttr, Template};
    pub use crate::error::ParseError;
    pub use crate::XmlUiPlugin;
}

#[derive(Default)]
pub struct XmlUiPlugin {
    auto_load_dirs: Vec<&'static str>,
    extension: &'static str,
}

impl XmlUiPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn auto_load(mut self, path: &str) -> Self {
        self
    }
}

impl Plugin for XmlUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            load::LoaderPlugin,
            build::BuildPlugin,
            bindings::BindingPlugin,
        ));

        // ------------------
        // auto load dirs
        // move to system
        // let mut to_load = self
        //     .auto_load_dirs
        //     .iter()
        //     .flat_map(|dir_path| {
        //         if let Ok(dir) = std::fs::read_dir(std::path::Path::new("assets").join(dir_path)) {
        //             return Some(dir);
        //         };
        //         warn!("[lommx ui] - cannot read dir `{dir_path}`");
        //         None
        //     })
        //     .map(|dir| dir.into_iter().flatten())
        //     .flatten()
        //     .map(|entry| {
        //         let name = entry.file_name().to_string_lossy().to_string();
        //         let path = entry.path();
        //         let handle: Handle<Template> = app.world().resource::<AssetServer>().load(path);
        //         (name, handle)
        //     })
        //     .collect::<Vec<_>>();
        //
        // let mut comp_registry = app.world_mut().resource_mut::<ComponentBindings>();
        //
        // for (name, handle) in to_load.drain(..) {
        //     comp_registry.register(name, move |mut cmd| {
        //         cmd.insert(HtmlBundle {
        //             handle: handle.clone(),
        //             ..default()
        //         });
        //     });
        // }
    }
}

// fn watch_comp_dirs(
//     plugin_config : Res<Plugin<LommixUiPlugin>>,
//
// ){
//
// }

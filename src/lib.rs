use std::{
    hash::{Hash, Hasher},
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer};
use build::TemplateBundle;
use data::Template;
use prelude::ComponentBindings;

mod bindings;
mod build;
mod compile;
mod data;
mod error;
mod load;
mod parse;
mod styles;

pub mod prelude {
    pub use crate::bindings::{ComponentBindings, FunctionBindings};
    pub use crate::build::{
        TemplateBundle, OnEnter, OnExit, OnPress, OnSpawn, ScopeEntity, Tag, Tags, TemplateState, UiId,
        UiTarget, UiWatch, UnbuildTag,
    };
    pub use crate::compile::{CompileContextEvent, CompileNodeEvent};
    pub use crate::data::{Action, Attribute, NodeType, StyleAttr, Template};
    pub use crate::error::ParseError;
    pub use crate::styles::NodeStyle;
    pub use crate::XmlUiPlugin;
}

/// @todo: inline docs
/// look at the examples
#[derive(Default, Resource, Clone)]
pub struct XmlUiPlugin {
    auto_load_dir: Option<&'static str>,
    extension: Option<&'static str>,
}

impl XmlUiPlugin {
    pub fn new() -> Self {
        Self::default()
    }
    /// auto_load("components","xml")
    pub fn auto_load(mut self, path: &'static str, ext: &'static str) -> Self {
        self.extension = Some(ext);
        self.auto_load_dir = Some(path);
        self
    }
}

impl Plugin for XmlUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            load::LoaderPlugin,
            build::BuildPlugin,
            bindings::BindingPlugin,
            styles::TransitionPlugin,
            compile::CompilePlugin,
        ));

        app.insert_resource(self.clone());

        #[cfg(debug_assertions)]
        app.add_systems(
            First,
            watch_autolaod_dirs.run_if(on_timer(Duration::from_secs(1))),
        );

        #[cfg(debug_assertions)]
        app.add_systems(Startup, watch_autolaod_dirs);
    }
}

fn watch_autolaod_dirs(
    config: Res<XmlUiPlugin>,
    server: Res<AssetServer>,
    mut comps: ResMut<ComponentBindings>,
    mut last_checksum: Local<u64>,
) {
    let (Some(dir), Some(ext)) = (config.auto_load_dir, config.extension) else {
        return;
    };

    let Ok((checksum, paths)) = gen_checksum(dir, ext) else {
        return;
    };

    if checksum == *last_checksum {
        return;
    }

    for path in paths.iter() {
        let handle: Handle<Template> = server.load(path);
        let name = std::path::Path::new(path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        comps.register(name.to_string(), move |mut cmd| {
            cmd.insert(TemplateBundle {
                handle: handle.clone(),
                ..default()
            });
        });

        info!("registered/reloaded component `{name}` at `{path}`");
    }

    *last_checksum = checksum;
}

fn gen_checksum(path: &str, allowed_ext: &str) -> Result<(u64, Vec<String>), std::io::Error> {
    let fullpath = format!("assets/{}", path);
    let dir = std::fs::read_dir(fullpath)?.flatten();

    let mut paths = vec![];
    let mut hasher = std::hash::DefaultHasher::default();

    for entry in dir {
        if !entry.file_type()?.is_file() {
            continue;
        }

        let is_ext = entry
            .path()
            .extension()
            .map(|s| s.to_str())
            .flatten()
            .map(|ext| ext == allowed_ext)
            .unwrap_or_default();

        if !is_ext {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        file_name.hash(&mut hasher);
        paths.push(format!("{}/{}", path, file_name));
    }

    Ok((hasher.finish(), paths))
}

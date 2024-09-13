use bevy::{prelude::*, utils::HashMap};

use crate::{
    build::{StyleAttributes, UnStyled},
    data::{Attribute, Property, StyleAttr},
    parse::parse_attribute,
    prelude::ComponentBindings,
};

pub struct PropertyPlugin;
impl Plugin for PropertyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, compile_properties);
    }
}

#[derive(Component, Default)]
pub struct ReCompile;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ToCompile(pub Vec<Property>);

#[derive(Component, Deref, Clone, DerefMut, Default)]
pub struct PropertyDefintions(HashMap<String, String>);

impl PropertyDefintions {
    pub fn new() -> Self{
        Self::default()
    }
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }
}

pub fn find_def<'a, 'b>(
    entity: Entity,
    key: &'b str,
    defs: &'a Query<&PropertyDefintions>,
    parents: &'a Query<&Parent>,
) -> Option<&'a str> {
    if let Some(v) = defs.get(entity).ok().map(|def| def.get(key)).flatten() {
        return Some(v.as_str());
    }

    let Ok(parent_ent) = parents.get(entity).map(|p| p.get()) else {
        return None;
    };

    find_def(parent_ent, key, defs, parents)
}

fn compile_properties(
    mut cmd: Commands,
    parents: Query<&Parent>,
    definitions: Query<&PropertyDefintions>,
    mut to_compile: Query<(Entity, &mut StyleAttributes, &mut ToCompile)>,
    spawn_bindings: Res<ComponentBindings>,
) {
    to_compile
        .iter_mut()
        .for_each(|(entity, mut styles, mut to_compile)| {
            // try to compile
            // --> success --> remove from stack
            // --> all compiled? --> remove Component
            // --> def unfindable? No Slotholder parent && end reached without propval found
            to_compile.retain(|raw_prop| {
                // try compile
                let Some(prop_val) = find_def(entity, &raw_prop.key, &definitions, &parents) else {
                    return true;
                };

                let prefix = raw_prop
                    .prefix
                    .as_ref()
                    .map(|p| format!("{}:", p))
                    .unwrap_or_default();

                let attribute_string = format!(r#"{}{}="{}""#, prefix, raw_prop.ident, prop_val);
                let Ok((_, attr)) = parse_attribute(attribute_string.as_bytes()) else {
                    warn!("failed to parse property `{}`", attribute_string);
                    return true;
                };

                // apply attr
                info!("success full compiled property {attribute_string}");

                // ---------------------------------------
                match attr {
                    Attribute::Style(s) => {
                        // upsert style
                        upsert_style(s, &mut styles);
                        cmd.entity(entity).insert(UnStyled);
                    }
                    Attribute::Action(action) => {
                        // insert action
                        action.self_insert(cmd.entity(entity));
                    }
                    Attribute::SpawnFunction(spawn) => {
                        // rerun spawn
                        spawn_bindings.try_spawn(&spawn, entity, &mut cmd);
                    }
                    Attribute::Path(_) => {
                        warn!("recursive includes not supported");
                    }
                    _ => {
                        warn!("recursive properties not supported");
                    }
                }

                // success --> false
                return false;
            });

            // timeout?

            if to_compile.is_empty() {
                // remove comp
                cmd.entity(entity).remove::<ToCompile>();
            }
        });
}

fn upsert_style(style: StyleAttr, target: &mut StyleAttributes) {
    for s in target.0.iter_mut() {
        if std::mem::discriminant(&style) == std::mem::discriminant(s) {
            *s = style;
            return;
        }
    }
    target.0.push(style);
}

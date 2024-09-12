use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    bindings::SpawnFunction,
    build::{StyleAttributes, UnStyled},
    data::{Attribute, Property, StyleAttr},
    parse::parse_attribute,
    prelude::ComponenRegistry,
};

pub struct PropertyPlugin;
impl Plugin for PropertyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PropTree>();
        app.add_systems(Update, compile_properties);
    }
}

// virtual tree required
// includes & slots move and park, bevy tree hirachy might be invalid at times.
// @todo: **leaks memory**, may have artefacts, fix later
#[derive(Default, Resource)]
pub struct PropTree {
    prop_defs: HashMap<Entity, PropertyDefintions>,
    parents: HashMap<Entity, Entity>,
}

impl PropTree {
    pub fn find_def_up<'a>(&'a self, entity: Entity, key: &'a str) -> Option<&'a String> {
        if let Some(props) = self
            .prop_defs
            .get(&entity)
            .map(|defs| defs.get(key))
            .flatten()
        {
            return Some(props);
        }

        match self.parents.get(&entity) {
            Some(parent) => self.find_def_up(*parent, key),
            None => None,
        }
    }

    pub fn set_parent(&mut self, child: Entity, parent: Entity) {
        self.parents.insert(child, parent);
    }

    pub fn insert(&mut self, entity: Entity, key: String, value: String) {
        _ = match self.prop_defs.get_mut(&entity) {
            Some(props) => {
                props.insert(key, value);
            }
            None => {
                let mut props = PropertyDefintions::default();
                props.insert(key, value);
                self.prop_defs.insert(entity, props);
            }
        }
    }

    /// does not overwrite
    pub fn try_insert(&mut self, entity: Entity, key: String, value: String) {

        info!("try insert! key: {key} val: {value}");

        _ = match self.prop_defs.get_mut(&entity) {
            Some(props) => {
                _ = props.try_insert(key, value);
            }
            None => {
                let mut props = PropertyDefintions::default();
                props.insert(key, value);
                self.prop_defs.insert(entity, props);
            }
        }
    }
}

#[derive(Component, Default)]
pub struct ReCompile;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ToCompile(pub Vec<Property>);

#[derive(Component, Deref, Clone, DerefMut, Default)]
pub struct PropertyDefintions(HashMap<String, String>);

fn compile_properties(
    mut cmd: Commands,
    prop_tree: Res<PropTree>,
    mut to_compile: Query<(Entity, &mut StyleAttributes, &mut ToCompile)>,
    spawn_bindings: Res<ComponenRegistry>,
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
                let Some(prop_val) = prop_tree.find_def_up(entity, &raw_prop.key) else {
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
                        action.apply(cmd.entity(entity));
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

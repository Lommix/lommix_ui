use bevy::{prelude::*, utils::HashMap};

use crate::{
    build::{StyleAttributes, Tags, UnStyled},
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

#[derive(Component)]
pub struct NeedsPropCompile;

#[derive(Component, Deref, DerefMut, Default)]
pub struct PropertyDefintions(HashMap<String, String>);

impl PropertyDefintions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Properties(Vec<Property>);

impl Properties {
    pub fn new(props: Vec<Property>) -> Self {
        Self(props)
    }

    pub fn has(&self, prop: &Property) -> bool {
        self.iter().any(|p| p.key == prop.key)
    }
}

// defs do not need to be mut
// fightning types, fix later
pub fn find_def<'a, 'b>(
    entity: Entity,
    key: &'b str,
    defs: &'a Query<&mut PropertyDefintions>,
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

///
/// compile once, no need to retry
/// don't compile before parents are done
fn compile_properties(
    mut cmd: Commands,
    mut tags: Query<&mut Tags>,
    needs_compile: Query<Entity, With<NeedsPropCompile>>,
    mut nodes: Query<(&mut StyleAttributes, &Properties)>,
    mut definitions: Query<&mut PropertyDefintions>,
    parents: Query<&Parent>,
    spawn_bindings: Res<ComponentBindings>,
) {
    needs_compile.iter().for_each(|entity| {
        if parents
            .get(entity)
            .map(|p| needs_compile.get(p.get()).is_ok())
            .unwrap_or_default()
        {
            // wait for parent to compile
            info!("waiting for parent");
            return;
        }

        let Ok((mut styles, properties)) = nodes.get_mut(entity) else {
            return;
        };

        // try to compile
        // --> success --> remove from stack
        // --> all compiled? --> remove Component
        // --> def unfindable? No Slotholder parent && end reached without propval found
        properties.iter().for_each(|raw_prop| {
            // try compile
            // check if the parent has uncompiled code

            let Some(prop_val) = find_def(entity, &raw_prop.key, &definitions, &parents) else {
                return;
            };

            let prefix = raw_prop
                .prefix
                .as_ref()
                .map(|p| format!("{}:", p))
                .unwrap_or_default();

            let attribute_string = format!(r#"{}{}="{}""#, prefix, raw_prop.ident, prop_val);
            let Ok((_, attr)) = parse_attribute(attribute_string.as_bytes()) else {
                warn!("failed to parse property `{}`", attribute_string);
                return;
            };

            // apply attr
            // info!("success full compiled property {attribute_string}");

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
                Attribute::PropertyDefinition(key, val) => {
                    _ = definitions.get_mut(entity).map(|mut defs| {
                        defs.insert(key, val);
                    });
                }
                Attribute::Custom(key, value) => {
                    _ = tags.get_mut(entity).map(|mut tag| {
                        tag.push(crate::build::Tag { key, value });
                    });
                }
                any => {
                    warn!("cannot overwrite value dynamic: {:?}", any);
                }
            };
        });

        cmd.entity(entity).remove::<NeedsPropCompile>();
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

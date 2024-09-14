use crate::{
    data::{Action, Attribute, NodeType, Property, XNode},
    prelude::{ComponentBindings, StyleAttr},
    properties::{find_def, NeedsPropCompile, Properties, PropertyDefintions},
};
use bevy::{prelude::*, utils::HashMap};
use nom::{
    bytes::{complete::is_not, streaming::tag},
    sequence::tuple,
    IResult,
};

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    hotreload,
                    spawn_ui,
                    move_children_to_slot,
                    compile_ui,
                    style_ui,
                )
                    .chain(),
                update_interaction,
            ),
        );
    }
}

#[derive(Component)]
pub struct SlotedNode;

#[derive(Component)]
pub struct SlotTag;

#[derive(Component)]
pub struct UnslotedChildren(Entity);

#[derive(Component, Deref, DerefMut)]
pub struct Tags(Vec<Tag>);

impl Tags {
    pub fn get_tag(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|entry| entry.key.eq(key))
            .map(|entry| entry.value.as_str())
    }
}

#[derive(Component)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

#[derive(Component, Deref)]
pub struct StyleAttributes(pub Vec<StyleAttr>);

/// the entities owned uid `id="my_id"`
#[derive(Component, Default, Hash, Deref, DerefMut)]
pub struct UiId(u64);

/// the entity behind `id` in `target="id"`
#[derive(Component, DerefMut, Deref)]
pub struct UiTarget(pub Entity);

#[derive(Component, Default)]
pub struct UnbuildTag;

/// tag fro reapplying styles
#[derive(Component, Default)]
pub struct UnStyled;

/// tag for recompiles text content
#[derive(Component)]
pub struct UnCompiled;

/// Eventlistener interaction transitions to Hover
#[derive(Component, Deref, DerefMut)]
pub struct OnPress(pub Vec<String>);

/// Eventlistener on spawning node
#[derive(Component, DerefMut, Deref)]
pub struct OnSpawn(pub Vec<String>);

/// Eventlistener for interaction transitions to Hover
#[derive(Component, DerefMut, Deref)]
pub struct OnEnter(pub Vec<String>);

/// Eventlistener for interaction transitions to None
#[derive(Component, Deref, DerefMut)]
pub struct OnExit(pub Vec<String>);

///
/// Spawns a ui template behind an asset.
///
#[derive(Bundle, Default)]
pub struct HtmlBundle {
    pub handle: Handle<XNode>,
    pub node: NodeBundle,
    pub unbuild: UnbuildTag,
    pub unstyled: UnStyled,
}

fn update_interaction(
    mut cmd: Commands,
    mut nodes: Query<
        (
            Entity,
            &mut Style,
            &StyleAttributes,
            &Interaction,
            Option<&mut Text>,
        ),
        Changed<Interaction>,
    >,
    server: Res<AssetServer>,
) {
    nodes.iter_mut().for_each(
        |(entity, mut style, style_attr, interaction, mut maybe_text)| match interaction {
            Interaction::Pressed => {
                style_attr.iter().for_each(|attr| {
                    if let StyleAttr::Pressed(val) = attr {
                        val.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server);
                    }
                });
            }
            Interaction::Hovered => {
                style_attr.iter().for_each(|attr| {
                    if let StyleAttr::Hover(val) = attr {
                        val.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server);
                    }
                });
            }
            Interaction::None => {
                *style = Style::default();
                style_attr.iter().for_each(|attr| match attr {
                    StyleAttr::Hover(_) | StyleAttr::Pressed(_) => (),
                    any => any.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server),
                });
            }
        },
    );
}

fn hotreload(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<XNode>>,
    templates: Query<(Entity, &Handle<XNode>)>,
    children: Query<&Children>,
    sloted_nodes: Query<Entity, With<SlotedNode>>,
) {
    events.read().for_each(|ev| {
        let id = match ev {
            AssetEvent::Modified { id } => id,
            _ => {
                return;
            }
        };

        templates
            .iter()
            .filter(|(_, handle)| handle.id() == *id)
            .for_each(|(entity, handle)| {
                let slots = find_sloted_children(entity, &children, &sloted_nodes, &templates);

                if slots.len() > 0 {
                    // info!("saved slots {}", slots.len());
                    let slot_holder = cmd.spawn_empty().push_children(&slots).id();
                    cmd.entity(entity).insert(UnslotedChildren(slot_holder));
                }

                cmd.entity(entity)
                    .despawn_descendants()
                    .retain::<KeepComps>()
                    .insert(UnbuildTag);
            });
    });
}

#[derive(Bundle)]
struct KeepComps {
    pub parent: Parent,
    pub children: Children,
    pub ui: HtmlBundle,
    pub sloted_nodes: SlotedNode,
    pub slot: SlotTag,
    pub unsloted: UnslotedChildren,
    pub props: PropertyDefintions,
    pub uncompiled: NeedsPropCompile,
}

fn find_sloted_children(
    entity: Entity,
    childrens: &Query<&Children>,
    sloted_nodes: &Query<Entity, With<SlotedNode>>,
    templates: &Query<(Entity, &Handle<XNode>)>,
) -> Vec<Entity> {
    let Ok(children) = childrens.get(entity) else {
        return vec![];
    };

    let mut out = children
        .iter()
        .filter(|c| sloted_nodes.get(**c).is_ok())
        .cloned()
        .collect::<Vec<_>>();

    for child in children.iter() {
        if templates.get(*child).is_ok() {
            continue;
        }
        out.extend(find_sloted_children(
            *child,
            childrens,
            sloted_nodes,
            templates,
        ));
    }

    out
}

fn style_ui(
    mut cmd: Commands,
    mut unstyled: Query<(Entity, &mut Style, &StyleAttributes, Option<&mut Text>), With<UnStyled>>,
    server: Res<AssetServer>,
) {
    unstyled
        .iter_mut()
        .for_each(|(entity, mut style, style_attr, mut maybe_text)| {
            style_attr.iter().for_each(|attr| match attr {
                StyleAttr::Hover(_) | StyleAttr::Pressed(_) => (),
                any => any.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server),
            });
            cmd.entity(entity).remove::<UnStyled>();
        });
}

fn move_children_to_slot(
    mut cmd: Commands,
    unsloted_includes: Query<(Entity, &UnslotedChildren)>,
    children: Query<&Children>,
    slots: Query<&SlotTag>,
    parent: Query<&Parent>,
) {
    unsloted_includes
        .iter()
        .for_each(|(entity, UnslotedChildren(slot_holder))| {
            // slot is a empty entity
            let Some(slot) = find_slot(entity, &slots, &children) else {
                warn!("this node does not have a slot");
                return;
            };

            // slot is a empty entity
            let Ok(slot_parent) = parent.get(slot).map(|p| p.get()) else {
                warn!("parentless slot, impossible");
                return;
            };

            // info!("found slot! {slot}");
            _ = children.get(*slot_holder).map(|children| {
                children.iter().for_each(|child| {
                    cmd.entity(slot_parent).add_child(*child);
                })
            });

            cmd.entity(entity).remove::<UnslotedChildren>();
            cmd.entity(slot).despawn_recursive();
            cmd.entity(*slot_holder).despawn();
        });
}

fn find_slot(
    entity: Entity,
    slots: &Query<&SlotTag>,
    children: &Query<&Children>,
) -> Option<Entity> {
    if slots.get(entity).is_ok() {
        return Some(entity);
    }

    let Ok(ent_children) = children.get(entity) else {
        return None;
    };

    for child in ent_children.iter() {
        if let Some(slot) = find_slot(*child, slots, children) {
            return Some(slot);
        }
    }

    None
}

#[derive(Default)]
struct IdLookUpTable {
    ids: HashMap<String, Entity>,
    targets: HashMap<Entity, String>,
}

fn spawn_ui(
    mut cmd: Commands,
    unbuild: Query<(Entity, &Handle<XNode>), With<UnbuildTag>>,
    assets: Res<Assets<XNode>>,
    server: Res<AssetServer>,
    custom_comps: Res<ComponentBindings>,
    mut defintions: Query<&mut PropertyDefintions>,
    mut properties: Query<&mut Properties>,
) {
    unbuild.iter().for_each(|(ent, handle)| {
        let Some(ui_node) = assets.get(handle) else {
            return;
        };

        let def = defintions.get_mut(ent).ok();
        let props = properties.get_mut(ent).ok();

        let mut id_table = IdLookUpTable::default();
        build_node(
            ent,
            &ui_node,
            &mut cmd,
            &assets,
            &server,
            &custom_comps,
            &mut id_table,
            def,
            props,
        );

        id_table
            .targets
            .iter()
            .for_each(|(entity, target_id)| match id_table.ids.get(target_id) {
                Some(tar) => {
                    cmd.entity(*entity).insert(UiTarget(*tar));
                }
                None => warn!("target `{target_id}` not found for entity {entity}"),
            });

        cmd.entity(ent).remove::<UnbuildTag>();
    });
}

/// big recursive boy
#[allow(clippy::too_many_arguments)]
fn build_node(
    entity: Entity,
    node: &XNode,
    cmd: &mut Commands,
    assets: &Assets<XNode>,
    server: &AssetServer,
    custom_comps: &ComponentBindings,
    id_table: &mut IdLookUpTable,
    defintions: Option<Mut<PropertyDefintions>>,
    properties: Option<Mut<Properties>>,
) {
    let mut attributes = SortedAttributes::from(&node.attributes);
    match defintions {
        // preserve already set props for includes
        // not on includes, on the first node of the include
        // only first node can already exist in the tree -> query in sys once
        Some(mut existing) => attributes.definitions.drain(..).for_each(|(key, val)| {
            _ = existing.try_insert(key, val);
        }),
        None => {
            let defs = attributes.definitions.drain(..).fold(
                PropertyDefintions::default(),
                |mut m, (key, val)| {
                    m.insert(key, val);
                    m
                },
            );
            cmd.entity(entity).insert(defs);
        }
    };

    if attributes.properties.len() > 0 {
        let mut props: Vec<Property> = attributes.properties.drain(..).collect();
        // check if already has some
        match properties {
            Some(mut parent_properties) => {
                for prop in props.drain(..) {
                    if parent_properties.has(&prop) {
                        continue;
                    }
                    parent_properties.push(prop);
                }
                cmd.entity(entity).insert(NeedsPropCompile);
            }
            None => {
                cmd.entity(entity)
                    .insert((Properties::new(props), NeedsPropCompile));
            }
        }
    }

    attributes.actions.drain(..).for_each(|action| {
        action.self_insert(cmd.entity(entity));
    });

    if let Some(id) = attributes.id {
        id_table.ids.insert(id, entity);
    }
    if let Some(target) = attributes.target {
        id_table.targets.insert(entity, target);
    }

    // ------------------
    if attributes.custom.len() > 0 {
        cmd.entity(entity).insert(Tags(
            attributes
                .custom
                .drain(..)
                .map(|(key, value)| Tag { key, value })
                .collect::<Vec<_>>(),
        ));
    }

    match &node.node_type {
        NodeType::Node => {
            cmd.entity(entity).insert((
                Name::new("Div"),
                NodeBundle::default(),
                StyleAttributes(attributes.styles.drain(..).collect::<Vec<_>>()),
                UnStyled,
            ));
        }
        NodeType::Image => {
            if let Some(path) = attributes.path {
                cmd.entity(entity).insert((
                    Name::new("Image"),
                    ImageBundle {
                        image: UiImage::new(server.load(path)),
                        ..default()
                    },
                    StyleAttributes(attributes.styles.drain(..).collect::<Vec<_>>()),
                    UnStyled,
                ));
            } else {
                warn!("trying to spawn image with no path")
            }
        }
        NodeType::Text => {
            let content = node.content.as_ref().cloned().unwrap_or_default();
            cmd.entity(entity).insert((
                Name::new("Text"),
                TextBundle::from_section(
                    content,
                    TextStyle {
                        font_size: 16., // default
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                StyleAttributes(attributes.styles.drain(..).collect::<Vec<_>>()),
                UnStyled,
                UnCompiled,
            ));
        }
        NodeType::Button => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                StyleAttributes(attributes.styles.drain(..).collect::<Vec<_>>()),
                UnStyled,
            ));
        }
        NodeType::Include => {
            let path = attributes.path.unwrap_or_default();
            let handle = server.load::<XNode>(path);

            cmd.entity(entity)
                .insert((handle, UnbuildTag, NodeBundle::default(), UnStyled));

            if node.children.len() > 0 {
                let slot_holder = cmd.spawn_empty().id();

                node.children.iter().for_each(|child_node| {
                    let child = cmd.spawn(SlotedNode).id();
                    build_node(
                        child,
                        child_node,
                        cmd,
                        assets,
                        server,
                        custom_comps,
                        id_table,
                        None,
                        None,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });
                cmd.entity(entity).insert(UnslotedChildren(slot_holder));
            }

            return;
        }
        NodeType::Slot => {
            cmd.entity(entity).insert((SlotTag, NodeBundle::default()));
            return;
        }
        NodeType::Custom(custom_tag) => {
            custom_comps.try_spawn(custom_tag, entity, cmd);
            if node.children.len() > 0 {
                let slot_holder = cmd.spawn_empty().id();

                node.children.iter().for_each(|child_node| {
                    let child = cmd.spawn(SlotedNode).id();
                    build_node(
                        child,
                        child_node,
                        cmd,
                        assets,
                        server,
                        custom_comps,
                        id_table,
                        None,
                        None,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });
                cmd.entity(entity).insert(UnslotedChildren(slot_holder));
            }

            return;
        }
    };

    for child_node in node.children.iter() {
        let child = cmd.spawn_empty().id();
        build_node(
            child,
            child_node,
            cmd,
            assets,
            server,
            custom_comps,
            id_table,
            None,
            None,
        );

        cmd.entity(entity).add_child(child);
    }
}

#[derive(Default)]
struct SortedAttributes {
    pub styles: Vec<StyleAttr>,
    pub actions: Vec<Action>,
    pub path: Option<String>,
    pub spawn_functions_keys: Vec<String>,
    pub definitions: Vec<(String, String)>,
    pub properties: Vec<Property>,
    pub target: Option<String>,
    pub id: Option<String>,
    pub custom: Vec<(String, String)>,
}

impl From<&Vec<Attribute>> for SortedAttributes {
    fn from(value: &Vec<Attribute>) -> Self {
        let mut sorted = SortedAttributes::default();
        for attr in value.iter().cloned() {
            _ = match attr {
                Attribute::Style(style) => sorted.styles.push(style),
                Attribute::Action(action) => sorted.actions.push(action),
                Attribute::Path(path) => sorted.path = Some(path),
                Attribute::SpawnFunction(spawn) => sorted.spawn_functions_keys.push(spawn),
                Attribute::PropertyDefinition(key, val) => sorted.definitions.push((key, val)),
                Attribute::UnCompiledProperty(prop) => sorted.properties.push(prop),
                Attribute::Target(target) => sorted.target = Some(target),
                Attribute::Id(id) => sorted.id = Some(id),
                Attribute::Custom(key, value) => {
                    info!("found custom {key} : {value}");
                    sorted.custom.push((key, value))
                }
            };
        }
        sorted
    }
}

fn compile_ui(
    mut cmd: Commands,
    mut texts: Query<(Entity, &mut Text), With<UnCompiled>>,
    parents: Query<&Parent>,
    definitions: Query<&mut PropertyDefintions>,
) {
    texts.iter_mut().for_each(|(entity, mut text)| {
        for section in text.sections.iter_mut() {
            let Ok((_, compiled)) = compile_content(entity, &section.value, |key| {
                find_def(entity, key, &definitions, &parents)
            }) else {
                continue;
            };
            section.value = compiled;
        }

        cmd.entity(entity).remove::<UnCompiled>();
    });
}

fn compile_content<'a, 'b>(
    entity: Entity,
    input: &'a str,
    find_val: impl Fn(&'a str) -> Option<&'a str>,
) -> IResult<&'a str, String> {
    let mut out = String::new();

    let (remaining, (text_content, _, prop_key, _)) =
        tuple((is_not("{"), tag("{"), is_not("}"), tag("}")))(input)?;

    out.push_str(text_content);

    match find_val(prop_key) {
        Some(val) => {
            out.push_str(val);
        }
        None => (),
    };

    if remaining.len() > 0 {
        if let Ok((_, str)) = compile_content(entity, remaining, find_val) {
            out.push_str(&str);
        } else {
            out.push_str(remaining);
        }
    }

    Ok(("", out))
}

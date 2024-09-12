use crate::{
    data::{Action, Attribute, NodeType, Property, XNode},
    prelude::{ComponenRegistry, StyleAttr},
    properties::{PropTree, PropertyDefintions, ToCompile},
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
                (hotreload, spawn_ui, move_children_to_slot, style_ui).chain(),
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

#[derive(Component, Deref)]
pub struct StyleAttributes(pub Vec<StyleAttr>);

#[derive(Default, Resource)]
pub struct IdLookUpTable {
    ids: HashMap<u64, Entity>,
    targets: HashMap<Entity, u64>,
}

#[derive(Component, Default, Hash)]
pub struct UiId(u64);

#[derive(Component, DerefMut, Deref)]
pub struct UiTarget(pub Entity);

#[derive(Component, Default)]
pub struct UnbuildTag;

#[derive(Component, Default)]
pub struct UnStyled;

#[derive(Component)]
pub struct OnPress(pub String);

#[derive(Component)]
pub struct OnSpawn(pub String);

#[derive(Component)]
pub struct OnEnter(pub String);

#[derive(Component)]
pub struct OnExit(pub String);

#[derive(Bundle, Default)]
pub struct UiBundle {
    pub handle: Handle<XNode>,
    pub _t1: UnbuildTag,
    pub _t2: UnStyled,
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
    pub node: NodeBundle,
    pub ui: UiBundle,
    pub sloted_nodes: SlotedNode,
    pub slot: SlotTag,
    pub unsloted: UnslotedChildren,
    pub props: PropertyDefintions,
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

fn spawn_ui(
    mut cmd: Commands,
    unbuild: Query<(Entity, &Handle<XNode>), With<UnbuildTag>>,
    assets: Res<Assets<XNode>>,
    server: Res<AssetServer>,
    custom_comps: Res<ComponenRegistry>,
    parents: Query<&Parent>,
    mut prop_tree: ResMut<PropTree>,
) {
    unbuild.iter().for_each(|(ent, handle)| {
        let Some(ui_node) = assets.get(handle) else {
            return;
        };

        // info!(
        //     "spawning ui {}",
        //     handle.path().map(|p| p.to_string()).unwrap_or_default()
        // );

        // if parents.get(ent).is_ok() {
        //     info!("hass prent ",);
        // }

        let mut id_table = IdLookUpTable::default();

        build_node(
            ent,
            &ui_node,
            &mut cmd,
            &assets,
            &server,
            &custom_comps,
            &mut prop_tree,
            &mut id_table,
        );

        id_table
            .targets
            .iter()
            .for_each(|(entity, target_id)| match id_table.ids.get(target_id) {
                Some(tar) => {
                    cmd.entity(*entity).insert(UiTarget(*tar));
                }
                None => warn!("target not found for entity {entity}"),
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
    custom_comps: &ComponenRegistry,
    prop_tree: &mut PropTree,
    id_table: &mut IdLookUpTable,
) {
    let mut attributes = SortedAttributes::from(&node.attributes);
    _ = match &node.node_type {
        // preserve already set props for includes
        NodeType::Include => attributes
            .definitions
            .drain(..)
            .for_each(|(key, val)| prop_tree.insert(entity, key, val)),
        _ => attributes
            .definitions
            .drain(..)
            .for_each(|(key, val)| prop_tree.try_insert(entity, key, val)),
    };

    if attributes.properties.len() > 0 {
        cmd.entity(entity).insert(ToCompile(
            attributes.properties.drain(..).collect::<Vec<_>>(),
        ));
    }

    if let Some(id) = attributes.id {
        // cmd.entity(entity).insert(UiId(id));
        id_table.ids.insert(id, entity);
    }

    if let Some(target) = attributes.target {
        id_table.targets.insert(entity, target);
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
            // compile text here? may break hotreload
            let content = node
                .content
                .as_ref()
                .map(|c| match compile_content(entity, c, prop_tree) {
                    Ok((_, compiled)) => compiled,
                    Err(_) => c.clone(),
                })
                .unwrap_or_default();

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
            ));
        }
        NodeType::Button => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                StyleAttributes(attributes.styles.drain(..).collect::<Vec<_>>()),
                UnStyled,
            ));

            for action in attributes.actions.drain(..) {
                action.apply(cmd.entity(entity));
            }
        }
        NodeType::Include => {
            let path = attributes.path.unwrap_or_default();
            // info!("loading component {path}");
            let handle = server.load::<XNode>(path);

            cmd.entity(entity)
                .insert((handle, UnbuildTag, NodeBundle::default(), UnStyled));

            if node.children.len() > 0 {
                let slot_holder = cmd.spawn_empty().id();

                node.children.iter().for_each(|child_node| {
                    let child = cmd.spawn(SlotedNode).id();

                    // save the real parent after slot insert
                    prop_tree.set_parent(child, entity);

                    build_node(
                        child,
                        child_node,
                        cmd,
                        assets,
                        server,
                        custom_comps,
                        prop_tree,
                        id_table,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });

                // info!("found unsloted children");
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
                    // save the real parent after slot insert
                    prop_tree.set_parent(child, entity);
                    build_node(
                        child,
                        child_node,
                        cmd,
                        assets,
                        server,
                        custom_comps,
                        prop_tree,
                        id_table,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });
                // info!("found unsloted children");
                cmd.entity(entity).insert(UnslotedChildren(slot_holder));
            }

            return;
        }
    };

    for child_node in node.children.iter() {
        let child = cmd.spawn_empty().id();
        // prop_tree.hirachy.insert(child, entity);
        prop_tree.set_parent(child, entity);

        build_node(
            child,
            child_node,
            cmd,
            assets,
            server,
            custom_comps,
            prop_tree,
            id_table,
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
    pub target: Option<u64>,
    pub id: Option<u64>,
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
                Attribute::Property(prop) => sorted.properties.push(prop),
                Attribute::Target(target) => sorted.target = Some(target),
                Attribute::Id(id) => sorted.id = Some(id),
            };
        }
        sorted
    }
}

fn compile_content<'a, 'b>(
    entity: Entity,
    input: &'a str,
    prop_tree: &'b PropTree,
) -> IResult<&'a str, String> {
    let mut out = String::new();

    let (remaining, (text_content, _, prop_key, _)) =
        tuple((is_not("{"), tag("{"), is_not("}"), tag("}")))(input)?;

    out.push_str(text_content);

    match prop_tree.find_def_up(entity, prop_key) {
        Some(val) => {
            out.push_str(val);
        }
        None => (),
    };

    if remaining.len() > 0 {
        if let Ok((_, str)) = compile_content(entity, remaining, prop_tree) {
            out.push_str(&str);
        } else {
            out.push_str(remaining);
        }
    }

    Ok(("", out))
}

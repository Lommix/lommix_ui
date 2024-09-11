use crate::{
    data::{Action, Attribute, NodeType, Property, XNode},
    prelude::{SpawnBindings, StyleAttr},
    properties::{PropTree, PropertyDefintions, ToCompile},
};
use bevy::prelude::*;

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

impl StyleAttributes {}

#[derive(Component, Default)]
pub struct UnbuildTag;

#[derive(Component, Default)]
pub struct UnStyled;

#[derive(Component)]
pub struct OnPress(pub String);

#[derive(Component)]
pub struct OnEnter(pub String);

#[derive(Component)]
pub struct OnExit(pub String);

#[derive(Bundle, Default)]
pub struct UiBundle {
    pub handle: Handle<XNode>,
    pub tag: UnbuildTag,
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
                // find slot
                let slots = find_sloted_children(entity, &children, &sloted_nodes, &templates);

                if slots.len() > 0 {
                    info!("saved slots {}", slots.len());
                    let slot_holder = cmd.spawn_empty().push_children(&slots).id();
                    cmd.entity(entity).insert(UnslotedChildren(slot_holder));
                }

                //todo: clean components
                // there maybe left over added compontens from spawn functions
                cmd.entity(entity)
                    .despawn_descendants()
                    .retain::<MinimalComponentList>()
                    .insert(UnbuildTag);
            });
    });
}

#[derive(Bundle)]
struct MinimalComponentList {
    pub parent: Parent,
    pub children: Children,
    pub node: NodeBundle,
    pub ui: UiBundle,
    pub sloted_nodes: SlotedNode,
    pub slot: SlotTag,
    pub unsloted: UnslotedChildren,
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

            info!("found slot! {slot}");
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
    spawn_bindings: Res<SpawnBindings>,
    parents: Query<&Parent>,
    loaded_props: Query<&PropertyDefintions>,
    mut prop_tree: ResMut<PropTree>,
) {
    unbuild.iter().for_each(|(ent, handle)| {
        let Some(ui_node) = assets.get(handle) else {
            return;
        };

        info!(
            "spawning ui {}",
            handle.path().map(|p| p.to_string()).unwrap_or_default()
        );

        if parents.get(ent).is_ok() {
            info!("hass prent ",);
        }

        build_node(
            ent,
            &ui_node,
            &mut cmd,
            &assets,
            &server,
            &spawn_bindings,
            &mut prop_tree,
        );

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
    spawn_bindings: &SpawnBindings,
    prop_tree: &mut PropTree,
) {
    let mut attributes = SortedAttributes::from(&node.attributes);
    // parent later
    for (key, value) in attributes.definitions.drain(..) {
        prop_tree.insert(entity, key, value);
    }

    if attributes.properties.len() > 0 {
        cmd.entity(entity).insert(ToCompile(
            attributes.properties.drain(..).collect::<Vec<_>>(),
        ));
    }

    // todo --> move to seperate system
    attributes.spawn_functions_keys.iter().for_each(|key| {
        spawn_bindings.maybe_run(key, entity, cmd);
    });

    // build node
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
            cmd.entity(entity).insert((
                Name::new("Text"),
                TextBundle::from_section(
                    node.content.as_ref().cloned().unwrap_or_default(),
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
            info!("loading component {path}");
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
                        spawn_bindings,
                        prop_tree,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });

                info!("found unsloted children");
                cmd.entity(entity).insert(UnslotedChildren(slot_holder));
            }

            return;
        }
        NodeType::Slot => {
            cmd.entity(entity).insert((SlotTag, NodeBundle::default()));
            return;
        }
        NodeType::Custom(custom_tag) => {
            warn!("your custom tag `{custom_tag}` is not supported yet");
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
            spawn_bindings,
            prop_tree,
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
            };
        }
        sorted
    }
}

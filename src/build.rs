use bevy::prelude::*;

use crate::{
    data::{Action, Attribute, NodeType, XNode},
    load::ClickAction,
    prelude::{SpawnBindings, StyleAttr},
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

#[derive(Component, Default)]
pub struct UnbuildTag;

#[derive(Component, Default)]
pub struct UnStyled;

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

fn find_parent_template(
    entity: Entity,
    parents: &Query<&Parent>,
    templates: &Query<(Entity, &Handle<XNode>)>,
) -> Option<(Entity, Handle<XNode>)> {
    let Ok(parent_ent) = parents.get(entity).map(|p| p.get()) else {
        return None;
    };

    if let Ok((entity, handle)) = templates.get(parent_ent) {
        return Some((entity, handle.clone()));
    }

    find_parent_template(parent_ent, parents, templates)
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
) {
    unbuild.iter().for_each(|(ent, handle)| {
        let Some(ui_node) = assets.get(handle) else {
            return;
        };

        info!(
            "spawning ui {}",
            handle.path().map(|p| p.to_string()).unwrap_or_default()
        );

        build_node(ent, &ui_node, &mut cmd, &assets, &server, &spawn_bindings);
        cmd.entity(ent).remove::<UnbuildTag>();
    });
}

fn filter_styles(attributes: &Vec<Attribute>) -> Vec<StyleAttr> {
    attributes
        .iter()
        .flat_map(|attr| {
            if let Attribute::Style(style) = attr {
                return Some(style.clone());
            }
            None
        })
        .collect::<Vec<_>>()
}

fn get_path(attributes: &Vec<Attribute>) -> Option<String> {
    for attr in attributes.iter() {
        if let Attribute::Path(path) = attr {
            return Some(path.to_string());
        }
    }
    None
}

fn filter_action(attributes: &Vec<Attribute>) -> Vec<Action> {
    attributes
        .iter()
        .flat_map(|attr| {
            if let Attribute::Action(action) = attr {
                return Some(action.clone());
            }
            None
        })
        .collect::<Vec<_>>()
}

fn filter_spawn_function(attributes: &Vec<Attribute>) -> Vec<String> {
    attributes
        .iter()
        .flat_map(|attr| {
            if let Attribute::SpawnFunction(action) = attr {
                return Some(action.clone());
            }
            None
        })
        .collect::<Vec<_>>()
}

fn build_node(
    entity: Entity,
    node: &XNode,
    cmd: &mut Commands,
    assets: &Assets<XNode>,
    server: &AssetServer,
    spawn_bindings: &SpawnBindings,
) {
    filter_spawn_function(&node.attributes)
        .iter()
        .for_each(|spawn_fn| {
            spawn_bindings.maybe_run(spawn_fn, entity, cmd);
        });

    // build node
    match &node.node_type {
        NodeType::Div => {
            cmd.entity(entity).insert((
                Name::new("Div"),
                NodeBundle::default(),
                StyleAttributes(filter_styles(&node.attributes)),
                UnStyled,
            ));
        }
        NodeType::Image => {
            if let Some(path) = get_path(&node.attributes) {
                cmd.entity(entity).insert((
                    Name::new("Image"),
                    ImageBundle {
                        image: UiImage::new(server.load(path)),
                        ..default()
                    },
                    StyleAttributes(filter_styles(&node.attributes)),
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
                StyleAttributes(filter_styles(&node.attributes)),
                UnStyled,
            ));
        }
        NodeType::Button => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                StyleAttributes(filter_styles(&node.attributes)),
                UnStyled,
            ));

            for action in filter_action(&node.attributes).drain(..) {
                cmd.entity(entity).insert(action);
            }
        }
        NodeType::Include => {
            let path = get_path(&node.attributes).unwrap_or_default();
            info!("loading component {path}");
            let handle = server.load::<XNode>(path);

            cmd.entity(entity)
                .insert((handle, UnbuildTag, NodeBundle::default(), UnStyled));

            if node.children.len() > 0 {
                let slot_holder = cmd.spawn_empty().id();

                node.children.iter().for_each(|child_node| {
                    let child = cmd.spawn(SlotedNode).id();
                    build_node(child, child_node, cmd, assets, server, spawn_bindings);
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
        build_node(child, child_node, cmd, assets, server, spawn_bindings);
        cmd.entity(entity).add_child(child);
    }

    // add slot
}

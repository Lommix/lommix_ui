use bevy::prelude::*;

use crate::{data::XNode, load::ClickAction, prelude::StyleAttr};

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
pub struct SlotTag;

#[derive(Component)]
pub struct UnslotedChildren(Entity);

#[derive(Component, Deref)]
pub struct StyleAttributes(pub Vec<StyleAttr>);

#[derive(Component, Default)]
pub struct UnbuildTag;

#[derive(Component, Default)]
pub struct JustIncluded;

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

// [] includes:preserve slot on hotreload
fn hotreload(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<XNode>>,
    nodes: Query<(Entity, Option<&Parent>, &Handle<XNode>), With<Node>>,
) {
    events.read().for_each(|ev| {
        let id = match ev {
            AssetEvent::Modified { id } => id,
            _ => {
                return;
            }
        };

        nodes
            .iter()
            .filter(|(_, _, handle)| handle.id() == *id)
            .for_each(|(ent, maybe_parent, handle)| {
                cmd.entity(ent).despawn_recursive();

                let ent = cmd
                    .spawn(UiBundle {
                        handle: handle.clone(),
                        ..default()
                    })
                    .id();

                if let Some(parent) = maybe_parent {
                    cmd.entity(parent.get()).add_child(ent);
                }
            });
    });
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
) {
    unsloted_includes
        .iter()
        .for_each(|(entity, UnslotedChildren(slot_holder))| {
            let Some(slot) = find_slot(entity, &slots, &children) else {
                return;
            };

            info!("found slot! {slot}");
            _ = children.get(*slot_holder).map(|children| {
                children.iter().for_each(|child| {
                    cmd.entity(slot).add_child(*child);
                })
            });

            cmd.entity(entity).remove::<UnslotedChildren>();
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
) {
    unbuild.iter().for_each(|(ent, handle)| {
        let Some(ui_node) = assets.get(handle) else {
            return;
        };

        info!(
            "spawning ui {}",
            handle.path().map(|p| p.to_string()).unwrap_or_default()
        );

        build_node(ent, &ui_node, &mut cmd, &assets, &server);
        cmd.entity(ent).remove::<UnbuildTag>();
    });
}

fn build_node(
    entity: Entity,
    node: &XNode,
    cmd: &mut Commands,
    assets: &Assets<XNode>,
    server: &AssetServer,
) {
    // build node
    let children = match &node {
        XNode::Div(div) => {
            cmd.entity(entity).insert((
                Name::new("Div"),
                NodeBundle::default(),
                StyleAttributes(div.styles.clone()),
                UnStyled,
            ));
            Some(&div.children)
        }
        XNode::Image(img) => {
            cmd.entity(entity).insert((
                Name::new("Image"),
                ImageBundle {
                    image: UiImage::new(server.load(&img.path)),
                    ..default()
                },
                StyleAttributes(img.styles.clone()),
                UnStyled,
            ));
            None
        }
        XNode::Text(text) => {
            cmd.entity(entity).insert((
                Name::new("Text"),
                TextBundle::from_section(
                    &text.content,
                    TextStyle {
                        font_size: 16., // default
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                StyleAttributes(text.styles.clone()),
                UnStyled,
            ));
            None
        }
        XNode::Button(btn) => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                StyleAttributes(btn.styles.clone()),
                ClickAction(btn.action.clone()),
                UnStyled,
            ));
            Some(&btn.children)
        }
        XNode::Include(inc) => {
            let handle = server.load::<XNode>(&inc.path);

            cmd.entity(entity)
                .insert((handle, UnbuildTag, NodeBundle::default(), UnStyled));

            if inc.children.len() > 0 {
                let slot_holder = cmd.spawn_empty().id();

                inc.children.iter().for_each(|child_node| {
                    let child = cmd.spawn_empty().id();
                    build_node(child, child_node, cmd, assets, server);
                    cmd.entity(slot_holder).add_child(child);
                });

                info!("found unsloted children");
                cmd.entity(entity).insert(UnslotedChildren(slot_holder));
            }

            None
        }
        XNode::Slot => {
            cmd.entity(entity).insert((SlotTag, NodeBundle::default()));
            return;
        }
        _ => {
            return;
        }
    };

    children.map(|children| {
        children.iter().for_each(|child_node| {
            let child = cmd.spawn_empty().id();
            build_node(child, child_node, cmd, assets, server);
            cmd.entity(entity).add_child(child);
        });
    });

    // add slot
}

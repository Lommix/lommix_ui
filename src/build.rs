use bevy::prelude::*;

use crate::{data::XNode, load::ClickAction, prelude::StyleAttr};

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((hotrealod, build_ui, style_ui).chain(), update_interaction),
        );
    }
}

#[derive(Component, Deref)]
pub struct StyleAttributes(pub Vec<StyleAttr>);

#[derive(Component, Default)]
pub struct UnbuildTag;

#[derive(Component, Default)]
pub struct UnStyled;

#[derive(Bundle, Default)]
pub struct RonUiBundle {
    pub handle: Handle<XNode>,
    pub tag: UnbuildTag,
}

fn update_interaction(
    mut cmd: Commands,
    mut nodes: Query<(Entity, &mut Style, &StyleAttributes, &Interaction), Changed<Interaction>>,
) {
    nodes.iter_mut().for_each(
        |(entity, mut style, style_attr, interaction)| match interaction {
            Interaction::Pressed => {
                style_attr.iter().for_each(|attr| {
                    if let StyleAttr::Active(val) = attr {
                        val.apply(entity, &mut cmd, &mut style);
                    }
                });
            }
            Interaction::Hovered => {
                style_attr.iter().for_each(|attr| {
                    if let StyleAttr::Hover(val) = attr {
                        val.apply(entity, &mut cmd, &mut style);
                    }
                });
            }
            Interaction::None => {
                *style = Style::default();
                style_attr.iter().for_each(|attr| match attr {
                    StyleAttr::Hover(_) | StyleAttr::Active(_) => (),
                    any => any.apply(entity, &mut cmd, &mut style),
                });
            }
        },
    );
}

fn hotrealod(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<XNode>>,
    nodes: Query<(Entity, &Handle<XNode>, Option<&Parent>), Without<UnbuildTag>>,
) {
    events.read().for_each(|ev| {
        let AssetEvent::LoadedWithDependencies { id } = ev else {
            return;
        };

        nodes
            .iter()
            .filter(|(_, handle, _)| handle.id() == *id)
            .for_each(|(ent, handle, parent)| {
                cmd.entity(ent).despawn_recursive();
                let id = cmd.spawn((handle.clone(), UnbuildTag)).id();
                parent.map(|p| {
                    cmd.entity(p.get()).add_child(id);
                });
            });
    });
}

fn style_ui(
    mut cmd: Commands,
    mut unstyled: Query<(Entity, &mut Style, &StyleAttributes), With<UnStyled>>,
) {
    unstyled
        .iter_mut()
        .for_each(|(entity, mut style, style_attr)| {
            cmd.entity(entity).remove::<UnStyled>();
            style_attr.iter().for_each(|attr| match attr {
                StyleAttr::Hover(_) | StyleAttr::Active(_) => (),
                any => any.apply(entity, &mut cmd, &mut style),
            });
        });
}

fn build_ui(
    mut cmd: Commands,
    unbuild: Query<(Entity, &Handle<XNode>), With<UnbuildTag>>,
    assets: Res<Assets<XNode>>,
    server: Res<AssetServer>,
) {
    unbuild.iter().for_each(|(ent, handle)| {
        let Some(ui_node) = assets.get(handle) else {
            return;
        };
        build_recursive(ent, &ui_node, &mut cmd, &assets, &server);
        cmd.entity(ent).remove::<UnbuildTag>();
    });
}

fn build_recursive(
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
            // Some(&img.children)
        }
        XNode::Text(text) => {
            cmd.entity(entity).insert((
                Name::new("Text"),
                TextBundle::from_section(
                    &text.content,
                    TextStyle {
                        font_size: 44.,
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
                .insert((UnStyled, Name::new("Include"), handle, UnbuildTag));

            // if let Some(click_action) = click {
            //     cmd.entity(entity).insert(ClickAction(click_action.into()));
            // }

            Some(&inc.children)
        }
        _ => {
            return;
        }
    };

    children.map(|children| {
        children.iter().for_each(|child_node| {
            let child = cmd.spawn_empty().id();
            cmd.entity(entity).add_child(child);
            build_recursive(child, child_node, cmd, assets, server);
        });
    });
}

use bevy::prelude::*;

use crate::{node::NNode, prelude::StyleAttr};

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
    pub handle: Handle<NNode>,
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
    mut events: EventReader<AssetEvent<NNode>>,
    nodes: Query<(Entity, &Handle<NNode>, Option<&Parent>), Without<UnbuildTag>>,
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
    unbuild: Query<(Entity, &Handle<NNode>), With<UnbuildTag>>,
    assets: Res<Assets<NNode>>,
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
    node: &NNode,
    cmd: &mut Commands,
    assets: &Assets<NNode>,
    server: &AssetServer,
) {
    // build node
    let children = match &node {
        NNode::Div(div) => {
            cmd.entity(entity).insert((
                Name::new("Div"),
                NodeBundle::default(),
                StyleAttributes(div.styles.clone()),
                UnStyled,
            ));
            Some(&div.children)
        }
        NNode::Image(img) => {
            cmd.entity(entity).insert((
                Name::new("Image"),
                ImageBundle {
                    image: UiImage::new(server.load(&img.path)),
                    ..default()
                },
                StyleAttributes(img.styles.clone()),
                UnStyled,
            ));
            Some(&img.children)
        }
        NNode::Text(text) => {
            cmd.entity(entity).insert((
                Name::new("Text"),
                TextBundle::from_section(
                    &text.content,
                    TextStyle {
                        font_size: 20.,
                        ..default()
                    },
                ),
                StyleAttributes(text.styles.clone()),
                UnStyled,
            ));
            Some(&text.children)
        }
        NNode::Button(btn) => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                StyleAttributes(btn.styles.clone()),
                UnStyled,
                //ClickAction
            ));
            None
        }
        NNode::Include(inc) => {
            let handle = server.load::<NNode>(&inc.path);
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

fn build_style(cmd: &mut Commands, target: Entity, style_attributes: &Vec<StyleAttr>) {
    let mut style = Style::default();

    style_attributes.iter().for_each(|attr| match attr {
        StyleAttr::Display(display) => style.display = *display,
        StyleAttr::Position(position) => style.position_type = *position,
        StyleAttr::Overflow(overflow) => style.overflow = *overflow,
        StyleAttr::Direction(dir) => style.direction = *dir,
        StyleAttr::Left(left) => style.left = *left,
        StyleAttr::Right(right) => style.right = *right,
        StyleAttr::Top(top) => style.top = *top,
        StyleAttr::Bottom(bottom) => style.bottom = *bottom,
        StyleAttr::Width(width) => style.width = *width,
        StyleAttr::Height(height) => style.height = *height,
        StyleAttr::FlexDirection(dir) => style.flex_direction = *dir,
        StyleAttr::Margin(rect) => style.margin = *rect,
        StyleAttr::Padding(rect) => style.padding = *rect,
        StyleAttr::Background(color) => {
            cmd.entity(target).insert(BackgroundColor(*color));
        }
        _ => (),
        // StyleAttr::MinWidth(_) => todo!(),
        // StyleAttr::MinHeight(_) => todo!(),
        // StyleAttr::MaxWidth(_) => todo!(),
        // StyleAttr::MaxHeight(_) => todo!(),
        // StyleAttr::AspectRatio(_) => todo!(),
        // StyleAttr::AlignItems(_) => todo!(),
        // StyleAttr::JustifyItems(_) => todo!(),
        // StyleAttr::AlignSelf(_) => todo!(),
        // StyleAttr::JustifySelf(_) => todo!(),
        // StyleAttr::AlignContent(_) => todo!(),
        // StyleAttr::JustifyContent(_) => todo!(),
        // StyleAttr::Border(_) => todo!(),
        // StyleAttr::FlexWrap(_) => todo!(),
        // StyleAttr::FlexGrow(_) => todo!(),
        // StyleAttr::FlexShrink(_) => todo!(),
        // StyleAttr::FlexBasis(_) => todo!(),
        // StyleAttr::RowGap(_) => todo!(),
        // StyleAttr::ColumnGap(_) => todo!(),
        // StyleAttr::GridAutoFlow(_) => todo!(),
        // StyleAttr::GridTemplateRows(_) => todo!(),
        // StyleAttr::GridTemplateColumns(_) => todo!(),
        // StyleAttr::GridAutoRows(_) => todo!(),
        // StyleAttr::GridAutoColumns(_) => todo!(),
        // StyleAttr::GridRow(_) => todo!(),
        // StyleAttr::GridColumn(_) => todo!(),
        // StyleAttr::FontSize(_) => todo!(),
        // StyleAttr::Font(_) => todo!(),
        // StyleAttr::FontColor(_) => todo!(),
        // StyleAttr::Background(_) => todo!(),
        // StyleAttr::BorderRadius(_) => todo!(),
    });

    cmd.entity(target).insert(style);
}

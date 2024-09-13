use std::f32::consts::PI;

use bevy::{
    ecs::system::EntityCommands, input::mouse::MouseWheel, prelude::*, window::WindowResolution,
};
use lommix_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Effortless Ui".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            LommixUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_puls, update_collapse, update_scroll))
        .run();
}

fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut function_bindings: ResMut<FunctionBindings>,
    mut custom_comps: ResMut<ComponentBindings>,
) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn(UiBundle {
        handle: server.load("menu.html"),
        ..default()
    });

    function_bindings.register("start_game", cmd.register_one_shot_system(start_game));
    function_bindings.register("add_comp", cmd.register_one_shot_system(add_comp));
    function_bindings.register("inventory", cmd.register_one_shot_system(init_inventory));
    function_bindings.register("scrollable", cmd.register_one_shot_system(init_scrollable));

    let panel_handle = server.load("panel.html");
    custom_comps.register("panel", move |mut entity_cmd: EntityCommands| {
        entity_cmd.insert((UiBundle {
            handle: panel_handle.clone(),
            ..default()
        },));
    });

    function_bindings.register(
        "collapse",
        cmd.register_one_shot_system(|In(entity), mut cmd: Commands| {
            cmd.entity(entity).insert(Collapse(false));
        }),
    );
}

fn update_collapse(
    mut interactions: Query<(&Interaction, &UiTarget, &mut Collapse), Changed<Interaction>>,
    mut style: Query<&mut Style>,
) {
    interactions
        .iter_mut()
        .for_each(|(interaction, target, mut collapse)| {
            let Interaction::Pressed = interaction else {
                return;
            };

            info!("collapsing {}", target.0);
            let display = match **collapse {
                true => {
                    **collapse = false;
                    Display::None
                }
                false => {
                    **collapse = true;
                    Display::Flex
                }
            };

            if let Ok(mut style) = style.get_mut(**target) {
                info!("change style");
                style.display = display;
            }
        });
}

#[derive(Component)]
pub struct Scrollable {
    offset: f32,
    speed: f32,
}
fn init_scrollable(In(entity): In<Entity>, mut cmd: Commands, tags: Query<&Tags>) {
    let speed = tags
        .get(entity)
        .ok()
        .map(|tags| {
            tags.get_tag("scroll_speed")
                .map(|fstr| fstr.parse::<f32>().ok())
                .flatten()
        })
        .flatten()
        .unwrap_or(10.);

    cmd.entity(entity).insert(Scrollable { speed, offset: 0. });
}

fn update_scroll(
    mut events: EventReader<MouseWheel>,
    mut scrollables: Query<(&mut Scrollable, &UiTarget, &Interaction)>,
    mut targets: Query<(&mut Style, &Node)>,
    time: Res<Time>,
) {
    // whatever
    events.read().for_each(|ev| {
        scrollables
            .iter_mut()
            .for_each(|(mut scroll, target, interaction)| {
                // match interaction {
                //     Interaction::Hovered => (),
                //     _ => return,
                // };

                let Ok((mut style, node)) = targets.get_mut(**target) else {
                    return;
                };

                scroll.offset = (scroll.offset
                    + ev.y.signum() * scroll.speed * time.delta_seconds())
                .clamp(-node.size().y, 0.);

                style.top = Val::Px(scroll.offset);
            });
    });
}

#[derive(Component)]
pub struct Puls(f32);

fn update_puls(mut query: Query<(&mut Style, &Puls)>, time: Res<Time>, mut elapsed: Local<f32>) {
    *elapsed += time.delta_seconds();

    query.iter_mut().for_each(|(mut style, rotatethis)| {
        style.width = Val::Percent((*elapsed * rotatethis.0).sin() * 5. + 90.);
        style.height = Val::Percent((*elapsed * rotatethis.0).sin() * 5. + 90.);
    });
}

fn init_inventory(In(entity): In<Entity>, mut cmd: Commands, server: Res<AssetServer>) {
    info!("spawning items");
    cmd.entity(entity).with_children(|cmd| {
        for i in 0..300 {
            cmd.spawn((
                UiBundle {
                    handle: server.load("card.html"),
                    ..default()
                },
                NodeBundle::default(),
                PropertyDefintions::new()
                    .with("title", format!("item {i}"))
                    .with("bordercolor", if i % 2 == 0 { "#FFF" } else { "#F88" }),
            ));
        }
    });
}

fn start_game(_: In<Entity>, mut _cmd: Commands) {
    info!("hello world from start game system");
}

fn add_comp(In(entity): In<Entity>, mut cmd: Commands, tags: Query<&Tags>) {
    let Ok(tags) = tags.get(entity) else {
        warn!("missing args");
        return;
    };

    match tags.get_tag("comp") {
        Some("collapse") => {
            cmd.entity(entity).insert(Collapse::default());
        }
        _ => warn!("missing `arg`"),
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Collapse(pub bool);

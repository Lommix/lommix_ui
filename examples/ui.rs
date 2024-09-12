use std::f32::consts::PI;

use bevy::{ecs::system::EntityCommands, prelude::*, window::WindowResolution};
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
        .add_systems(Update, (update_puls, update_collapse))
        .run();
}

fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut function_bindings: ResMut<FunctionBindings>,
    mut custom_comps: ResMut<ComponenRegistry>,
) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn(UiBundle {
        handle: server.load("menu.html"),
        ..default()
    });

    function_bindings.register("delete_me", cmd.register_one_shot_system(delete_me));
    function_bindings.register("start_game", cmd.register_one_shot_system(start_game));
    function_bindings.register(
        "add_comp_collapse",
        cmd.register_one_shot_system(add_comp_collapse),
    );

    let panel_handle = server.load("panel.html");
    custom_comps.register("panel", move |mut entity_cmd: EntityCommands| {
        info!("spawning custom node!");
        entity_cmd.insert((UiBundle {
            handle: panel_handle.clone(),
            ..default()
        },));
    });
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
                style.display = display;
            }
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

fn delete_me(callback: In<Callback>, mut cmd: Commands) {
    info!("hehe I delete {}", callback.entity);
    cmd.entity(callback.entity).despawn_recursive();
}

fn start_game(_entity: In<Callback>, mut _cmd: Commands) {
    info!("hello world from start game system");
}

fn add_comp_collapse(callback: In<Callback>, mut cmd: Commands) {
    info!("added collapse comp");
    cmd.entity(callback.entity).insert(Collapse::default());
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Collapse(pub bool);

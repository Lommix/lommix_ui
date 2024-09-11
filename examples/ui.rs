use std::f32::consts::PI;

use bevy::{prelude::*, window::WindowResolution};
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
        .add_systems(Update, update_puls)
        .run();
}

fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut function_bindings: ResMut<FunctionBindings>,
    mut spawn_bindings: ResMut<SpawnBindings>,
) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn(UiBundle {
        handle: server.load("menu.html"),
        ..default()
    });

    function_bindings.register("delete_me", cmd.register_one_shot_system(delete_me));
    function_bindings.register("start_game", cmd.register_one_shot_system(start_game));

    spawn_bindings.register("puls", &|mut entity_cmd| {
        entity_cmd.insert(Puls(20.));
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

fn delete_me(entity: In<Entity>, mut cmd: Commands) {
    info!("hehe I delete {}", *entity);
    cmd.entity(*entity).despawn_recursive();
}

fn start_game(entity: In<Entity>, mut cmd: Commands) {
    info!("hello world from start game system");
}

fn add_collapse(entity: In<Entity>, mut cmd: Commands) {
    info!("hello world from start game system");
}

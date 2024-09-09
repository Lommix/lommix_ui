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
        .run();
}

fn setup(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn(UiBundle {
        handle: server.load("menu.html"),
        ..default()
    });
}

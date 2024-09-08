use bevy::prelude::*;
use lommix_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            // bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            LommixUiPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default());

    cmd.spawn(UiBundle {
        handle: server.load("demo_ui.html"),
        ..default()
    });

    // let root = cmd.spawn(NodeBundle::default()).id();
    // let ui = server.load::<UiNode>("demo_ui.ron");
    // cmd.spawn_empty().insert(ui);
    // cmd.select_ui(root).spawn(NodeBundle::default(), |n| {
    //     n.spawn(ButtonBundle::default(), |n| {});
    // });
}

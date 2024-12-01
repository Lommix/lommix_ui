use bevy::prelude::*;
use bevy_hui::prelude::*;
use bevy_hui_widgets::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            HuiPlugin,
            HuiSliderWidgetPlugin,
            HuiInputWidgetPlugin,
        ))
        .add_systems(Startup, (register_widgets, setup_scene).chain())
        .run();
}

fn register_widgets(mut html_comps: HtmlComponents, server: Res<AssetServer>) {
    html_comps.register("slider", server.load("slider.html"));
    html_comps.register("input", server.load("input.html"));
}

fn setup_scene(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    cmd.spawn(HtmlNode(server.load("widgets_demo.html")));
}

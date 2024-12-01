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
            HuiSelectWidgetPlugin,
        ))
        .add_systems(Startup, (register_widgets, setup_scene).chain())
        .add_systems(Update, update_slider_target_text)
        .run();
}

fn register_widgets(mut html_comps: HtmlComponents, server: Res<AssetServer>) {
    html_comps.register("vslider", server.load("vertical_slider.html"));
    html_comps.register("hslider", server.load("horizontal_slider.html"));
    html_comps.register("input", server.load("input.html"));
    html_comps.register("select", server.load("select.html"));
    html_comps.register("option", server.load("option.html"));
}

fn setup_scene(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    cmd.spawn(HtmlNode(server.load("widgets_demo.html")));
}

// -----------------
// example, custom user extension, update a value display of a slider
fn update_slider_target_text(
    mut events: EventReader<SliderChangedEvent>,
    targets: Query<&UiTarget>,
    mut texts: Query<&mut Text>,
) {
    for event in events.read() {
        let Ok(target) = targets.get(event.slider) else {
            continue;
        };

        let Ok(mut text) = texts.get_mut(**target) else {
            continue;
        };

        text.0 = format!("{:.2}", event.value);
    }
}

use bevy::prelude::*;
use bevy_hui::prelude::*;
use bevy_hui_widgets::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin {
                default_sampler: bevy::image::ImageSamplerDescriptor::nearest(),
            }),
            HuiPlugin,
            HuiSliderWidgetPlugin,
            HuiInputWidgetPlugin,
            HuiSelectWidgetPlugin,
        ))
        .add_systems(Startup, (register_widgets, setup_scene, register_user_functions))
        .add_systems(Update, update_slider_target_text)
        .run();
}

fn register_widgets(mut html_comps: HtmlComponents, server: Res<AssetServer>) {
    html_comps.register("vslider", server.load("widgets/vertical_slider.html"));
    html_comps.register("hslider", server.load("widgets/horizontal_slider.html"));
    html_comps.register("input", server.load("widgets/input.html"));
    html_comps.register("select", server.load("widgets/select.html"));
    html_comps.register("option", server.load("widgets/option.html"));
}

fn setup_scene(mut cmd: Commands, server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    cmd.spawn(HtmlNode(server.load("widgets/widgets_demo.html")));
}

/// If you trigger the [bevy_hui::prelude::UiChangedEvent] in your widget
/// code, you can bind custom functions in html to this event.
fn register_user_functions(mut html_funcs: HtmlFunctions) {
    html_funcs.register(
        "notify_slider_change",
        |In(entity), sliders: Query<&Slider>| {
            let Ok(slider) = sliders.get(entity) else {
                return;
            };
            info!("Slider {entity} changed, new value: {:.2}", slider.value);
        },
    );

    html_funcs.register(
        "notify_input_change",
        |In(entity), sliders: Query<&TextInput>| {
            let Ok(input) = sliders.get(entity) else {
                return;
            };
            info!("Input {entity} changed, new value: `{}`", input.value);
        },
    );

    html_funcs.register(
        "notify_select_change",
        |In(entity), sliders: Query<&SelectInput>| {
            let Ok(select) = sliders.get(entity) else {
                return;
            };
            info!("Select {entity} changed, new value: {:?}", select.value);
        },
    );
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

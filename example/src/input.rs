use bevy::{
    ecs::event::EventCursor, input::{
        keyboard::{Key, KeyboardInput},
        mouse::MouseButtonInput,
    }, prelude::*
};
use bevy_html_ui::prelude::*;

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HtmlUiPlugin::default()))
        .add_plugins(TextInputPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, write_input)
        .run();
}

fn setup_scene(mut cmd: Commands, mut functions: HtmlFunctions, server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    cmd.spawn(HtmlNode(server.load("textinput/menu.xml")));
    functions.register("submit", on_submit);
}

// Example a simple submit form

fn on_submit(
    In(entity): In<Entity>,
    inputs: Query<(&TextInput, &UiId)>,
    targets: Query<&UiTarget>,
    children: Query<&Children>,
) {
    info!("submit");

    let Ok(form_target) = targets.get(entity) else {
        warn!("no parent of inputs nodes as target provided for `{entity}`");
        return;
    };

    let Ok(children) = children.get(**form_target) else {
        warn!("form has no children");
        return;
    };

    for child in children.iter() {
        let Ok((text, id)) = inputs.get(*child) else {
            continue;
        };
        info!("Submitted input: `{}` form: `{}`", text.0, **id);
    }
}

// ----------------------------------------------
// Example Text Input Component Plugin

pub struct TextInputPlugin;
impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (focus /* , write_input */, sync_display_text, unfocus),
        );
    }
}

#[derive(Component, Default)]
pub struct TextInput(pub String);

#[derive(Component)]
pub struct Focused;

fn setup(mut components: HtmlComponents, server: Res<AssetServer>) {
    let handle = server.load("textinput/textinput.xml");

    components.register_with_spawn_fn("input", handle, |mut cmd| {
        cmd.insert((
            // Directly adding the Textinput-Component, because the first node is the button holding
            // the input state. This saves us a `onspawn` binding
            TextInput::default(),
        ));
    });
}

fn focus(
    mut cmd: Commands,
    interations: Query<(Entity, &Interaction), (With<TextInput>, Changed<Interaction>)>,
) {
    interations
        .iter()
        .for_each(|(entity, interaction)| match interaction {
            Interaction::Pressed => {
                // add focus state
                cmd.entity(entity).insert((Focused, UiActive));
            }
            Interaction::Hovered => (),
            Interaction::None => (),
        });
}

fn unfocus(
    mut cmd: Commands,
    mut mouse_events: EventReader<MouseButtonInput>,
    interations: Query<(Entity, &Interaction), With<Focused>>,
) {
    mouse_events.read().for_each(|ev| {
        match ev.button {
            MouseButton::Left | MouseButton::Middle | MouseButton::Right => (),
            _ => {
                return;
            }
        };

        if !ev.state.is_pressed() {
            return;
        }

        interations.iter().for_each(|(entity, interaction)| {
            if let Interaction::Pressed = interaction {
                return;
            }
            cmd.entity(entity).remove::<UiActive>().remove::<Focused>();
        });
    });
}

fn write_input(
    mut cmd: Commands,
    key_events: Res<Events<KeyboardInput>>,
    mut reader: Local<EventCursor<KeyboardInput>>,
    mut text_inputs: Query<(Entity, &mut TextInput), With<Focused>>,
) {
    if text_inputs.is_empty() {
        return;
    }

    if reader.clone().read(&key_events).next().is_none() {
        return;
    }

    for input in reader.clone().read(&key_events) {
        if !input.state.is_pressed() {
            return;
        }

        match input.logical_key {
            Key::Character(ref char) => {
                text_inputs
                    .iter_mut()
                    .for_each(|(_, mut txt)| txt.0.push_str(char));
            }
            Key::Enter => {
                text_inputs.iter().for_each(|(ent, _)| {
                    cmd.entity(ent).remove::<UiActive>().remove::<Focused>();
                });
            }
            Key::Backspace => {
                text_inputs.iter_mut().for_each(|(_, mut txt)| {
                    _ = txt.0.pop();
                });
            }
            Key::Space => {
                text_inputs
                    .iter_mut()
                    .for_each(|(_, mut txt)| txt.0.push_str(" "));
            }
            _ => (),
        }
    }

    reader.clear(&key_events);
}

fn sync_display_text(
    text_inputs: Query<(&TextInput, &UiTarget), Changed<TextInput>>,
    mut texts: Query<&mut Text>,
) {
    text_inputs.iter().for_each(|(input, target)| {
        _ = texts.get_mut(**target).map(|mut txt| {
            **txt = input.0.clone();
        });
    });
}

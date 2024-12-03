use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::MouseButtonInput,
    },
    prelude::*,
};
use bevy_hui::prelude::*;

/// # Input Component Plugin
///
///
/// ## defintion:
///
/// A input node is just a button with a text child.
/// follow these instructions to make it work for your
/// template.
///
/// ## instruction:
///
/// -   add the `HuiInputWidgetPlugin`
/// -   create a template/custom component.
/// -   attach the init_input function to the root node
/// -   use optional `tag:filter="text/number"` (similar to `type` in html)
/// -   define a `target` on the root node. The target can be any Text node
///     in the template. This will display the current value
/// -   the input consumes any key events, when `UiActive` is attached.
///     you can use conditional styles with `active:border_color="..`
///
/// ## Minimal template example:
///
/// ```html
/// <template>
///     <button
///         on_spawn="init_input"
///         target="text_value"
/// 		tag:filter="text"
///         background="#222"
///         border="2px"
/// 		min_height="40px"
///         border_color="#555"
///         padding="5px"
///         active:border_color="#FFF"
///     >
///         <text id="text_value" font_size="20">Placeholder</text>
///     </button>
/// </template>
/// ```
pub struct HuiInputWidgetPlugin;
impl Plugin for HuiInputWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TextInput>();
        app.register_type::<TextFilter>();
        app.add_systems(
            Update,
            (
                unfocus,
                focus.after(unfocus),
                write_input,
                sync_text_preview,
            ),
        );
        app.add_systems(Startup, setup);
    }
}

pub const TAG_FILTER: &'static str = "filter";

#[derive(Component, Default, Reflect)]
#[reflect]
pub struct TextInput {
    pub value: String,
    pub filter: TextFilter,
}

#[derive(Default, Reflect)]
#[reflect]
pub enum TextFilter {
    #[default]
    None,
    Text,
    Number,
}

impl TextFilter {
    pub fn apply(&self, c: &char) -> bool {
        match self {
            TextFilter::None => true,
            TextFilter::Text => c.is_ascii_punctuation() || c.is_alphabetic(),
            TextFilter::Number => c.is_numeric() || c == &'.',
        }
    }
}

impl From<&str> for TextFilter {
    fn from(value: &str) -> Self {
        match value {
            "text" => TextFilter::Text,
            "number" => TextFilter::Number,
            _ => TextFilter::None,
        }
    }
}

fn setup(mut html_funcs: HtmlFunctions) {
    html_funcs.register(
        "init_input",
        |In(entity), mut cmd: Commands, tags: Query<&Tags>| {
            let filter = tags
                .get(entity)
                .ok()
                .map(|tags| {
                    tags.get(TAG_FILTER)
                        .map(|str_val| TextFilter::from(str_val.as_str()))
                })
                .flatten()
                .unwrap_or_default();

            cmd.entity(entity).insert(TextInput {
                value: default(),
                filter,
            });
        },
    );
}

fn focus(
    mut cmd: Commands,
    text_inputs: Query<(Entity, &Interaction), (With<TextInput>, Without<UiActive>)>,
) {
    for (entity, interaction) in text_inputs.iter() {
        if matches!(interaction, Interaction::Pressed) {
            cmd.entity(entity).insert(UiActive);
        }
    }
}

fn unfocus(
    mut cmd: Commands,
    text_inputs: Query<Entity, (With<TextInput>, With<UiActive>)>,
    mut mouse_events: EventReader<MouseButtonInput>,
) {
    for event in mouse_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        for entity in text_inputs.iter() {
            cmd.entity(entity).remove::<UiActive>();
        }
    }
}

fn write_input(
    mut cmd: Commands,
    mut events: EventReader<KeyboardInput>,
    mut text_inputs: Query<(Entity, &mut TextInput), With<UiActive>>,
) {
    if text_inputs.is_empty() {
        return;
    }

    for input in events.read() {
        if !input.state.is_pressed() {
            continue;
        }

        match input.logical_key {
            Key::Character(ref char) => {
                text_inputs.iter_mut().for_each(|(_, mut txt)| {
                    for c in char.chars() {
                        if txt.filter.apply(&c) {
                            txt.value.push(c);
                        }
                    }
                });
            }
            Key::Enter => {
                text_inputs.iter().for_each(|(ent, _)| {
                    cmd.entity(ent).remove::<UiActive>().remove::<UiActive>();
                });
            }
            Key::Backspace => {
                text_inputs.iter_mut().for_each(|(_, mut txt)| {
                    _ = txt.value.pop();
                });
            }
            Key::Space => {
                text_inputs
                    .iter_mut()
                    .for_each(|(_, mut txt)| txt.value.push(' '));
            }
            _ => (),
        }
    }
}

fn sync_text_preview(
    mut cmd: Commands,
    inputs: Query<(Entity, &TextInput, &UiTarget), Changed<TextInput>>,
    mut texts: Query<&mut Text>,
) {
    for (entity, text_input, target) in inputs.iter() {
        _ = texts.get_mut(**target).map(|mut text| {
            text.0.clone_from(&text_input.value);
        });

        cmd.trigger_targets(UiChangedEvent, entity);
    }
}

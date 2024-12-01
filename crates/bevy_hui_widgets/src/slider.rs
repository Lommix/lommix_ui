use bevy::prelude::*;
use bevy_hui::prelude::*;

/// # Slider Component Plugin
/// A slider is a node of certain size
/// with an absolute button as child. The button
/// can be dragged on a fixed axis.
///
/// Instructions:
/// -   add the `HuiSliderWidgetPlugin`
/// -   create a template/custom component.
/// -   attach the init_slider function to the root node
/// -   the root node holds the `Slider` component with the
///     current value. It must own an absolute button child.
/// -   when using your slider component, attach another `on_spawn` fn to
///     mark it for your intended use. example: `MasterVolumeSlider`
///
/// minimal template example:
///
/// ```html
///<template>
///    <node
///        on_spawn="init_slider"
///        tag:axis="x"
///        width="255px"
///        height="20px"
///		background="#000"
///    >
///        <button
///            background="#00F"
///            position="absolute"
///            width="20px"
///            height="20px"
///        ></button>
///    </node>
///</template>
/// ```
pub struct HuiSliderWidgetPlugin;
impl Plugin for HuiSliderWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SliderAxis>();
        app.register_type::<Slider>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_drag);
    }
}

pub const TAG_AXIS: &'static str = "axis";

#[derive(Default, Reflect)]
#[reflect]
pub enum SliderAxis {
    #[default]
    Horizontal,
    Vertical,
}

impl From<&str> for SliderAxis {
    fn from(value: &str) -> Self {
        match value {
            "y" => SliderAxis::Vertical,
            _ => SliderAxis::Horizontal,
        }
    }
}

/// Slider Component holds the current value
#[derive(Component, Reflect)]
#[reflect]
pub struct Slider {
    value: f32,
    axis: SliderAxis,
}

/// Slider Nob, which represent the button
#[derive(Component, Reflect)]
#[reflect]
pub struct SliderNob {
    slider: Entity,
}

impl Slider {
    pub fn value(&self) -> f32 {
        self.value
    }
}

fn setup(mut html_funcs: HtmlFunctions) {
    html_funcs.register("init_slider", init_slider);
}

fn init_slider(
    In(entity): In<Entity>,
    children: Query<&Children>,
    tags: Query<&Tags>,
    buttons: Query<(), With<Button>>,
    mut cmd: Commands,
) {
    let Some(nob_entity) = children
        .get(entity)
        .ok()
        .map(|children| {
            children
                .iter()
                .find_map(|child| buttons.get(*child).ok().map(|_| *child))
        })
        .flatten()
    else {
        error!("Your slider needs to have a button child, which will be the draggable nob");
        return;
    };

    let axis = tags
        .get(entity)
        .ok()
        .map(|tags| {
            tags.get(TAG_AXIS)
                .map(|str_val| SliderAxis::from(str_val.as_str()))
        })
        .flatten()
        .unwrap_or_default();

    cmd.entity(entity).insert(Slider { value: 0., axis });
    cmd.entity(nob_entity).insert(SliderNob { slider: entity });
}

fn update_drag(
    mut events: EventReader<bevy::input::mouse::MouseMotion>,
    mut nobs: Query<(Entity, &SliderNob, &mut HtmlStyle, &Interaction)>,
    mut sliders: Query<&mut Slider>,
    computed_nodes: Query<&ComputedNode>,
) {
    for event in events.read() {
        let delta_x = event.delta.x;
        nobs.iter_mut()
            .filter(|(_, _, _, interaction)| matches!(interaction, Interaction::Pressed))
            .for_each(|(nob_entity, nob, mut style, _)| {
                let Ok(slider_computed) = computed_nodes.get(nob.slider) else {
                    return;
                };

                let Ok(nob_computed) = computed_nodes.get(nob_entity) else {
                    return;
                };

                let current_pos = match style.computed.node.left {
                    Val::Px(pos) => pos,
                    _ => 0.,
                };

                let max_pos = slider_computed.unrounded_size().x
                    * slider_computed.inverse_scale_factor()
                    - nob_computed.unrounded_size().x * nob_computed.inverse_scale_factor();

                let next_pos = (current_pos + delta_x).min(max_pos).max(0.);
                style.computed.node.left = Val::Px(next_pos);

                let slider_value = next_pos / max_pos;

                _ = sliders.get_mut(nob.slider).map(|mut slider| {
                    slider.value = slider_value;
                });
            });
    }
}

use bevy::prelude::*;
use bevy_html_ui::prelude::*;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin {
                default_sampler: bevy::render::texture::ImageSamplerDescriptor::nearest(),
            }),
            HtmlUiPlugin::default(),
        ))
        .add_plugins(SliderPlugin)
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut cmd: Commands, server: Res<AssetServer>) {
    // --
    cmd.spawn(Camera2d::default());
    cmd.spawn((
        HtmlBundle {
            html: HtmlNode(server.load("slider/menu.xml")),
            ..default()
        },
        Slider(0.),
    ));
}

pub struct SliderPlugin;
impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_slider);
        app.add_systems(Update, (update_state, update_drag));
    }
}

fn setup_slider(
    mut components: ResMut<ComponentBindings>,
    mut html_functions: HtmlFunctions,
    server: Res<AssetServer>,
) {
    let handle = server.load("slider/slider.xml");
    components.register("slider", move |mut cmd| {
        cmd.insert((
            HtmlBundle {
                html: HtmlNode(handle.clone()),
                ..default()
            },
            // add our state to the root, so way may easly have access in other templates logic by
            // just adding a `id` tag to the custom component.
            Slider(0.),
        ));
    });

    html_functions.register("init_slider_btn", |In(entity), mut cmd: Commands| {
        cmd.entity(entity).insert(SliderNob::Rested);
    });
}

#[derive(Component)]
pub struct Slider(f32);

#[derive(Component)]
pub enum SliderNob {
    Dragged,
    Rested,
}
fn update_state(mut sliders: Query<(&Interaction, &mut SliderNob), Changed<Interaction>>) {
    sliders.iter_mut().for_each(|(interaction, mut state)| {
        // --
        match interaction {
            Interaction::Pressed => {
                *state = SliderNob::Dragged;
            }
            _ => {
                *state = SliderNob::Rested;
            }
        }
    });
}

fn update_drag(
    mut events: EventReader<bevy::input::mouse::MouseMotion>,
    mut sliders: Query<(&mut Slider, &TemplateState)>,
    mut nobs: Query<(&mut HtmlStyle, &Children, &UiTarget, &SliderNob)>,
    mut text: Query<&mut Text>,
) {
    events.read().for_each(|event| {
        nobs.iter_mut()
            .for_each(|(mut node_style, children, target, nob)| {
                let SliderNob::Dragged = nob else {
                    return;
                };

                // @todo: add width property
                let Ok((mut slide, _state)) = sliders.get_mut(**target) else {
                    return;
                };

                slide.0 = (slide.0 + event.delta.x / 200.).clamp(0., 1.);
                node_style.computed.node.left = Val::Px(slide.0 * 200.);

                // update nob text
                // we cheat here, because we know by design, that the text is
                // part of the first child of our slider nob.

                children.first().map(|child| {
                    _ = text.get_mut(*child).map(|mut txt| {
                        **txt = format!("{:.0}", slide.0 * 100.);
                    });
                });
            });
    });
}

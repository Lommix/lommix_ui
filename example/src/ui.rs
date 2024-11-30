use bevy::{image::ImageSamplerDescriptor, input::mouse::MouseWheel, prelude::*};
use bevy_aseprite_ultra::prelude::*;
use bevy_hui::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor::nearest(),
            }),
            AsepriteUltraPlugin,
            HuiPlugin,
            HuiAutoLoadPlugin::new(&["components"]),
        ))
        .add_systems(OnEnter(AutoLoadState::Finished), setup)
        .add_systems(
            Update,
            (update_puls, update_collapse, update_scroll, cleaner),
        )
        .run();
}

fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,

    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
) {
    cmd.spawn(Camera2d);
    cmd.spawn((
        HtmlNode(server.load("demo/menu.html")),
        TemplateProperties::default().with("title", "Test-title"),
    ));

    // register function bindings
    html_funcs.register("greet", greet);
    html_funcs.register("inventory", init_inventory);
    html_funcs.register("scrollable", init_scrollable);
    html_funcs.register("play_beep", play_beep);
    html_funcs.register("collapse", |In(entity), mut cmd: Commands| {
        cmd.entity(entity).insert(Collapse(true));
    });

    html_funcs.register(
        "attach_aseprite",
        |In(entity), tags: Query<&Tags>, mut cmd: Commands, server: Res<AssetServer>| {
            let Ok(tags) = tags.get(entity) else {
                return;
            };

            let Some(ase_path) = tags.get("source") else {
                warn!("missing `source` for aseprite component {entity}");
                return;
            };

            let animation = tags
                .get("animation")
                .map(|s| Animation::tag(s))
                .unwrap_or(Animation::default());

            cmd.entity(entity).insert(AseUiAnimation {
                aseprite: server.load(ase_path),
                animation,
            });
        },
    );

    // register custom node by passing a template handle
    html_comps.register("panel", server.load("demo/panel.html"));
    html_comps.register("aseprite", server.load("demo/aseprite.html"));

    // a function that updates a property and triggers a recompile
    html_funcs.register(
        "debug",
        |In(entity),
         mut cmd: Commands,
         mut template_props: Query<&mut TemplateProperties>,
         scopes: Query<&TemplateScope>| {
            let Ok(scope) = scopes.get(entity) else {
                return;
            };

            let Ok(mut props) = template_props.get_mut(**scope) else {
                return;
            };

            let rng = rand::random::<u32>();
            props.insert("title".to_string(), format!("{}", rng));
            cmd.trigger_targets(CompileContextEvent, **scope);
        },
    );
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Collapse(pub bool);

fn update_collapse(
    mut interactions: Query<(&Interaction, &UiTarget, &mut Collapse), Changed<Interaction>>,
    mut style: Query<&mut HtmlStyle>,
) {
    interactions
        .iter_mut()
        .for_each(|(interaction, target, mut collapse)| {
            let Interaction::Pressed = interaction else {
                return;
            };

            let display = match **collapse {
                true => {
                    **collapse = false;
                    Display::None
                }
                false => {
                    **collapse = true;
                    Display::Flex
                }
            };

            if let Ok(mut style) = style.get_mut(**target) {
                style.computed.node.display = display;
            }
        });
}

#[derive(Component)]
pub struct Scrollable {
    offset: f32,
    speed: f32,
}

fn init_scrollable(In(entity): In<Entity>, mut cmd: Commands, tags: Query<&Tags>) {
    let speed = tags
        .get(entity)
        .ok()
        .and_then(|tags| {
            tags.get("scroll_speed")
                .and_then(|fstr| fstr.parse::<f32>().ok())
        })
        .unwrap_or(10.);

    cmd.entity(entity).insert(Scrollable { speed, offset: 0. });
}

fn update_scroll(
    mut events: EventReader<MouseWheel>,
    mut scrollables: Query<(&mut Scrollable, &UiTarget, &Interaction)>,
    mut targets: Query<&mut HtmlStyle>,
    time: Res<Time>,
) {
    // whatever
    events.read().for_each(|ev| {
        scrollables.iter_mut().for_each(|(mut scroll, target, _)| {
            let Ok(mut style) = targets.get_mut(**target) else {
                return;
            };

            scroll.offset = scroll.offset + ev.y.signum() * scroll.speed * time.delta_secs();
            style.computed.node.top = Val::Px(scroll.offset);
        });
    });
}

#[derive(Component)]
pub struct Puls(f32);

fn update_puls(mut query: Query<(&mut Node, &Puls)>, time: Res<Time>, mut elapsed: Local<f32>) {
    *elapsed += time.delta_secs();

    query.iter_mut().for_each(|(mut style, rotatethis)| {
        style.width = Val::Percent((*elapsed * rotatethis.0).sin() * 5. + 90.);
        style.height = Val::Percent((*elapsed * rotatethis.0).sin() * 5. + 90.);
    });
}

fn init_inventory(In(entity): In<Entity>, mut cmd: Commands, server: Res<AssetServer>) {
    cmd.entity(entity).with_children(|cmd| {
        for i in 0..200 {
            cmd.spawn((
                HtmlNode(server.load("demo/card.html")),
                TemplateProperties::default()
                    .with("title", &format!("item {i}"))
                    .with("bordercolor", if i % 2 == 0 { "#FFF" } else { "#F88" }),
            ));
        }
    });
}

fn play_beep(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmd: Commands,
    server: Res<AssetServer>,
) {
    let Some(path) = tags
        .get(entity)
        .ok()
        .and_then(|t| t.get("source").map(|s| s.to_string()))
    else {
        return;
    };

    let beep: Handle<AudioSource> = server.load(&path);
    cmd.spawn((
        AudioPlayer(beep),
        PlaybackSettings::ONCE,
        LifeTime::new(0.5),
    ));
}

#[derive(Component, Deref, DerefMut)]
struct LifeTime(Timer);
impl LifeTime {
    pub fn new(s: f32) -> Self {
        LifeTime(Timer::new(
            std::time::Duration::from_secs_f32(s),
            TimerMode::Once,
        ))
    }
}

fn cleaner(mut expired: Query<(Entity, &mut LifeTime)>, mut cmd: Commands, time: Res<Time>) {
    expired.iter_mut().for_each(|(entity, mut lifetime)| {
        if lifetime.tick(time.delta()).finished() {
            cmd.entity(entity).despawn_recursive();
        }
    });
}

fn greet(In(entity): In<Entity>, mut _cmd: Commands) {
    info!("greetings from `{entity}`");
}

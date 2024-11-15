use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_html_ui::prelude::*;

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
            HtmlUiPlugin,
            HtmlAutoLoadPlugin::new(&["components"]),
            // bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
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

    cmd.spawn(HtmlBundle {
        html: HtmlNode(server.load("demo/menu.xml")),
        ..default()
    });

    html_funcs.register("greet", greet);
    html_funcs.register("inventory", init_inventory);
    html_funcs.register("scrollable", init_scrollable);
    html_funcs.register("play_beep", play_beep);

    // register custom node by hand
    let panel_handle: Handle<HtmlTemplate> = server.load("demo/panel.xml");
    html_comps.register("panel", panel_handle);

    html_funcs.register("collapse", |In(entity), mut cmd: Commands| {
        cmd.entity(entity).insert(Collapse(true));
    });

    html_funcs.register(
        "debug",
        |In(entity),
         mut cmd: Commands,
         mut state: Query<&mut TemplateState>,
         scopes: Query<&ScopeEntity>| {
            let Ok(scope) = scopes.get(entity) else {
                return;
            };

            let Ok(mut state) = state.get_mut(**scope) else {
                return;
            };

            let rng = rand::random::<u32>();
            state.set_prop("title", format!("{}", rng));
            cmd.trigger_targets(CompileContextEvent, **scope);
        },
    );
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Collapse(pub bool);

fn update_collapse(
    mut interactions: Query<(&Interaction, &UiTarget, &mut Collapse), Changed<Interaction>>,
    mut style: Query<&mut NodeStyle>,
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
                style.regular.style.display = display;
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
            tags.get_tag("scroll_speed")
                .and_then(|fstr| fstr.parse::<f32>().ok())
        })
        .unwrap_or(10.);

    cmd.entity(entity).insert(Scrollable { speed, offset: 0. });
}

fn update_scroll(
    mut events: EventReader<MouseWheel>,
    mut scrollables: Query<(&mut Scrollable, &UiTarget, &Interaction)>,
    mut targets: Query<&mut NodeStyle>,
    time: Res<Time>,
) {
    // whatever
    events.read().for_each(|ev| {
        scrollables.iter_mut().for_each(|(mut scroll, target, _)| {
            let Ok(mut style) = targets.get_mut(**target) else {
                return;
            };

            scroll.offset = scroll.offset + ev.y.signum() * scroll.speed * time.delta_secs();
            style.regular.style.top = Val::Px(scroll.offset);
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
                HtmlBundle {
                    html: HtmlNode(server.load("demo/card.xml")),
                    ..default()
                },
                TemplateState::new()
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
        .and_then(|t| t.get_tag("source").map(|s| s.to_string()))
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

use bevy::{
    ecs::system::{EntityCommands, SystemId},
    prelude::*,
    utils::HashMap,
};

pub struct BindingPlugin;
impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FunctionBindings>();
        app.init_resource::<ComponentBindings>();
        app.add_systems(Update, (observe_interactions, observe_on_spawn));
    }
}

pub type SpawnFunction = dyn Fn(EntityCommands) + Send + Sync + 'static;

/// # Register custom node tags
///
/// then use in your templats `<my_comp></my_comp>`
/// `
/// ComponenRegistry.register("my_comp", &|mut cmd: EntityCommands| cmd.insert(MyBundle::default()))
/// `
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ComponentBindings(HashMap<String, Box<SpawnFunction>>);

impl ComponentBindings {
    pub fn register<F>(&mut self, key: impl Into<String>, f: F)
    where
        F: Fn(EntityCommands) + Send + Sync + 'static,
    {
        let key: String = key.into();
        self.insert(key, Box::new(f));
    }

    pub fn try_spawn(&self, key: &String, entity: Entity, cmd: &mut Commands) {
        self.get(key)
            .map(|f| {
                let cmd = cmd.entity(entity);
                f(cmd);
            })
            .unwrap_or_else(|| warn!("custom tag `{key}` is not bound"));
    }
}

/// # Function binding resource
///
/// maps an oneshot system to a callable action, passing the Entity the action is
/// bound to.
///
/// in templates: `click="start_game"`
///
/// backend:
///
/// `
/// let system_id = app.register_system(|entity: In<Entity>| {})
/// FunctionBindings.register("start_game", system_id);
/// `
#[derive(Resource, Default, Deref, DerefMut)]
pub struct FunctionBindings(HashMap<String, SystemId<Entity>>);

impl FunctionBindings {
    pub fn register(&mut self, key: impl Into<String>, system_id: SystemId<Entity>) {
        let key: String = key.into();
        self.insert(key, system_id);
    }

    pub fn maybe_run(&self, key: &String, entity: Entity, cmd: &mut Commands) {
        self.get(key)
            .map(|id| {
                cmd.run_system_with_input(*id, entity);
            })
            .unwrap_or_else(|| warn!("function `{key}` is not bound"));
    }
}

fn observe_on_spawn(
    mut cmd: Commands,
    function_bindings: Res<FunctionBindings>,
    on_spawn: Query<(Entity, &crate::prelude::OnSpawn)>,
) {
    on_spawn.iter().for_each(|(entity, on_spawn)| {
        for spawn_fn in on_spawn.iter() {
            function_bindings.maybe_run(spawn_fn, entity, &mut cmd);
        }

        cmd.entity(entity).remove::<crate::prelude::OnSpawn>();
    });
}

#[rustfmt::skip]
fn observe_interactions(
    mut cmd: Commands,
    interactions: Query<(Entity, &Interaction), Changed<Interaction>>,
    function_bindings: Res<FunctionBindings>,
    on_pressed : Query<&crate::prelude::OnPress>,
    on_enter : Query<&crate::prelude::OnEnter>,
    on_exit : Query<&crate::prelude::OnExit>,
){
    interactions.iter().for_each(|(entity, interaction)|{
        match interaction {
            Interaction::Pressed => {
                if let Ok(crate::prelude::OnPress(funcs)) = on_pressed.get(entity){
                    for fn_str in funcs.iter(){
                        function_bindings.maybe_run(fn_str, entity, &mut cmd);
                    }
                }
            }
            Interaction::Hovered => {
                if let Ok(crate::prelude::OnEnter(funcs)) = on_enter.get(entity){
                    for fn_str in funcs.iter(){
                        function_bindings.maybe_run(fn_str, entity, &mut cmd);
                    }
                }
            },
            Interaction::None => {
                if let Ok(crate::prelude::OnExit(funcs)) = on_exit.get(entity){
                    for fn_str in funcs.iter(){
                        function_bindings.maybe_run(fn_str, entity, &mut cmd);
                    }
                }
            },
        }
    });
}

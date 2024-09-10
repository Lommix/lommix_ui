use bevy::{
    ecs::system::{EntityCommands, SystemId},
    prelude::*,
    utils::HashMap,
};

use crate::data::Action;

pub struct BindingPlugin;
impl Plugin for BindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FunctionBindings>();
        app.init_resource::<SpawnBindings>();
        app.add_systems(Update, observe_interactions);
    }
}

pub type SpawnFunction = dyn Fn(EntityCommands) + Send + Sync + 'static;

/// # Spawn binding resource
///
/// maps spawn functions on any node identitfied by a string
/// makes it possible to mark nodes with any custom spawn logic/components
///
/// in templates: `comp="slider_tag"`
///
/// backend:
///
/// `
/// SpawnBindings.register("slider_tag", &|mut cmd: EntityCommands| cmd.insert(SliderTag::default()))
/// `
#[derive(Resource, Default, Deref, DerefMut)]
pub struct SpawnBindings(HashMap<String, &'static SpawnFunction>);

impl SpawnBindings {
    pub fn register<F>(&mut self, key: impl Into<String>, f: &'static F)
    where
        F: Fn(EntityCommands) + Send + Sync + 'static,
    {
        let key: String = key.into();
        self.insert(key, f);
    }

    pub fn maybe_run(&self, key: &String, entity: Entity, cmd: &mut Commands) {
        self.get(key).map(|f| {
            let cmd = cmd.entity(entity);
            f(cmd);
        });
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
        self.get(key).map(|id| {
            cmd.run_system_with_input(*id, entity);
        });
    }
}

#[rustfmt::skip]
fn observe_interactions(
    mut cmd: Commands,
    interactions: Query<(Entity, &Interaction, &Action), Changed<Interaction>>,
    function_bindings: Res<FunctionBindings>,
){
    interactions.iter().for_each(|(entity, interaction, action)|{
        //-----
        match interaction {
            Interaction::Pressed =>{
                if let Action::OnPress(fn_str) = action {
                    function_bindings.maybe_run(fn_str, entity, &mut cmd);
                }
            }
            Interaction::Hovered => {

            },
            Interaction::None => {

            },
        }
    });
}

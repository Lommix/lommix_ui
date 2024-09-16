pub use bevy::prelude::*;

use crate::build::ScopeEntity;

pub struct StatePlugin;
impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CompileVars>();
    }
}


#[derive(Event)]
pub struct CompileVars;

fn recompile_vars(trigger: Trigger<Entity>, Query: Query<&ScopeEntity>) {

}

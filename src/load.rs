use crate::{error::ParseError, node::NNode, parse::parse_xml_new};
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    ecs::system::SystemId,
    prelude::*,
    utils::HashMap,
};

pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<NNode>();
        app.init_asset_loader::<LayoutLoader>();
        app.add_systems(Update, interaction_observer);

        let mut map = UiFnMap::default();

        let id = app.register_system(on_click);
        map.insert("start_game".into(), id);

        app.insert_resource(map);
        // map.insert("start_game".into(), ob);
        // map.insert("start_game".into(), Box::new(observe));
        // map.insert("end_game".into(), Box::new(observe));
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct UiFnMap(HashMap<String, SystemId<Entity>>);

#[derive(Event)]
pub struct ClickEvent;

#[derive(Event)]
pub struct HoverEvent;

#[derive(Component)]
pub struct HoverAction(String);

#[derive(Component)]
pub struct ClickAction(pub String);

fn interaction_observer(
    mut cmd: Commands,
    interactions: Query<(Entity, &Interaction, &ClickAction), Changed<Interaction>>,
    fnmap: Res<UiFnMap>,
) {
    interactions
        .iter()
        .for_each(|(ent, int, action)| match int {
            Interaction::Pressed => {
                if let Some(id) = fnmap.get(&action.0) {
                    cmd.run_system_with_input(*id, ent);
                }
            }
            Interaction::Hovered => {
                // if let Some(id) = fnmap.get(&action.0) {
                //     cmd.run_system_with_input(*id, ent);
                // }
            }
            Interaction::None => (),
        });
}

fn on_click(ent: In<Entity>, cmd: Commands, server: Res<AssetServer>) {
    print!("hello world \n");
}

// #[derive(Asset, TypePath)]
// pub struct XmlUi {
//     pub root: XNode,
// }

#[derive(Default)]
pub struct LayoutLoader;
impl AssetLoader for LayoutLoader {
    type Asset = NNode;
    type Settings = ();
    type Error = ParseError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|err| ParseError::Failed(err.to_string()))?;

            let root = parse_xml_new(&bytes).unwrap();
            Ok(root)
        })
    }
}

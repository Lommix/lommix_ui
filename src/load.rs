use crate::{data::XNode, error::ParseError, parse::parse_bytes};
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};

pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<XNode>();
        app.init_asset_loader::<LayoutLoader>();
    }
}

#[derive(Default)]
pub struct LayoutLoader;
impl AssetLoader for LayoutLoader {
    type Asset = XNode;
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
                .map_err(|err| ParseError::FailedToRead(err.to_string()))?;

            parse_bytes(&bytes)
        })
    }
}

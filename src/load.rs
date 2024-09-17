use crate::{data::Template, error::ParseError, parse::parse_template};
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};

pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Template>();
        app.init_asset_loader::<XmlUiLoader>();
    }
}

#[derive(Default)]
pub struct XmlUiLoader;
impl AssetLoader for XmlUiLoader {
    type Asset = Template;
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

            parse_template(&bytes)
        })
    }
}

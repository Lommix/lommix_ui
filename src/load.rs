use crate::{data::HtmlTemplate, error::ParseError, parse::parse_template};
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};

pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HtmlTemplate>();
        app.init_asset_loader::<HtmlUiLoader>();
    }
}

#[derive(Default)]
pub struct HtmlUiLoader;
impl AssetLoader for HtmlUiLoader {
    type Asset = HtmlTemplate;
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

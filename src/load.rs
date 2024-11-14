use crate::{data::HtmlTemplate, error::ParseError, parse::parse_template};
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};

pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HtmlTemplate>();
        app.init_asset_loader::<HtmlAssetLoader>();
    }
}

#[derive(Default)]
pub struct HtmlAssetLoader;
impl AssetLoader for HtmlAssetLoader {
    type Asset = HtmlTemplate;
    type Settings = ();
    type Error = ParseError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|err| ParseError::FailedToRead(err.to_string()))?;

        match parse_template::<nom::error::VerboseError<&[u8]>>(&bytes) {
            Ok((_, template)) => Ok(template),
            Err(err) => {
                let msg = crate::parse::convert_verbose_error(&bytes, err);
                Err(ParseError::Nom(msg))
            }
        }
    }

    fn extensions(&self) -> &[&str] {
        &["html", "xml"]
    }
}

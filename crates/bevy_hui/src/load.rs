use crate::{data::HtmlTemplate, error::ParseError, parse::parse_template};
use bevy::{
    asset::{io::Reader, AssetLoader},
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

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|err| ParseError::FailedToRead(err.to_string()))?;

        let file_path = load_context.path().to_str().unwrap_or_default();
        match parse_template::<crate::error::VerboseHtmlError>(&bytes) {
            Ok((_, template)) => Ok(template),
            Err(err) => match err {
                nom::Err::Incomplete(_) => Err(ParseError::Incomplete),
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    Err(ParseError::Nom(err.format(&bytes, file_path)))
                }
            },
        }
    }

    fn extensions(&self) -> &[&str] {
        &["html", "xml"]
    }
}

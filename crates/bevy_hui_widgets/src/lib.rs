#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(rustdoc::redundant_explicit_links)]
#![doc = include_str!("../README.md")]

use bevy::app::Plugin;
mod input;
mod select;
mod slider;

/// Widget Component lib for bevy_hui
pub mod prelude {
    pub use super::input::{HuiInputWidgetPlugin, TextInput};
    pub use super::slider::{HuiSliderWidgetPlugin, Slider, SliderAxis, SliderChangedEvent};
    pub use super::select::{HuiSelectWidgetPlugin, SelectionChangedEvent, SelectInput};
    pub use super::HuiWidgetCompletePlugin;
}

/// # The complete widget library
///
/// for implementation details check out the
/// specific widget plugins.
///
/// [slider::HuiSliderWidgetPlugin]
/// [input::HuiInputWidgetPlugin]
/// [select::HuiSelectWidgetPlugin]
///
pub struct HuiWidgetCompletePlugin;
impl Plugin for HuiWidgetCompletePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            slider::HuiSliderWidgetPlugin,
            input::HuiInputWidgetPlugin,
            select::HuiSelectWidgetPlugin,
        ));
    }
}

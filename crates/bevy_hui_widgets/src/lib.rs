mod slider;
mod text_input;

/// Widget Component lib for bevy_hui
pub mod prelude {
    pub use super::slider::{HuiSliderWidgetPlugin, Slider, SliderAxis};
    pub use super::text_input::{HuiInputWidgetPlugin, TextInput};
}

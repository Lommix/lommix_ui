# 0.2.0

- added support for `overflow_clip_margin`

- added new `on_changed` function binding attribute for all nodes.
  Does nothing by default. Must be triggered by the user. For notifying change in
  custom widgets.

- added `scroll` to overflow values.

- added `shadow_color`,`shadow_blur`,`shadow_spread` and `shadow_offset` for the new box shadow.
  These values can all be used with conditional styling and easing!

- added `image_region` style attribute. `(float,float)(float,float)` -> maps to `Rect`
  and can be used together with the `image`-Node.

- added new optional `bevy_hui_widgets` sub crate. Provides helper functions for some basic
  widgets. Checkout the new example.

# 0.1.0

- First release

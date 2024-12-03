# Bevy_hui Cheatsheet

## Default nodes

| Html                 | Bevy                                |
| -------------------- | ----------------------------------- |
| `<template>`         | component entry / root element      |
| `<node>`             | `Node`                              |
| `<image>`            | `UiImage`                           |
| `<button>`           | `Button`                            |
| `<text>`             | `Text`                              |
| `<slot\>`            | component slot marker               |
| `<property name="">` | template property def with fallback |

## Basic Values

| Valid in Html                                                     | Bevy        |
| ----------------------------------------------------------------- | ----------- |
| `float + px/%/vw/vh/vmin/vmax`                                    | `Val`       |
| `auto`,`min`,`max` `float + px/fr/flex/vh/vw/%`                   | `GridTrack` |
| all:`Val` axis:`Val Val` full:`Val Val Val Val`                   | `UiRect`    |
| `#FFF` `#FFFA` `#FFFFFF` `#FFFFFFAA` `rgb(1,1,1)` `rgba(1,1,1,1)` | `Color`     |

## Basic Style Attributes

**The Rule: bevy style but snake case**

| Html                  | valid values                                                                                             |
| --------------------- | -------------------------------------------------------------------------------------------------------- |
| position              | `absolute`, `relative`                                                                                   |
| display               | `none`, `flex`, `block`, `grid`                                                                          |
| overflow              | `X Y` (xAxis yAxis) with values: `hidden` `visible` `clip` `scroll`                                      |
| overflow_clip_margin  | `content_box float` `padding_box float` `border_box float`                                      |
| align_self            | `auto` `start` `flex_end` `stretch` `end` `flex_start`                                                   |
| align_items           | `default` `center` `start` `flex_end` `stretch` `end` `baseline` `flex_start`                            |
| align_content         | `space_evenly` `space_around` `space_between` `center` `start` `flex_end` `stretch` `end` `flex_start`   |
| justify_self          | `auto` `center` `start` `stretch` `end` `baseline`                                                       |
| justify_items         | `default` `center` `start` `end` `baseline`                                                              |
| justify_content       | `center` `start` `flex_start` `stretch` `end` `space_evenly` `space_around` `space_between` `flex_start` |
| flex_direction        | `auto` `center` `start` `stretch` `end` `baseline`                                                       |
| flex_wrap             | `wrap` `no_wrap` `wrap_reverse`                                                                          |
| flex_grow             | float                                                                                                    |
| flex_shrink           | float                                                                                                    |
| flex_basis            | ref `Val`                                                                                                |
| row_gap               | ref `Val`                                                                                                |
| column_gap            | ref `Val`                                                                                                |
| grid_auto_flow        | `row`, `column`, `dense_column`, `dense_row`                                                             |
| grid_auto_rows        | ref `GridTrack ...` (array of `GridTrack`)                                                               |
| grid_auto_columns     | ref `GridTrack ...` (array of `GridTrack`)                                                               |
| grid_template_rows    | `(5, auto)(2, 1fr)..` ref `GridTrack`                                                                    |
| grid_template_columns | `(5, auto)(2, 1fr)..` ref `GridTrack`                                                                    |
| grid_row              | `auto` `span(u16)` `start_span(i16,u16)` `end_span(i16,u16)` `end(i16)` `start(i16)`                     |
| grid_column           | `auto` `start_span(5,5)` `end_span(5,5)` `end(5)` `start(5)`                                             |
| image_mode            | `auto` `stretch` slice: `20px tiled(scale) stretch scale` tiled: `bool bool scale` **(scale=float)**     |
| image_region          | `(float,float)(float,float)` = min->max image region rect                                                |
| shadow_color          | ref `Color`                                                                                              |
| shadow_blur           | ref `Val`                                                                                                |
| shadow_spread         | ref `Val`                                                                                                |
| shadow_offset         | ref `Val` `Val` shadow_offset="10px 10px"                                                                |
| font                  | asset path                                                                                               |
| font_color            | ref `Color`                                                                                              |
| font_size             | float                                                                                                    |
| delay                 | `100ms` `5s`                                                                                             |
| ease                  | `bevy_math::EaseFunction` snake case `sine_in` `quintic_in_out`...                                       |
| max_height            | ref `Val`                                                                                                |
| max_width             | ref `Val`                                                                                                |
| min_height            | ref `Val`                                                                                                |
| min_width             | ref `Val`                                                                                                |
| bottom                | ref `Val`                                                                                                |
| top                   | ref `Val`                                                                                                |
| right                 | ref `Val`                                                                                                |
| left                  | ref `Val`                                                                                                |
| height                | ref `Val`                                                                                                |
| width                 | ref `Val`                                                                                                |
| padding               | ref `UiRect` top->right->bottom->left                                                                    |
| margin                | ref `UiRect` top->right->bottom->left                                                                    |
| border                | ref `UiRect` top->right->bottom->left                                                                    |
| border_radius         | ref `UiRect` top->right->bottom->left                                                                    |
| outline               | `Val Val Color`                                                                                          |
| background            | ref `Color`                                                                                              |
| border_color          | ref `Color`                                                                                              |
| src                   | an asset path for image nodes                                                                            |

## Conditional Styles

transition animation in combination with `ease` and `delay`

| Html style prefix | valid values                       |
| ----------------- | ---------------------------------- |
| `hover:..`        | active on `Interaction::Hover`     |
| `pressed:..`      | active on `Interaction::Press`     |
| `active:..`       | active if has component `UiActive` |

## Events

Each event accepts a list of comma separated function bindings

`on_spawn="my_func, my_second_func"`

| Html        | Explanation                                         |
| ----------- | --------------------------------------------------- |
| `on_spawn`  | called on spawning                                  |
| `on_press`  | called on `Interaction::Press`                      |
| `on_enter`  | called on enter `Interaction::Hover`                |
| `on_exit`   | called on enter `Interaction::None`                 |
| `on_change` | triggered by the user. Used to build custom widgets |

## Special Helpers

These are local to the template and cannot be referenced outside.

| Html               | Explanation                                                          |
| ------------------ | -------------------------------------------------------------------- |
| `id="my_node"`     | id marker (Adds `UiId(String)` Component)                            |
| `target="my_node"` | target marker (Adds `UiTarget(Entity)` Component (resolved at build) |
| `watch="my_node"`  | 'watch' another nodes `Interaction` for conditional styles           |

## Custom tags

Any attribute marked with `tag:my_value=""` can be accessed on the `Tag` Component
owned by the node. It's just a HashMap. Great for passing args in combination with
events. Checkout `ui` example to play a custom sound, defined by a tag on enter.

## Binding Functions & Custom Components

```rust

fn setup_templates(
    mut cmd: Commands,
    server: Res<AssetServer>,
    mut html_funcs: HtmlFunctions,
    mut html_comps: HtmlComponents,
) {

    // function
    html_funcs.register("hello_world", |In(entity): In<Entity>|{
        println!("hello world");
    });

    // component
    html_comps.register("panel", server.load("panel.html"));


    // component with spawn function
    html_comps.register("slider", server.load("slider.html"), |mut entity_commands|{
        entity_commands.inser(SliderState(0.));
    });

    // advanced function using tags
    html_funcs.register("play_beep", play_beep);
}

fn play_beep(
    In(entity): In<Entity>,
    tags: Query<&Tags>,
    mut cmd: Commands,
    server: Res<AssetServer>,
) {
    let Some(path) = tags
        .get(entity)
        .ok()
        .and_then(|t| t.get("source").map(|s| s.to_string()))
    else {
        return;
    };

    cmd.spawn((
        AudioPlayer(server.load(&path)),
        PlaybackSettings::ONCE,
    ));
}
```

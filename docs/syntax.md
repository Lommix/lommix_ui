# Style Values

## Function bindings

provides basics events for the `Interaction` Component and on `Spawn`.

-   `onspawn="init_friend_list"`
-   `onenter="play_beep"`
-   `onpress="start_game"`
-   `onexit="fade_out"`

in bevy your register your function on startup in the

```rust
fn setup(
    mut function_bindings: ResMut<FunctionBindings>,
) {
    function_bindings.register(
        "collapse",
        cmd.register_one_shot_system(|In(entity), mut cmd: Commands| {
            cmd.entity(entity).insert(Collapse(true));
        }),
    );
}
```

## Register a custom node

if autoloading from files is not enough, you need more control over the bundle

```rust
fn setup(
    mut custom_comps: ResMut<ComponentBindings>,
) {
    let panel_handle = server.load("panel.xml");
    custom_comps.register("panel", move |mut entity_cmd: EntityCommands| {
        entity_cmd.insert((HtmlBundle {
            handle: panel_handle.clone(),
            ..default()
        },));
    });
}
```

## Create connections, observe other nodes

-   `id="my_node"`: gives an identifier to a node, that is only valid inside the component.
-   `target="my_node"`: adds a target component to the node, if `my_var` is a valid node id.
-   `watch="my_node"`: any `hover:style` and `pressed:style` gets applied when the watched node is interacted with.

## Default elements

default elements with events/special attributes.

-   `<node>` : `NodeBundle`
    -   `onspawn=".."`,
-   `<img>` : `ImageBundle`
    -   `src="image_path"`
    -   `onspawn=".."`,
-   `<button>` : `ButtonBundle`
    -   `onpress=".."`,
    -   `onenter=".."`,
    -   `onexit=".."`,
    -   `onspawn=".."`,
-   `<text>` : `TexBundle`
    -   `onspawn=".."`,
-   `<Include>`
    -   `src="include_path"`
    -   `onspawn=".."`,
-   `<slot>` : marks a slot if block is included. There is currently only support for 1 slot.

## Conditional Styles

apply on condition, for elements using the `Interaction` Component like buttons.

-   `hover:background_color="#665"`
-   `pressed:background_color="#666"`

## Style tags

-   `padding="UiRect"`
-   `margin="UiRect"`
-   `border="UiRect"`
-   `border_color="Color"`
-   `background="Color"`
-   `width="Val"`
-   `height="Val"`

-   `display="grid"`
-   `postion="aboslute"`
-   `overflow="visible visible"`
-   `min_width="5%"`
-   `max_width="5vw"`
-   `max_height="5%"`
-   `min_height="56px"`
-   `aspect_ratio="50"` : flaot
-   `duration="50"` -> adds a Duration Component, [WIP], animations

## `Flex`

-   `flex_direction="row"`: [`row`, `column`, `column_reverse`, `row_reverse`, `default`]
-   `flex_warp="warp"`: [`wrap`, `no_wrap`, `wrap_reverse` ]
-   `justify_self="center"`: [`auto`, `center`, `start`, `stretch`, `end`, `baseline` ]
-   `justify_content="center"`: [`space_evenly`,`space_around`,`space_between`, `center`, `start`, `stretch`, `end`,`flex_end`, `flex_start` ]
-   `justify_content="items"`: [`default`,`center`,`start`, `stretch`, `end`, `baseline`]

## `Grid`

-   `grid_auto_flow="row"`: [`row`, `column`, `dense_column`, `dense_row`]
-   `grid_template_rows="(5, auto)(2, 1fr).."` : `Vec<(u16, GridTrack)>`
-   `grid_template_columns="(5, auto)(2, 1fr).."` : `Vec<(u16, GridTrack)>`
-   `grid_row="start_span(-5,6)"` : `GridPlacement::start_span(-5,6)>`
-   `grid_column="span(6)"` : `GridPlacement::span(-5,6)>`

## `GridTrack`

-   `auto`: `GridTrack::auto()`
-   `min`: `GridTrack::min_content()`
-   `max`: `GridTrack::max_content()`
-   `1fr`: `GridTrack::fr(1.)`
-   `1flex`: `GridTrack::flex(1.)`
-   `20px`: `GridTrack::px(20.)`
-   `20vh`: `GridTrack::vh(20.)`
-   `20vw`: `GridTrack::vw(20.)`
-   `20%`: `GridTrack::percent(20.)`

## Gap

-   `column_gap="5px"` `Val`
-   `row_ga="5px"` `Val`

## Font

-   `font_size="20"` flaot
-   `font="font.ttf"` asset path
-   `font_color="#000"` any color

## Border

-   `border_color="rgba(1,1,1,1)"` any `Color`
-   `border_radius="10px 10px 10px 10%"` any `UiRect`
-   `border="10px 10px 10px 10%"` any `UiRect`

## GridPlacement

-   `span(5)` : `GridPlacement::span(5)`
-   `start_span(-5, 5)` : `GridPlacement::start_span(5)`
-   `end_span(5)` : `GridPlacement::end_span(0,2)`
-   `end(5)` : `GridPlacement::end(5)`

## Val

-   `20px`: `Val::Px(20.)`
-   `20vmin`: `Val::VMin(20.)`
-   `20vmax`: `Val::VMax(20.)`
-   `20vh`: `Val::Vh(20.)`
-   `20vw`: `Val::Vw(20.)`
-   `20%`: `Val::Percent(20.)`

## UiRect

-   `20px`: Uirect::all(value)
-   `20px 10px`: Uirect::axis(value, value)
-   `20px 10p% 5vw 3px`: Uirect::new(value,value,value,value)

## `Color`

-   `#FFF`
-   `#FFFA` with alpha
-   `#FFFFFF`
-   `#FFFFFFAA` with alpha
-   `rgb(1,1,0.5)`
-   `rgba(1,1,0.8,1)`

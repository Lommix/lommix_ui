# Effortless Bevy ui

Opionionated `html/Xml`-like bevy ui parser. First class support for hotreloading.

## Featuring

-   Hotreload, iteratate fast.
-   Conditional styling with prefix like `hover:..` & `pressed:..`
-   Component Templates with `<include>`
-   Function- and Spawnbindings like `on:pess="fn_id"` or `spawn="add_my_component"`
    -   `fn_id` maps a string to a one shot system, passing the entity.
    -   `add_my_component` maps a string to a function with access to `EntityCommands`.
-   Template `props`. expose values in components for reuseablity.
-   Very little dependecies. Minimal designed. No Widgets. Just the tools to make your own fast.

## Example

you write html, you get bevy ui. hotreload, happy dev noises.

```html
<div padding="10px">
    <button
        padding="5px"
        hover:background="#0000FF"
        background="#000000"
        height="80px"
        width="210px"
        on:press="start_game"
        border="10px"
        border_color="#FFF"
        border_radius="30px"
    >
        <text font_size="20" font_color="#FFF">click me</text>
    </button>
</div>
```

## Syntax

There is only one truth: `snake_case`. Take any Bevy naming, make it `snake_case`, you found your value.

[checkout the full syntax here](docs/syntax.md)

## Customize

Add your own conmponent templates. `UiTarget` and `UiId` Components are resolved on build to contain the
Entities of the corresponding element. With this, you can shape any custom logic and make it reuseable.

`id` and `target` are always local to the file and do not propagate on includes.

```html
<!-- menu.html -->
<my_panel>
    <text>Hello Worl</text>
</my_panel color="#000" close_button_text="X">
```

```html
<!-- panel.html -->
<node background_color={color} >
    <button on_spawn="add_collapse_comp" target="collapse_me">
        <text>{close_button_text}</text>
    </button>

    <node id="collapse_me">
        <slot />
    </node>
</nonde>
```

```rust
fn setup(
    server: Res<AssetServer>,
    mut custom_comps: ResMut<ComponenRegistry>,
) {
    let panel_handle = server.load("panel.html");
    custom_comps.register("panel", move |mut entity_cmd: EntityCommands| {
        entity_cmd.insert((UiBundle {
            handle: panel_handle.clone(),
            ..default()
        },));
    });
}
```

## Goal

The goal is to provide a very thin layer of ui syntax abstraction for seamless and fast iteration on your design.

This crate will not provide any ui components/widgets like sliders and so on. Rather it will provide all the tooling neccessary to build your own
on top.

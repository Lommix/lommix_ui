# lommix' effortless bevy ui

`html/Xml`-like bevy ui parser. First class support for hotreloading, custom nodes
and Templating.

This is a [MVP]. Expect alot of change.

## Featuring

-   Hotreload, iteratate fast.
-   Conditional styling with prefix like `hover:..` & `pressed:..`
-   Component Templates with `<include>`
-   Event Hooks that bind to your `OneShotSystems`: `onpress="fn_id"` or `onspawn="init_item_preview"`
-   Add Custom tags for custom logic `tag:scroll_speed="20"` .. `Query<&Tags>`
-   `Properties`. Expose and inject your Values. `<item_card prop:bg="{item_rarity_color}" ..`
-   Very little dependecies. Minimal designed. No Widgets. Just the tools to make your own.

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

`snake_case` all the way. Take any BeVy naming, make it `snake_case`, you found your value.

[checkout the full syntax here](docs/syntax.md)

## Customize

Add your own conmponent templates. `UiTarget` and `UiId` Components are resolved on build to contain the
Entities of the corresponding element. With this, you can shape any custom logic and make it reuseable.

Any `tag:my_tag` gets added as `Tags` Component to the node entity.

`id` and `target` are always local to the file and do not propagate to other templates (slots work).

```html
<!-- menu.html -->
<my_panel>
    <text>Hello Worl</text>
</my_panel color="#000" close_button_text="X">
```

```html
<!-- panel.html -->
<node background_color="{color}" >
    <button on_spawn="init_collapse" target="preview" tag:collapse_speed="20">
        <text>{close_button_text}</text>
    </button>

    <node id="preview">
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
        entity_cmd.insert((HtmlBundle {
            handle: panel_handle.clone(),
            ..default()
        },));
    });
}
```

## Goal

The goal is to provide a very thin layer of ui syntax abstraction for seamless and fast iteration on your design.

## Why Html (-like)

To make us of existing tooling like syntax highlights, auto format and basic linting. T

## Animations & Transitions

Animations and transitions, just like in bevy, are application level. There will probably be
some defaults behind features at some point.

## Errors

They are messy, parser rewrite soon.

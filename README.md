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

[checkout the full syntax here](docs/styles.md)

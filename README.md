# Effortless Bevy ui

bevy ui loader for html-like syntax files. Work in Progress.

## Featuring

-   Hotreload, iteratate fast.
-   Conditional styling with prefix like `hover:..` & `pressed:..`
-   Simple templating with `<include>` elements.
-   Function bindings mapping strings to `systemId`
-   Transition animations.

## Example

```html
<div padding="10px">
    <button
        padding="5px"
        hover:background="#0000FF"
        background="#000000"
        height="80px"
        width="210px"
        click="start_game"
        border="10px"
        border_color="#FFF"
        border_radius="30px"
    >
        <text font_size="20" font_color="#FFF">click me</text>
    </button>
</div>
```

## Syntax

strict unified syntax. Take any Bevy naming, make it `snake_case`.

[checkout the full syntax here](docs/styles.md)

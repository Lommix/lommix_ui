# Bevy Xml Ui

`Xml` ui syntax parser & builder. Create reusable component
templates in plain `Xml`. Enjoy hot reloading, autocomplete, formatting and linting (schema.xsd).

Because there is nothing worse than waiting on compilation.

## Featuring

-   A style attribute for each use. `padding="10px auto 5% 60vw"` -> `Uirect`
-   Hook into events with `onspawn`,`onenter`,`onexit`,`onpress`
-   Conditional styling with prefix like `hover:..` & `pressed:..`
-   Use `id`, `target` to connect elements and have access in bevy systems.
-   `watch="target_id"` to hook to other elements interactions.

## How To

Add the plugin. Use an optional auto load path (filename = component name).

```rust
app.add_plugins(XmlUiPlugin::new().auto_load("components"));
```

Create components.

```html
<!-- /assets/components/super_button.xml-->
<template>
    <property name="text">Press me</property>
    <property name="primary">#123</property>
    <property name="secondary">#503</property>
    <node padding="10px">
        <button
            id="button"
            padding="5px"
            background="{primary}"
            border_color="{primary}"
            hover:background="{secondary}"
            hover:border_color="{secondary}"
            height="80px"
            width="210px"
            border="10px"
            border_radius="30px"
            on:press="start_game"
        >
            <text watch="button" font_size="20" font_color="#FFF" hover:font_color="#752">{text}</text>
        </button>
    </node>
</template>
```

Component auto registers into your running app.
Use them in the next template.

```html
<!-- menu.xml -->
<template>
    <property name="title">My Game</property>
    ...
    <node display="grid" grid_template_columns="(2, auto)">
        <super_button text="Start Game" press="start_game" />
        <super_button text="Settings" press="to_settings" />
        <super_button text="Credits" press="to_credits" />
        <super_button text="Exit" press="quit_game" />
    </node>
    ...
</template>
```

How to load your UI root:

```rust
fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,
) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn(HtmlBundle {
        handle: server.load("menu.xml"),
        state: TemplateState::new()
            .with("title", "My actual translated title"))
        ..default()
    });
}
```

Checkout the examples for advanced interactions.

## Syntax

`snake_case` all the way. Take any Bevy naming, make it `snake_case`, you found your value.

[checkout the full syntax here](docs/syntax.md)

## Autocomplete & Formatting & Linting

not perfect, but getting there. Checkout the example on how to use the provided
schema.xsd. Feel free to extend it to your needs.

[schema.xsd](schema.xsd)

## Goal

The goal is to provide a very thin layer of UI syntax abstraction for seamless and fast iteration on your design.

## Why Xml/Html(-like)

To make us of existing tooling like syntax highlights, auto format, basic linting and even autocomplete.

## Animations & Transitions

Animations and transitions, just like in bevy, are application level. There will probably be
some defaults behind features at some point.

## Known limitations

-   Do not recursive import. [WIP]
-   One root node per component.

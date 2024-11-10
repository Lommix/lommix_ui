# Bevy Html Ui

[WIP][MVP]

Bevy UI serialized to html. Create reusable templates with variable properties
and simple one-shot systems to event bindings.

Use attributes to describe style. Enjoy hot reloading, autocomplete, formatting and linting (schema.xsd).

Because there is nothing worse than waiting on compilation.

https://github.com/user-attachments/assets/4eb22305-7762-404e-9093-806b6a155ede

## Featuring

A keyword for every bevy related UI style. Take any Bevy naming, make it `snake_case`, you found your value.

-   A simple way to describe complex styles via attributes. `padding="20px 0 5% 1vw"`, `grid_template_columns="(3,auto)"`
-   Wire up your bevy systems with `onspawn`,`onenter`,`onexit`,`onpress` events.
-   Use `id`, `target` to connect elements and have access as components in bevy systems.
-   Conditional style transitions with `hover:`,`pressed:`,`active:`,`delay` & `ease`.
-   propagate style transitions with `watch`.
-   very thin dependencies.

## How To

Add the plugin.

```rust
app.add_plugins(HtmlUiPlugin);
```

Create components in html or xml, same syntax.

```html
<!-- /assets/my_button.xml-->
<template>
    <property name="action">greet</property>
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
            delay="0.2"
            ease="cubic_out"
            height="80px"
            hover:height="100px"
            width="210px"
            hover:width="230px"
            border="10px"
            border_radius="30px"
            onpress="{action}"
        >
            <text
                watch="button"
                font_size="20"
                font_color="#FFF"
                hover:font_color="#752"
            >
                {text}
            </text>
        </button>
    </node>
</template>
```

You can now use the `include` node to import your component:

```
<include src="my_button.xml" text="Start Game" action="start_game" />
```

Or even better, register it as a custom component in a startup system to use
it in any other template!

```rust
fn startup(
    server: Res<AssetServer>,
    mut custom_comps: ResMut<ComponentBindings>,
) {
    let button_template: Handle<HtmlTemplate> = server.load("my_button.xml");
    custom_comps.register("my_button", move |mut entity_cmd: EntityCommands| {
        entity_cmd.insert(HtmlBundle {
            handle: button_template.clone(),
            ..default()
        });
    });
```

And now you have a new node!

```html
<!-- menu.xml -->
<template>
    <property name="title">My Game</property>
    ...
    <img
        display="grid"
        grid_template_columns="(2, auto)"
        src="ui_panel.png"
        image_scale_mode="10px tile(1) tile(1) 4"
        >
            <my_button text="Start Game" action="start_game" />
            <my_button text="Settings" action="to_settings" />
            <my_button text="Credits" action="to_credits" />
            <my_button text="Exit" action="quit_game" />
    </img>
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
            .with("title", "I'm injecting my values"))
        ..default()
    });
}
```

Checkout the examples for advanced interactions, play with the assets. Keep in mind these are
very crude as proof of concept.

```bash
# basic menu demo
cargo run --example ui

# simple text inputs with a submit form
cargo run --example input

# simple sliders
cargo run --example slider
```

## Syntax

[checkout the full syntax here](docs/syntax.md)

## Autocomplete, Formatting & Linting

Not perfect, but getting there. Checkout the example on how to use the provided
schema.xsd. Feel free to extend it to your needs.

[schema.xsd](schema.xsd)

## Goal

The goal is to provide a very thin layer of UI syntax abstraction for seamless and fast iteration on your design,
while keeping any kind of logic in bevy.

## Why Xml/Html(-like)

To make us of existing tooling like syntax highlights, auto format, basic linting and even autocomplete.

## Trade offs

-   You loose control over all `Style` related Components for all nodes part of a template. Instead use `NodeStyle` which holds
    the `regular` state and `hover`,`pressed` + `active` style attributes.

## Known limitations and Pitfalls

-   Do not recursive import. [mem stonks]
-   One root node per component.
-   .xsd schema is broken/unfinished.
-   docs are uncomplete and sometimes outdated.

# Bevy_hui

Build `bevy_ui` design in pseudo Html. Keep your logic in bevy, while iterating fast on design
with hot reloading. Create reusable templates in the style of Web Components.

**starting with bevy 0.15!**

https://github.com/user-attachments/assets/4eb22305-7762-404e-9093-806b6a155ede

## Features

-   In build support for conditional styles and transitions. Hover animations by default!
-   Any value can be a dynamic property and injected into a template at runtime. (recursive!)
-   Simple but effective event system. Register any bevy system via function binding and use it
    in your templates `on_press="start_game"`.
-   No widgets, no themes. Just bevy UI serialized with all the tools necessary to build anything
    in a reusable way.

## Example

Like most crates, don't forget to register the plugin!

```rust
app.add_plugins((
    HuiPlugin,
    // Optional auto loading. Any template this folder will register as custom component
    // using the file name.
    HuiAutoLoadPlugin::new(&["components"]),
));

```

## Getting Started

[Bevy html syntax Cheatsheet](docs/syntax.md)

Create your first component template with external properties!

```html
<!-- /assets/my_button.html-->
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
            delay="0.2s"
            ease="cubic_in"
            height="80px"
            width="210px"
            border="10px"
            border_radius="30px"
            hover:height="100px"
            hover:background="{secondary}"
            hover:border_color="{secondary}"
            hover:width="230px"
            on_press="{action}"
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

## Register your component and make a custom binding

To use your new component in any other templates, we have to register it first.
You can either use the `HuiAutoLoadPlugin` feature (experimental), which
is great for simple components or register the component yourself in a startup system.

This also allows for custom spawning functions. With is great if you need to add custom components as well!

```rust
fn startup(
    server: Res<AssetServer>,
    mut html_comps: HtmlComponents,
    mut html_funcs: HtmlFunctions,
) {
    // simple register
    html_comps.register("my_button", server.load("my_button.html"));

    // advanced register, with spawn functions
    html_comps.register_with_spawn_fn("my_button", server.load("my_button.html"), |mut entity_commands| {
        entity_commands.insert(MyCustomComponent);
    })

    // create a system binding that will change the game state.
    // any (one-shot) system with `In<Entity>` is valid!
    // the entity represents the node, the function is called on
    html_funcs.register("start_game", |In(entity): In<Entity>, mut state : ResMut<NextState<GameState>> |{
        state.set(GameState::Play);
    });

```

## Putting it all together

Time to be creative. Include your component in the next template.

```html
<!-- menu.html -->
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

## Spawning your Template

required components make it super simple.

```rust
fn setup(
    mut cmd: Commands,
    server: Res<AssetServer>,
) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn(HtmlNode(server.load("menu.html"));
}
```

## Hot reload and advanced examples

Hot reload requires bevy `file_watcher` feature to be enabled.

Checkout the examples for advanced interactions, play with the assets. Keep in mind these are
very crude as proof of concept.

```bash
# basic menu demo
cargo run -p example --bin ui

# simple text inputs with a submit form
cargo run -p example --bin input

# simple sliders
cargo run -p example --bin slider
```

## Help wanted

I do not plan to offer any widgets on the templating side, but I would like
to have common components and system for a general reusable widget toolkit like
sliders, drop downs, dragables and so on.

Checkout the examples, if you come up with some really cool widgets, I would be happy
to merge them into a new sub crate behind a feature flag!

### More examples

I am not the greatest designer. I am actively looking for some really fancy and cool examples, using
this crate to include in the example crate.

## Known limitations and Pitfalls

-   Any manual changes to bevy's styling components will be overwritten
-   Do not recursive import. [mem stonks, bug]
-   One root node per component.
-   docs are uncomplete and sometimes outdated.

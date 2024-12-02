# Bevy Hui Widgets

A collection of bevy components & systems to build widgets
with `bevy_hui`.

It is highly suggest to checkout the widget example in the example-crate.

## Slider

A slider is a node with an absolute button as child.
The button can be dragged on a fixed axis. The slider state/value
is stored on the root node.

The important part:

`on_spawn="init_slider"`
`tag:axis="x"`

A minimal template you have to implement:

```html
<template>
    <node
        on_spawn="init_slider"
        tag:axis="x"
        width="255px"
        height="20px"
        background="#000"
    >
        <button
            background="#FFF"
            position="absolute"
            width="20px"
            height="20px"
        ></button>
    </node>
</template>
```

## Input

A input node is just a button with a text node somewhere in it's hierarchy.

```html
<template>
    <button
        on_spawn="init_input"
        target="text_value"
        tag:filter="text"
        active:border_color="#FFF"
    >
        <text id="text_value">Placeholder</text>
    </button>
</template>
```

## Select [WIP]

A select is a button with a text child and a hidden container node with options.
The current selected value is represented by the entity of the value node.
It's up to the user to add any Component/value to that node.

Minimal template:

Select Template:

```html
<template>
    <button
        on_spawn="init_select"
        target="options"
    >
        <text font_size="20">None</text>
        <node
            display="none"
            top="30px"
            id="options"
            position="absolute"
        >
            <slot />
        </node>
    </button>
</template>
```

Option Template:

```html
<template>
    <property name="value"></property>
    <button tag:value="{value}">
        <text font_size="20">{value}</text>
    </button>
</template>
```

Usage:

```html
<select>
    <option value="option 1" />
    <option value="option 2" />
    <option value="option 3" />
</select>
```

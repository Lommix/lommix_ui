# Style Values

## Elements

-   `<div>` : `NodeBundle`
-   `<img>` : `ImageBundle`
    -   `path="image_path"`
-   `<button>` : `ButtonBundle`
    -   `click="function_key"`
-   `<text>` : `TexBundle`
-   `<Include>` : Includes another file
    -   `path="include_path"`
-   `<slot>` : marks a slot if block is included [WIP]

## Style tags

-   `padding="UiRect"`
-   `margin="UiRect"`
-   `border="UiRect"`
-   `border_color="Color"`
-   `background="Color"`
-   `width="Val"`
-   `height="Val"`

## Style tags prefix

apply on condition, for elements using the `Interaction` Component like buttons.

-   `hover:background_colo="Color"`
-   `pressed:background_colo="Color"`

## Valid Bevy `Val`

-   `20px`: Val::Px(20.)
-   `20vw`: Val::Vw(20.)
-   `20%`: Val::Percent(20.)

## Valid Bevy `UiRect`

-   `20px`: Uirect::all(value)
-   `20px 10px`: Uirect::axis(value, value)
-   `20px 10p% 5vw 3px`: Uirect::new(value,value,value,value)

## Valid Bevy `Color`

-   `#FFF`
-   `#FFFA` with alpha
-   `#FFFFFF`
-   `#FFFFFFAA` with alpha
-   `rgb(1,1,0.5)`
-   `rgba(1,1,0.8,1)`

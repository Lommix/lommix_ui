use bevy::prelude::*;
use bevy_hui::prelude::*;
use maud::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HuiPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut cmd: Commands, mut templates: ResMut<Assets<HtmlTemplate>>) {
    cmd.spawn(Camera2d);

    let html = greet_button("Maud").render();

    let template = match parse_template::<VerboseHtmlError>(html.0.as_bytes()) {
        Ok((_, template)) => template,
        Err(err) => {
            let e = err.map(|e| e.format(html.0.as_bytes(), "maud"));
            dbg!(e);
            return;
        }
    };

    let handle = templates.add(template);
    cmd.spawn(HtmlNode(handle));
}

fn greet_button(name: &str) -> Markup {
    html!(
        template {
            node background="#000" padding="50px" border_radius="20px"
            {
                button background="#333" padding="10px" border_radius="10px" {
                    text font_size="32" {"Hello "(name)"!"}
                }
            }
        }
    )
}

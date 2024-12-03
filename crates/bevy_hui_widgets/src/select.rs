use bevy::prelude::*;
use bevy_hui::prelude::*;

/// # Select Widget
///
/// A select is a button with 2 children. The current
/// selected node and a hidden node, holding the options.
///
pub struct HuiSelectWidgetPlugin;
impl Plugin for HuiSelectWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SelectInput>();
        app.register_type::<SelectOption>();
        app.register_type::<SelectionChangedEvent>();
        app.add_event::<SelectionChangedEvent>();
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (
                open_list,
                selection,
                update_selection.run_if(on_event::<SelectionChangedEvent>),
            ),
        );
    }
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect]
pub struct SelectInput {
    // points to the current select node
    pub value: Option<Entity>,
}

#[derive(Component, Debug, Reflect)]
#[reflect]
pub struct SelectOption {
    select: Entity,
}

#[derive(Event, Reflect, Debug)]
#[reflect]
pub struct SelectionChangedEvent {
    pub select: Entity,
    pub option: Entity,
}

fn setup(mut html_funcs: HtmlFunctions) {
    html_funcs.register("init_select", init_select);
}

fn init_select(
    In(entity): In<Entity>,
    mut cmd: Commands,
    children: Query<&Children>,
    targets: Query<&UiTarget>,
) {
    cmd.entity(entity).insert(SelectInput::default());

    let Ok(option_holder) = targets.get(entity) else {
        warn!("your select does not have a target option list");
        return;
    };

    _ = children.get(**option_holder).map(|children| {
        children.iter().for_each(|option| {
            cmd.entity(*option).insert(SelectOption { select: entity });
        });
    });
}

fn open_list(
    selects: Query<(&Interaction, &UiTarget), (With<SelectInput>, Changed<Interaction>)>,
    mut styles: Query<&mut HtmlStyle>,
) {
    for (interaction, target) in selects.iter() {
        let Ok(mut list_style) = styles.get_mut(**target) else {
            continue;
        };

        match interaction {
            Interaction::Pressed => {
                list_style.computed.node.display = Display::Grid;
            }
            _ => (),
        }
    }
}

fn selection(
    mut events: EventWriter<SelectionChangedEvent>,
    options: Query<(Entity, &Parent, &Interaction, &SelectOption), Changed<Interaction>>,
    mut styles: Query<&mut HtmlStyle>,
) {
    for (entity, parent, interaction, option) in options.iter() {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        events.send(SelectionChangedEvent {
            select: option.select,
            option: entity,
        });

        // close the list
        _ = styles.get_mut(**parent).map(|mut style| {
            style.computed.node.display = Display::None;
        });
    }
}

fn update_selection(
    mut cmd: Commands,
    mut events: EventReader<SelectionChangedEvent>,
    mut texts: Query<&mut Text>,
    children: Query<&Children>,
    tags: Query<&Tags>,
) {
    for event in events.read() {
        let Some(mut text) = children
            .get(event.select)
            .ok()
            .map(|children| {
                children
                    .iter()
                    .filter(|child| texts.get(**child).is_ok())
                    .next()
            })
            .flatten()
            .map(|c| texts.get_mut(*c).ok())
            .flatten()
        else {
            continue;
        };

        _ = tags
            .get(event.option)
            .map(|tags| tags.get("value").map(|s| s.as_str()).unwrap_or_default())
            .map(|t| text.0 = t.into());

        cmd.trigger_targets(UiChangedEvent, event.select);
    }
}

use std::time::Duration;

use crate::{
    compile::{compile_content, CompileContextEvent},
    data::{AttrTokens, HtmlTemplate, NodeType, XNode},
    prelude::ComponentBindings,
    styles::{HoverTimer, NodeStyle, PressedTimer},
};
use bevy::{prelude::*, utils::HashMap};
use nom::{
    bytes::complete::{is_not, tag, take_until},
    character::complete::multispace0,
    sequence::{delimited, preceded, tuple},
};

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (hotreload, spawn_ui, move_children_to_slot).chain());
        app.register_type::<StateSubscriber>();
        app.register_type::<TemplateExpresions>();
        app.register_type::<TemplateState>();
        app.register_type::<ScopeEntity>();
    }
}

#[derive(Component, Clone, Deref, DerefMut, Copy, Reflect)]
#[reflect]
pub struct ScopeEntity(Entity);
impl ScopeEntity {
    pub fn get(&self) -> Entity {
        return **self;
    }
}

#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect]
pub struct TemplateState {
    pub props: HashMap<String, String>,
}

#[derive(Component, Clone, Default, Deref, DerefMut, Reflect)]
#[reflect]
pub struct StateSubscriber(Vec<Entity>);

impl TemplateState {
    pub fn try_set_prop(&mut self, key: &str, value: String) {
        _ = self.props.try_insert(key.to_string(), value);
    }
    pub fn set_prop(&mut self, key: &str, value: String) {
        self.props.insert(key.to_string(), value);
    }
    pub fn get_prop(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.props.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Component)]
pub struct InsideSlot {
    owner: Entity,
}

#[derive(Component, Reflect)]
#[reflect]
pub struct SlotPlaceholder {
    owner: Entity,
}

#[derive(Component, Reflect)]
#[reflect]
pub struct UnslotedChildren(Entity);

#[derive(Component, DerefMut, Deref)]
pub struct InteractionObverser(Vec<Entity>);

// map to vtable one day
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect]
pub struct TemplateExpresions(Vec<AttrTokens>);

#[derive(Component, Deref, DerefMut)]
pub struct Tags(Vec<Tag>);

/// @todo:refactor, currently cloning each nodes
/// text template as component
#[derive(Component, Deref, DerefMut)]
pub struct RawContent(String);

impl Tags {
    pub fn get_tag(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|entry| entry.key.eq(key))
            .map(|entry| entry.value.as_str())
    }
}

#[derive(Component)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

/// the entities owned uid hashed as u64
#[derive(Component, Default, Hash, Deref, DerefMut)]
pub struct UiId(String);

/// the entity behind `id` in `target="id"`
#[derive(Component, DerefMut, Deref)]
pub struct UiTarget(pub Entity);

/// watch interaction of another entity
#[derive(Component, DerefMut, Deref)]
pub struct UiWatch(pub Entity);

#[derive(Component, Default)]
pub struct UnbuildTag;

/// Eventlistener interaction transitions to Hover
#[derive(Component, Deref, DerefMut)]
pub struct OnPress(pub Vec<String>);

/// Eventlistener on spawning node
#[derive(Component, DerefMut, Deref)]
pub struct OnSpawn(pub Vec<String>);

/// Eventlistener for interaction transitions to Hover
#[derive(Component, DerefMut, Deref)]
pub struct OnEnter(pub Vec<String>);

/// Eventlistener for interaction transitions to None
#[derive(Component, Deref, DerefMut)]
pub struct OnExit(pub Vec<String>);

/// Spawns a ui template behind an asset.
#[derive(Bundle, Default)]
pub struct HtmlBundle {
    pub handle: Handle<HtmlTemplate>,
    pub node: NodeBundle,
    pub unbuild: UnbuildTag,
    pub state: TemplateState,
}

fn hotreload(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<HtmlTemplate>>,
    templates: Query<(Entity, &Handle<HtmlTemplate>)>,
    sloted_nodes: Query<(Entity, &InsideSlot)>,
) {
    events.read().for_each(|ev| {
        let id = match ev {
            AssetEvent::Modified { id } => id,
            _ => {
                return;
            }
        };
        templates
            .iter()
            .filter(|(_, handle)| handle.id() == *id)
            .for_each(|(entity, _)| {
                let slots = sloted_nodes
                    .iter()
                    .flat_map(|(slot_entity, slot)| (slot.owner == entity).then_some(slot_entity))
                    .collect::<Vec<_>>();

                if slots.len() > 0 {
                    let slot_holder = cmd.spawn_empty().push_children(&slots).id();
                    cmd.entity(entity).insert(UnslotedChildren(slot_holder));
                } else {
                }

                cmd.entity(entity)
                    .despawn_descendants()
                    .retain::<KeepComps>()
                    .insert(UnbuildTag);
            });
    });
}

#[derive(Bundle)]
struct KeepComps {
    pub parent: Parent,
    pub children: Children,
    pub ui: HtmlBundle,
    pub unsloed: UnslotedChildren,
    pub slot: SlotPlaceholder,
    pub inside: InsideSlot,
    pub scope: ScopeEntity,
}

fn move_children_to_slot(
    mut cmd: Commands,
    unsloted_includes: Query<(Entity, &UnslotedChildren)>,
    children: Query<&Children>,
    slots: Query<(Entity, &SlotPlaceholder)>,
    parent: Query<&Parent>,
) {
    unsloted_includes
        .iter()
        .for_each(|(entity, UnslotedChildren(slot_holder))| {
            let Some(placeholder_entity) = slots
                .iter()
                .find_map(|(slot_ent, slot)| (slot.owner == entity).then_some(slot_ent))
            else {
                return;
            };

            let Ok(slot_parent) = parent.get(placeholder_entity).map(|p| p.get()) else {
                error!("parentless slot, impossible");
                return;
            };

            _ = children.get(*slot_holder).map(|children| {
                children.iter().for_each(|child| {
                    if *child != slot_parent {
                        cmd.entity(*child).insert(InsideSlot { owner: entity });
                        cmd.entity(slot_parent).add_child(*child);
                    }
                })
            });

            cmd.entity(entity).remove::<UnslotedChildren>();
            cmd.entity(placeholder_entity).despawn_recursive();
            cmd.entity(*slot_holder).despawn();
        });
}

#[derive(Default)]
struct IdLookUpTable {
    ids: HashMap<String, Entity>,
    targets: HashMap<Entity, String>,
    watch: HashMap<String, Vec<Entity>>,
}

impl IdLookUpTable {
    pub fn subscribe(&mut self, target: String, entity: Entity) {
        match self.watch.get_mut(&target) {
            Some(list) => {
                list.push(entity);
            }
            None => {
                self.watch.insert(target, vec![entity]);
            }
        }
    }
}

fn spawn_ui(
    mut cmd: Commands,
    mut unbuild: Query<(Entity, &Handle<HtmlTemplate>, &mut TemplateState), With<UnbuildTag>>,
    assets: Res<Assets<HtmlTemplate>>,
    server: Res<AssetServer>,
    custom_comps: Res<ComponentBindings>,
) {
    unbuild
        .iter_mut()
        .for_each(|(root_entity, handle, mut state)| {
            let Some(template) = assets.get(handle) else {
                return;
            };

            let mut subscriber = StateSubscriber::default();
            let mut id_table = IdLookUpTable::default();
            let scope = ScopeEntity(root_entity);

            template.properties.iter().for_each(|(key, val)| {
                _ = state.try_set_prop(key, val.clone());
            });

            build_node(
                0,
                root_entity,
                scope,
                &template.root[0],
                &mut cmd,
                &assets,
                &server,
                &custom_comps,
                &mut id_table,
                &mut state,
                &mut subscriber,
            );

            id_table.ids.iter().for_each(|(id_string, entity)| {
                cmd.entity(*entity).insert(UiId(id_string.clone()));
            });

            id_table.targets.iter().for_each(|(entity, target_id)| {
                match id_table.ids.get(target_id) {
                    Some(tar) => {
                        cmd.entity(*entity).insert(UiTarget(*tar));
                    }
                    None => warn!("target `{target_id}` not found for entity {entity}"),
                }
            });

            id_table.watch.iter().for_each(|(target_str, obs_list)| {
                match id_table.ids.get(target_str) {
                    Some(to_observe) => {
                        cmd.entity(*to_observe)
                            .insert(InteractionObverser(obs_list.clone()));
                    }
                    None => warn!("undefined watch target `{target_str}`"),
                }
            });

            cmd.entity(root_entity)
                .insert(subscriber)
                .remove::<UnbuildTag>();

            cmd.trigger_targets(CompileContextEvent, root_entity);
        });
}

/// big recursive boy
#[allow(clippy::too_many_arguments)]
fn build_node(
    depth: u32,
    entity: Entity,
    scope: ScopeEntity,
    node: &XNode,
    cmd: &mut Commands,
    assets: &Assets<HtmlTemplate>,
    server: &AssetServer,
    custom_comps: &ComponentBindings,
    id_table: &mut IdLookUpTable,
    state: &mut TemplateState,
    state_subscriber: &mut StateSubscriber,
) {
    // first node is the state holder
    if depth > 0 {
        cmd.entity(entity).insert(scope.clone());
    }

    if node.uncompiled.len() > 0 {
        cmd.entity(entity).insert(TemplateExpresions(
            node.uncompiled.iter().cloned().collect(),
        ));
        state_subscriber.push(entity);
    }

    let style_attributes = NodeStyle::from(node.styles.clone());
    cmd.entity(entity)
        .insert(PressedTimer::new(Duration::from_secs_f32(
            style_attributes.regular.delay.max(0.01),
        )))
        .insert(HoverTimer::new(Duration::from_secs_f32(
            style_attributes.regular.delay.max(0.01),
        )));

    let passed_state =
        node.defs
            .iter()
            .cloned()
            .fold(TemplateState::default(), |mut m, (key, value)| {
                // info!("passing {key}:{value} on type {:?}", &node.node_type);
                m.props.insert(key, value);
                m
            });

    node.event_listener.iter().for_each(|listener| {
        listener.clone().self_insert(cmd.entity(entity));
    });

    // ------------------ link
    if let Some(id) = &node.id {
        id_table.ids.insert(id.clone(), entity);
    }
    if let Some(target) = &node.target {
        id_table.targets.insert(entity, target.clone());
    }

    if let Some(watch) = &node.watch {
        id_table.subscribe(watch.clone(), entity);
    }
    // ------------------
    if node.tags.len() > 0 {
        cmd.entity(entity).insert(Tags(
            node.tags
                .iter()
                .cloned()
                .map(|(key, value)| Tag { key, value })
                .collect::<Vec<_>>(),
        ));
    }

    match &node.node_type {
        NodeType::Template => todo!(),
        NodeType::Node => {
            cmd.entity(entity)
                .insert((Name::new("Div"), NodeBundle::default(), style_attributes));
        }
        NodeType::Image => {
            //compile if is passed down
            let path = node.src.clone().unwrap_or_default();
            let handle = server.load::<Image>(path);

            if let Some(scale_mode) = style_attributes.regular.image_scale_mode.as_ref() {
                cmd.entity(entity).insert(scale_mode.clone());
            }

            cmd.entity(entity).insert((
                Name::new("Image"),
                ImageBundle {
                    image: UiImage::new(handle),
                    ..default()
                },
                style_attributes,
            ));
        }
        NodeType::Text => {
            let content = node
                .content
                .as_ref()
                .map(|str| compile_content(str, state))
                .unwrap_or_default();

            // @todo: double check bad, refactor
            if node
                .content
                .as_ref()
                .map(|s| is_templated(s.as_str()))
                .unwrap_or_default()
            {
                cmd.entity(entity)
                    .insert(RawContent(node.content.clone().unwrap_or_default()));
                state_subscriber.push(entity);
            }

            cmd.entity(entity).insert((
                Name::new("Text"),
                TextBundle::from_section(
                    content,
                    TextStyle {
                        font_size: 16., // default
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                style_attributes,
            ));
        }
        NodeType::Button => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                style_attributes,
            ));
        }
        NodeType::Include => {
            let path = node.src.clone().unwrap_or_default();
            let handle = server.load::<HtmlTemplate>(path);
            if node.children.len() > 0 {
                let slot_holder = cmd.spawn(NodeBundle::default()).id();
                node.children.iter().for_each(|child_node| {
                    let child = cmd.spawn_empty().id();
                    build_node(
                        depth + 1,
                        child,
                        scope,
                        child_node,
                        cmd,
                        assets,
                        server,
                        custom_comps,
                        id_table,
                        state,
                        state_subscriber,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });
                cmd.entity(entity).insert((UnslotedChildren(slot_holder),));
            }

            cmd.entity(entity)
                .insert((handle, UnbuildTag, NodeBundle::default(), passed_state));

            return;
        }
        NodeType::Custom(custom_tag) => {
            custom_comps.try_spawn(custom_tag, entity, cmd);
            if node.children.len() > 0 {
                let slot_holder = cmd.spawn(NodeBundle::default()).id();
                node.children.iter().for_each(|child_node| {
                    let child = cmd.spawn_empty().id();
                    build_node(
                        depth + 1,
                        child,
                        scope,
                        child_node,
                        cmd,
                        assets,
                        server,
                        custom_comps,
                        id_table,
                        state,
                        state_subscriber,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });

                cmd.entity(entity).insert((UnslotedChildren(slot_holder),));
            }

            cmd.entity(entity).insert((passed_state, UnbuildTag));
            return;
        }
        NodeType::Slot => {
            cmd.entity(entity).insert((
                SlotPlaceholder { owner: scope.get() },
                NodeBundle::default(),
            ));
            return;
        }
    };

    for child_node in node.children.iter() {
        let child = cmd.spawn_empty().id();
        build_node(
            depth + 1,
            child,
            scope,
            child_node,
            cmd,
            assets,
            server,
            custom_comps,
            id_table,
            state,
            state_subscriber,
        );

        cmd.entity(entity).add_child(child);
    }
}

//@todo:dirty af
pub fn is_templated(input: &str) -> bool {
    let parts: Result<(&str, (&str, &str)), nom::Err<nom::error::Error<&str>>> = tuple((
        take_until("{"),
        delimited(tag("{"), preceded(multispace0, is_not("}")), tag("}")),
    ))(input);

    parts.is_ok()
}

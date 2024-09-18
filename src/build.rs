use crate::{
    data::{Action, AttrTokens, Attribute, NodeType, Template, XNode},
    prelude::{ComponentBindings, StyleAttr},
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
        app.register_type::<TemplateState>();

        app.add_event::<CompileStateEvent>();
        app.add_systems(
            Update,
            (
                (hotreload, spawn_ui, move_children_to_slot, style_ui).chain(),
                update_interaction,
            ),
        );
        app.observe(compile_state);
    }
}

#[derive(Component, Clone, Deref, DerefMut, Copy)]
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

#[derive(Component, Clone, Default, Deref, DerefMut)]
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

#[derive(Component)]
pub struct SlotPlaceholder {
    owner: Entity,
}

#[derive(Component)]
pub struct UnslotedChildren(Entity);

#[derive(Component, DerefMut, Deref)]
pub struct InteractionObverser(Vec<Entity>);

// map to vtable one day
#[derive(Component, Deref, DerefMut)]
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

#[derive(Component, DerefMut, Deref, Clone)]
pub struct StyleAttributes(pub Vec<StyleAttr>);

impl StyleAttributes {
    pub fn replace_or_insert(&mut self, attr: StyleAttr) {
        if let Some(i) = self
            .iter()
            .position(|at| std::mem::discriminant(at) == std::mem::discriminant(&attr))
        {
            self[i] = attr;
        } else {
            self.push(attr);
        }
    }

    pub fn try_insert(&mut self, attr: StyleAttr) {
        if self
            .iter()
            .find(|at| std::mem::discriminant(*at) == std::mem::discriminant(&attr))
            .is_none()
        {
            self.push(attr);
        }
    }
}

/// the entities owned uid `id="my_id"`
#[derive(Component, Default, Hash, Deref, DerefMut)]
pub struct UiId(u64);

/// the entity behind `id` in `target="id"`
#[derive(Component, DerefMut, Deref)]
pub struct UiTarget(pub Entity);

/// watch interaction of another entity
#[derive(Component, DerefMut, Deref)]
pub struct UiWatch(pub Entity);

#[derive(Component, Default)]
pub struct UnbuildTag;

/// tag fro reapplying styles
#[derive(Component, Default)]
pub struct UnstyledTag;

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

///
/// Spawns a ui template behind an asset.
///
#[derive(Bundle, Default)]
pub struct HtmlBundle {
    pub handle: Handle<Template>,
    pub node: NodeBundle,
    pub unbuild: UnbuildTag,
    pub unstyled: UnstyledTag,
    pub state: TemplateState,
}

fn update_interaction(
    mut cmd: Commands,
    interactions: Query<(Entity, &Interaction, Option<&InteractionObverser>), Changed<Interaction>>,
    mut targets: Query<(&mut Style, &StyleAttributes, Option<&mut Text>)>,
    server: Res<AssetServer>,
) {
    interactions
        .iter()
        .for_each(|(entity, interaction, maybe_oberser)| {
            if let Ok((mut style, style_attr, mut maybe_text)) = targets.get_mut(entity) {
                apply_interaction(
                    entity,
                    &interaction,
                    style_attr,
                    &mut style,
                    &mut maybe_text,
                    &server,
                    &mut cmd,
                );
            };

            if let Some(observer) = maybe_oberser {
                for obs in observer.iter() {
                    if let Ok((mut style, style_attr, mut maybe_text)) = targets.get_mut(*obs) {
                        apply_interaction(
                            *obs,
                            &interaction,
                            style_attr,
                            &mut style,
                            &mut maybe_text,
                            &server,
                            &mut cmd,
                        );
                    };
                }
            };
        });
}

fn apply_interaction(
    entity: Entity,
    interaction: &Interaction,
    style_attr: &StyleAttributes,
    style: &mut Style,
    maybe_text: &mut Option<Mut<Text>>,
    server: &AssetServer,
    cmd: &mut Commands,
) {
    match interaction {
        Interaction::Pressed => {
            style_attr.iter().for_each(|attr| {
                if let StyleAttr::Pressed(val) = attr {
                    val.apply(entity, cmd, style, maybe_text, &server);
                }
            });
        }
        Interaction::Hovered => {
            style_attr.iter().for_each(|attr| {
                if let StyleAttr::Hover(val) = attr {
                    val.apply(entity, cmd, style, maybe_text, &server);
                }
            });
        }
        Interaction::None => {
            *style = Style::default();
            style_attr.iter().for_each(|attr| match attr {
                StyleAttr::Hover(_) | StyleAttr::Pressed(_) => (),
                any => any.apply(entity, cmd, style, maybe_text, &server),
            });
        }
    };
}

// what if a slot uses another include?
fn hotreload(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<Template>>,
    templates: Query<(Entity, &Handle<Template>)>,
    sloted_nodes: Query<(Entity, &InsideSlot)>,
    scopes: Query<&ScopeEntity>,
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

fn style_ui(
    mut cmd: Commands,
    mut unstyled: Query<
        (Entity, &mut Style, &StyleAttributes, Option<&mut Text>),
        With<UnstyledTag>,
    >,
    server: Res<AssetServer>,
) {
    unstyled
        .iter_mut()
        .for_each(|(entity, mut style, style_attr, mut maybe_text)| {
            style_attr.iter().for_each(|attr| match attr {
                StyleAttr::Hover(_) | StyleAttr::Pressed(_) => (),
                any => any.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server),
            });
            cmd.entity(entity).remove::<UnstyledTag>();
        });
}

#[derive(Event)]
pub struct CompileStateEvent;

fn compile_state(
    trigger: Trigger<CompileStateEvent>,
    mut cmd: Commands,
    mut state: Query<(&mut TemplateState, &StateSubscriber)>,
    scopes: Query<&ScopeEntity>,
    textcontent: Query<&RawContent>,
    expressions: Query<&TemplateExpresions>,
    server: Res<AssetServer>,
    // diff scope, but sub?
    mut ui_images: Query<&mut UiImage>,
    mut styles: Query<(&mut Style, Option<&mut Text>)>,
) {
    let entity = trigger.entity();

    // ---------------------------------
    // self compile state holding node
    let mut to_insert = vec![];
    if let Ok(expr) = expressions.get(entity) {
        // get parent state
        let Some((pstate, _)) = scopes
            .get(entity)
            .ok()
            .map(|scope| state.get(**scope).ok())
            .flatten()
        else {
            return;
        };

        expr.iter().for_each(|exp| {
            let Some(Attribute::PropertyDefinition(key, val)) = exp.compile(pstate) else {
                return;
            };
            to_insert.push((key, val));
        });
    }

    to_insert.drain(..).for_each(|(key, val)| {
        state.get_mut(entity).map(|(mut state, _)| {
            state.set_prop(&key, val);
        });
    });

    // ---------------------------------
    // compile down
    let Ok((state, subscriber)) = state.get(entity) else {
        return;
    };

    for sub in subscriber.iter() {
        if *sub == entity {
            continue;
        }

        let Ok((mut style, mut txt)) = styles.get_mut(*sub) else {
            // warn!("uncompile node, missing scope, style & expressions");
            return;
        };
        // ---------------------------------
        // expressions
        if let Ok(expressions) = expressions.get(*sub) {
            for attr_tokens in expressions.iter() {
                if let Some(attr) = attr_tokens.compile(state) {
                    match attr {
                        Attribute::Style(style_attr) => {
                            style_attr.apply(entity, &mut cmd, &mut style, &mut txt, &server);
                        }
                        Attribute::Action(action) => {
                            action.self_insert(cmd.entity(entity));
                        }
                        // can only be another include
                        Attribute::PropertyDefinition(_key, _val) => {
                            cmd.trigger_targets(CompileStateEvent, *sub);
                        }
                        Attribute::Path(path) => {
                            //is image, reload
                            if let Ok(mut img) = ui_images.get_mut(*sub) {
                                img.texture = server.load(path);
                            }
                        }
                        _any => {
                            continue;
                        }
                    }
                }
            }
        }
        // ---------------------------------
        // text content
        if let Some(mut text) = txt {
            if let Ok(raw) = textcontent.get(*sub) {
                text.sections.iter_mut().for_each(|section| {
                    section.value = compile_content(raw, state);
                });
            } else {
                warn!("missing template content");
            }
        };
    }
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
                // warn!("this node does not have a slot {entity}");
                return;
            };

            // slot is a empty entity
            let Ok(slot_parent) = parent.get(placeholder_entity).map(|p| p.get()) else {
                warn!("parentless slot, impossible");
                return;
            };

            // info!("found slot! {slot}");
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
    mut unbuild: Query<(Entity, &Handle<Template>, &mut TemplateState), With<UnbuildTag>>,
    assets: Res<Assets<Template>>,
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

            // info!("spawned with {:?}", state);

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

            cmd.trigger_targets(CompileStateEvent, root_entity);
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
    assets: &Assets<Template>,
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

    let style_attributes = StyleAttributes(node.styles.clone());
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
            cmd.entity(entity).insert((
                Name::new("Div"),
                NodeBundle::default(),
                style_attributes,
                UnstyledTag,
            ));
        }
        NodeType::Image => {
            //compile if is passed down
            let path = node.src.clone().unwrap_or_default();
            let handle = server.load::<Image>(path);

            cmd.entity(entity).insert((
                Name::new("Image"),
                ImageBundle {
                    image: UiImage::new(handle),
                    ..default()
                },
                style_attributes,
                UnstyledTag,
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
                UnstyledTag,
            ));
        }
        NodeType::Button => {
            cmd.entity(entity).insert((
                Name::new("Button"),
                ButtonBundle::default(),
                style_attributes,
                UnstyledTag,
            ));
        }
        NodeType::Include => {
            let path = node.src.clone().unwrap_or_default();
            let handle = server.load::<Template>(path);
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

pub fn is_templated(input: &str) -> bool {
    let parts: Result<(&str, (&str, &str)), nom::Err<nom::error::Error<&str>>> = tuple((
        take_until("{"),
        delimited(tag("{"), preceded(multispace0, is_not("}")), tag("}")),
    ))(input);

    parts.is_ok()
}

pub(crate) fn compile_content(input: &str, defs: &TemplateState) -> String {
    let mut compiled = String::new();

    let parts: Result<(&str, (&str, &str)), nom::Err<nom::error::Error<&str>>> = tuple((
        take_until("{"),
        delimited(tag("{"), preceded(multispace0, is_not("}")), tag("}")),
    ))(input);

    let Ok((input, (literal, key))) = parts else {
        compiled.push_str(input);
        return compiled;
    };

    compiled.push_str(literal);

    if let Some(value) = defs.get_prop(key.trim_end()) {
        compiled.push_str(value);
    }

    if input.len() > 0 {
        compiled.push_str(&compile_content(input, defs));
    }

    compiled
}

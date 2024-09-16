use crate::{
    data::{Action, AttrTokens, Attribute, NodeType, XNode},
    prelude::{ComponentBindings, StyleAttr},
};
use bevy::{prelude::*, utils::HashMap};
use nom::{
    bytes::complete::{is_not, tag, take_until},
    character::complete::multispace0,
    sequence::{delimited, preceded, tuple},
};
use std::sync::{Arc, RwLock};

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (hotreload, spawn_ui, move_children_to_slot, style_ui).chain(),
                update_interaction,
            ),
        );
    }
}

#[derive(Component, Clone, Deref, DerefMut, Copy)]
pub struct ScopeEntity(Entity);
impl ScopeEntity {
    pub fn get(&self) -> Entity {
        return **self;
    }
}

#[derive(Component, Clone, Default)]
pub struct TemplateState {
    pub vars: HashMap<String, String>,
    pub props: HashMap<String, String>,
}

#[derive(Component, Clone, Default)]
pub struct StateSubscriber(Vec<Entity>);

impl TemplateState {
    pub fn try_set_prop(&mut self, key: &str, value: String) {
        _ = self.props.try_insert(key.to_string(), value);
    }
    pub fn try_set_val(&mut self, key: &str, value: String) {
        _ = self.vars.try_insert(key.to_string(), value);
    }
    pub fn set_prop(&mut self, key: &str, value: String) {
        self.props.insert(key.to_string(), value);
    }
    pub fn set_val(&mut self, key: &str, value: String) {
        self.vars.insert(key.to_string(), value);
    }
    pub fn get_prop(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }
    pub fn get_val(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
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

#[derive(Component, Deref, DerefMut)]
pub struct Tags(Vec<Tag>);

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

#[derive(Component, Deref, DerefMut, Default)]
pub struct PropertyDefintions(HashMap<String, String>);

impl PropertyDefintions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }
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
    pub handle: Handle<XNode>,
    pub node: NodeBundle,
    pub unbuild: UnbuildTag,
    pub unstyled: UnstyledTag,
    pub properties: PropertyDefintions,
    pub state: TemplateState,
}

fn update_interaction(
    mut cmd: Commands,
    mut nodes: Query<
        (
            Entity,
            &mut Style,
            &StyleAttributes,
            &Interaction,
            Option<&mut Text>,
        ),
        Changed<Interaction>,
    >,
    server: Res<AssetServer>,
) {
    nodes.iter_mut().for_each(
        |(entity, mut style, style_attr, interaction, mut maybe_text)| match interaction {
            Interaction::Pressed => {
                style_attr.iter().for_each(|attr| {
                    if let StyleAttr::Pressed(val) = attr {
                        val.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server);
                    }
                });
            }
            Interaction::Hovered => {
                style_attr.iter().for_each(|attr| {
                    if let StyleAttr::Hover(val) = attr {
                        val.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server);
                    }
                });
            }
            Interaction::None => {
                *style = Style::default();
                style_attr.iter().for_each(|attr| match attr {
                    StyleAttr::Hover(_) | StyleAttr::Pressed(_) => (),
                    any => any.apply(entity, &mut cmd, &mut style, &mut maybe_text, &server),
                });
            }
        },
    );
}

// what if a slot uses another include?
fn hotreload(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<XNode>>,
    templates: Query<(Entity, &Handle<XNode>)>,
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
                // let slots = find_sloted_children(entity, &children, &sloted_nodes, &templates);

                info!("found slots: {}", sloted_nodes.iter().count());

                // same include children are also replaced!!!
                let slots = sloted_nodes
                    .iter()
                    .flat_map(|(slot_entity, slot)| (slot.owner == entity).then_some(slot_entity))
                    .collect::<Vec<_>>();

                // check if there are already children in queue!

                if slots.len() > 0 {
                    let slot_holder = cmd.spawn_empty().push_children(&slots).id();
                    info!("preserving {} children for {entity}", slots.len());
                    cmd.entity(entity).insert(UnslotedChildren(slot_holder));
                } else {
                    info!("no children found");
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
                warn!("this node does not have a slot {entity}");
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

            info!("filled slot of {entity} successfull");

            cmd.entity(entity).remove::<UnslotedChildren>();
            cmd.entity(placeholder_entity).despawn_recursive();
            cmd.entity(*slot_holder).despawn();
        });
}

#[derive(Default)]
struct IdLookUpTable {
    ids: HashMap<String, Entity>,
    targets: HashMap<Entity, String>,
}

fn spawn_ui(
    mut cmd: Commands,
    mut unbuild: Query<
        (
            Entity,
            &Handle<XNode>,
            &mut PropertyDefintions,
            &mut TemplateState,
        ),
        With<UnbuildTag>,
    >,
    assets: Res<Assets<XNode>>,
    server: Res<AssetServer>,
    custom_comps: Res<ComponentBindings>,
) {
    unbuild
        .iter_mut()
        .for_each(|(root_entity, handle, mut defs, mut state)| {
            let Some(root_node) = assets.get(handle) else {
                return;
            };

            let mut subscriber = StateSubscriber::default();
            let mut id_table = IdLookUpTable::default();
            let scope = ScopeEntity(root_entity);

            // append build template vars & props here

            // add defaults on first call
            build_node(
                0,
                root_entity,
                scope,
                &root_node,
                &mut cmd,
                &assets,
                &server,
                &custom_comps,
                &mut id_table,
                &mut defs,
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

            cmd.entity(root_entity)
                .insert(subscriber)
                .remove::<UnbuildTag>();
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
    assets: &Assets<XNode>,
    server: &AssetServer,
    custom_comps: &ComponentBindings,
    id_table: &mut IdLookUpTable,
    defintions: &mut PropertyDefintions,
    state: &mut TemplateState,
    subscriber: &mut StateSubscriber,
) {
    // add any default properties on first node here
    if depth == 0 {
        node.attributes.iter().for_each(|attr| match attr {
            Attribute::PropertyDefinition(key, value) => {
                _ = defintions.try_insert(key.clone(), value.clone());
            }
            _ => (),
        });

        match &node.node_type {
            NodeType::Include | NodeType::Template | NodeType::Custom(_) => (),
            _ => {
                cmd.entity(entity).insert(scope.clone());
            }
        };
    } else {
        // dont touch include scopes
        // match ui_node.node_type {
        //     NodeType::Include | NodeType::Template | NodeType::Custom(_) => (),
        //     _ => {
        cmd.entity(entity).insert(scope.clone());
        // }
        // };
    }

    // compile properties
    let mut attributes = SortedAttributes::new(&node.attributes, &defintions);
    let style_attributes = StyleAttributes(attributes.styles.drain(..).collect());

    // any defintion not on inlucde/custom gets discarded
    let include_definitions = attributes.definitions.drain(..).fold(
        PropertyDefintions::default(),
        |mut m, (key, value)| {
            m.insert(key, value);
            m
        },
    );

    attributes.actions.drain(..).for_each(|action| {
        action.self_insert(cmd.entity(entity));
    });

    if let Some(id) = attributes.id {
        id_table.ids.insert(id, entity);
    }
    if let Some(target) = attributes.target {
        id_table.targets.insert(entity, target);
    }

    // ------------------
    if attributes.custom.len() > 0 {
        cmd.entity(entity).insert(Tags(
            attributes
                .custom
                .drain(..)
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
            cmd.entity(entity).insert((
                Name::new("Image"),
                ImageBundle {
                    image: UiImage::new(server.load(attributes.path.unwrap_or_default())),
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
                .map(|str| compile_content(str, defintions))
                .unwrap_or_default();

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
            let path = attributes.path.unwrap_or_default();
            let handle = server.load::<XNode>(path);

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
                        defintions,
                        state,
                        subscriber,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });

                cmd.entity(entity).insert((
                    UnslotedChildren(slot_holder),
                    handle,
                    include_definitions,
                    UnbuildTag,
                    NodeBundle::default(),
                    TemplateState::default(),
                ));
            }

            return;
        }
        NodeType::Slot => {
            cmd.entity(entity).insert((
                SlotPlaceholder { owner: scope.get() },
                NodeBundle::default(),
            ));
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
                        defintions,
                        state,
                        subscriber,
                    );
                    cmd.entity(slot_holder).add_child(child);
                });

                cmd.entity(entity).insert((
                    UnslotedChildren(slot_holder),
                    include_definitions,
                    UnbuildTag,
                ));
            }

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
            defintions,
            state,
            subscriber,
        );

        cmd.entity(entity).add_child(child);
    }
}

#[derive(Default)]
struct SortedAttributes {
    pub styles: Vec<StyleAttr>,
    pub actions: Vec<Action>,
    pub path: Option<String>,
    pub definitions: Vec<(String, String)>,
    pub target: Option<String>,
    pub id: Option<String>,
    pub custom: Vec<(String, String)>,
    pub code: Vec<AttrTokens>,
}

impl SortedAttributes {
    pub fn new(unsorted: &Vec<Attribute>, props: &PropertyDefintions) -> Self {
        let mut sorted = SortedAttributes::default();
        unsorted
            .iter()
            .cloned()
            .for_each(|attr| sorted.add_attr(attr, props));
        sorted
    }

    pub fn add_attr(&mut self, attr: Attribute, defs: &PropertyDefintions) {
        match attr {
            Attribute::Style(style) => self.styles.push(style),
            Attribute::Action(action) => self.actions.push(action),
            Attribute::Path(path) => self.path = Some(path),
            Attribute::PropertyDefinition(key, val) => self.definitions.push((key, val)),
            Attribute::Uncompiled(tokens) => {
                self.code.push(tokens.clone());
                // should no longer happen here, remove later
                if let Some(attr) = tokens.compile(defs) {
                    self.add_attr(attr, defs);
                };
            }
            Attribute::Target(target) => self.target = Some(target),
            Attribute::Id(id) => self.id = Some(id),
            Attribute::Tag(key, value) => self.custom.push((key, value)),
        };
    }
}

fn compile_content(input: &str, defs: &PropertyDefintions) -> String {
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

    if let Some(value) = defs.get(key.trim_end()) {
        compiled.push_str(value);
    }

    if input.len() > 0 {
        compiled.push_str(&compile_content(input, defs));
    }

    compiled
}

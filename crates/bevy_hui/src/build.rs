use crate::{
    compile::CompileContextEvent,
    data::{AttrTokens, HtmlTemplate, NodeType, XNode},
    prelude::ComponentBindings,
    styles::{HoverTimer, HtmlStyle, PressedTimer},
    util::SlotId,
};
use bevy::{prelude::*, utils::HashMap};
use nom::{
    bytes::complete::{is_not, tag, take_until},
    character::complete::multispace0,
    sequence::{delimited, preceded, tuple},
};
use std::time::Duration;

pub struct BuildPlugin;
impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (hotreload, spawn_ui, move_children_to_slot).chain());
        app.register_type::<TemplatePropertySubscriber>();
        app.register_type::<TemplateExpresions>();
        app.register_type::<TemplateProperties>();
        app.register_type::<TemplateScope>();
        app.register_type::<OnUiExit>();
        app.register_type::<OnUiEnter>();
        app.register_type::<OnUiPress>();
        app.register_type::<OnUiSpawn>();
        app.register_type::<UiTarget>();
        app.register_type::<UiId>();
        app.register_type::<SlotPlaceholder>();
        app.register_type::<UnslotedChildren>();
        app.register_type::<HtmlNode>();
    }
}

/// Holds the reference to the template root entity,
/// which owns the template properties
#[derive(Component, Clone, Deref, DerefMut, Copy, Reflect)]
#[reflect]
pub struct TemplateScope(Entity);

/// The property definition of a template,
/// this component can be found on the template root
/// entity, use `TemplateScope` (exists on all nodes, part of a template)
/// to get access to the root.
#[derive(Component, Debug, Clone, Default, Reflect, Deref, DerefMut)]
#[reflect]
pub struct TemplateProperties(HashMap<String, String>);

impl TemplateProperties {
    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.insert(key.to_string(), value.to_string());
        self
    }
}

/// Entites that need to be notified, when the
/// template properties change.
#[derive(Component, Clone, Default, Deref, DerefMut, Reflect)]
#[reflect]
pub struct TemplatePropertySubscriber(pub Vec<Entity>);

#[derive(Component)]
pub struct InsideSlot {
    owner: Entity,
}

#[derive(Component, Reflect)]
#[reflect]
pub struct SlotPlaceholder {
    owner: Entity,
}

/// ref to unresolved nodes that
/// need to move to the `<slot/>`
/// when the template is loaded.
#[derive(Component, Reflect)]
#[reflect]
pub struct UnslotedChildren(Entity);

/// entities subscribed to the owners interaction
/// component
#[derive(Component, DerefMut, Deref)]
pub struct InteractionObverser(Vec<Entity>);

/// unresolved expresssions that can be compiled
/// to a solid attribute
#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect]
pub struct TemplateExpresions(Vec<AttrTokens>);

/// Any attribute prefixed with `tag:my_tag="my_value"`
/// will be availble here.
#[derive(Component, Deref, DerefMut)]
pub struct Tags(HashMap<String, String>);

/// holds ref to the raw uncompiled text content
#[derive(Component, Deref, DerefMut)]
pub struct ContentId(SlotId);

/// the entities owned uid hashed as u64
#[derive(Component, Default, Hash, Deref, DerefMut, Reflect)]
#[reflect]
pub struct UiId(String);

/// the entity behind `id` in `target="id"`
#[derive(Component, DerefMut, Deref, Reflect)]
#[reflect]
pub struct UiTarget(pub Entity);

/// watch interaction of another entity
#[derive(Component, DerefMut, Deref, Reflect)]
#[reflect]
pub struct UiWatch(pub Entity);

#[derive(Component, Default)]
pub struct FullyBuild;

/// Eventlistener interaction transition to Hover
#[derive(Component, Deref, DerefMut, Reflect)]
#[reflect]
pub struct OnUiPress(pub Vec<String>);

/// Eventlistener on spawning node
#[derive(Component, DerefMut, Deref, Reflect)]
#[reflect]
pub struct OnUiSpawn(pub Vec<String>);

/// Eventlistener for interaction transition to Hover
#[derive(Component, DerefMut, Deref, Reflect)]
#[reflect]
pub struct OnUiEnter(pub Vec<String>);

/// Eventlistener for interaction transition to None
#[derive(Component, Deref, DerefMut, Reflect)]
#[reflect]
pub struct OnUiExit(pub Vec<String>);

/// Html Ui Node
/// pass it a handle, it will spawn an UI.
#[derive(Component, Default, Deref, DerefMut, Reflect)]
#[require(Node, TemplateProperties)]
#[reflect]
pub struct HtmlNode(pub Handle<HtmlTemplate>);

fn hotreload(
    mut cmd: Commands,
    mut events: EventReader<AssetEvent<HtmlTemplate>>,
    templates: Query<(Entity, &HtmlNode)>,
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
            .filter(|(_, html)| html.id() == *id)
            .for_each(|(entity, _)| {
                let slots = sloted_nodes
                    .iter()
                    .flat_map(|(slot_entity, slot)| (slot.owner == entity).then_some(slot_entity))
                    .collect::<Vec<_>>();

                if slots.len() > 0 {
                    let slot_holder = cmd.spawn_empty().add_children(&slots).id();
                    cmd.entity(entity).insert(UnslotedChildren(slot_holder));
                }

                cmd.entity(entity)
                    .despawn_descendants()
                    .retain::<KeepComps>();
            });
    });
}

#[derive(Bundle)]
struct KeepComps {
    pub parent: Parent,
    pub children: Children,
    pub ui: HtmlNode,
    pub unsloed: UnslotedChildren,
    pub slot: SlotPlaceholder,
    pub inside: InsideSlot,
    pub scope: TemplateScope,
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

fn spawn_ui(
    mut cmd: Commands,
    mut unbuild: Query<(Entity, &HtmlNode, &mut TemplateProperties), Without<FullyBuild>>,
    assets: Res<Assets<HtmlTemplate>>,
    server: Res<AssetServer>,
    custom_comps: Res<ComponentBindings>,
) {
    unbuild
        .iter_mut()
        .for_each(|(root_entity, handle, mut state)| {
            let Some(template) = assets.get(&**handle) else {
                return;
            };

            template.properties.iter().for_each(|(key, val)| {
                _ = state.try_insert(key.to_owned(), val.clone());
            });

            let mut builder = TemplateBuilder::new(
                root_entity,
                cmd.reborrow(),
                &server,
                &custom_comps,
                &template,
            );

            if let Some(node) = template.root.first() {
                builder.build_tree(node);
                builder.finalize_relations();
                cmd.trigger_targets(CompileContextEvent, root_entity);
            } else {
                warn!("template has no root node!");
            }

            if template.root.len() > 1 {
                warn!("templates currently only support one root node, ignoring the rest");
            }
        });
}

struct TemplateBuilder<'w, 's> {
    cmd: Commands<'w, 's>,
    server: &'w AssetServer,
    scope: Entity,
    comps: &'w ComponentBindings,
    subscriber: TemplatePropertySubscriber,
    ids: HashMap<String, Entity>,
    targets: HashMap<Entity, String>,
    watch: HashMap<String, Vec<Entity>>,
    template: &'w HtmlTemplate,
}

impl<'w, 's> TemplateBuilder<'w, 's> {
    pub fn new(
        scope: Entity,
        cmd: Commands<'w, 's>,
        server: &'w AssetServer,
        comps: &'w ComponentBindings,
        template: &'w HtmlTemplate,
    ) -> Self {
        Self {
            cmd,
            scope,
            server,
            comps,
            template,
            subscriber: Default::default(),
            ids: Default::default(),
            targets: Default::default(),
            watch: Default::default(),
        }
    }

    pub fn finalize_relations(mut self) {
        self.ids.iter().for_each(|(id_string, entity)| {
            self.cmd.entity(*entity).insert(UiId(id_string.clone()));
        });

        self.targets
            .iter()
            .for_each(|(entity, target_id)| match self.ids.get(target_id) {
                Some(tar) => {
                    self.cmd.entity(*entity).insert(UiTarget(*tar));
                }
                None => warn!("target `{target_id}` not found for entity {entity}"),
            });

        self.watch
            .iter()
            .for_each(|(target_str, obs_list)| match self.ids.get(target_str) {
                Some(to_observe) => {
                    self.cmd
                        .entity(*to_observe)
                        .insert(InteractionObverser(obs_list.clone()));
                }
                None => warn!("undefined watch target `{target_str}`"),
            });

        self.cmd
            .entity(self.scope)
            .insert((std::mem::take(&mut self.subscriber), FullyBuild));
    }

    pub fn build_tree(&mut self, root: &XNode) {
        self.build_node(self.scope, root);
    }

    fn build_node(&mut self, entity: Entity, node: &XNode) {
        let styles = HtmlStyle::from(node.styles.clone());

        // ----------------------
        // timers
        self.cmd
            .entity(entity)
            .insert(PressedTimer::new(Duration::from_secs_f32(
                styles.computed.delay.max(0.01),
            )))
            .insert(HoverTimer::new(Duration::from_secs_f32(
                styles.computed.delay.max(0.01),
            )));

        if entity != self.scope {
            self.cmd.entity(entity).insert(TemplateScope(self.scope));
        }

        // ----------------------
        //register prop listner
        if node.uncompiled.len() > 0 {
            self.cmd.entity(entity).insert(TemplateExpresions(
                node.uncompiled.iter().cloned().collect(),
            ));
            self.subscriber.push(entity);
        }

        // ----------------------
        //tags
        if node.tags.len() > 0 {
            self.cmd.entity(entity).insert(Tags(node.tags.clone()));
        }

        // ----------------------
        // connections
        if let Some(id) = &node.id {
            self.ids.insert(id.clone(), entity);
        }
        if let Some(target) = &node.target {
            self.targets.insert(entity, target.clone());
        }

        if let Some(watch) = &node.watch {
            match self.watch.get_mut(watch) {
                Some(list) => {
                    list.push(entity);
                }
                None => {
                    self.watch.insert(watch.clone(), vec![entity]);
                }
            };
        }

        // ----------------------
        // events
        node.event_listener.iter().for_each(|listener| {
            listener.clone().self_insert(self.cmd.entity(entity));
        });

        // ----------------------
        // dirty outline
        if let Some(outline) = styles.computed.outline.as_ref() {
            self.cmd.entity(entity).insert(outline.clone());
        }

        match &node.node_type {
            // --------------------------------
            // div node
            NodeType::Node => {
                self.cmd.entity(entity).insert((Node::default(), styles));
            }
            // --------------------------------
            // spawn image
            NodeType::Image => {
                self.cmd.entity(entity).insert((
                    UiImage {
                        image: node
                            .src
                            .as_ref()
                            .map(|path| self.server.load(path))
                            .unwrap_or_default(),
                        image_mode: styles
                            .computed
                            .image_mode
                            .as_ref()
                            .cloned()
                            .unwrap_or_default(),
                        ..default()
                    },
                    styles,
                ));
            }
            // --------------------------------
            // spawn image
            NodeType::Text => {
                let content = self
                    .template
                    .content
                    .get(node.content_id)
                    .map(|t| t.trim().to_string())
                    .unwrap_or_default();

                if is_templated(&content) {
                    self.cmd.entity(entity).insert(ContentId(node.content_id));
                    self.subscriber.push(entity);
                }

                self.cmd.entity(entity).insert((Text(content), styles));
            }
            // --------------------------------
            // spawn button
            NodeType::Button => {
                self.cmd.entity(entity).insert((Button, styles));
            }
            // --------------------------------
            // spawn include
            NodeType::Include => {
                // mark children
                let template: Handle<HtmlTemplate> = node
                    .src
                    .as_ref()
                    .map(|path| self.server.load(path))
                    .unwrap_or_default();

                let entity = self
                    .cmd
                    .entity(entity)
                    .insert((
                        Node::default(),
                        HtmlNode(template),
                        TemplateProperties(node.defs.clone()),
                    ))
                    .id();

                if node.uncompiled.len() > 0 {
                    self.subscriber.push(entity);
                }

                if node.children.len() > 0 {
                    let slot_holder = self.cmd.spawn(Node::default()).id();
                    for child_node in node.children.iter() {
                        let child_entity = self.cmd.spawn_empty().id();
                        self.build_node(child_entity, child_node);
                        self.cmd.entity(slot_holder).add_child(child_entity);
                    }
                    self.cmd
                        .entity(entity)
                        .insert(UnslotedChildren(slot_holder));
                }
                return;
            }
            NodeType::Custom(custom) => {
                // mark children
                self.comps.try_spawn(custom, entity, &mut self.cmd);
                if node.children.len() > 0 {
                    let slot_holder = self.cmd.spawn(Node::default()).id();
                    for child_node in node.children.iter() {
                        let child_entity = self.cmd.spawn_empty().id();
                        self.build_node(child_entity, child_node);
                        self.cmd.entity(slot_holder).add_child(child_entity);
                    }

                    self.cmd
                        .entity(entity)
                        .insert((UnslotedChildren(slot_holder),));
                }

                self.cmd
                    .entity(entity)
                    .insert(TemplateProperties(node.defs.clone()));

                if node.uncompiled.len() > 0 {
                    self.subscriber.push(entity);
                }

                return;
            }
            // --------------------------------
            // spawn slot
            NodeType::Slot => {
                self.cmd
                    .entity(entity)
                    .insert((Node::default(), SlotPlaceholder { owner: self.scope }));
            }
            // --------------------------------
            // don't render
            NodeType::Template | NodeType::Property => {
                return;
            }
        };

        for child in node.children.iter() {
            let child_entity = self.cmd.spawn_empty().id();
            self.build_node(child_entity, child);
            self.cmd.entity(entity).add_child(child_entity);
        }
    }
}

//@todo:dirty AF
pub fn is_templated(input: &str) -> bool {
    let parts: Result<(&str, (&str, &str)), nom::Err<nom::error::Error<&str>>> = tuple((
        take_until("{"),
        delimited(tag("{"), preceded(multispace0, is_not("}")), tag("}")),
    ))(input);

    parts.is_ok()
}

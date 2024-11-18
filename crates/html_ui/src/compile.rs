use crate::{
    build::{
        ContentId, HtmlNode, TemplateExpresions, TemplateProperties, TemplatePropertySubscriber,
        TemplateScope,
    },
    data::HtmlTemplate,
    styles::HtmlStyle,
};
use bevy::prelude::*;
use nom::{
    bytes::complete::{is_not, tag, take_until},
    character::complete::multispace0,
    sequence::{delimited, preceded, tuple},
};

pub struct CompilePlugin;
impl Plugin for CompilePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CompileNodeEvent>();
        app.add_event::<CompileContextEvent>();
        app.add_event::<CompileContentEvent>();
        app.add_observer(compile_node);
        app.add_observer(compile_context);
        app.add_observer(compile_text);
    }
}

#[derive(Event)]
pub struct CompileContentEvent;

fn compile_text(
    trigger: Trigger<CompileContentEvent>,
    mut nodes: Query<(&ContentId, &TemplateScope, &mut Text)>,
    root: Query<(&HtmlNode, &TemplateProperties)>,
    templates: Res<Assets<HtmlTemplate>>,
) {
    let entity = trigger.entity();
    let Ok((content_id, scope, mut text)) = nodes.get_mut(entity) else {
        warn!("trying to compile content for {entity}, that does not have any");
        return;
    };

    let Some((template, props)) = root
        .get(**scope)
        .ok()
        .map(|(handle, props)| templates.get(&**handle).map(|d| (d, props)))
        .flatten()
    else {
        warn!("{entity} has no scope!");
        return;
    };

    _ = template
        .content
        .get(**content_id)
        .map(|raw| compile_content(raw, &props))
        .map(|compiled| **text = compiled);
}

#[derive(Event)]
pub struct CompileNodeEvent;
fn compile_node(
    trigger: Trigger<CompileNodeEvent>,
    mut cmd: Commands,
    mut nodes: Query<(&mut HtmlStyle, &TemplateScope)>,
    mut images: Query<&mut UiImage>,
    expressions: Query<&TemplateExpresions>,
    contexts: Query<&TemplateProperties>,
    server: Res<AssetServer>,
) {
    let entity = trigger.entity();
    let Ok((mut node_style, scope)) = nodes.get_mut(entity) else {
        // unbuild nodes also complain
        // warn!("Trying to compile a non ui node");
        return;
    };

    let Some(context) = contexts.get(**scope).ok() else {
        warn!("Node has no context scope");
        return;
    };

    if let Ok(expressions) = expressions.get(entity) {
        expressions
            .iter()
            .for_each(|expr| match expr.compile(context) {
                Some(compiled) => {
                    // info!("compiled {:?}", compiled);
                    match compiled {
                        crate::data::Attribute::Style(style_attr) => {
                            node_style.add_style_attr(style_attr)
                        }
                        crate::data::Attribute::Action(action) => {
                            action.self_insert(cmd.entity(entity))
                        }
                        crate::data::Attribute::Path(path) => {
                            _ = images.get_mut(entity).map(|mut img| {
                                img.image = server.load(path);
                            });
                        }
                        rest => {
                            warn!("attribute of this kind cannot be dynamic `{:?}`", rest);
                        }
                    };
                }
                None => {
                    warn!("expression failed to compile `{:?}`", expr);
                    return;
                }
            });
    }
}

#[derive(Event)]
pub struct CompileContextEvent;

fn compile_context(
    trigger: Trigger<CompileContextEvent>,
    expressions: Query<(&TemplateExpresions, Option<&TemplateScope>)>,
    text_nodes: Query<(), With<ContentId>>,
    subscriber: Query<&TemplatePropertySubscriber>,
    mut context: Query<&mut TemplateProperties>,
    mut cmd: Commands,
) {
    let entity = trigger.entity();
    if let Ok((expressions, scope)) = expressions.get(entity) {
        // compile
        if let Some(parent_context) = scope.map(|s| context.get(**s).ok()).flatten() {
            let mut compiled_defintions = vec![];
            for expr in expressions.iter() {
                match expr.compile(parent_context) {
                    Some(compiled) => match compiled {
                        crate::data::Attribute::PropertyDefinition(key, value) => {
                            compiled_defintions.push((key, value));
                        }
                        _ => {
                            error!("cannot compile to unimplementd attribute `{:?}`", compiled);
                        }
                    },
                    None => {
                        error!("{:#?}", expr);
                    }
                }
            }
            _ = context.get_mut(entity).map(|mut context| {
                compiled_defintions.drain(..).for_each(|(key, value)| {
                    context.insert(key, value);
                });
            });
        };
    };

    if let Ok(subs) = subscriber.get(entity) {
        for sub in subs.iter() {
            if *sub != entity && context.get(*sub).is_ok() {
                cmd.trigger_targets(CompileContextEvent, *sub);
            } else {
                cmd.trigger_targets(CompileNodeEvent, *sub);
            }
            if text_nodes.get(*sub).is_ok() {
                cmd.trigger_targets(CompileContentEvent, *sub);
            }
        }
    }
}

pub(crate) fn compile_content(input: &str, defs: &TemplateProperties) -> String {
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

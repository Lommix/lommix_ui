use std::time::Duration;

use bevy::prelude::*;

use crate::{build::InteractionObverser, data::StyleAttr};

pub struct TransitionPlugin;
impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_transitions, update_interactions, update_node_style),
        );

        app.register_type::<PressedTimer>();
        app.register_type::<HoverTimer>();
        app.register_type::<ComputedStyle>();
        app.register_type::<NodeStyle>();
    }
}

/*
tyl
 A -> B
 A <- B

 Vals

 Animation: list of Vec<style->style>
*/
fn update_interactions(
    mut cmd: Commands,
    interactions: Query<(Entity, &Interaction, &NodeStyle), Changed<Interaction>>,
    observer: Query<&InteractionObverser>,
) {
    interactions
        .iter()
        .for_each(|(entity, interaction, node_style)| {
            let iter = observer
                .get(entity)
                .map(|obs| obs.iter())
                .unwrap_or_default()
                .chain(std::iter::once(&entity));

            let delay = Duration::from_secs_f32(node_style.regular.delay);
            match interaction {
                Interaction::Pressed => {
                    iter.for_each(|ent| {
                        cmd.entity(*ent).insert(PressedTimer::new(delay));
                    });
                }
                Interaction::Hovered => {
                    iter.for_each(|ent| {
                        cmd.entity(*ent).insert(HoverTimer::new(delay));
                    });
                }
                Interaction::None => {
                    iter.for_each(|ent| {
                        cmd.entity(*ent)
                            .remove::<PressedTimer>()
                            .remove::<HoverTimer>();
                    });
                }
            }
        });
}

// @todo: broadcast update
fn update_transitions(
    mut pressed_timers: Query<(Entity, &mut PressedTimer)>,
    mut hover_timers: Query<(Entity, &mut HoverTimer)>,
    time: Res<Time>,
) {
    hover_timers.iter_mut().for_each(|(_, mut trans)| {
        trans.tick(time.delta());
    });
    pressed_timers.iter_mut().for_each(|(_, mut trans)| {
        trans.tick(time.delta());
    });
}

// @todo: split
fn update_node_style(
    mut nodes: Query<(Entity, &mut Style, &NodeStyle)>,
    mut bg: Query<&mut BackgroundColor>,
    mut bradius: Query<&mut BorderRadius>,
    mut bcolor: Query<&mut BorderColor>,
    mut texts: Query<&mut Text>,
    server: Res<AssetServer>,
    hover_timer: Query<&HoverTimer>,
    pressed_timer: Query<&PressedTimer>,
) {
    nodes
        .iter_mut()
        .for_each(|(entity, mut style, node_styles)| {
            let pressed_timer = pressed_timer.get(entity).ok();
            let hover_timer = hover_timer.get(entity).ok();
            let bg = bg.get_mut(entity).ok();
            let bradius = bradius.get_mut(entity).ok();
            let bcolor = bcolor.get_mut(entity).ok();
            let text = texts.get_mut(entity).ok();

            node_styles.apply_style(
                &mut style,
                bg,
                bradius,
                bcolor,
                text,
                &server,
                hover_timer,
                pressed_timer,
            );
        });
}

#[derive(Component, Reflect, Default, Deref, DerefMut)]
#[reflect]
pub struct PressedTimer(Timer);
impl PressedTimer {
    pub fn new(d: Duration) -> Self {
        Self(Timer::new(d, TimerMode::Once))
    }
}

#[derive(Component, Default, Reflect, Deref, DerefMut)]
#[reflect]
pub struct HoverTimer(Timer);
impl HoverTimer {
    pub fn new(d: Duration) -> Self {
        Self(Timer::new(d, TimerMode::Once))
    }
}

#[derive(Debug, Reflect)]
#[reflect]
pub struct ComputedStyle {
    style: Style,
    border_color: Color,
    border_radius: UiRect,
    background: Color,
    font: Handle<Font>,
    font_size: f32,
    font_color: Color,
    delay: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            style: Style::default(),
            border_color: Color::NONE,
            border_radius: UiRect::default(),
            background: Color::NONE,
            font: Handle::default(),
            font_size: 12.,
            font_color: Color::WHITE,
            delay: 0.,
        }
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect]
pub struct NodeStyle {
    regular: ComputedStyle,
    #[reflect(ignore)]
    hover: Vec<StyleAttr>,
    #[reflect(ignore)]
    pressed: Vec<StyleAttr>,
}

impl NodeStyle {
    pub fn apply_style(
        &self,
        style: &mut Style,
        mut bg: Option<Mut<BackgroundColor>>,
        mut bradius: Option<Mut<BorderRadius>>,
        mut bcolor: Option<Mut<BorderColor>>,
        mut text: Option<Mut<Text>>,
        server: &AssetServer,
        hover_timer: Option<&HoverTimer>,
        pressed_timer: Option<&PressedTimer>,
    ) {
        style.clone_from(&self.regular.style);
        bg.as_mut().map(|bg| bg.0 = self.regular.background);

        bcolor
            .as_mut()
            .map(|bcolor| bcolor.0 = self.regular.border_color);

        bradius.as_mut().map(|bradius| {
            bradius.top_left = self.regular.border_radius.top;
            bradius.top_right = self.regular.border_radius.right;
            bradius.bottom_right = self.regular.border_radius.bottom;
            bradius.bottom_left = self.regular.border_radius.left;
        });

        text.as_mut().map(|txt| {
            txt.sections.iter_mut().for_each(|s| {
                s.style.font_size = self.regular.font_size;
                s.style.color = self.regular.font_color;
                s.style.font = self.regular.font.clone();
            });
        });

        hover_timer.map(|timer| {
            if timer.fraction() > 0.01 {
                for attr in self.hover.iter() {
                    apply_lerp_style(
                        attr,
                        timer.fraction(),
                        &self.regular,
                        style,
                        &mut bg,
                        &mut bradius,
                        &mut bcolor,
                        &mut text,
                        server,
                    );
                }
            }
        });

        pressed_timer.map(|timer| {
            if timer.fraction() > 0.01 {
                for attr in self.hover.iter() {
                    apply_lerp_style(
                        attr,
                        timer.fraction(),
                        &self.regular,
                        style,
                        &mut bg,
                        &mut bradius,
                        &mut bcolor,
                        &mut text,
                        server,
                    );
                }
            }
        });
    }

    pub fn add_style_attr(&mut self, attr: StyleAttr) {
        match attr {
            StyleAttr::Hover(style) => {
                let style = *style;
                match self
                    .hover
                    .iter()
                    .position(|s| std::mem::discriminant(s) == std::mem::discriminant(&style))
                {
                    Some(index) => self.hover.insert(index, style),
                    None => self.hover.push(style),
                }
            }
            StyleAttr::Pressed(style) => {
                let style = *style;
                match self
                    .pressed
                    .iter()
                    .position(|s| std::mem::discriminant(s) == std::mem::discriminant(&style))
                {
                    Some(index) => self.hover.insert(index, style),
                    None => self.pressed.push(style),
                }
            }
            StyleAttr::Display(display) => self.regular.style.display = display,
            StyleAttr::Position(position_type) => self.regular.style.position_type = position_type,
            StyleAttr::Overflow(overflow) => self.regular.style.overflow = overflow,
            StyleAttr::Left(val) => self.regular.style.left = val,
            StyleAttr::Right(val) => self.regular.style.right = val,
            StyleAttr::Top(val) => self.regular.style.top = val,
            StyleAttr::Bottom(val) => self.regular.style.bottom = val,
            StyleAttr::Width(val) => self.regular.style.width = val,
            StyleAttr::Height(val) => self.regular.style.height = val,
            StyleAttr::MinWidth(val) => self.regular.style.min_width = val,
            StyleAttr::MinHeight(val) => self.regular.style.min_height = val,
            StyleAttr::MaxWidth(val) => self.regular.style.max_width = val,
            StyleAttr::MaxHeight(val) => self.regular.style.min_height = val,
            StyleAttr::AspectRatio(f) => self.regular.style.aspect_ratio = Some(f),
            StyleAttr::AlignItems(align_items) => self.regular.style.align_items = align_items,
            StyleAttr::JustifyItems(justify_items) => {
                self.regular.style.justify_items = justify_items
            }
            StyleAttr::AlignSelf(align_self) => self.regular.style.align_self = align_self,
            StyleAttr::JustifySelf(justify_self) => self.regular.style.justify_self = justify_self,
            StyleAttr::AlignContent(align_content) => {
                self.regular.style.align_content = align_content
            }
            StyleAttr::JustifyContent(justify_content) => {
                self.regular.style.justify_content = justify_content
            }
            StyleAttr::Margin(ui_rect) => self.regular.style.margin = ui_rect,
            StyleAttr::Padding(ui_rect) => self.regular.style.padding = ui_rect,
            StyleAttr::Border(ui_rect) => self.regular.style.border = ui_rect,
            StyleAttr::BorderColor(color) => self.regular.border_color = color,
            StyleAttr::BorderRadius(ui_rect) => self.regular.border_radius = ui_rect,
            StyleAttr::FlexDirection(flex_direction) => {
                self.regular.style.flex_direction = flex_direction
            }
            StyleAttr::FlexWrap(flex_wrap) => self.regular.style.flex_wrap = flex_wrap,
            StyleAttr::FlexGrow(f) => self.regular.style.flex_grow = f,
            StyleAttr::FlexShrink(f) => self.regular.style.flex_shrink = f,
            StyleAttr::FlexBasis(val) => self.regular.style.flex_basis = val,
            StyleAttr::RowGap(val) => self.regular.style.row_gap = val,
            StyleAttr::ColumnGap(val) => self.regular.style.column_gap = val,
            StyleAttr::GridAutoFlow(grid_auto_flow) => {
                self.regular.style.grid_auto_flow = grid_auto_flow
            }
            StyleAttr::GridTemplateRows(vec) => self.regular.style.grid_template_rows = vec,
            StyleAttr::GridTemplateColumns(vec) => self.regular.style.grid_template_columns = vec,
            StyleAttr::GridAutoRows(vec) => self.regular.style.grid_auto_rows = vec,
            StyleAttr::GridAutoColumns(vec) => self.regular.style.grid_auto_columns = vec,
            StyleAttr::GridRow(grid_placement) => self.regular.style.grid_row = grid_placement,
            StyleAttr::GridColumn(grid_placement) => {
                self.regular.style.grid_column = grid_placement
            }
            StyleAttr::Direction(direction) => self.regular.style.direction = direction,
            StyleAttr::FontSize(f) => self.regular.font_size = f,
            StyleAttr::FontColor(color) => self.regular.font_color = color,
            StyleAttr::Background(color) => self.regular.background = color,
            StyleAttr::Delay(f) => self.regular.delay = f,
            StyleAttr::Font(_) => todo!(),
        };
    }
}

fn apply_lerp_style(
    attr: &StyleAttr,
    ratio: f32,
    default: &ComputedStyle,
    style: &mut Style,
    bg: &mut Option<Mut<BackgroundColor>>,
    bradius: &mut Option<Mut<BorderRadius>>,
    bcolor: &mut Option<Mut<BorderColor>>,
    text: &mut Option<Mut<Text>>,
    server: &AssetServer,
) {
    match attr {
        StyleAttr::Display(display) => style.display = *display,
        StyleAttr::Position(position_type) => style.position_type = *position_type,
        StyleAttr::Overflow(overflow) => style.overflow = *overflow,
        StyleAttr::Left(val) => style.left = lerp_val(&default.style.left, val, ratio),
        StyleAttr::Right(val) => style.right = lerp_val(&default.style.right, val, ratio),
        StyleAttr::Top(val) => style.top = lerp_val(&default.style.top, val, ratio),
        StyleAttr::Bottom(val) => style.bottom = lerp_val(&default.style.bottom, val, ratio),
        StyleAttr::Width(val) => style.width = lerp_val(&default.style.width, val, ratio),
        StyleAttr::Height(val) => style.height = lerp_val(&default.style.height, val, ratio),
        StyleAttr::MinWidth(val) => {
            style.min_width = lerp_val(&default.style.min_width, val, ratio)
        }
        StyleAttr::MinHeight(val) => {
            style.min_height = lerp_val(&default.style.min_height, val, ratio)
        }
        StyleAttr::MaxWidth(val) => {
            style.max_width = lerp_val(&default.style.max_width, val, ratio)
        }
        StyleAttr::MaxHeight(val) => {
            style.max_height = lerp_val(&default.style.max_height, val, ratio)
        }
        StyleAttr::AspectRatio(f) => {
            style.aspect_ratio = default.style.aspect_ratio.map(|a| a.lerp(*f, ratio))
        }
        StyleAttr::AlignItems(align_items) => style.align_items = *align_items,
        StyleAttr::JustifyItems(justify_items) => style.justify_items = *justify_items,
        StyleAttr::AlignSelf(align_self) => style.align_self = *align_self,
        StyleAttr::JustifySelf(justify_self) => style.justify_self = *justify_self,
        StyleAttr::AlignContent(align_content) => style.align_content = *align_content,
        StyleAttr::JustifyContent(justify_content) => style.justify_content = *justify_content,
        StyleAttr::Margin(ui_rect) => {
            style.margin = lerp_rect(&default.style.margin, ui_rect, ratio)
        }
        StyleAttr::Padding(ui_rect) => {
            style.padding = lerp_rect(&default.style.padding, ui_rect, ratio)
        }
        StyleAttr::Border(ui_rect) => {
            style.border = lerp_rect(&default.style.border, ui_rect, ratio)
        }
        StyleAttr::BorderColor(color) => {
            bcolor
                .as_mut()
                .map(|bcolor| bcolor.0 = lerp_color(&default.border_color, color, ratio));
        }
        StyleAttr::BorderRadius(ui_rect) => {
            bradius.as_mut().map(|bradius| {
                bradius.top_left = lerp_val(&default.border_radius.top, &ui_rect.top, ratio);
                bradius.top_right = lerp_val(&default.border_radius.right, &ui_rect.right, ratio);
                bradius.bottom_right =
                    lerp_val(&default.border_radius.bottom, &ui_rect.bottom, ratio);
                bradius.bottom_left = lerp_val(&default.border_radius.left, &ui_rect.left, ratio);
            });
        }
        StyleAttr::FlexDirection(flex_direction) => style.flex_direction = *flex_direction,
        StyleAttr::FlexWrap(flex_wrap) => style.flex_wrap = *flex_wrap,
        StyleAttr::FlexGrow(g) => style.flex_grow = default.style.flex_grow.lerp(*g, ratio),
        StyleAttr::FlexShrink(s) => style.flex_shrink = default.style.flex_shrink.lerp(*s, ratio),
        StyleAttr::FlexBasis(val) => {
            style.flex_basis = lerp_val(&default.style.flex_basis, val, ratio)
        }
        StyleAttr::RowGap(val) => style.row_gap = lerp_val(&default.style.row_gap, val, ratio),
        StyleAttr::ColumnGap(val) => {
            style.column_gap = lerp_val(&default.style.max_height, val, ratio)
        }
        StyleAttr::GridAutoFlow(grid_auto_flow) => style.grid_auto_flow = *grid_auto_flow,
        StyleAttr::GridTemplateRows(vec) => style.grid_template_rows = vec.clone(),
        StyleAttr::GridTemplateColumns(vec) => style.grid_template_columns = vec.clone(),
        StyleAttr::GridAutoRows(vec) => style.grid_auto_rows = vec.clone(),
        StyleAttr::GridAutoColumns(vec) => style.grid_auto_columns = vec.clone(),
        StyleAttr::GridRow(grid_placement) => style.grid_row = *grid_placement,
        StyleAttr::GridColumn(grid_placement) => style.grid_column = *grid_placement,
        StyleAttr::Direction(direction) => style.direction = *direction,
        StyleAttr::Background(color) => {
            bg.as_mut()
                .map(|bg| bg.0 = lerp_color(&default.border_color, color, ratio));
        }
        StyleAttr::FontColor(color) => {
            text.as_mut().map(|txt| {
                txt.sections.iter_mut().for_each(|sec| {
                    sec.style.color = *color;
                });
            });
        }
        StyleAttr::FontSize(s) => {
            text.as_mut().map(|txt| {
                txt.sections.iter_mut().for_each(|sec| {
                    sec.style.font_size = *s;
                });
            });
        }
        StyleAttr::Font(h) => {
            text.as_mut().map(|txt| {
                txt.sections.iter_mut().for_each(|sec| {
                    sec.style.font = server.load(h);
                });
            });
        }
        _ => (),
        // StyleAttr::Hover(style_attr) => todo!(),
        // StyleAttr::Pressed(style_attr) => todo!(),
        // StyleAttr::Delay(_) => todo!(),
    }
}

impl From<Vec<StyleAttr>> for NodeStyle {
    fn from(mut styles: Vec<StyleAttr>) -> Self {
        let mut out = NodeStyle::default();
        for style in styles.drain(..) {
            out.add_style_attr(style);
        }
        out
    }
}

fn lerp_color(start: &Color, end: &Color, ratio: f32) -> Color {
    let lin = start
        .to_linear()
        .to_vec4()
        .lerp(end.to_linear().to_vec4(), ratio);

    Color::LinearRgba(LinearRgba::from_vec4(lin))
}

fn lerp_rect(start: &UiRect, end: &UiRect, ratio: f32) -> UiRect {
    UiRect::new(
        lerp_val(&start.left, &end.left, ratio),
        lerp_val(&start.right, &end.right, ratio),
        lerp_val(&start.top, &end.top, ratio),
        lerp_val(&start.bottom, &end.bottom, ratio),
    )
}

fn lerp_val(start: &Val, end: &Val, ratio: f32) -> Val {
    match (start, end) {
        (Val::Percent(start), Val::Percent(end)) => {
            Val::Percent((end - start).mul_add(ratio, *start))
        }
        (Val::Px(start), Val::Px(end)) => Val::Px((end - start).mul_add(ratio, *start)),
        _ => *start,
    }
}

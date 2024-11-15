use crate::{build::InteractionObverser, data::StyleAttr};
use bevy::prelude::*;
use interpolation::Ease;
use std::time::Duration;

pub struct TransitionPlugin;
impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (continues_interaction_checking, update_node_style));
        app.register_type::<PressedTimer>();
        app.register_type::<HoverTimer>();
        app.register_type::<ComputedStyle>();
        app.register_type::<NodeStyle>();
    }
}

#[derive(Component, Default, Reflect)]
#[reflect]
pub struct InteractionTimer {
    elapsed: Duration,
    max: Duration,
}

#[derive(Component)]
pub struct UiActive;

impl InteractionTimer {
    pub fn new(max: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            max,
        }
    }

    pub fn fraction(&self) -> f32 {
        self.elapsed.div_duration_f32(self.max)
    }

    pub fn forward(&mut self, delta: Duration) {
        self.elapsed = self
            .elapsed
            .checked_add(delta)
            .map(|d| d.min(self.max))
            .unwrap_or(self.elapsed);
    }

    pub fn backward(&mut self, delta: Duration) {
        self.elapsed = self.elapsed.checked_sub(delta).unwrap_or(Duration::ZERO);
    }
}

// @mvp
// something is off with interactions of transforming nodes
fn continues_interaction_checking(
    interactions: Query<(Entity, &Interaction), With<NodeStyle>>,
    mut hovers: Query<&mut HoverTimer>,
    mut presseds: Query<&mut PressedTimer>,
    observer: Query<&InteractionObverser>,
    time: Res<Time<Real>>,
) {
    interactions.iter().for_each(|(entity, interaction)| {
        let subs = observer
            .get(entity)
            .map(|obs| obs.iter())
            .unwrap_or_default()
            .chain(std::iter::once(&entity));

        match interaction {
            Interaction::Pressed => {
                // ++ pressed ++ hover
                subs.for_each(|sub| {
                    if let (Ok(mut htimer), Ok(mut ptimer)) =
                        (hovers.get_mut(*sub), presseds.get_mut(*sub))
                    {
                        ptimer.forward(time.delta());
                        htimer.forward(time.delta());
                    } else {
                        warn!("non interacting node obsering `{sub}`")
                    }
                });
            }
            Interaction::Hovered => {
                // ++ hover -- pressed
                subs.for_each(|sub| {
                    if let (Ok(mut htimer), Ok(mut ptimer)) =
                        (hovers.get_mut(*sub), presseds.get_mut(*sub))
                    {
                        ptimer.backward(time.delta());
                        htimer.forward(time.delta());
                    } else {
                        warn!("non interacting node obsering `{sub}`")
                    }
                });
            }
            Interaction::None => {
                // -- hover --pressed
                subs.for_each(|sub| {
                    if let (Ok(mut htimer), Ok(mut ptimer)) =
                        (hovers.get_mut(*sub), presseds.get_mut(*sub))
                    {
                        ptimer.backward(time.delta());
                        htimer.backward(time.delta());
                    } else {
                        warn!("non interacting node obsering `{sub}`")
                    }
                });
            }
        };
    });
}

// @todo: split, event based updates
fn update_node_style(
    mut nodes: Query<(Entity, &mut Node, &NodeStyle, Option<&UiActive>)>,
    mut bg: Query<&mut BackgroundColor>,
    mut bradius: Query<&mut BorderRadius>,
    mut bcolor: Query<&mut BorderColor>,
    mut text_fonts: Query<&mut TextFont>,
    mut text_colors: Query<&mut TextColor>,
    server: Res<AssetServer>,
    hover_timer: Query<&HoverTimer>,
    pressed_timer: Query<&PressedTimer>,
) {
    nodes
        .iter_mut()
        .for_each(|(entity, mut style, node_styles, active_state)| {
            let pressed_timer = pressed_timer.get(entity).ok();
            let hover_timer = hover_timer.get(entity).ok();
            let bg = bg.get_mut(entity).ok();
            let bradius = bradius.get_mut(entity).ok();
            let bcolor = bcolor.get_mut(entity).ok();
            let text_font = text_fonts.get_mut(entity).ok();
            let text_color = text_colors.get_mut(entity).ok();

            node_styles.apply_style(
                &mut style,
                active_state.is_some(),
                bg,
                bradius,
                bcolor,
                text_color,
                text_font,
                &server,
                hover_timer,
                pressed_timer,
            );
        });
}

#[derive(Component, Reflect, Default, Deref, DerefMut)]
#[reflect]
pub struct PressedTimer(InteractionTimer);

impl PressedTimer {
    pub fn new(d: Duration) -> Self {
        Self(InteractionTimer::new(d))
    }
}

#[derive(Component, Default, Reflect, Deref, DerefMut)]
#[reflect]
pub struct HoverTimer(InteractionTimer);

impl HoverTimer {
    pub fn new(d: Duration) -> Self {
        Self(InteractionTimer::new(d))
    }
}

#[derive(Debug, Reflect)]
#[reflect]
pub struct ComputedStyle {
    pub style: Node,
    pub border_color: Color,
    pub border_radius: UiRect,
    pub image_scale_mode: Option<NodeImageMode>,
    pub background: Color,
    pub font: Handle<Font>,
    pub font_size: f32,
    pub font_color: Color,
    pub delay: f32,
    #[reflect(ignore)]
    pub easing: Option<interpolation::EaseFunction>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            style: Node::default(),
            border_color: Color::NONE,
            border_radius: UiRect::default(),
            background: Color::NONE,
            image_scale_mode: None,
            font: Handle::default(),
            font_size: 12.,
            font_color: Color::WHITE,
            delay: 0.,
            easing: Some(interpolation::EaseFunction::CubicInOut),
        }
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect]
pub struct NodeStyle {
    pub regular: ComputedStyle,
    #[reflect(ignore)]
    pub hover: Vec<StyleAttr>,
    #[reflect(ignore)]
    pub pressed: Vec<StyleAttr>,
    #[reflect(ignore)]
    pub active: Vec<StyleAttr>,
}

impl NodeStyle {
    pub fn apply_style(
        &self,
        style: &mut Node,
        is_active: bool,
        mut bg: Option<Mut<BackgroundColor>>,
        mut bradius: Option<Mut<BorderRadius>>,
        mut bcolor: Option<Mut<BorderColor>>,
        mut text_color: Option<Mut<TextColor>>,
        mut text_style: Option<Mut<TextFont>>,
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

        text_style.as_mut().map(|txt_style| {
            txt_style.font_size = self.regular.font_size;
            txt_style.font = self.regular.font.clone();
        });

        text_color.as_mut().map(|txt_color| {
            ***txt_color = self.regular.font_color;
        });

        hover_timer.map(|timer| {
            if timer.fraction() > 0.01 {
                let ratio = self
                    .regular
                    .easing
                    .map(|ez| timer.fraction().calc(ez))
                    .unwrap_or(timer.fraction());

                for attr in self.hover.iter() {
                    apply_lerp_style(
                        attr,
                        ratio,
                        &self.regular,
                        style,
                        &mut bg,
                        &mut bradius,
                        &mut bcolor,
                        &mut text_color,
                        &mut text_style,
                        server,
                    );
                }
            }
        });

        pressed_timer.map(|timer| {
            if timer.fraction() > 0.01 {
                let ratio = self
                    .regular
                    .easing
                    .map(|ez| timer.fraction().calc(ez))
                    .unwrap_or(timer.fraction());

                for attr in self.pressed.iter() {
                    apply_lerp_style(
                        attr,
                        ratio,
                        &self.regular,
                        style,
                        &mut bg,
                        &mut bradius,
                        &mut bcolor,
                        &mut text_color,
                        &mut text_style,
                        server,
                    );
                }
            }
        });

        if is_active {
            for attr in self.active.iter() {
                apply_lerp_style(
                    attr,
                    1., //@todo: fade-in/out?
                    &self.regular,
                    style,
                    &mut bg,
                    &mut bradius,
                    &mut bcolor,
                    &mut text_color,
                    &mut text_style,
                    server,
                );
            }
        }
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
                    Some(index) => self.pressed.insert(index, style),
                    None => self.pressed.push(style),
                }
            }
            StyleAttr::Active(style) => {
                let style = *style;
                match self
                    .active
                    .iter()
                    .position(|s| std::mem::discriminant(s) == std::mem::discriminant(&style))
                {
                    Some(index) => self.active.insert(index, style),
                    None => self.active.push(style),
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
            StyleAttr::FontSize(f) => self.regular.font_size = f,
            StyleAttr::FontColor(color) => self.regular.font_color = color,
            StyleAttr::Background(color) => self.regular.background = color,
            StyleAttr::Delay(f) => self.regular.delay = f,
            StyleAttr::Easing(ease) => self.regular.easing = Some(ease),
            StyleAttr::ImageScaleMode(mode) => self.regular.image_scale_mode = Some(mode),
            // StyleAttr::Font(font) => self.regular.font = server
            _ => (),
        };
    }
}

fn apply_lerp_style(
    attr: &StyleAttr,
    ratio: f32,
    default: &ComputedStyle,
    style: &mut Node,
    bg: &mut Option<Mut<BackgroundColor>>,
    bradius: &mut Option<Mut<BorderRadius>>,
    bcolor: &mut Option<Mut<BorderColor>>,
    text_color: &mut Option<Mut<TextColor>>,
    text_style: &mut Option<Mut<TextFont>>,
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
        StyleAttr::Background(color) => {
            bg.as_mut()
                .map(|bg| bg.0 = lerp_color(&default.background, color, ratio));
        }
        StyleAttr::FontColor(color) => {
            text_color.as_mut().map(|tc| {
                ***tc = lerp_color(&default.font_color, color, ratio);
            });
        }
        StyleAttr::FontSize(s) => {
            text_style.as_mut().map(|txt| {
                txt.font_size = default.font_size.lerp(*s, ratio);
            });
        }
        StyleAttr::Font(h) => {
            text_style.as_mut().map(|txt| {
                txt.font = server.load(h);
            });
        }
        _ => (),
        // StyleAttr::Active(style_attr) => todo!(),
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

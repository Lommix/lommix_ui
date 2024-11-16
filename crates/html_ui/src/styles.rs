use crate::{build::InteractionObverser, data::StyleAttr};
use bevy::{
    ecs::{query::QueryEntityError, system::SystemParam},
    prelude::*,
};
use std::time::Duration;

pub struct TransitionPlugin;
impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (continues_interaction_checking, update_node_style));
        app.register_type::<PressedTimer>();
        app.register_type::<HoverTimer>();
        app.register_type::<ComputedStyle>();
        app.register_type::<HtmlStyle>();
        app.register_type::<InteractionTimer>();
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

fn continues_interaction_checking(
    interactions: Query<(Entity, &Interaction), With<HtmlStyle>>,
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

#[derive(SystemParam)]
pub struct UiStyleQuery<'w, 's> {
    pub server: Res<'w, AssetServer>,
    pub node: Query<'w, 's, &'static mut Node>,
    pub text_fonts: Query<'w, 's, &'static mut TextFont>,
    pub text_colors: Query<'w, 's, &'static mut TextColor>,
    pub background: Query<'w, 's, &'static mut BackgroundColor>,
    pub border_radius: Query<'w, 's, &'static mut BorderRadius>,
    pub border_color: Query<'w, 's, &'static mut BorderColor>,
}

impl<'w, 's> UiStyleQuery<'w, 's> {
    pub fn apply_computed(&mut self, entity: Entity, computed: &ComputedStyle) {
        _ = self.node.get_mut(entity).map(|mut node| {
            node.clone_from(&computed.node);
        });

        _ = self.text_fonts.get_mut(entity).map(|mut font| {
            font.font_size = computed.font_size;
            font.font = computed.font.clone();
        });

        _ = self.text_colors.get_mut(entity).map(|mut color| {
            **color = computed.font_color;
        });

        _ = self.background.get_mut(entity).map(|mut background| {
            background.0 = computed.background;
        });

        _ = self.border_radius.get_mut(entity).map(|mut radius| {
            radius.top_left = computed.border_radius.top;
            radius.top_right = computed.border_radius.right;
            radius.bottom_right = computed.border_radius.bottom;
            radius.bottom_left = computed.border_radius.left;
        });

        _ = self.border_color.get_mut(entity).map(|mut color| {
            color.0 = computed.border_color;
        });
    }

    pub fn apply_interpolated<'a>(
        &mut self,
        entity: Entity,
        ratio: f32,
        computed: &ComputedStyle,
        attr: &StyleAttr,
    ) -> Result<(), QueryEntityError> {
        let mut style = self.node.get_mut(entity)?;
        match attr {
            StyleAttr::Display(display) => style.display = *display,
            StyleAttr::Position(position_type) => style.position_type = *position_type,
            StyleAttr::Overflow(overflow) => style.overflow = *overflow,
            StyleAttr::Left(val) => style.left = lerp_val(&computed.node.left, val, ratio),
            StyleAttr::Right(val) => style.right = lerp_val(&computed.node.right, val, ratio),
            StyleAttr::Top(val) => style.top = lerp_val(&computed.node.top, val, ratio),
            StyleAttr::Bottom(val) => style.bottom = lerp_val(&computed.node.bottom, val, ratio),
            StyleAttr::Width(val) => style.width = lerp_val(&computed.node.width, val, ratio),
            StyleAttr::Height(val) => style.height = lerp_val(&computed.node.height, val, ratio),
            StyleAttr::MinWidth(val) => {
                style.min_width = lerp_val(&computed.node.min_width, val, ratio)
            }
            StyleAttr::MinHeight(val) => {
                style.min_height = lerp_val(&computed.node.min_height, val, ratio)
            }
            StyleAttr::MaxWidth(val) => {
                style.max_width = lerp_val(&computed.node.max_width, val, ratio)
            }
            StyleAttr::MaxHeight(val) => {
                style.max_height = lerp_val(&computed.node.max_height, val, ratio)
            }
            StyleAttr::AspectRatio(f) => {
                style.aspect_ratio = computed.node.aspect_ratio.map(|a| a.lerp(*f, ratio))
            }
            StyleAttr::AlignItems(align_items) => style.align_items = *align_items,
            StyleAttr::JustifyItems(justify_items) => style.justify_items = *justify_items,
            StyleAttr::AlignSelf(align_self) => style.align_self = *align_self,
            StyleAttr::JustifySelf(justify_self) => style.justify_self = *justify_self,
            StyleAttr::AlignContent(align_content) => style.align_content = *align_content,
            StyleAttr::JustifyContent(justify_content) => style.justify_content = *justify_content,
            StyleAttr::Margin(ui_rect) => {
                style.margin = lerp_rect(&computed.node.margin, ui_rect, ratio)
            }
            StyleAttr::Padding(ui_rect) => {
                style.padding = lerp_rect(&computed.node.padding, ui_rect, ratio)
            }
            StyleAttr::Border(ui_rect) => {
                style.border = lerp_rect(&computed.node.border, ui_rect, ratio)
            }
            StyleAttr::BorderColor(color) => {
                _ = self
                    .border_color
                    .get_mut(entity)
                    .map(|mut bcolor| bcolor.0 = lerp_color(&computed.border_color, color, ratio));
            }
            StyleAttr::BorderRadius(ui_rect) => {
                _ = self.border_radius.get_mut(entity).map(|mut bradius| {
                    bradius.top_left = lerp_val(&computed.border_radius.top, &ui_rect.top, ratio);
                    bradius.top_right =
                        lerp_val(&computed.border_radius.right, &ui_rect.right, ratio);
                    bradius.bottom_right =
                        lerp_val(&computed.border_radius.bottom, &ui_rect.bottom, ratio);
                    bradius.bottom_left =
                        lerp_val(&computed.border_radius.left, &ui_rect.left, ratio);
                });
            }
            StyleAttr::FlexDirection(flex_direction) => style.flex_direction = *flex_direction,
            StyleAttr::FlexWrap(flex_wrap) => style.flex_wrap = *flex_wrap,
            StyleAttr::FlexGrow(g) => style.flex_grow = computed.node.flex_grow.lerp(*g, ratio),
            StyleAttr::FlexShrink(s) => {
                style.flex_shrink = computed.node.flex_shrink.lerp(*s, ratio)
            }
            StyleAttr::FlexBasis(val) => {
                style.flex_basis = lerp_val(&computed.node.flex_basis, val, ratio)
            }
            StyleAttr::RowGap(val) => style.row_gap = lerp_val(&computed.node.row_gap, val, ratio),
            StyleAttr::ColumnGap(val) => {
                style.column_gap = lerp_val(&computed.node.max_height, val, ratio)
            }
            StyleAttr::GridAutoFlow(grid_auto_flow) => style.grid_auto_flow = *grid_auto_flow,
            StyleAttr::GridTemplateRows(vec) => style.grid_template_rows = vec.clone(),
            StyleAttr::GridTemplateColumns(vec) => style.grid_template_columns = vec.clone(),
            StyleAttr::GridAutoRows(vec) => style.grid_auto_rows = vec.clone(),
            StyleAttr::GridAutoColumns(vec) => style.grid_auto_columns = vec.clone(),
            StyleAttr::GridRow(grid_placement) => style.grid_row = *grid_placement,
            StyleAttr::GridColumn(grid_placement) => style.grid_column = *grid_placement,
            StyleAttr::Background(color) => {
                _ = self
                    .background
                    .get_mut(entity)
                    .map(|mut bg| bg.0 = lerp_color(&computed.background, color, ratio));
            }
            StyleAttr::FontColor(color) => {
                _ = self.text_colors.get_mut(entity).map(|mut tc| {
                    **tc = lerp_color(&computed.font_color, color, ratio);
                });
            }
            StyleAttr::FontSize(s) => {
                _ = self.text_fonts.get_mut(entity).map(|mut txt| {
                    txt.font_size = computed.font_size.lerp(*s, ratio);
                });
            }
            StyleAttr::Font(h) => {
                _ = self.text_fonts.get_mut(entity).map(|mut txt| {
                    txt.font = self.server.load(h);
                });
            }
            _ => (),
            // StyleAttr::Active(style_attr) => todo!(),
            // StyleAttr::Hover(style_attr) => todo!(),
            // StyleAttr::Pressed(style_attr) => todo!(),
            // StyleAttr::Delay(_) => todo!(),
        }

        Ok(())
    }
}

fn update_node_style(
    nodes: Query<(Entity, &HtmlStyle, Has<UiActive>)>,
    mut ui_style: UiStyleQuery,
    hover_timer: Query<&HoverTimer>,
    press_timer: Query<&PressedTimer>,
) {
    for (entity, html_style, is_active) in nodes.iter() {
        ui_style.apply_computed(entity, &html_style.computed);

        let hover_ratio = hover_timer
            .get(entity)
            .map(|t| t.fraction())
            .unwrap_or_default();

        for hover_style in html_style.hover.iter() {
            ui_style
                .apply_interpolated(entity, hover_ratio, &html_style.computed, hover_style)
                .expect("node has no style, impossible");
        }

        let press_ratio = press_timer
            .get(entity)
            .map(|t| t.fraction())
            .unwrap_or_default();

        for press_style in html_style.pressed.iter() {
            ui_style
                .apply_interpolated(entity, press_ratio, &html_style.computed, press_style)
                .expect("node has no style, impossible");
        }

        let active_ratio = is_active.then_some(1.).unwrap_or_default();
        for active_style in html_style.active.iter() {
            ui_style
                .apply_interpolated(entity, active_ratio, &html_style.computed, active_style)
                .expect("node has no style, impossible");
        }
    }
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
    pub node: Node,
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
            node: Node::default(),
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

// ----------------

#[derive(Component, Default, Debug, Reflect)]
#[reflect]
pub struct HtmlStyle {
    pub computed: ComputedStyle,
    #[reflect(ignore)]
    pub hover: Vec<StyleAttr>,
    #[reflect(ignore)]
    pub pressed: Vec<StyleAttr>,
    #[reflect(ignore)]
    pub active: Vec<StyleAttr>,
}

impl From<Vec<StyleAttr>> for HtmlStyle {
    fn from(mut styles: Vec<StyleAttr>) -> Self {
        let mut out = HtmlStyle::default();
        for style in styles.drain(..) {
            out.add_style_attr(style);
        }
        out
    }
}

impl HtmlStyle {
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
            StyleAttr::Display(display) => self.computed.node.display = display,
            StyleAttr::Position(position_type) => self.computed.node.position_type = position_type,
            StyleAttr::Overflow(overflow) => self.computed.node.overflow = overflow,
            StyleAttr::Left(val) => self.computed.node.left = val,
            StyleAttr::Right(val) => self.computed.node.right = val,
            StyleAttr::Top(val) => self.computed.node.top = val,
            StyleAttr::Bottom(val) => self.computed.node.bottom = val,
            StyleAttr::Width(val) => self.computed.node.width = val,
            StyleAttr::Height(val) => self.computed.node.height = val,
            StyleAttr::MinWidth(val) => self.computed.node.min_width = val,
            StyleAttr::MinHeight(val) => self.computed.node.min_height = val,
            StyleAttr::MaxWidth(val) => self.computed.node.max_width = val,
            StyleAttr::MaxHeight(val) => self.computed.node.min_height = val,
            StyleAttr::AspectRatio(f) => self.computed.node.aspect_ratio = Some(f),
            StyleAttr::AlignItems(align_items) => self.computed.node.align_items = align_items,
            StyleAttr::JustifyItems(justify_items) => {
                self.computed.node.justify_items = justify_items
            }
            StyleAttr::AlignSelf(align_self) => self.computed.node.align_self = align_self,
            StyleAttr::JustifySelf(justify_self) => self.computed.node.justify_self = justify_self,
            StyleAttr::AlignContent(align_content) => {
                self.computed.node.align_content = align_content
            }
            StyleAttr::JustifyContent(justify_content) => {
                self.computed.node.justify_content = justify_content
            }
            StyleAttr::Margin(ui_rect) => self.computed.node.margin = ui_rect,
            StyleAttr::Padding(ui_rect) => self.computed.node.padding = ui_rect,
            StyleAttr::Border(ui_rect) => self.computed.node.border = ui_rect,
            StyleAttr::BorderColor(color) => self.computed.border_color = color,
            StyleAttr::BorderRadius(ui_rect) => self.computed.border_radius = ui_rect,
            StyleAttr::FlexDirection(flex_direction) => {
                self.computed.node.flex_direction = flex_direction
            }
            StyleAttr::FlexWrap(flex_wrap) => self.computed.node.flex_wrap = flex_wrap,
            StyleAttr::FlexGrow(f) => self.computed.node.flex_grow = f,
            StyleAttr::FlexShrink(f) => self.computed.node.flex_shrink = f,
            StyleAttr::FlexBasis(val) => self.computed.node.flex_basis = val,
            StyleAttr::RowGap(val) => self.computed.node.row_gap = val,
            StyleAttr::ColumnGap(val) => self.computed.node.column_gap = val,
            StyleAttr::GridAutoFlow(grid_auto_flow) => {
                self.computed.node.grid_auto_flow = grid_auto_flow
            }
            StyleAttr::GridTemplateRows(vec) => self.computed.node.grid_template_rows = vec,
            StyleAttr::GridTemplateColumns(vec) => self.computed.node.grid_template_columns = vec,
            StyleAttr::GridAutoRows(vec) => self.computed.node.grid_auto_rows = vec,
            StyleAttr::GridAutoColumns(vec) => self.computed.node.grid_auto_columns = vec,
            StyleAttr::GridRow(grid_placement) => self.computed.node.grid_row = grid_placement,
            StyleAttr::GridColumn(grid_placement) => {
                self.computed.node.grid_column = grid_placement
            }
            StyleAttr::FontSize(f) => self.computed.font_size = f,
            StyleAttr::FontColor(color) => self.computed.font_color = color,
            StyleAttr::Background(color) => self.computed.background = color,
            StyleAttr::Delay(f) => self.computed.delay = f,
            StyleAttr::Easing(ease) => self.computed.easing = Some(ease),
            StyleAttr::ImageScaleMode(mode) => self.computed.image_scale_mode = Some(mode),
            // StyleAttr::Font(font) => self.regular.font = server
            _ => (),
        };
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

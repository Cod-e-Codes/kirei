use crate::gui::core::{Event, Rect, SizeConstraints, Widget, WidgetContext};
use glam::Vec2;
use glyphon::Wrap;
use std::sync::{Arc, Mutex};

// Per-widget state types

#[derive(Debug, Clone, Default)]
pub struct ButtonState {
    pub hovered: bool,
    pub pressed: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SliderState {
    pub value: f32,
    pub hovered: bool,
    pub dragging: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CheckboxState {
    pub checked: bool,
    pub hovered: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RadioButtonsState {
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    pub text: String,
    pub focused: bool,
    pub cursor_pos: usize,
    pub selection_start: Option<usize>,
    pub hovered: bool,
    pub error_message: Option<String>,
    pub has_interacted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TextAreaState {
    pub text: String,
    pub focused: bool,
    pub cursor_pos: usize,
    pub selection_start: Option<usize>,
    pub hovered: bool,
    pub preferred_height: Option<f32>,
    pub error_message: Option<String>,
    pub has_interacted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ScrollViewState {
    pub offset: f32,
    pub dragging: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DropdownState {
    pub selected_index: Option<usize>,
    pub hovered: bool,
    pub hovered_option: Option<usize>,
    pub open: bool,
    pub scroll_offset: f32,
    pub scroll_range: f32,
}

#[derive(Debug, Clone, Default)]
pub struct TabsState {
    pub selected_index: Option<usize>,
    pub hovered_index: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct LabelState {
    pub text: Option<String>,
}

pub struct Label {
    pub text: String,
    pub rect: Rect,
    pub color: Option<[f32; 4]>,
    explicit_id: Option<String>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            rect: Rect::default(),
            color: None,
            explicit_id: None,
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.explicit_id = Some(id.into());
        self
    }
}

impl Widget for Label {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.push_explicit_id(self.explicit_id.as_deref());

        // Extract values before borrowing ctx mutably
        let default_text = self.text.clone();
        let color = self.color.unwrap_or(ctx.theme.colors.text);
        let font_size = ctx.theme.font.size_body;
        let pos = self.rect.pos;

        // Load state and determine text to use
        let state: LabelState = ctx.get_state();
        let text = state.text.unwrap_or(default_text);

        ctx.painter
            .draw_text(&text, pos, color, font_size, Wrap::Word);

        ctx.pop_explicit_id();
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let size = ctx
            .painter
            .get_text_size(&self.text, ctx.theme.font.size_body);
        constraints.constrain(size)
    }

    fn handle_event(&mut self, _ctx: &mut WidgetContext, _event: &Event) -> bool {
        false
    }
}

pub struct Button {
    pub text: String,
    pub rect: Rect,
    pub on_click: Option<Box<dyn FnMut()>>,
    explicit_id: Option<String>,
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            rect: Rect::default(),
            on_click: None,
            explicit_id: None,
        }
    }

    pub fn on_click<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.explicit_id = Some(id.into());
        self
    }
}

impl Widget for Button {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.push_explicit_id(self.explicit_id.as_deref());

        // Load state
        let state: ButtonState = ctx.get_state();
        let is_hovered = state.hovered;
        let is_pressed = state.pressed;

        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;

        let border_color = if is_hovered || is_pressed {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.border
        };

        let bg_color = if is_pressed {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.widget_bg
        };

        // Draw border (outer rounded rect)
        ctx.painter
            .draw_rounded_rect(self.rect, border_color, radius);

        // Draw background (inner rounded rect, inset by border width)
        let inset = border_width;
        let inner_rect = Rect::new(
            self.rect.pos.x + inset,
            self.rect.pos.y + inset,
            self.rect.size.x - inset * 2.0,
            self.rect.size.y - inset * 2.0,
        );
        ctx.painter
            .draw_rounded_rect(inner_rect, bg_color, (radius - inset).max(0.0));

        if is_hovered && !is_pressed {
            ctx.painter.draw_rounded_rect(
                inner_rect,
                ctx.theme.colors.hover_overlay,
                (radius - inset).max(0.0),
            );
        }

        let text_size = ctx
            .painter
            .get_text_size(&self.text, ctx.theme.font.size_body);
        let text_pos = Vec2::new(
            self.rect.pos.x + (self.rect.size.x - text_size.x) / 2.0,
            self.rect.pos.y + (self.rect.size.y - text_size.y) / 2.0,
        );

        let text_color = ctx.theme.colors.text;

        ctx.painter.draw_text(
            &self.text,
            text_pos,
            text_color,
            ctx.theme.font.size_body,
            Wrap::Word,
        );

        ctx.pop_explicit_id();
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let text_size = ctx
            .painter
            .get_text_size(&self.text, ctx.theme.font.size_body);
        let size = Vec2::new(
            text_size.x + ctx.theme.metrics.padding * 2.0,
            text_size.y + ctx.theme.metrics.padding * 2.0,
        );
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        ctx.push_explicit_id(self.explicit_id.as_deref());

        // Load state
        let mut state: ButtonState = ctx.get_state();

        let result = match event {
            Event::Move(pos) => {
                let was_hovered = state.hovered;
                state.hovered = self.rect.contains(*pos);
                was_hovered != state.hovered
            }
            Event::Press(pos) => {
                if self.rect.contains(*pos) {
                    state.pressed = true;
                    true
                } else {
                    false
                }
            }
            Event::Release(pos) => {
                let was_pressed = state.pressed;
                state.pressed = false;
                if was_pressed && self.rect.contains(*pos) {
                    if let Some(callback) = &mut self.on_click {
                        callback();
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        // Save state
        ctx.set_state(state);
        ctx.pop_explicit_id();
        result
    }
}

pub struct Checkbox {
    pub label: String,
    pub rect: Rect,
    pub on_change: Option<Box<dyn FnMut(bool)>>,
    initial_checked: bool, // Used to initialize state on first draw
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            rect: Rect::default(),
            on_change: None,
            initial_checked: checked,
        }
    }

    pub fn on_change<F: FnMut(bool) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for Checkbox {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Load state
        let mut state: CheckboxState = ctx.get_state();
        // Initialize state from constructor parameter if not set
        if !state.checked && self.initial_checked {
            state.checked = true;
        }
        let checked = state.checked;
        let is_hovered = state.hovered;
        // Save state if we modified it
        if !state.checked && self.initial_checked {
            ctx.set_state(state);
        }

        let box_size = 20.0;
        let box_rect = Rect::new(
            self.rect.pos.x,
            self.rect.pos.y + (self.rect.size.y - box_size) / 2.0,
            box_size,
            box_size,
        );

        // Draw box with rounded border
        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;

        let border_color = if is_hovered {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.border
        };

        // Draw border (outer rounded rect)
        ctx.painter
            .draw_rounded_rect(box_rect, border_color, radius);

        // Draw background (inner rounded rect)
        let inset = border_width;
        let inner_rect = Rect::new(
            box_rect.pos.x + inset,
            box_rect.pos.y + inset,
            box_size - inset * 2.0,
            box_size - inset * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.widget_bg,
            (radius - inset).max(0.0),
        );

        // Draw check if checked
        if checked {
            let check_inset = 4.0;
            let check_rect = Rect::new(
                box_rect.pos.x + check_inset,
                box_rect.pos.y + check_inset,
                box_size - check_inset * 2.0,
                box_size - check_inset * 2.0,
            );
            ctx.painter.draw_rounded_rect(
                check_rect,
                ctx.theme.colors.primary,
                (radius - check_inset).max(0.0),
            );
        }

        // Draw label
        let text_pos = Vec2::new(
            self.rect.pos.x + box_size + ctx.theme.metrics.spacing,
            self.rect.pos.y + (self.rect.size.y - ctx.theme.font.size_body) / 2.0,
        );
        ctx.painter.draw_text(
            &self.label,
            text_pos,
            ctx.theme.colors.text,
            ctx.theme.font.size_body,
            Wrap::Word,
        );
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let text_size = ctx
            .painter
            .get_text_size(&self.label, ctx.theme.font.size_body);
        let box_size = 20.0;
        let size = Vec2::new(
            box_size + ctx.theme.metrics.spacing + text_size.x,
            box_size.max(text_size.y),
        );
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        // Load state
        let mut state: CheckboxState = ctx.get_state();
        // Initialize state from constructor parameter if not set
        if !state.checked && self.initial_checked {
            state.checked = true;
        }

        let result = match event {
            Event::Move(pos) => {
                state.hovered = self.rect.contains(*pos);
                false
            }
            Event::Press(pos) => {
                if self.rect.contains(*pos) {
                    state.checked = !state.checked;
                    if let Some(callback) = &mut self.on_change {
                        callback(state.checked);
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        // Save state
        ctx.set_state(state);
        result
    }
}

type RadioButtonsCallback = Box<dyn FnMut(usize, &str)>;

pub struct RadioButtons {
    pub options: Vec<String>,
    pub rect: Rect,
    pub on_change: Option<RadioButtonsCallback>,
    initial_selected: Option<usize>,
}

impl RadioButtons {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            rect: Rect::default(),
            on_change: None,
            initial_selected: None,
        }
    }

    pub fn select(mut self, index: usize) -> Self {
        self.initial_selected = Some(index);
        self
    }

    pub fn on_change<F: FnMut(usize, &str) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    fn indicator_size(&self) -> f32 {
        18.0
    }

    fn row_spacing(&self, spacing: f32) -> f32 {
        spacing * 0.5
    }

    fn option_height(&self, font_size: f32, padding: f32) -> f32 {
        self.indicator_size().max(font_size + padding)
    }

    fn option_rect(&self, option_height: f32, spacing: f32, index: usize) -> Rect {
        let y = self.rect.pos.y + index as f32 * (option_height + spacing);
        Rect::new(self.rect.pos.x, y, self.rect.size.x, option_height)
    }

    fn hit_test(&self, pos: Vec2, option_height: f32, spacing: f32) -> Option<usize> {
        if !self.rect.contains(pos) {
            return None;
        }

        for i in 0..self.options.len() {
            let option_rect = self.option_rect(option_height, spacing, i);
            if option_rect.contains(pos) {
                return Some(i);
            }
        }

        None
    }
}

impl Widget for RadioButtons {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        if self.options.is_empty() {
            return;
        }

        let mut state: RadioButtonsState = ctx.get_state();
        let mut state_dirty = false;

        if state.selected_index.is_none()
            && let Some(initial) = self.initial_selected
            && !self.options.is_empty()
        {
            let clamped = initial.min(self.options.len() - 1);
            state.selected_index = Some(clamped);
            state_dirty = true;
        }

        let indicator_size = self.indicator_size();
        let row_spacing = self.row_spacing(ctx.theme.metrics.spacing);
        let option_height = self.option_height(ctx.theme.font.size_body, ctx.theme.metrics.padding);
        let border_width = ctx.theme.metrics.border_width;
        let highlight_radius = ctx.theme.metrics.radius;

        for (index, option) in self.options.iter().enumerate() {
            let option_rect = self.option_rect(option_height, row_spacing, index);
            let indicator_rect = Rect::new(
                option_rect.pos.x,
                option_rect.pos.y + (option_rect.size.y - indicator_size) / 2.0,
                indicator_size,
                indicator_size,
            );

            let is_selected = state.selected_index == Some(index);
            let is_hovered = state.hovered_index == Some(index);

            if is_hovered {
                ctx.painter.draw_rounded_rect(
                    option_rect,
                    ctx.theme.colors.hover_overlay,
                    highlight_radius,
                );
            }

            // Outer indicator ring
            ctx.painter.draw_rounded_rect(
                indicator_rect,
                if is_selected {
                    ctx.theme.colors.primary
                } else {
                    ctx.theme.colors.border
                },
                indicator_size / 2.0,
            );

            // Inner background
            let inner_size = (indicator_size - border_width * 2.0).max(0.0);
            let inner_rect = Rect::new(
                indicator_rect.pos.x + border_width,
                indicator_rect.pos.y + border_width,
                inner_size,
                inner_size,
            );
            let inner_radius = (indicator_size / 2.0 - border_width).max(0.0);
            ctx.painter
                .draw_rounded_rect(inner_rect, ctx.theme.colors.widget_bg, inner_radius);

            // Selection dot
            if is_selected {
                let dot_inset = border_width + 3.0;
                let dot_size = (indicator_size - dot_inset * 2.0).max(0.0);
                let dot_rect = Rect::new(
                    indicator_rect.pos.x + dot_inset,
                    indicator_rect.pos.y + dot_inset,
                    dot_size,
                    dot_size,
                );
                ctx.painter
                    .draw_rounded_rect(dot_rect, ctx.theme.colors.primary, dot_size / 2.0);
            }

            // Option label
            let text_pos = Vec2::new(
                indicator_rect.pos.x + indicator_size + ctx.theme.metrics.spacing,
                option_rect.pos.y + (option_rect.size.y - ctx.theme.font.size_body) / 2.0,
            );
            ctx.painter.draw_text(
                option,
                text_pos,
                ctx.theme.colors.text,
                ctx.theme.font.size_body,
                Wrap::Word,
            );
        }

        if state_dirty {
            ctx.set_state(state);
        }
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        if self.options.is_empty() {
            return constraints.constrain(Vec2::new(0.0, 0.0));
        }

        let indicator_size = self.indicator_size();
        let row_spacing = self.row_spacing(ctx.theme.metrics.spacing);
        let option_height = self.option_height(ctx.theme.font.size_body, ctx.theme.metrics.padding);

        let mut max_text_width: f32 = 0.0;
        for option in &self.options {
            let text_size = ctx.painter.get_text_size(option, ctx.theme.font.size_body);
            max_text_width = max_text_width.max(text_size.x);
        }

        let width =
            indicator_size + ctx.theme.metrics.spacing + max_text_width + ctx.theme.metrics.padding;
        let count = self.options.len();
        let total_height =
            option_height * count as f32 + row_spacing * (count.saturating_sub(1) as f32);

        constraints.constrain(Vec2::new(width, total_height))
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        if self.options.is_empty() {
            return false;
        }

        let mut state: RadioButtonsState = ctx.get_state();
        let option_height = self.option_height(ctx.theme.font.size_body, ctx.theme.metrics.padding);
        let row_spacing = self.row_spacing(ctx.theme.metrics.spacing);

        match event {
            Event::Move(pos) => {
                let hovered = self.hit_test(*pos, option_height, row_spacing);
                if hovered != state.hovered_index {
                    state.hovered_index = hovered;
                    ctx.set_state(state);
                    hovered.is_some()
                } else {
                    false
                }
            }
            Event::Press(pos) => {
                if let Some(index) = self.hit_test(*pos, option_height, row_spacing) {
                    let selection_changed = state.selected_index != Some(index);
                    state.selected_index = Some(index);
                    state.hovered_index = Some(index);

                    if selection_changed && let Some(callback) = &mut self.on_change {
                        callback(index, &self.options[index]);
                    }

                    ctx.set_state(state);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

type DropdownCallback = Box<dyn FnMut(usize, &str)>;

pub struct Dropdown {
    pub options: Vec<String>,
    pub rect: Rect,
    pub on_change: Option<DropdownCallback>,
    pub placeholder: Option<String>,
}

impl Dropdown {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            rect: Rect::default(),
            on_change: None,
            placeholder: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn on_change<F: FnMut(usize, &str) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for Dropdown {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Load state
        let state: DropdownState = ctx.get_state();
        let selected_index = state.selected_index;
        let is_hovered = state.hovered;
        let is_open = state.open;

        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;
        let padding = ctx.theme.metrics.padding;

        // Draw main button/trigger
        let border_color = if is_hovered || is_open {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.border
        };

        // Draw border (outer rounded rect)
        ctx.painter
            .draw_rounded_rect(self.rect, border_color, radius);

        // Draw background (inner rounded rect)
        let inset = border_width;
        let inner_rect = Rect::new(
            self.rect.pos.x + inset,
            self.rect.pos.y + inset,
            self.rect.size.x - inset * 2.0,
            self.rect.size.y - inset * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.widget_bg,
            (radius - inset).max(0.0),
        );

        if is_hovered && !is_open {
            ctx.painter.draw_rounded_rect(
                inner_rect,
                ctx.theme.colors.hover_overlay,
                (radius - inset).max(0.0),
            );
        }

        // Draw selected text or placeholder
        let display_text = if let Some(idx) = selected_index {
            if idx < self.options.len() {
                &self.options[idx]
            } else {
                self.placeholder.as_deref().unwrap_or("Select...")
            }
        } else {
            self.placeholder.as_deref().unwrap_or("Select...")
        };

        let text_size = ctx
            .painter
            .get_text_size(display_text, ctx.theme.font.size_body);
        let text_pos = Vec2::new(
            self.rect.pos.x + padding,
            self.rect.pos.y + (self.rect.size.y - text_size.y) / 2.0,
        );

        let text_color = if selected_index.is_some() {
            ctx.theme.colors.text
        } else {
            ctx.theme.colors.text_dim
        };

        ctx.painter.draw_text(
            display_text,
            text_pos,
            text_color,
            ctx.theme.font.size_body,
            Wrap::Word,
        );

        // Draw dropdown arrow
        let arrow_size = 8.0;
        let arrow_x = self.rect.pos.x + self.rect.size.x - padding - arrow_size;
        let arrow_y = self.rect.pos.y + (self.rect.size.y - arrow_size) / 2.0;
        let arrow_rect = Rect::new(arrow_x, arrow_y, arrow_size, arrow_size);

        // Simple arrow indicator (small triangle or chevron)
        // For now, just draw a small rect as placeholder - could be enhanced with actual arrow
        let arrow_color = ctx.theme.colors.text_dim;
        ctx.painter.draw_rect(arrow_rect, arrow_color);

        // Menu will be drawn in draw_overlay() to appear on top of everything
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        // Load state - need to ensure state is up to date
        let mut state: DropdownState = ctx.get_state();
        let selected_index = state.selected_index;
        let is_open = state.open;

        // Draw dropdown menu if open (in overlay phase, after all other widgets)
        if is_open && !self.options.is_empty() {
            // Ensure we have a valid rect (should be set from layout phase)
            if self.rect.size.x == 0.0 || self.rect.size.y == 0.0 {
                return; // Widget not laid out yet, skip drawing
            }

            // Clear any scissor so menu renders on top of everything
            let previous_scissor = ctx.painter.set_scissor(None);

            let radius = ctx.theme.metrics.radius;
            let border_width = ctx.theme.metrics.border_width;
            let padding = ctx.theme.metrics.padding;
            let scrollbar_width = ctx.theme.metrics.scrollbar_width;
            let menu_item_height = 32.0;
            let max_visible_items = 5;
            let visible_height = menu_item_height * max_visible_items as f32;
            let total_height = self.options.len() as f32 * menu_item_height;
            let has_scroll = total_height > visible_height;
            let menu_height = if has_scroll {
                visible_height
            } else {
                total_height
            };
            let max_scroll = (total_height - menu_height).max(0.0);
            if (state.scroll_range - max_scroll).abs() > f32::EPSILON {
                state.scroll_range = max_scroll;
            }
            let scroll_offset = state.scroll_offset.clamp(0.0, max_scroll);
            if (scroll_offset - state.scroll_offset).abs() > f32::EPSILON {
                state.scroll_offset = scroll_offset;
            }

            let menu_rect = Rect::new(
                self.rect.pos.x,
                self.rect.pos.y + self.rect.size.y,
                self.rect.size.x,
                menu_height,
            );

            // Draw menu border (outer rounded rect)
            ctx.painter
                .draw_rounded_rect(menu_rect, ctx.theme.colors.border, radius);

            // Draw menu background (inner rounded rect, inset by border width)
            let inset = border_width;
            let inner_menu_rect = Rect::new(
                menu_rect.pos.x + inset,
                menu_rect.pos.y + inset,
                menu_rect.size.x - inset * 2.0,
                menu_rect.size.y - inset * 2.0,
            );
            ctx.painter.draw_rounded_rect(
                inner_menu_rect,
                ctx.theme.colors.surface,
                (radius - inset).max(0.0),
            );

            // Clip menu items to visible area
            let menu_clip_rect = Rect::new(
                inner_menu_rect.pos.x,
                inner_menu_rect.pos.y,
                inner_menu_rect.size.x - if has_scroll { scrollbar_width } else { 0.0 },
                inner_menu_rect.size.y,
            );
            let previous_menu_scissor = ctx.painter.set_scissor(Some(menu_clip_rect));

            // Draw menu items with scroll offset
            let start_item_index = (scroll_offset / menu_item_height) as usize;
            let end_item_index =
                ((scroll_offset + visible_height) / menu_item_height).ceil() as usize;
            let end_item_index = end_item_index.min(self.options.len());

            for i in start_item_index..end_item_index {
                let item_y_offset = i as f32 * menu_item_height - scroll_offset;
                let item_rect = Rect::new(
                    inner_menu_rect.pos.x,
                    inner_menu_rect.pos.y + item_y_offset,
                    inner_menu_rect.size.x - if has_scroll { scrollbar_width } else { 0.0 },
                    menu_item_height,
                );

                let is_selected = selected_index == Some(i);
                let is_option_hovered = state.hovered_option == Some(i);

                // Highlight hovered or selected item
                if is_selected || is_option_hovered {
                    let bg_color = if is_option_hovered {
                        ctx.theme.colors.hover_overlay
                    } else {
                        ctx.theme.colors.primary
                    };
                    ctx.painter.draw_rounded_rect(item_rect, bg_color, 0.0);
                }

                // Draw option text
                let option_text_pos = Vec2::new(
                    item_rect.pos.x + padding,
                    item_rect.pos.y + (item_rect.size.y - ctx.theme.font.size_body) / 2.0,
                );
                ctx.painter.draw_text(
                    &self.options[i],
                    option_text_pos,
                    ctx.theme.colors.text,
                    ctx.theme.font.size_body,
                    Wrap::Word,
                );
            }

            // Restore menu scissor
            ctx.painter.set_scissor(previous_menu_scissor);

            // Draw scrollbar if needed
            if has_scroll {
                let view_h = inner_menu_rect.size.y;
                let ratio = view_h / total_height;
                let thumb_h = (view_h * ratio).max(20.0);
                let max_scroll = (total_height - view_h).max(0.0);
                let track_ratio = if max_scroll > 0.0 {
                    scroll_offset / max_scroll
                } else {
                    0.0
                };
                let thumb_y = inner_menu_rect.pos.y + track_ratio * (view_h - thumb_h);

                let thumb_rect = Rect::new(
                    inner_menu_rect.pos.x + inner_menu_rect.size.x - scrollbar_width,
                    thumb_y,
                    scrollbar_width,
                    thumb_h,
                );

                ctx.painter.draw_rect(thumb_rect, ctx.theme.colors.border);
            }

            // Restore the previous scissor (if any)
            ctx.painter.set_scissor(previous_scissor);

            // Save state back (in case we modified anything)
            ctx.set_state(state);
        }
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let padding = ctx.theme.metrics.padding;
        let min_width = 120.0;

        // Calculate width based on longest option or placeholder
        let mut max_text_width: f32 = 0.0;
        if let Some(ref placeholder) = self.placeholder {
            let text_size = ctx
                .painter
                .get_text_size(placeholder, ctx.theme.font.size_body);
            max_text_width = max_text_width.max(text_size.x);
        }
        for option in &self.options {
            let text_size = ctx.painter.get_text_size(option, ctx.theme.font.size_body);
            max_text_width = max_text_width.max(text_size.x);
        }

        // Add space for arrow and padding
        let arrow_width = 20.0;
        let width = (max_text_width + padding * 2.0 + arrow_width).max(min_width);
        let height = ctx.theme.font.size_body + padding * 2.0;

        let size = Vec2::new(width, height);
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        // Load state
        let mut state: DropdownState = ctx.get_state();

        let result = match event {
            Event::Move(pos) => {
                let was_hovered = state.hovered;
                state.hovered = self.rect.contains(*pos);

                if state.open {
                    // Check if hovering over menu items
                    let menu_item_height = 32.0;
                    let max_visible_items = 5;
                    let visible_height = menu_item_height * max_visible_items as f32;
                    let total_height = self.options.len() as f32 * menu_item_height;
                    let has_scroll = total_height > visible_height;
                    let menu_height = if has_scroll {
                        visible_height
                    } else {
                        total_height
                    };

                    let menu_rect = Rect::new(
                        self.rect.pos.x,
                        self.rect.pos.y + self.rect.size.y,
                        self.rect.size.x,
                        menu_height,
                    );

                    if menu_rect.contains(*pos) {
                        let relative_y = pos.y - menu_rect.pos.y + state.scroll_offset;
                        let item_index = (relative_y / menu_item_height) as usize;
                        if item_index < self.options.len() {
                            state.hovered_option = Some(item_index);
                        } else {
                            state.hovered_option = None;
                        }
                    } else {
                        state.hovered_option = None;
                    }
                    true
                } else {
                    was_hovered != state.hovered
                }
            }
            Event::Press(pos) => {
                if self.rect.contains(*pos) {
                    // Toggle dropdown
                    state.open = !state.open;
                    state.hovered = true;
                    if !state.open {
                        state.scroll_offset = 0.0;
                        state.scroll_range = 0.0;
                    }
                    true
                } else if state.open {
                    // Check if clicking on menu item
                    let menu_item_height = 32.0;
                    let max_visible_items = 5;
                    let visible_height = menu_item_height * max_visible_items as f32;
                    let total_height = self.options.len() as f32 * menu_item_height;
                    let has_scroll = total_height > visible_height;
                    let menu_height = if has_scroll {
                        visible_height
                    } else {
                        total_height
                    };

                    let menu_rect = Rect::new(
                        self.rect.pos.x,
                        self.rect.pos.y + self.rect.size.y,
                        self.rect.size.x,
                        menu_height,
                    );

                    if menu_rect.contains(*pos) {
                        let relative_y = pos.y - menu_rect.pos.y + state.scroll_offset;
                        let item_index = (relative_y / menu_item_height) as usize;
                        if item_index < self.options.len() {
                            state.selected_index = Some(item_index);
                            if let Some(callback) = &mut self.on_change {
                                callback(item_index, &self.options[item_index]);
                            }
                            state.open = false;
                            state.hovered_option = None;
                            state.scroll_offset = 0.0; // Reset scroll when closing
                            state.scroll_range = 0.0;
                        }
                        true
                    } else {
                        // Clicked outside - close dropdown
                        state.open = false;
                        state.hovered_option = None;
                        state.scroll_offset = 0.0; // Reset scroll when closing
                        state.scroll_range = 0.0;
                        true
                    }
                } else {
                    false
                }
            }
            Event::Release(_) => {
                // Don't close on release, only on press
                false
            }
            Event::Scroll(delta) => {
                if state.open {
                    let menu_item_height = 32.0;
                    let max_visible_items = 5;
                    let visible_height = menu_item_height * max_visible_items as f32;
                    let total_height = self.options.len() as f32 * menu_item_height;
                    let max_scroll = (total_height - visible_height).max(0.0);
                    if (state.scroll_range - max_scroll).abs() > f32::EPSILON {
                        state.scroll_range = max_scroll;
                    }
                    if max_scroll > 0.0 {
                        let scroll_speed = 20.0;
                        let new_offset =
                            (state.scroll_offset - delta * scroll_speed).clamp(0.0, max_scroll);
                        if (new_offset - state.scroll_offset).abs() > f32::EPSILON {
                            state.scroll_offset = new_offset;
                        }
                    } else if state.scroll_offset != 0.0 {
                        state.scroll_offset = 0.0;
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        // Save state
        ctx.set_state(state);
        result
    }
}

pub struct ContextMenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub destructive: bool,
    pub on_select: Option<Box<dyn FnMut()>>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            enabled: true,
            destructive: false,
            on_select: None,
        }
    }

    pub fn shortcut(mut self, text: impl Into<String>) -> Self {
        self.shortcut = Some(text.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    pub fn on_select<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
}

#[derive(Debug, Clone)]
pub struct ContextMenuState {
    pub open: bool,
    pub hovered_index: Option<usize>,
    pub anchor_pos: Vec2,
    pub menu_rect: Rect,
    pub item_height: f32,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            open: false,
            hovered_index: None,
            anchor_pos: Vec2::ZERO,
            menu_rect: Rect::default(),
            item_height: 28.0,
        }
    }
}

pub struct ContextMenu {
    trigger: Box<dyn Widget>,
    pub rect: Rect,
    pub items: Vec<ContextMenuItem>,
    pub offset: Vec2,
    pub min_width: f32,
    pub global: bool,
}

impl ContextMenu {
    pub fn new(trigger: impl Widget + 'static) -> Self {
        Self {
            trigger: Box::new(trigger),
            rect: Rect::default(),
            items: Vec::new(),
            offset: Vec2::new(4.0, 4.0),
            min_width: 160.0,
            global: false,
        }
    }

    pub fn with_items(mut self, items: Vec<ContextMenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn add_item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn global(mut self, enabled: bool) -> Self {
        self.global = enabled;
        self
    }

    fn ensure_layout(&self, ctx: &mut WidgetContext, state: &mut ContextMenuState) {
        if self.items.is_empty() {
            state.menu_rect = Rect::default();
            return;
        }

        let padding = ctx.theme.metrics.padding;
        let shortcut_gap = ctx.theme.metrics.spacing;
        let mut max_width = self.min_width;

        for item in &self.items {
            let label_size = ctx
                .painter
                .get_text_size(&item.label, ctx.theme.font.size_body);
            let mut width = label_size.x;
            if let Some(shortcut) = &item.shortcut {
                let shortcut_size = ctx
                    .painter
                    .get_text_size(shortcut, ctx.theme.font.size_small);
                width += shortcut_gap + shortcut_size.x;
            }
            max_width = max_width.max(width + padding * 2.0);
        }

        let item_height = ctx.theme.font.size_body + padding;
        let total_height = item_height * self.items.len() as f32;
        let mut pos = state.anchor_pos + self.offset;
        pos.x = pos.x.max(0.0);
        pos.y = pos.y.max(0.0);

        state.menu_rect = Rect::new(pos.x, pos.y, max_width, total_height);
        state.item_height = item_height.max(24.0);
    }

    fn item_at(&self, state: &ContextMenuState, pos: Vec2) -> Option<usize> {
        if !state.menu_rect.contains(pos) || state.item_height <= 0.0 {
            return None;
        }
        let relative = pos.y - state.menu_rect.pos.y;
        if relative < 0.0 {
            return None;
        }
        let index = (relative / state.item_height).floor() as usize;
        if index < self.items.len() {
            Some(index)
        } else {
            None
        }
    }

    fn activate_item(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index)
            && item.enabled
            && let Some(callback) = &mut item.on_select
        {
            (callback)();
        }
    }
}

impl Widget for ContextMenu {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.push_path(0);
        self.trigger.draw(ctx);
        ctx.pop_path();
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
        ctx.push_path(0);
        self.trigger.layout(ctx, available_space);
        ctx.pop_path();
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        ctx.push_path(0);
        let hint = self.trigger.size_hint(ctx, constraints);
        ctx.pop_path();
        hint
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut state: ContextMenuState = ctx.get_state();
        let mut consumed = false;

        match event {
            Event::ContextClick(pos) => {
                if self.items.is_empty() {
                    // Nothing to show, let event propagate
                } else if self.global || self.rect.contains(*pos) {
                    state.open = true;
                    state.hovered_index = None;
                    state.anchor_pos = *pos;
                    self.ensure_layout(ctx, &mut state);
                    consumed = true;
                } else if state.open && !state.menu_rect.contains(*pos) {
                    state.open = false;
                    state.hovered_index = None;
                }
            }
            Event::Press(pos) => {
                if state.open {
                    self.ensure_layout(ctx, &mut state);
                    if let Some(index) = self.item_at(&state, *pos) {
                        state.hovered_index = Some(index);
                        if self
                            .items
                            .get(index)
                            .map(|item| item.enabled)
                            .unwrap_or(false)
                        {
                            state.open = false;
                            self.activate_item(index);
                        }
                        consumed = true;
                    } else if !self.rect.contains(*pos) {
                        state.open = false;
                        state.hovered_index = None;
                    }
                }
            }
            Event::Move(pos) => {
                if state.open {
                    self.ensure_layout(ctx, &mut state);
                    state.hovered_index = self.item_at(&state, *pos);
                    if state.hovered_index.is_some() {
                        consumed = true;
                    }
                }
            }
            Event::Scroll(_) => {
                if state.open {
                    consumed = true;
                }
            }
            Event::Key(key, _) => {
                use winit::keyboard::NamedKey;
                if state.open {
                    match key {
                        winit::keyboard::Key::Named(NamedKey::Escape) => {
                            state.open = false;
                            state.hovered_index = None;
                            consumed = true;
                        }
                        winit::keyboard::Key::Named(NamedKey::ArrowDown) => {
                            let len = self.items.len();
                            if len > 0 {
                                let next =
                                    state.hovered_index.map(|idx| (idx + 1) % len).unwrap_or(0);
                                state.hovered_index = Some(next);
                            }
                            consumed = true;
                        }
                        winit::keyboard::Key::Named(NamedKey::ArrowUp) => {
                            let len = self.items.len();
                            if len > 0 {
                                let prev = state
                                    .hovered_index
                                    .map(|idx| (idx + len - 1) % len)
                                    .unwrap_or(len - 1);
                                state.hovered_index = Some(prev);
                            }
                            consumed = true;
                        }
                        winit::keyboard::Key::Named(NamedKey::Enter) => {
                            if let Some(index) = state.hovered_index
                                && self
                                    .items
                                    .get(index)
                                    .map(|item| item.enabled)
                                    .unwrap_or(false)
                            {
                                state.open = false;
                                self.activate_item(index);
                            }
                            consumed = true;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        ctx.push_path(0);
        let child_handled = self.trigger.handle_event(ctx, event);
        ctx.pop_path();

        if child_handled {
            consumed = true;
        }

        ctx.set_state(state);
        consumed
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        ctx.push_path(0);
        self.trigger.draw_overlay(ctx);
        ctx.pop_path();

        if self.items.is_empty() {
            return;
        }

        let mut state: ContextMenuState = ctx.get_state();
        if !state.open {
            ctx.set_state(state);
            return;
        }

        self.ensure_layout(ctx, &mut state);

        let previous_scissor = ctx.painter.set_scissor(None);

        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;
        ctx.painter
            .draw_rounded_rect(state.menu_rect, ctx.theme.colors.border, radius);

        let inset = border_width;
        let inner_rect = Rect::new(
            state.menu_rect.pos.x + inset,
            state.menu_rect.pos.y + inset,
            state.menu_rect.size.x - inset * 2.0,
            state.menu_rect.size.y - inset * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.surface,
            (radius - inset).max(0.0),
        );

        let padding = ctx.theme.metrics.padding;
        for (index, item) in self.items.iter().enumerate() {
            let item_rect = Rect::new(
                inner_rect.pos.x,
                inner_rect.pos.y + index as f32 * state.item_height,
                inner_rect.size.x,
                state.item_height,
            );

            if state.hovered_index == Some(index) && item.enabled {
                ctx.painter
                    .draw_rect(item_rect, ctx.theme.colors.hover_overlay);
            }

            let text_color = if !item.enabled {
                ctx.theme.colors.text_dim
            } else if item.destructive {
                ctx.theme.colors.error
            } else {
                ctx.theme.colors.text
            };

            let text_pos = Vec2::new(
                item_rect.pos.x + padding,
                item_rect.pos.y + (state.item_height - ctx.theme.font.size_body) / 2.0,
            );

            ctx.painter.draw_text(
                &item.label,
                text_pos,
                text_color,
                ctx.theme.font.size_body,
                Wrap::None,
            );

            if let Some(shortcut) = &item.shortcut {
                let shortcut_size = ctx
                    .painter
                    .get_text_size(shortcut, ctx.theme.font.size_small);
                let shortcut_pos = Vec2::new(
                    item_rect.pos.x + item_rect.size.x - padding - shortcut_size.x,
                    text_pos.y,
                );
                ctx.painter.draw_text(
                    shortcut,
                    shortcut_pos,
                    ctx.theme.colors.text_dim,
                    ctx.theme.font.size_small,
                    Wrap::None,
                );
            }
        }

        ctx.painter.set_scissor(previous_scissor);
        ctx.set_state(state);
    }
}

pub struct Slider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub rect: Rect,
    pub on_change: Option<Box<dyn FnMut(f32)>>,
    explicit_id: Option<String>,
}

impl Slider {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            rect: Rect::default(),
            on_change: None,
            explicit_id: None,
        }
    }

    pub fn on_change<F: FnMut(f32) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.explicit_id = Some(id.into());
        self
    }

    fn update_value_from_pos(&mut self, x: f32) {
        let width = self.rect.size.x;
        let relative_x = (x - self.rect.pos.x).clamp(0.0, width);
        let t = relative_x / width;
        self.value = self.min + t * (self.max - self.min);

        if let Some(callback) = &mut self.on_change {
            callback(self.value);
        }
    }
}

impl Widget for Slider {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.push_explicit_id(self.explicit_id.as_deref());

        // Load state
        let mut state: SliderState = ctx.get_state();

        // Initialize from widget constructor value if state is default
        if state.value == 0.0 && self.value != 0.0 {
            state.value = self.value;
        }

        let value = state.value;
        let is_hovered = state.hovered;
        let is_dragging = state.dragging;

        // Update widget's value to match state (for on_change callback)
        self.value = value;

        // Draw track with rounded border
        let track_height = 4.0;
        let track_y = self.rect.pos.y + (self.rect.size.y - track_height) / 2.0;
        let track_rect = Rect::new(self.rect.pos.x, track_y, self.rect.size.x, track_height);
        let track_radius = track_height / 2.0; // Fully rounded (pill shape)

        // Draw track border (outer rounded rect)
        let border_width = ctx.theme.metrics.border_width * 0.5; // Thinner border for track
        let outer_rect = Rect::new(
            self.rect.pos.x,
            track_y - border_width,
            self.rect.size.x,
            track_height + border_width * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            outer_rect,
            ctx.theme.colors.border,
            track_radius + border_width,
        );

        // Draw track background
        ctx.painter
            .draw_rounded_rect(track_rect, ctx.theme.colors.widget_bg, track_radius);

        // Draw knob
        let t = (value - self.min) / (self.max - self.min);
        let knob_width = 16.0;
        let knob_height = 24.0;
        let center_x = self.rect.pos.x + t * self.rect.size.x;
        let knob_x = center_x - knob_width / 2.0;
        let knob_y = self.rect.pos.y + (self.rect.size.y - knob_height) / 2.0;

        let knob_rect = Rect::new(knob_x, knob_y, knob_width, knob_height);

        // Draw knob with rounded border
        let knob_radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width * 1.5; // Slightly thicker for visibility

        // Draw border when hovered or dragging (outer rounded rect)
        if is_hovered || is_dragging {
            ctx.painter
                .draw_rounded_rect(knob_rect, ctx.theme.colors.primary, knob_radius);
        }

        // Draw knob background (inner rounded rect if bordered, full rect otherwise)
        let knob_color = if is_dragging {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.text_dim
        };

        if is_hovered || is_dragging {
            let inset = border_width;
            let inner_knob = Rect::new(
                knob_x + inset,
                knob_y + inset,
                knob_width - inset * 2.0,
                knob_height - inset * 2.0,
            );
            ctx.painter
                .draw_rounded_rect(inner_knob, knob_color, (knob_radius - inset).max(0.0));
        } else {
            ctx.painter
                .draw_rounded_rect(knob_rect, knob_color, knob_radius);
        }

        ctx.pop_explicit_id();
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, _ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let size = Vec2::new(100.0, 30.0); // Min width 100, height 30
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        ctx.push_explicit_id(self.explicit_id.as_deref());

        // Load state
        let mut state: SliderState = ctx.get_state();

        // Initialize from widget constructor value if state is default
        if state.value == 0.0 && self.value != 0.0 {
            state.value = self.value;
        }

        let result = match event {
            Event::Press(pos) => {
                if self.rect.contains(*pos) {
                    state.dragging = true;
                    self.update_value_from_pos(pos.x);
                    state.value = self.value;
                    true
                } else {
                    false
                }
            }
            Event::Release(_) => {
                if state.dragging {
                    state.dragging = false;
                    true
                } else {
                    false
                }
            }
            Event::Move(pos) => {
                state.hovered = self.rect.contains(*pos);
                if state.dragging {
                    self.update_value_from_pos(pos.x);
                    state.value = self.value;
                    true
                } else {
                    false
                }
            }
            _ => false,
        };

        // Save state
        ctx.set_state(state);
        ctx.pop_explicit_id();
        result
    }
}

pub struct ProgressBar {
    pub progress: f32, // 0.0 to 1.0
    pub rect: Rect,
}

impl ProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            rect: Rect::default(),
        }
    }
}

impl Widget for ProgressBar {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // ProgressBar reuses SliderState (same fields)
        let mut state: SliderState = ctx.get_state();

        // Initialize from widget constructor value if state is default
        if state.value == 0.0 && self.progress != 0.0 {
            state.value = self.progress;
        }

        // Use state value if available, otherwise use widget's progress
        let progress = state.value.clamp(0.0, 1.0);

        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;

        // Draw border (outer rounded rect)
        ctx.painter
            .draw_rounded_rect(self.rect, ctx.theme.colors.border, radius);

        // Draw background (inner rounded rect)
        let inset = border_width;
        let inner_rect = Rect::new(
            self.rect.pos.x + inset,
            self.rect.pos.y + inset,
            self.rect.size.x - inset * 2.0,
            self.rect.size.y - inset * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.widget_bg,
            (radius - inset).max(0.0),
        );

        // Fill
        if progress > 0.0 {
            let fill_width = inner_rect.size.x * progress;
            let fill_rect = Rect::new(
                inner_rect.pos.x,
                inner_rect.pos.y,
                fill_width,
                inner_rect.size.y,
            );
            ctx.painter.draw_rounded_rect(
                fill_rect,
                ctx.theme.colors.primary,
                (radius - inset).max(0.0),
            );
        }
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, _ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let size = Vec2::new(100.0, 20.0);
        constraints.constrain(size)
    }

    fn handle_event(&mut self, _ctx: &mut WidgetContext, _event: &Event) -> bool {
        false
    }
}

type InputValidator = Box<dyn Fn(&str) -> Option<String> + 'static>;
type InputSanitizer = Box<dyn Fn(&str) -> String + 'static>;

pub struct TextInput {
    pub text: String,
    pub hint: String,
    pub rect: Rect,
    pub focused: bool,
    pub on_change: Option<Box<dyn FnMut(String)>>,
    validator: Option<InputValidator>,
    error_message: Option<String>,
    error_visible: bool,
    max_chars: Option<usize>,
    sanitizer: Option<InputSanitizer>,
    pub cursor_pos: usize,              // Cursor position in characters
    pub selection_start: Option<usize>, // Start of selection (if any)
    pub is_dragging: bool,              // Currently selecting text with mouse
    char_positions: Vec<f32>,           // Cached character positions for accurate cursor placement
    font_size: f32,                     // Cached font size
    cached_text: String,                // Cached text to detect changes
    positions_dirty: bool,              // Flag to indicate positions need recalculation
    scroll_offset: f32,                 // Horizontal scroll offset to keep cursor visible
    ime_composition: Option<String>,    // Current IME composition text (pre-edit string)
    is_password: bool,                  // Render masked text
    mask_char: char,                    // Character used for masking
    allow_password_copy: bool,          // Whether clipboard operations are allowed in password mode
    display_text: String,               // Cached display string (masked or plain)
    password_visibility: Option<Arc<Mutex<bool>>>, // Shared toggle to reveal password
    last_mask_state: bool,              // Tracks last mask state to refresh layout when toggled
}

impl TextInput {
    fn base_height(&self, ctx: &WidgetContext) -> f32 {
        ctx.theme.font.size_body + ctx.theme.metrics.padding * 2.0
    }

    fn error_height(&self, ctx: &WidgetContext) -> f32 {
        if self.error_visible {
            ctx.theme.metrics.spacing * 0.5 + ctx.theme.font.size_small
        } else {
            0.0
        }
    }

    pub fn new(hint: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            hint: hint.into(),
            rect: Rect::default(),
            focused: false,
            on_change: None,
            validator: None,
            error_message: None,
            error_visible: false,

            max_chars: None,
            sanitizer: None,
            cursor_pos: 0,
            selection_start: None,
            is_dragging: false,
            char_positions: Vec::new(),
            font_size: 16.0, // Default, will be updated in draw
            cached_text: String::new(),
            positions_dirty: true, // Start dirty so it calculates on first draw
            scroll_offset: 0.0,
            ime_composition: None,
            is_password: false,
            mask_char: '•',
            allow_password_copy: false,
            display_text: String::new(),
            password_visibility: None,
            last_mask_state: false,
        }
    }

    pub fn password(hint: impl Into<String>) -> Self {
        Self::new(hint).with_password(true)
    }

    pub fn with_password(mut self, is_password: bool) -> Self {
        self.is_password = is_password;
        self.positions_dirty = true;
        self
    }

    pub fn with_mask_char(mut self, mask_char: char) -> Self {
        self.mask_char = mask_char;
        self.positions_dirty = true;
        self
    }

    pub fn allow_password_copy(mut self, allow: bool) -> Self {
        self.allow_password_copy = allow;
        self
    }

    pub fn bind_password_visibility(mut self, flag: Arc<Mutex<bool>>) -> Self {
        self.password_visibility = Some(flag);
        self.positions_dirty = true;
        self
    }

    fn remaining_capacity(&self) -> Option<usize> {
        self.max_chars
            .map(|limit| limit.saturating_sub(self.text.chars().count()))
    }

    fn apply_sanitization(&mut self) {
        if let Some(sanitizer) = &self.sanitizer {
            let sanitized = sanitizer(&self.text);
            if sanitized != self.text {
                self.text = sanitized;
                self.positions_dirty = true;
                self.cursor_pos = self.cursor_pos.min(self.text.chars().count());
            }
        }
    }

    fn enforce_max_chars(&mut self) {
        if let Some(limit) = self.max_chars {
            let mut chars: Vec<char> = self.text.chars().collect();
            if chars.len() > limit {
                chars.truncate(limit);
                self.text = chars.into_iter().collect();
                self.positions_dirty = true;
                self.cursor_pos = self.cursor_pos.min(self.text.chars().count());
            }
        }
    }

    fn evaluate_validator(&mut self) {
        if let Some(validator) = &self.validator {
            self.error_message = validator(&self.text);
        } else {
            self.error_message = None;
        }
    }

    fn finalize_text_change(&mut self, state: &mut TextInputState, previous_text: String) -> bool {
        self.apply_sanitization();
        self.enforce_max_chars();
        let text_changed = self.text != previous_text;
        if text_changed {
            self.positions_dirty = true;
        }

        self.cursor_pos = self.cursor_pos.min(self.text.chars().count());
        state.cursor_pos = self.cursor_pos;
        state.selection_start = self.selection_start;
        state.text = self.text.clone();
        state.has_interacted = true;

        self.evaluate_validator();
        state.error_message = self.error_message.clone();

        if text_changed && let Some(callback) = &mut self.on_change {
            callback(self.text.clone());
        }

        text_changed
    }

    fn insert_text(&mut self, text: &str) -> bool {
        let mut changed = self.delete_selection();
        let mut chars: Vec<char> = self.text.chars().collect();
        let mut remaining = self.remaining_capacity();

        for ch in text.chars() {
            if let Some(rem) = remaining.as_mut() {
                if *rem == 0 {
                    break;
                }
                *rem -= 1;
            }
            chars.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
            changed = true;
        }

        if changed {
            self.text = chars.into_iter().collect();
            self.positions_dirty = true;
        }

        changed
    }

    /// Update character positions only if text or font size changed
    fn update_char_positions(&mut self, ctx: &mut WidgetContext) {
        let mask_active = self.should_mask();
        if mask_active != self.last_mask_state {
            self.positions_dirty = true;
            self.last_mask_state = mask_active;
        }

        let current_font_size = ctx.theme.font.size_body;

        // Check if we need to recalculate
        if self.positions_dirty
            || self.cached_text != self.text
            || self.font_size != current_font_size
        {
            self.font_size = current_font_size;
            self.cached_text = self.text.clone();
            self.positions_dirty = false;

            if mask_active {
                let char_count = self.text.chars().count();
                self.display_text = std::iter::repeat_n(self.mask_char, char_count).collect();
            } else {
                self.display_text = self.text.clone();
            }

            let chars: Vec<char> = self.display_text.chars().collect();
            self.char_positions.clear();
            self.char_positions.reserve(chars.len() + 1);

            for i in 0..=chars.len() {
                let text_before: String = chars[..i].iter().collect();
                let width = ctx.painter.get_text_size(&text_before, self.font_size).x;
                self.char_positions.push(width);
            }
        }
    }

    fn password_visible(&self) -> bool {
        if let Some(flag) = &self.password_visibility
            && let Ok(visible) = flag.lock()
        {
            return *visible;
        }
        false
    }

    fn should_mask(&self) -> bool {
        self.is_password && !self.password_visible()
    }

    fn get_cursor_pos_from_mouse(&self, mouse_x: f32) -> usize {
        let text_pos_x = self.rect.pos.x + 5.0; // Same offset used in draw
        let relative_x = mouse_x - text_pos_x + self.scroll_offset;

        if relative_x <= 0.0 {
            return 0;
        }

        // Use cached character positions for accurate cursor placement
        if self.char_positions.is_empty() {
            return 0;
        }

        // Find the closest character position
        for i in 0..self.char_positions.len() {
            if self.char_positions[i] > relative_x {
                // Check which position is closer
                if i > 0 {
                    let mid_point = (self.char_positions[i - 1] + self.char_positions[i]) / 2.0;
                    return if relative_x < mid_point { i - 1 } else { i };
                }
                return 0;
            }
        }

        // Clicked past the end of text
        self.char_positions.len() - 1
    }

    /// Get the normalized selection range (start, end) where start <= end
    fn get_selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start < self.cursor_pos {
                (start, self.cursor_pos)
            } else {
                (self.cursor_pos, start)
            }
        })
    }

    /// Get the selected text as a string
    fn get_selected_text(&self) -> Option<String> {
        self.get_selection_range().map(|(start, end)| {
            let chars: Vec<char> = self.text.chars().collect();
            chars[start..end].iter().collect()
        })
    }

    /// Delete the current selection and return true if a selection was deleted
    fn delete_selection(&mut self) -> bool {
        if let Some((sel_start, sel_end)) = self.get_selection_range() {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.drain(sel_start..sel_end);
            self.text = chars.into_iter().collect();
            self.cursor_pos = sel_start;
            self.selection_start = None;
            self.positions_dirty = true; // Mark for recalculation
            true
        } else {
            false
        }
    }

    /// Clear the selection without deleting text
    fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    /// Start or extend selection based on shift modifier
    fn handle_selection_with_shift(&mut self, extend: bool) {
        if extend {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_pos);
            }
        } else {
            self.clear_selection();
        }
    }

    /// Update scroll offset to ensure cursor stays visible
    fn update_scroll_offset(&mut self) {
        if self.char_positions.is_empty() {
            self.scroll_offset = 0.0;
            return;
        }

        let inner_width = (self.rect.size.x - 10.0).max(0.0); // Account for padding
        let cursor_x = if self.cursor_pos < self.char_positions.len() {
            self.char_positions[self.cursor_pos]
        } else {
            *self.char_positions.last().unwrap_or(&0.0)
        };
        let text_width = *self.char_positions.last().unwrap_or(&0.0);
        let max_scroll = (text_width - inner_width).max(0.0);

        // Calculate visible range based on current scroll offset
        let visible_start = self.scroll_offset;
        let visible_end = self.scroll_offset + inner_width;

        // Adjust scroll to keep cursor visible
        if cursor_x < visible_start {
            // Cursor is to the left of visible area
            self.scroll_offset = cursor_x.max(0.0);
        } else if cursor_x > visible_end {
            // Cursor is to the right of visible area
            self.scroll_offset = (cursor_x - inner_width).max(0.0);
        }

        // Ensure we don't scroll past the text bounds
        self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
    }

    /// Copy selected text to clipboard with error handling
    fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        arboard::Clipboard::new()
            .map_err(|e| format!("Failed to open clipboard: {}", e))?
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard text: {}", e))
    }

    /// Get text from clipboard with error handling
    fn get_from_clipboard(&self) -> Result<String, String> {
        arboard::Clipboard::new()
            .map_err(|e| format!("Failed to open clipboard: {}", e))?
            .get_text()
            .map_err(|e| format!("Failed to get clipboard text: {}", e))
    }

    pub fn on_change<F: FnMut(String) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Option<String> + 'static,
    {
        self.validator = Some(Box::new(f));
        self
    }

    pub fn max_chars(mut self, limit: usize) -> Self {
        self.max_chars = Some(limit);
        self
    }

    pub fn sanitize_with<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + 'static,
    {
        self.sanitizer = Some(Box::new(f));
        self
    }
}

impl Widget for TextInput {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Update character positions only if needed
        self.update_char_positions(ctx);

        // Load state
        let state: TextInputState = ctx.get_state();
        // Sync widget fields from state
        self.text = state.text.clone();
        self.cursor_pos = state.cursor_pos;
        self.focused = state.focused;
        self.selection_start = state.selection_start;
        self.error_message = state.error_message.clone();
        let is_hovered = state.hovered;
        let show_error = state.has_interacted && self.error_message.is_some();
        self.error_visible = show_error;
        let display_error = if show_error {
            self.error_message.clone()
        } else {
            None
        };

        // Update scroll offset to keep cursor visible (do this early, before using ctx)
        self.update_scroll_offset();

        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;

        let border_color = if show_error {
            ctx.theme.colors.error
        } else if self.focused || is_hovered {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.border
        };

        // Draw border (outer rounded rect)
        ctx.painter
            .draw_rounded_rect(self.rect, border_color, radius);

        // Draw background (inner rounded rect)
        let inset = border_width;
        let inner_rect = Rect::new(
            self.rect.pos.x + inset,
            self.rect.pos.y + inset,
            self.rect.size.x - inset * 2.0,
            self.rect.size.y - inset * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.widget_bg,
            (radius - inset).max(0.0),
        );

        let placeholder_active = self.text.is_empty() && !self.focused;
        let text_to_draw = if placeholder_active {
            &self.hint
        } else {
            &self.display_text
        };

        let text_color = if placeholder_active {
            ctx.theme.colors.text_dim
        } else {
            ctx.theme.colors.text
        };

        let text_pos = Vec2::new(
            self.rect.pos.x + 5.0 - self.scroll_offset,
            self.rect.pos.y + (self.rect.size.y - ctx.theme.font.size_body) / 2.0,
        );

        // Set scissor to clip text to the inner rect bounds
        let previous_scissor = ctx.painter.set_scissor(Some(inner_rect));

        // Draw selection highlight
        if let Some((start, end)) = self.get_selection_range() {
            let text_before_sel: String = self.display_text.chars().take(start).collect();
            let selected_text: String = self
                .display_text
                .chars()
                .skip(start)
                .take(end - start)
                .collect();

            let start_offset = ctx
                .painter
                .get_text_size(&text_before_sel, ctx.theme.font.size_body);
            let sel_width = ctx
                .painter
                .get_text_size(&selected_text, ctx.theme.font.size_body);
            let mut sel_rect = Rect::new(
                text_pos.x + start_offset.x,
                text_pos.y,
                sel_width.x,
                ctx.theme.font.size_body,
            );

            // Clamp highlight horizontally so it stays within the inner bounds even before scissoring
            let inner_start = inner_rect.pos.x;
            let inner_end = inner_rect.pos.x + inner_rect.size.x;
            let highlight_start = sel_rect.pos.x;
            let highlight_end = sel_rect.pos.x + sel_rect.size.x;
            let clamped_start = highlight_start.max(inner_start);
            let clamped_end = highlight_end.min(inner_end);

            if clamped_end > clamped_start {
                sel_rect.pos.x = clamped_start;
                sel_rect.size.x = clamped_end - clamped_start;
                ctx.painter.draw_rect(sel_rect, ctx.theme.colors.primary);
            }
        }

        // Draw the main text
        ctx.painter.draw_text(
            text_to_draw,
            text_pos,
            text_color,
            ctx.theme.font.size_body,
            Wrap::None,
        );

        // Draw IME composition text if active
        if let Some(ref composition) = self.ime_composition {
            let text_before_cursor: String =
                self.display_text.chars().take(self.cursor_pos).collect();
            let before_size = ctx
                .painter
                .get_text_size(&text_before_cursor, ctx.theme.font.size_body);
            let composition_pos = Vec2::new(text_pos.x + before_size.x, text_pos.y);
            let composition_display = if self.is_password {
                std::iter::repeat_n(self.mask_char, composition.chars().count()).collect::<String>()
            } else {
                composition.clone()
            };
            // Draw composition text with underline or different style
            ctx.painter.draw_text(
                &composition_display,
                composition_pos,
                ctx.theme.colors.primary,
                ctx.theme.font.size_body,
                Wrap::None,
            );
        }

        // Restore previous scissor
        ctx.painter.set_scissor(previous_scissor);

        // Cursor (only show when no selection and no IME composition)
        if self.focused && self.selection_start.is_none() && self.ime_composition.is_none() {
            // Position cursor at cursor_pos
            let text_before_cursor: String =
                self.display_text.chars().take(self.cursor_pos).collect();
            let text_size = ctx
                .painter
                .get_text_size(&text_before_cursor, ctx.theme.font.size_body);
            let cursor_x = text_pos.x + text_size.x;
            // Only draw cursor if it's within the visible area
            if cursor_x >= self.rect.pos.x + 5.0
                && cursor_x <= self.rect.pos.x + self.rect.size.x - 5.0
            {
                let cursor_rect = Rect::new(cursor_x, text_pos.y, 2.0, ctx.theme.font.size_body);
                ctx.painter.draw_rect(cursor_rect, ctx.theme.colors.primary);
            }
        }

        if let Some(ref message) = display_error {
            let text_pos = Vec2::new(
                self.rect.pos.x,
                self.rect.pos.y + self.rect.size.y + ctx.theme.metrics.spacing * 0.5,
            );
            ctx.painter.draw_text(
                message,
                text_pos,
                ctx.theme.colors.error,
                ctx.theme.font.size_small,
                Wrap::None,
            );
        }
    }

    fn layout(&mut self, ctx: &mut WidgetContext, mut available_space: Rect) {
        let base_height = self.base_height(ctx);
        let error_height = self.error_height(ctx);
        let mut input_height = (available_space.size.y - error_height).max(0.0);
        input_height = input_height.min(base_height);
        available_space.size.y = input_height;
        self.rect = available_space;

        // Register as focusable widget
        if self.is_focusable() {
            ctx.focus_manager.register_focusable(ctx.current_id());
        }
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let base_height = self.base_height(ctx);
        let error_height = self.error_height(ctx);
        constraints.constrain(Vec2::new(200.0, base_height + error_height))
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        // Load state at start
        let mut state: TextInputState = ctx.get_state();

        // Sync widget fields from state
        self.text = state.text.clone();
        self.cursor_pos = state.cursor_pos;
        self.focused = state.focused;
        self.selection_start = state.selection_start;
        self.error_message = state.error_message.clone();
        let mut needs_scroll_update = false;

        let result = match event {
            Event::Move(pos) => {
                let was_hovered = state.hovered;
                state.hovered = self.rect.contains(*pos);

                // Handle text selection dragging
                if self.is_dragging && self.focused {
                    self.cursor_pos = self.get_cursor_pos_from_mouse(pos.x);
                    state.cursor_pos = self.cursor_pos;
                    needs_scroll_update = true;
                    true
                } else {
                    was_hovered != state.hovered
                }
            }
            Event::Press(pos) => {
                let was_focused = state.focused;
                state.focused = self.rect.contains(*pos);
                self.focused = state.focused;

                if self.focused {
                    // Position cursor at click location
                    self.cursor_pos = self.get_cursor_pos_from_mouse(pos.x);
                    state.cursor_pos = self.cursor_pos;
                    self.selection_start = Some(self.cursor_pos);
                    state.selection_start = self.selection_start;
                    self.is_dragging = true;
                    needs_scroll_update = true;
                    true
                } else {
                    // Lost focus, clear selection
                    self.selection_start = None;
                    state.selection_start = None;
                    self.is_dragging = false;
                    if was_focused {
                        state.has_interacted = true;
                    }
                    was_focused != state.focused
                }
            }
            Event::Release(_pos) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    // If selection start equals cursor pos, clear selection (just a click, not a drag)
                    if let Some(start) = self.selection_start
                        && start == self.cursor_pos
                    {
                        self.selection_start = None;
                        state.selection_start = None;
                    }
                    true
                } else {
                    false
                }
            }
            Event::Char(c) => {
                if self.focused {
                    // Handle space explicitly (including tab as space)
                    if *c == ' ' || *c == '\t' || !c.is_control() {
                        let previous_text = self.text.clone();
                        let mut buffer = String::new();
                        buffer.push(*c);
                        if self.insert_text(&buffer)
                            && self.finalize_text_change(&mut state, previous_text)
                        {
                            needs_scroll_update = true;
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Event::Key(key, modifiers) => {
                if self.focused {
                    use winit::keyboard::{Key, NamedKey};

                    // Handle Ctrl+A (Select All)
                    if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("a"))
                    {
                        self.selection_start = Some(0);
                        state.selection_start = self.selection_start;
                        self.cursor_pos = self.text.chars().count();
                        state.cursor_pos = self.cursor_pos;
                        needs_scroll_update = true;
                        true
                    }
                    // Handle Ctrl+C (Copy)
                    else if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("c"))
                    {
                        let mask_active = self.should_mask();
                        if mask_active && !self.allow_password_copy {
                            true
                        } else {
                            if let Some(selected) = self.get_selected_text()
                                && let Err(e) = self.copy_to_clipboard(&selected)
                            {
                                eprintln!("Clipboard copy failed: {}", e);
                            }
                            true
                        }
                    }
                    // Handle Ctrl+X (Cut)
                    else if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("x"))
                    {
                        let mask_active = self.should_mask();
                        if mask_active && !self.allow_password_copy {
                            let previous_text = self.text.clone();
                            if self.delete_selection()
                                && self.finalize_text_change(&mut state, previous_text)
                            {
                                needs_scroll_update = true;
                            }
                            true
                        } else {
                            if let Some(selected) = self.get_selected_text() {
                                if let Err(e) = self.copy_to_clipboard(&selected) {
                                    eprintln!("Clipboard copy failed: {}", e);
                                }
                                let previous_text = self.text.clone();
                                if self.delete_selection()
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                            }
                            true
                        }
                    }
                    // Handle Ctrl+V (Paste)
                    else if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("v"))
                    {
                        match self.get_from_clipboard() {
                            Ok(text) => {
                                let previous_text = self.text.clone();
                                if self.insert_text(&text)
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                            }
                            Err(e) => {
                                eprintln!("Clipboard paste failed: {}", e);
                            }
                        }
                        true
                    } else {
                        match key {
                            // Handle Space key explicitly
                            Key::Named(NamedKey::Space) => {
                                let previous_text = self.text.clone();
                                if self.insert_text(" ")
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            // Handle Tab key for focus navigation
                            Key::Named(NamedKey::Tab) => {
                                if modifiers.shift {
                                    ctx.focus_manager.focus_prev();
                                } else {
                                    ctx.focus_manager.focus_next();
                                }
                                ctx.request_redraw();
                                // Lose focus - the FocusManager will set focus on the next widget
                                state.focused = false;
                                self.focused = false;
                                self.selection_start = None;
                                state.selection_start = None;
                                ctx.set_state(state);
                                return true; // Consume Tab event and exit early
                            }
                            Key::Named(NamedKey::Backspace) => {
                                let previous_text = self.text.clone();
                                let mut changed = self.delete_selection();
                                if !changed && self.cursor_pos > 0 {
                                    let mut chars: Vec<char> = self.text.chars().collect();
                                    chars.remove(self.cursor_pos - 1);
                                    self.cursor_pos -= 1;
                                    self.text = chars.into_iter().collect();
                                    self.positions_dirty = true;
                                    changed = true;
                                }

                                if changed && self.finalize_text_change(&mut state, previous_text) {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::Delete) => {
                                let previous_text = self.text.clone();
                                let mut changed = self.delete_selection();
                                if !changed {
                                    let char_count = self.text.chars().count();
                                    if self.cursor_pos < char_count {
                                        let mut chars: Vec<char> = self.text.chars().collect();
                                        chars.remove(self.cursor_pos);
                                        self.text = chars.into_iter().collect();
                                        self.positions_dirty = true;
                                        changed = true;
                                    }
                                }

                                if changed && self.finalize_text_change(&mut state, previous_text) {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::ArrowLeft) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                if self.cursor_pos > 0 {
                                    self.cursor_pos -= 1;
                                    state.cursor_pos = self.cursor_pos;
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::ArrowRight) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                let char_count = self.text.chars().count();
                                if self.cursor_pos < char_count {
                                    self.cursor_pos += 1;
                                    state.cursor_pos = self.cursor_pos;
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::Home) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                if self.cursor_pos != 0 {
                                    self.cursor_pos = 0;
                                    state.cursor_pos = 0;
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::End) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                let end = self.text.chars().count();
                                if self.cursor_pos != end {
                                    self.cursor_pos = end;
                                    state.cursor_pos = self.cursor_pos;
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            _ => false,
                        }
                    }
                } else {
                    false
                }
            }
            Event::Ime(ime_event) => {
                if self.focused {
                    use crate::gui::core::ImeEvent;
                    match ime_event {
                        ImeEvent::Start => {
                            // Start IME composition
                            self.ime_composition = Some(String::new());
                            true
                        }
                        ImeEvent::Update(text) => {
                            // Update IME composition text
                            self.ime_composition = Some(text.clone());
                            self.positions_dirty = true;
                            true
                        }
                        ImeEvent::Commit(text) => {
                            let previous_text = self.text.clone();
                            self.insert_text(text.as_str());
                            self.ime_composition = None;
                            if self.finalize_text_change(&mut state, previous_text) {
                                needs_scroll_update = true;
                            }
                            true
                        }
                        ImeEvent::Cancel => {
                            // Cancel IME composition
                            self.ime_composition = None;
                            self.positions_dirty = true;
                            true
                        }
                    }
                } else {
                    false
                }
            }
            Event::ContextClick(_) => false,
            Event::Scroll(_) => false,
        };

        if needs_scroll_update {
            self.update_char_positions(ctx);
            self.update_scroll_offset();
        }

        self.evaluate_validator();

        // Sync widget fields to state before saving
        state.text = self.text.clone();
        state.cursor_pos = self.cursor_pos;
        state.focused = self.focused;
        state.selection_start = self.selection_start;
        state.error_message = self.error_message.clone();
        self.error_visible = state.has_interacted && state.error_message.is_some();

        // Check if we should gain focus from FocusManager
        if !self.focused && ctx.focus_manager.has_focus(ctx.current_id()) {
            state.focused = true;
            self.focused = true;
        }

        // Save state at end
        ctx.set_state(state);
        result
    }

    fn is_focusable(&self) -> bool {
        true // TextInput can receive keyboard focus
    }
}

#[derive(Clone)]
struct LineLayout {
    start: usize,
    end: usize,
    text: String,
    cursor_positions: Vec<f32>,
    baseline: f32,
}

pub struct TextArea {
    pub text: String,
    pub hint: String,
    pub rect: Rect,
    pub focused: bool,
    pub cursor_pos: usize,
    pub selection_start: Option<usize>,
    pub on_change: Option<Box<dyn FnMut(String)>>,
    validator: Option<InputValidator>,
    error_message: Option<String>,
    error_visible: bool,
    max_chars: Option<usize>,
    sanitizer: Option<InputSanitizer>,
    pub ime_composition: Option<String>,
    pub is_dragging: bool,
    pub scroll_offset_y: f32,
    pub positions_dirty: bool,
    pub cached_text: String,
    pub cached_width: f32,
    pub font_size: f32,
    pub cursor_positions: Vec<Vec2>,
    line_layouts: Vec<LineLayout>,
    pub content_height: f32,
    pub auto_grow: bool,
    pub min_lines: usize,
    pub max_lines: Option<usize>,
    pub resizable: bool,
    pub resizing: bool,
    pub resize_start_mouse: Vec2,
    pub resize_start_height: f32,
    pub preferred_height: Option<f32>,
    auto_height: Option<f32>,
    pub allow_tab: bool,
    pub vertical_anchor_x: Option<f32>,
}

impl TextArea {
    fn error_height(&self, ctx: &WidgetContext) -> f32 {
        if self.error_visible {
            ctx.theme.metrics.spacing * 0.5 + ctx.theme.font.size_small
        } else {
            0.0
        }
    }

    const LINE_SPACING: f32 = 1.35;
    const PADDING_X: f32 = 6.0;
    const PADDING_Y: f32 = 6.0;
    const RESIZE_HANDLE: f32 = 12.0;

    pub fn new(hint: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            hint: hint.into(),
            rect: Rect::default(),
            focused: false,
            cursor_pos: 0,
            selection_start: None,
            on_change: None,
            validator: None,
            error_message: None,
            error_visible: false,
            max_chars: None,
            sanitizer: None,
            ime_composition: None,
            is_dragging: false,
            scroll_offset_y: 0.0,
            positions_dirty: true,
            cached_text: String::new(),
            cached_width: 0.0,
            font_size: 16.0,
            cursor_positions: Vec::new(),
            line_layouts: Vec::new(),
            content_height: 0.0,
            auto_grow: false,
            min_lines: 3,
            max_lines: None,
            resizable: false,
            resizing: false,
            resize_start_mouse: Vec2::ZERO,
            resize_start_height: 0.0,
            preferred_height: None,
            auto_height: None,
            allow_tab: false,
            vertical_anchor_x: None,
        }
    }

    pub fn on_change<F: FnMut(String) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn with_min_lines(mut self, lines: usize) -> Self {
        self.min_lines = lines.max(1);
        self
    }

    pub fn with_max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines.max(1));
        self
    }

    pub fn auto_grow(mut self, enabled: bool) -> Self {
        self.auto_grow = enabled;
        self
    }

    pub fn resizable(mut self, enabled: bool) -> Self {
        self.resizable = enabled;
        self
    }

    pub fn allow_tab(mut self, allow: bool) -> Self {
        self.allow_tab = allow;
        self
    }

    fn remaining_capacity(&self) -> Option<usize> {
        self.max_chars
            .map(|limit| limit.saturating_sub(self.text.chars().count()))
    }

    fn apply_sanitization(&mut self) {
        if let Some(sanitizer) = &self.sanitizer {
            let sanitized = sanitizer(&self.text);
            if sanitized != self.text {
                self.text = sanitized;
                self.positions_dirty = true;
                self.cursor_pos = self.cursor_pos.min(self.text.chars().count());
            }
        }
    }

    fn enforce_max_chars(&mut self) {
        if let Some(limit) = self.max_chars {
            let mut chars: Vec<char> = self.text.chars().collect();
            if chars.len() > limit {
                chars.truncate(limit);
                self.text = chars.into_iter().collect();
                self.positions_dirty = true;
                self.cursor_pos = self.cursor_pos.min(self.text.chars().count());
            }
        }
    }

    fn evaluate_validator(&mut self) {
        if let Some(validator) = &self.validator {
            self.error_message = validator(&self.text);
        } else {
            self.error_message = None;
        }
    }

    fn finalize_text_change(&mut self, state: &mut TextAreaState, previous_text: String) -> bool {
        self.apply_sanitization();
        self.enforce_max_chars();
        let changed = self.text != previous_text;
        if changed {
            self.positions_dirty = true;
        }

        self.cursor_pos = self.cursor_pos.min(self.text.chars().count());
        state.cursor_pos = self.cursor_pos;
        state.selection_start = self.selection_start;
        state.text = self.text.clone();
        state.has_interacted = true;

        self.evaluate_validator();
        state.error_message = self.error_message.clone();

        if changed && let Some(callback) = &mut self.on_change {
            callback(self.text.clone());
        }

        changed
    }

    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Option<String> + 'static,
    {
        self.validator = Some(Box::new(f));
        self
    }

    pub fn max_chars(mut self, limit: usize) -> Self {
        self.max_chars = Some(limit);
        self
    }

    pub fn sanitize_with<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + 'static,
    {
        self.sanitizer = Some(Box::new(f));
        self
    }

    fn line_height(&self) -> f32 {
        self.font_size * Self::LINE_SPACING
    }

    fn text_area_height(&self, inner_rect: &Rect) -> f32 {
        (inner_rect.size.y - Self::PADDING_Y * 2.0).max(0.0)
    }

    fn inner_rect(&self, ctx: &WidgetContext) -> Rect {
        let inset = ctx.theme.metrics.border_width;
        Rect::new(
            self.rect.pos.x + inset,
            self.rect.pos.y + inset,
            self.rect.size.x - inset * 2.0,
            self.rect.size.y - inset * 2.0,
        )
    }

    fn text_origin(&self, inner_rect: &Rect) -> Vec2 {
        Vec2::new(
            inner_rect.pos.x + Self::PADDING_X,
            inner_rect.pos.y + Self::PADDING_Y - self.scroll_offset_y,
        )
    }

    fn measure_line(&self, ctx: &mut WidgetContext, text: &str) -> Vec<f32> {
        let mut positions = Vec::with_capacity(text.chars().count() + 1);
        positions.push(0.0);
        if text.is_empty() {
            return positions;
        }

        let mut prefix = String::new();
        for ch in text.chars() {
            prefix.push(ch);
            let width = ctx.painter.get_text_size(&prefix, self.font_size).x;
            positions.push(width);
        }
        positions
    }

    fn push_line(&mut self, start: usize, end: usize, text: String, cursor_positions: Vec<f32>) {
        let baseline = self.line_layouts.len() as f32 * self.line_height();
        for (i, x) in cursor_positions.iter().enumerate() {
            let idx = start + i;
            if idx < self.cursor_positions.len() {
                self.cursor_positions[idx] = Vec2::new(*x, baseline);
            }
        }
        self.line_layouts.push(LineLayout {
            start,
            end,
            text,
            cursor_positions,
            baseline,
        });
    }

    fn wrap_segment(
        &mut self,
        ctx: &mut WidgetContext,
        segment: &[char],
        global_start: usize,
        inner_width: f32,
    ) {
        let width_limit = (inner_width - Self::PADDING_X * 2.0).max(1.0);
        if segment.is_empty() {
            self.push_line(global_start, global_start, String::new(), vec![0.0]);
            return;
        }

        let segment_text: String = segment.iter().collect();
        let measurements = self.measure_line(ctx, &segment_text);
        let len = segment.len();
        let mut local_start = 0;

        while local_start < len {
            let mut local_end = local_start;
            let mut last_break = None;

            while local_end < len {
                let width = measurements[local_end + 1] - measurements[local_start];
                if width > width_limit {
                    break;
                }
                if segment[local_end].is_whitespace() {
                    last_break = Some(local_end + 1);
                }
                local_end += 1;
            }

            let break_idx = if local_end == len {
                len
            } else if let Some(candidate) = last_break {
                candidate.max(local_start + 1)
            } else {
                (local_end).max(local_start + 1)
            };

            let text_slice: String = segment[local_start..break_idx].iter().collect();
            let mut rel_positions = Vec::with_capacity(break_idx - local_start + 1);
            for i in 0..=(break_idx - local_start) {
                let idx = local_start + i;
                let x = measurements[idx] - measurements[local_start];
                rel_positions.push(x);
            }

            let start_idx = global_start + local_start;
            let end_idx = global_start + break_idx;
            self.push_line(start_idx, end_idx, text_slice, rel_positions);

            local_start = break_idx;
        }
    }

    fn rebuild_layout(&mut self, ctx: &mut WidgetContext) {
        let font_size = ctx.theme.font.size_body;
        let inner_rect = self.inner_rect(ctx);
        let inner_width = inner_rect.size.x.max(0.0);

        if !self.positions_dirty
            && (self.font_size - font_size).abs() < f32::EPSILON
            && (self.cached_width - inner_width).abs() < f32::EPSILON
            && self.cached_text == self.text
        {
            return;
        }

        self.font_size = font_size;
        self.cached_width = inner_width;
        self.cached_text = self.text.clone();
        self.positions_dirty = false;

        self.line_layouts.clear();
        self.cursor_positions.clear();

        let chars: Vec<char> = self.text.chars().collect();
        self.cursor_positions
            .resize(chars.len() + 1, Vec2::new(0.0, 0.0));

        let mut segment_start = 0;
        for (idx, ch) in chars.iter().enumerate() {
            if *ch == '\n' {
                self.wrap_segment(ctx, &chars[segment_start..idx], segment_start, inner_width);
                segment_start = idx + 1;
            }
        }
        self.wrap_segment(ctx, &chars[segment_start..], segment_start, inner_width);

        if self.line_layouts.is_empty() {
            self.wrap_segment(ctx, &[], chars.len(), inner_width);
        }

        let line_count = self.line_layouts.len().max(1);
        self.content_height = line_count as f32 * self.line_height();

        if self.auto_grow && self.preferred_height.is_none() {
            let mut lines = self.line_layouts.len().max(self.min_lines);
            if let Some(max_lines) = self.max_lines {
                lines = lines.min(max_lines);
            }
            let mut auto_height = lines as f32 * self.line_height()
                + Self::PADDING_Y * 2.0
                + ctx.theme.metrics.border_width * 2.0;
            if let Some(max_lines) = self.max_lines {
                let max_height = max_lines as f32 * self.line_height()
                    + Self::PADDING_Y * 2.0
                    + ctx.theme.metrics.border_width * 2.0;
                auto_height = auto_height.min(max_height);
            }
            self.auto_height = Some(auto_height);
        } else {
            self.auto_height = None;
        }
    }

    fn max_scroll(&self, view_height: f32) -> f32 {
        (self.content_height - view_height).max(0.0)
    }

    fn ensure_cursor_visible(&mut self, view_height: f32) {
        if self.cursor_positions.is_empty() {
            return;
        }
        let idx = self.cursor_pos.min(self.cursor_positions.len() - 1);
        let cursor = self.cursor_positions[idx];
        let cursor_top = cursor.y;
        let cursor_bottom = cursor.y + self.line_height();
        let visible_top = self.scroll_offset_y;
        let visible_bottom = self.scroll_offset_y + view_height;

        if cursor_top < visible_top {
            self.scroll_offset_y = cursor_top.max(0.0);
        } else if cursor_bottom > visible_bottom {
            self.scroll_offset_y = (cursor_bottom - view_height).max(0.0);
        }

        let max_scroll = self.max_scroll(view_height);
        self.scroll_offset_y = self.scroll_offset_y.clamp(0.0, max_scroll);
    }

    fn get_selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start < self.cursor_pos {
                (start, self.cursor_pos)
            } else {
                (self.cursor_pos, start)
            }
        })
    }

    fn get_selected_text(&self) -> Option<String> {
        self.get_selection_range().map(|(start, end)| {
            let chars: Vec<char> = self.text.chars().collect();
            chars[start..end].iter().collect()
        })
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.get_selection_range() {
            let mut chars: Vec<char> = self.text.chars().collect();
            chars.drain(start..end);
            self.text = chars.into_iter().collect();
            self.cursor_pos = start;
            self.selection_start = None;
            self.positions_dirty = true;
            self.vertical_anchor_x = None;
            true
        } else {
            false
        }
    }

    fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    fn handle_selection_with_shift(&mut self, extend: bool) {
        if extend {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_pos);
            }
        } else {
            self.clear_selection();
        }
    }

    fn cursor_from_point(&self, point: Vec2, text_origin: Vec2) -> usize {
        if self.cursor_positions.is_empty() {
            return 0;
        }

        let relative = Vec2::new(
            point.x - text_origin.x,
            point.y - text_origin.y + self.scroll_offset_y,
        );
        let mut best_idx = 0;
        let mut best_score = f32::MAX;
        for (idx, pos) in self.cursor_positions.iter().enumerate() {
            let dx = (pos.x - relative.x).abs();
            let dy = (pos.y - relative.y).abs();
            let score = dy * 4.0 + dx;
            if score < best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        best_idx
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        arboard::Clipboard::new()
            .map_err(|e| format!("Failed to open clipboard: {}", e))?
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard text: {}", e))
    }

    fn get_from_clipboard(&self) -> Result<String, String> {
        arboard::Clipboard::new()
            .map_err(|e| format!("Failed to open clipboard: {}", e))?
            .get_text()
            .map_err(|e| format!("Failed to get clipboard text: {}", e))
    }

    fn insert_text(&mut self, text: &str) -> bool {
        let mut changed = self.delete_selection();
        let mut chars: Vec<char> = self.text.chars().collect();
        let mut remaining = self.remaining_capacity();
        for ch in text.chars() {
            if let Some(rem) = remaining.as_mut() {
                if *rem == 0 {
                    break;
                }
                *rem -= 1;
            }
            chars.insert(self.cursor_pos, ch);
            self.cursor_pos += 1;
            changed = true;
        }
        if changed {
            self.text = chars.into_iter().collect();
            self.positions_dirty = true;
            self.vertical_anchor_x = None;
        }
        changed
    }

    fn line_index_for(&self, idx: usize) -> usize {
        for (line_idx, line) in self.line_layouts.iter().enumerate() {
            if idx >= line.start && idx <= line.end {
                return line_idx;
            }
        }
        self.line_layouts.len().saturating_sub(1)
    }

    fn line_bounds(&self, idx: usize) -> (usize, usize) {
        if self.line_layouts.is_empty() {
            return (0, 0);
        }
        let line_idx = self.line_index_for(idx);
        let line = &self.line_layouts[line_idx];
        (line.start, line.end)
    }

    fn move_to_line_edge(&mut self, start: bool) {
        let (line_start, line_end) = self.line_bounds(self.cursor_pos);
        self.cursor_pos = if start { line_start } else { line_end };
        self.vertical_anchor_x = None;
    }

    fn move_vertical(&mut self, direction: i32) {
        if self.cursor_positions.is_empty() {
            return;
        }
        let len = self.cursor_positions.len();
        let current_idx = self.cursor_pos.min(len - 1);
        let current = self.cursor_positions[current_idx];
        let target_y = current.y + direction as f32 * self.line_height();
        let anchor_x = self
            .vertical_anchor_x
            .unwrap_or_else(|| self.cursor_positions[current_idx].x);
        self.vertical_anchor_x = Some(anchor_x);

        let mut best_idx = current_idx;
        let mut best_score = f32::MAX;
        let mut found = false;
        for (idx, pos) in self.cursor_positions.iter().enumerate() {
            if direction < 0 && pos.y >= current.y {
                continue;
            }
            if direction > 0 && pos.y <= current.y {
                continue;
            }
            let dy = (pos.y - target_y).abs();
            let dx = (pos.x - anchor_x).abs();
            let score = dy * 4.0 + dx;
            if score < best_score {
                best_score = score;
                best_idx = idx;
                found = true;
            }
        }
        if found {
            self.cursor_pos = best_idx;
        }
    }

    fn draw_selection(&self, ctx: &mut WidgetContext, inner_rect: &Rect, text_origin: Vec2) {
        if let Some((start, end)) = self.get_selection_range() {
            if start == end {
                return;
            }
            let clip_start = inner_rect.pos.x;
            let clip_end = inner_rect.pos.x + inner_rect.size.x;
            for line in &self.line_layouts {
                let line_start = line.start;
                let line_end = line.end;
                if end <= line_start || start >= line_end {
                    continue;
                }
                let local_start = start.max(line_start) - line_start;
                let local_end = end.min(line_end) - line_start;
                if local_start == local_end {
                    continue;
                }
                let start_x = line
                    .cursor_positions
                    .get(local_start)
                    .copied()
                    .unwrap_or(0.0);
                let end_x = line
                    .cursor_positions
                    .get(local_end)
                    .copied()
                    .unwrap_or(start_x);
                let y = text_origin.y + line.baseline;
                let mut rect = Rect::new(
                    text_origin.x + start_x,
                    y,
                    (end_x - start_x).max(1.0),
                    self.line_height(),
                );
                let rect_end = rect.pos.x + rect.size.x;
                rect.pos.x = rect.pos.x.max(clip_start);
                let clamped_end = rect_end.min(clip_end);
                rect.size.x = (clamped_end - rect.pos.x).max(0.0);
                if rect.size.x > 0.0 {
                    ctx.painter.draw_rect(rect, ctx.theme.colors.primary);
                }
            }
        }
    }

    fn draw_lines(&self, ctx: &mut WidgetContext, text_origin: Vec2, inner_rect: &Rect) {
        let view_top = inner_rect.pos.y;
        let view_bottom = inner_rect.pos.y + inner_rect.size.y;
        for line in &self.line_layouts {
            let line_top = text_origin.y + line.baseline;
            let line_bottom = line_top + self.line_height();
            if line_bottom < view_top || line_top > view_bottom {
                continue;
            }
            ctx.painter.draw_text(
                &line.text,
                Vec2::new(text_origin.x, line_top),
                ctx.theme.colors.text,
                self.font_size,
                Wrap::None,
            );
        }
    }

    fn resize_handle_rect(&self) -> Rect {
        Rect::new(
            self.rect.pos.x + self.rect.size.x - Self::RESIZE_HANDLE,
            self.rect.pos.y + self.rect.size.y - Self::RESIZE_HANDLE,
            Self::RESIZE_HANDLE,
            Self::RESIZE_HANDLE,
        )
    }

    fn min_visual_height(&self, ctx: &WidgetContext) -> f32 {
        (self.min_lines as f32 * self.line_height())
            + Self::PADDING_Y * 2.0
            + ctx.theme.metrics.border_width * 2.0
    }
}

impl Widget for TextArea {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        let mut state: TextAreaState = ctx.get_state();
        self.text = state.text.clone();
        self.focused = state.focused;
        self.cursor_pos = state.cursor_pos.min(self.text.chars().count());
        self.selection_start = state.selection_start;
        self.preferred_height = state.preferred_height;
        self.error_message = state.error_message.clone();
        self.error_visible = state.has_interacted && self.error_message.is_some();
        let show_error = self.error_visible;

        self.rebuild_layout(ctx);

        let inner_rect = self.inner_rect(ctx);
        let view_height = self.text_area_height(&inner_rect);
        self.ensure_cursor_visible(view_height);

        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;
        let border_color = if show_error {
            ctx.theme.colors.error
        } else if self.focused || state.hovered {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.border
        };

        ctx.painter
            .draw_rounded_rect(self.rect, border_color, radius);

        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.widget_bg,
            (radius - border_width).max(0.0),
        );

        let previous_scissor = ctx.painter.set_scissor(Some(inner_rect));
        let text_origin = self.text_origin(&inner_rect);

        let placeholder_active = self.text.is_empty() && !self.focused;
        if placeholder_active {
            ctx.painter.draw_text(
                &self.hint,
                Vec2::new(text_origin.x, inner_rect.pos.y + Self::PADDING_Y),
                ctx.theme.colors.text_dim,
                self.font_size,
                Wrap::Word,
            );
        } else {
            self.draw_selection(ctx, &inner_rect, text_origin);
            self.draw_lines(ctx, text_origin, &inner_rect);
        }

        if let Some(ref composition) = self.ime_composition {
            let cursor_idx = self.cursor_pos.min(self.cursor_positions.len() - 1);
            let cursor_pos = self.cursor_positions[cursor_idx];
            let ime_pos = Vec2::new(text_origin.x + cursor_pos.x, text_origin.y + cursor_pos.y);
            ctx.painter.draw_text(
                composition,
                ime_pos,
                ctx.theme.colors.primary,
                self.font_size,
                Wrap::None,
            );
        }

        if self.focused && self.selection_start.is_none() && self.ime_composition.is_none() {
            let idx = self.cursor_pos.min(self.cursor_positions.len() - 1);
            let cursor = self.cursor_positions[idx];
            let cursor_x = text_origin.x + cursor.x;
            let cursor_y = text_origin.y + cursor.y;
            if cursor_y >= inner_rect.pos.y - self.line_height()
                && cursor_y <= inner_rect.pos.y + inner_rect.size.y
            {
                let cursor_rect = Rect::new(cursor_x, cursor_y, 2.0, self.line_height());
                ctx.painter.draw_rect(cursor_rect, ctx.theme.colors.primary);
            }
        }

        ctx.painter.set_scissor(previous_scissor);

        if show_error && let Some(ref message) = self.error_message {
            let text_pos = Vec2::new(
                self.rect.pos.x,
                self.rect.pos.y + self.rect.size.y + ctx.theme.metrics.spacing * 0.5,
            );
            ctx.painter.draw_text(
                message,
                text_pos,
                ctx.theme.colors.error,
                ctx.theme.font.size_small,
                Wrap::None,
            );
        }

        if self.content_height > view_height {
            let track_start = inner_rect.pos.y + Self::PADDING_Y;
            let track_height = inner_rect.size.y - Self::PADDING_Y * 2.0;
            let ratio = view_height / self.content_height;
            let thumb_height = (track_height * ratio).max(20.0);
            let max_scroll = self.max_scroll(view_height);
            let scroll_ratio = if max_scroll > 0.0 {
                self.scroll_offset_y / max_scroll
            } else {
                0.0
            };
            let thumb_y = track_start + scroll_ratio * (track_height - thumb_height);
            let thumb_rect = Rect::new(
                inner_rect.pos.x + inner_rect.size.x - 4.0,
                thumb_y,
                3.0,
                thumb_height,
            );
            ctx.painter.draw_rect(thumb_rect, ctx.theme.colors.border);
        }

        if self.resizable {
            let handle = self.resize_handle_rect();
            ctx.painter.draw_rect(handle, ctx.theme.colors.border);
        }

        state.text = self.text.clone();
        state.focused = self.focused;
        state.cursor_pos = self.cursor_pos;
        state.selection_start = self.selection_start;
        state.preferred_height = self.preferred_height;
        state.error_message = self.error_message.clone();

        ctx.set_state(state);
    }

    fn layout(&mut self, ctx: &mut WidgetContext, mut available_space: Rect) {
        let error_height = self.error_height(ctx);
        let content_space = (available_space.size.y - error_height).max(0.0);
        available_space.size.y = content_space;
        self.rect = available_space;
        if let Some(desired) = self.preferred_height.or(self.auto_height) {
            self.rect.size.y = desired.min(content_space);
        }
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let line_count = self.text.split('\n').count().max(self.min_lines).max(1);
        let mut height = line_count as f32 * ctx.theme.font.size_body * Self::LINE_SPACING
            + Self::PADDING_Y * 2.0
            + ctx.theme.metrics.border_width * 2.0;
        if let Some(max_lines) = self.max_lines {
            height = height.min(
                max_lines as f32 * ctx.theme.font.size_body * Self::LINE_SPACING
                    + Self::PADDING_Y * 2.0
                    + ctx.theme.metrics.border_width * 2.0,
            );
        }
        if let Some(pref) = self.preferred_height.or(self.auto_height) {
            height = pref;
        }
        height += self.error_height(ctx);
        let width = 280.0;
        constraints.constrain(Vec2::new(width, height))
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut state: TextAreaState = ctx.get_state();

        self.text = state.text.clone();
        self.focused = state.focused;
        self.cursor_pos = state.cursor_pos.min(self.text.chars().count());
        self.selection_start = state.selection_start;
        self.preferred_height = state.preferred_height;

        self.rebuild_layout(ctx);

        let inner_rect = self.inner_rect(ctx);
        let text_origin = self.text_origin(&inner_rect);
        let view_height = self.text_area_height(&inner_rect);
        let mut needs_scroll_update = false;

        let handled = match event {
            Event::Move(pos) => {
                let was_hovered = state.hovered;
                state.hovered = self.rect.contains(*pos);
                if self.resizing {
                    let delta = pos.y - self.resize_start_mouse.y;
                    let new_height =
                        (self.resize_start_height + delta).max(self.min_visual_height(ctx));
                    self.preferred_height = Some(new_height);
                    state.preferred_height = self.preferred_height;
                    needs_scroll_update = true;
                    true
                } else if self.is_dragging && self.focused {
                    self.cursor_pos = self.cursor_from_point(*pos, text_origin);
                    state.cursor_pos = self.cursor_pos;
                    needs_scroll_update = true;
                    true
                } else {
                    was_hovered != state.hovered
                }
            }
            Event::Press(pos) => {
                if self.resizable && self.resize_handle_rect().contains(*pos) {
                    self.resizing = true;
                    self.resize_start_mouse = *pos;
                    self.resize_start_height = self.preferred_height.unwrap_or(self.rect.size.y);
                    true
                } else {
                    let was_focused = state.focused;
                    state.focused = self.rect.contains(*pos);
                    self.focused = state.focused;

                    if self.focused {
                        self.cursor_pos = self.cursor_from_point(*pos, text_origin);
                        state.cursor_pos = self.cursor_pos;
                        self.selection_start = Some(self.cursor_pos);
                        state.selection_start = self.selection_start;
                        self.is_dragging = true;
                        needs_scroll_update = true;
                        true
                    } else {
                        self.selection_start = None;
                        state.selection_start = None;
                        self.is_dragging = false;
                        if was_focused {
                            state.has_interacted = true;
                        }
                        was_focused != state.focused
                    }
                }
            }
            Event::Release(_pos) => {
                if self.resizing {
                    self.resizing = false;
                    state.preferred_height = self.preferred_height;
                    true
                } else if self.is_dragging {
                    self.is_dragging = false;
                    if let Some(start) = self.selection_start
                        && start == self.cursor_pos
                    {
                        self.selection_start = None;
                        state.selection_start = None;
                    }
                    true
                } else {
                    false
                }
            }
            Event::Scroll(delta) => {
                if state.hovered || self.focused {
                    let scroll_speed = 20.0;
                    let new = self.scroll_offset_y - delta * scroll_speed;
                    let max_scroll = self.max_scroll(view_height);
                    self.scroll_offset_y = new.clamp(0.0, max_scroll);
                    true
                } else {
                    false
                }
            }
            Event::Char(c) => {
                if self.focused {
                    match *c {
                        '\r' | '\n' => {
                            let previous_text = self.text.clone();
                            if self.insert_text("\n")
                                && self.finalize_text_change(&mut state, previous_text)
                            {
                                needs_scroll_update = true;
                            }
                            true
                        }
                        '\t' => {
                            if self.allow_tab {
                                let previous_text = self.text.clone();
                                if self.insert_text("\t")
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                                true
                            } else {
                                false
                            }
                        }
                        _ => {
                            if !c.is_control() {
                                let previous_text = self.text.clone();
                                let mut buffer = String::new();
                                buffer.push(*c);
                                if self.insert_text(&buffer)
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                                true
                            } else {
                                false
                            }
                        }
                    }
                } else {
                    false
                }
            }
            Event::Key(key, modifiers) => {
                if !self.focused {
                    false
                } else {
                    use winit::keyboard::{Key, NamedKey};
                    let mut handled_key = false;

                    if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("a"))
                    {
                        self.selection_start = Some(0);
                        state.selection_start = self.selection_start;
                        self.cursor_pos = self.text.chars().count();
                        state.cursor_pos = self.cursor_pos;
                        needs_scroll_update = true;
                        handled_key = true;
                    } else if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("c"))
                    {
                        if let Some(selected) = self.get_selected_text()
                            && let Err(e) = self.copy_to_clipboard(&selected)
                        {
                            eprintln!("Clipboard copy failed: {}", e);
                        }
                        handled_key = true;
                    } else if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("x"))
                    {
                        if let Some(selected) = self.get_selected_text() {
                            if let Err(e) = self.copy_to_clipboard(&selected) {
                                eprintln!("Clipboard cut failed: {}", e);
                            }
                            let previous_text = self.text.clone();
                            if self.delete_selection()
                                && self.finalize_text_change(&mut state, previous_text)
                            {
                                needs_scroll_update = true;
                            }
                        }
                        handled_key = true;
                    } else if modifiers.ctrl
                        && matches!(key, Key::Character(c) if c.as_ref().eq_ignore_ascii_case("v"))
                    {
                        match self.get_from_clipboard() {
                            Ok(text) => {
                                let previous_text = self.text.clone();
                                if self.insert_text(&text)
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                            }
                            Err(e) => eprintln!("Clipboard paste failed: {}", e),
                        }
                        handled_key = true;
                    }

                    if handled_key {
                        true
                    } else {
                        match key {
                            Key::Named(NamedKey::Backspace) => {
                                let previous_text = self.text.clone();
                                let mut changed = self.delete_selection();
                                if !changed && self.cursor_pos > 0 {
                                    let mut chars: Vec<char> = self.text.chars().collect();
                                    chars.remove(self.cursor_pos - 1);
                                    self.cursor_pos -= 1;
                                    self.text = chars.into_iter().collect();
                                    self.positions_dirty = true;
                                    self.vertical_anchor_x = None;
                                    changed = true;
                                }

                                if changed && self.finalize_text_change(&mut state, previous_text) {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::Delete) => {
                                let previous_text = self.text.clone();
                                let mut changed = self.delete_selection();
                                if !changed {
                                    let len = self.text.chars().count();
                                    if self.cursor_pos < len {
                                        let mut chars: Vec<char> = self.text.chars().collect();
                                        chars.remove(self.cursor_pos);
                                        self.text = chars.into_iter().collect();
                                        self.positions_dirty = true;
                                        self.vertical_anchor_x = None;
                                        changed = true;
                                    }
                                }

                                if changed && self.finalize_text_change(&mut state, previous_text) {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::ArrowLeft) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                if self.cursor_pos > 0 {
                                    self.cursor_pos -= 1;
                                    state.cursor_pos = self.cursor_pos;
                                    needs_scroll_update = true;
                                }
                                self.vertical_anchor_x = None;
                                true
                            }
                            Key::Named(NamedKey::ArrowRight) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                let len = self.text.chars().count();
                                if self.cursor_pos < len {
                                    self.cursor_pos += 1;
                                    state.cursor_pos = self.cursor_pos;
                                    needs_scroll_update = true;
                                }
                                self.vertical_anchor_x = None;
                                true
                            }
                            Key::Named(NamedKey::ArrowUp) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                self.move_vertical(-1);
                                state.cursor_pos = self.cursor_pos;
                                needs_scroll_update = true;
                                true
                            }
                            Key::Named(NamedKey::ArrowDown) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                self.move_vertical(1);
                                state.cursor_pos = self.cursor_pos;
                                needs_scroll_update = true;
                                true
                            }
                            Key::Named(NamedKey::Home) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                self.move_to_line_edge(true);
                                state.cursor_pos = self.cursor_pos;
                                needs_scroll_update = true;
                                true
                            }
                            Key::Named(NamedKey::End) => {
                                self.handle_selection_with_shift(modifiers.shift);
                                state.selection_start = self.selection_start;
                                self.move_to_line_edge(false);
                                state.cursor_pos = self.cursor_pos;
                                needs_scroll_update = true;
                                true
                            }
                            Key::Named(NamedKey::Enter) => {
                                let previous_text = self.text.clone();
                                if self.insert_text("\n")
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::Space) => {
                                let previous_text = self.text.clone();
                                if self.insert_text(" ")
                                    && self.finalize_text_change(&mut state, previous_text)
                                {
                                    needs_scroll_update = true;
                                }
                                true
                            }
                            Key::Named(NamedKey::Tab) => {
                                if self.allow_tab {
                                    let previous_text = self.text.clone();
                                    if self.insert_text("\t")
                                        && self.finalize_text_change(&mut state, previous_text)
                                    {
                                        needs_scroll_update = true;
                                    }
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        }
                    }
                }
            }
            Event::Ime(ime_event) => {
                if self.focused {
                    use crate::gui::core::ImeEvent;
                    match ime_event {
                        ImeEvent::Start => {
                            self.ime_composition = Some(String::new());
                            true
                        }
                        ImeEvent::Update(text) => {
                            self.ime_composition = Some(text.clone());
                            self.positions_dirty = true;
                            true
                        }
                        ImeEvent::Commit(text) => {
                            let previous_text = self.text.clone();
                            self.insert_text(text.as_str());
                            self.ime_composition = None;
                            if self.finalize_text_change(&mut state, previous_text) {
                                needs_scroll_update = true;
                            }
                            true
                        }
                        ImeEvent::Cancel => {
                            self.ime_composition = None;
                            self.positions_dirty = true;
                            true
                        }
                    }
                } else {
                    false
                }
            }
            Event::ContextClick(_) => false,
        };

        if needs_scroll_update {
            self.rebuild_layout(ctx);
            self.ensure_cursor_visible(view_height);
        }

        self.evaluate_validator();

        state.text = self.text.clone();
        state.focused = self.focused;
        state.cursor_pos = self.cursor_pos;
        state.selection_start = self.selection_start;
        state.preferred_height = self.preferred_height;
        state.error_message = self.error_message.clone();
        self.error_visible = state.has_interacted && state.error_message.is_some();

        ctx.set_state(state);
        handled
    }
}

pub struct ImageWidget {
    pub texture_id: usize,
    pub rect: Rect,
    pub size: Option<Vec2>,
}

impl ImageWidget {
    pub fn new(texture_id: usize) -> Self {
        Self {
            texture_id,
            rect: Rect::default(),
            size: None,
        }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }
}

impl Widget for ImageWidget {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.painter.draw_image(self.rect, self.texture_id);
    }

    fn layout(&mut self, _ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
    }

    fn size_hint(&self, _ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let size = self.size.unwrap_or(Vec2::new(100.0, 100.0));
        constraints.constrain(size)
    }

    fn handle_event(&mut self, _ctx: &mut WidgetContext, _event: &Event) -> bool {
        false
    }
}

pub enum Align {
    Start,
    Center,
    End,
    Stretch, // Fill the cross-axis (e.g., vertically in a column, horizontally in a row)
}

/// Flex configuration for widgets, controlling how they grow/shrink in flex containers
#[derive(Debug, Clone, Copy)]
pub struct FlexConfig {
    /// Flex grow factor (0.0 = don't grow, >0.0 = proportional growth)
    pub flex_grow: f32,
    /// Flex shrink factor (0.0 = don't shrink, >0.0 = proportional shrinkage)
    pub flex_shrink: f32,
    /// Minimum width constraint (0.0 = no minimum)
    pub min_width: f32,
    /// Maximum width constraint (f32::INFINITY = no maximum)
    pub max_width: f32,
    /// Minimum height constraint (0.0 = no minimum)
    pub min_height: f32,
    /// Maximum height constraint (f32::INFINITY = no maximum)
    pub max_height: f32,
}

impl Default for FlexConfig {
    fn default() -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }
}

impl FlexConfig {
    /// Create a flex config that doesn't grow or shrink
    pub fn fixed() -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..Default::default()
        }
    }

    /// Create a flex config that grows to fill available space
    pub fn grow(factor: f32) -> Self {
        Self {
            flex_grow: factor,
            flex_shrink: 1.0,
            ..Default::default()
        }
    }

    /// Set minimum size constraints
    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Set maximum size constraints
    pub fn with_max_size(mut self, width: f32, height: f32) -> Self {
        self.max_width = width;
        self.max_height = height;
        self
    }
}

/// Wrapper to attach flex configuration to a widget
pub struct FlexWidget {
    pub widget: Box<dyn Widget>,
    pub flex: FlexConfig,
}

impl FlexWidget {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self {
            widget: Box::new(widget),
            flex: FlexConfig::default(),
        }
    }

    pub fn with_flex(mut self, flex: FlexConfig) -> Self {
        self.flex = flex;
        self
    }
}

impl Widget for FlexWidget {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        self.widget.draw(ctx);
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.widget.layout(ctx, available_space);
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        // Apply flex min/max constraints
        let constrained = SizeConstraints::new(
            constraints.min_width.max(self.flex.min_width),
            constraints.max_width.min(self.flex.max_width),
            constraints.min_height.max(self.flex.min_height),
            constraints.max_height.min(self.flex.max_height),
        );
        self.widget.size_hint(ctx, constrained)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        self.widget.handle_event(ctx, event)
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        self.widget.draw_overlay(ctx);
    }
}

pub struct Column {
    pub children: Vec<FlexWidget>,
    pub rect: Rect,
    pub align_items: Align,
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Column {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            rect: Rect::default(),
            align_items: Align::Start,
        }
    }

    pub fn push(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(FlexWidget::new(widget));
        self
    }

    pub fn push_flex(mut self, widget: impl Widget + 'static, flex: FlexConfig) -> Self {
        self.children.push(FlexWidget::new(widget).with_flex(flex));
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }
}

impl Widget for Column {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Note: For viewport culling in a ScrollView, you can check
        // if each child is visible before drawing. This is left to the user to
        // implement when needed.
        // Example:
        //   if scroll_view.is_visible(&child_rect) {
        //       child.draw(ctx);
        //   }
        for (i, child) in self.children.iter_mut().enumerate() {
            ctx.push_path(i);
            child.draw(ctx);
            ctx.pop_path();
        }
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
        let spacing = ctx.theme.metrics.spacing;

        // Calculate total flex grow and shrink factors
        let total_flex_grow: f32 = self.children.iter().map(|c| c.flex.flex_grow).sum();
        let total_flex_shrink: f32 = self.children.iter().map(|c| c.flex.flex_shrink).sum();

        // First pass: get preferred sizes (natural size, but with proper constraints)
        let mut child_hints = Vec::new();
        let mut total_min_height = 0.0; // Minimum space needed (preferred for fixed, preferred for flex-grow as min)
        let total_spacing = spacing * (self.children.len().saturating_sub(1)) as f32;
        let width = available_space.size.x;

        for child in &self.children {
            // Pass constraints that include:
            // - Cross-axis (width) constraint for stretch alignment
            // - Flex min/max constraints
            let child_constraints = SizeConstraints::new(
                match self.align_items {
                    Align::Stretch => width.max(child.flex.min_width),
                    _ => child.flex.min_width,
                },
                match self.align_items {
                    Align::Stretch => width.min(child.flex.max_width),
                    _ => child.flex.max_width,
                },
                child.flex.min_height,
                child.flex.max_height,
            );
            let hint = child.size_hint(ctx, child_constraints);
            child_hints.push(hint);
            // All widgets need at least their preferred size (or min_height if larger)
            // This is the "flex-basis" - the minimum they need before flex distribution
            total_min_height += hint.y.max(child.flex.min_height);
        }

        let available_height = available_space.size.y - total_spacing;
        // Excess space is what's left after all widgets get their minimum (preferred) size
        // This excess will be distributed to flex-grow widgets proportionally
        let excess_height = available_height - total_min_height;

        // Second pass: distribute space using flex factors
        let mut y_cursor = available_space.pos.y;
        let x_start = available_space.pos.x;

        for (i, (child, hint)) in self.children.iter_mut().zip(child_hints.iter()).enumerate() {
            let mut final_height;

            if child.flex.flex_grow > 0.0 {
                // Flex-grow widget: gets proportional share of excess space
                // Minimum size is the preferred size (flex-basis) or min_height, whichever is larger
                let min_size = hint.y.max(child.flex.min_height);

                if excess_height > 0.0 && total_flex_grow > 0.0 {
                    // Distribute excess space proportionally
                    let grow_factor = child.flex.flex_grow / total_flex_grow;
                    // Each flex-grow widget gets: min_size + (excess * their_proportion)
                    final_height = min_size + (excess_height * grow_factor);
                } else {
                    // No excess or no flex-grow widgets, use minimum size
                    final_height = min_size;
                }
            } else {
                // Fixed widget: use preferred size, but may shrink if needed
                final_height = hint.y;

                if excess_height < 0.0 && total_flex_shrink > 0.0 && child.flex.flex_shrink > 0.0 {
                    // Shrink proportionally if there's a deficit
                    let shrink_factor = child.flex.flex_shrink / total_flex_shrink;
                    final_height += excess_height * shrink_factor;
                }
            }

            // Apply min/max constraints
            final_height = final_height
                .max(child.flex.min_height)
                .min(child.flex.max_height);

            // Calculate cross-axis alignment
            let child_width = match self.align_items {
                Align::Stretch => width, // Fill cross-axis
                _ => hint.x.min(width),  // Use preferred width, but don't exceed available
            };

            let child_x = match self.align_items {
                Align::Start => x_start,
                Align::Center => x_start + (width - child_width) / 2.0,
                Align::End => x_start + width - child_width,
                Align::Stretch => x_start,
            };

            let child_rect = Rect::new(child_x, y_cursor, child_width, final_height);
            ctx.push_path(i);
            child.layout(ctx, child_rect);
            ctx.pop_path();
            y_cursor += final_height + spacing;
        }
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let mut w = 0.0f32;
        let mut h = 0.0f32;
        let spacing = ctx.theme.metrics.spacing;

        for (i, child) in self.children.iter().enumerate() {
            // Pass constraints to children, ensuring cross-axis (width) constraint is respected
            // This is critical when Column is inside a Row with Align::Stretch
            let child_constraints = SizeConstraints::new(
                constraints.min_width.max(child.flex.min_width),
                constraints.max_width.min(child.flex.max_width),
                child.flex.min_height,
                child.flex.max_height,
            );
            ctx.push_path(i);
            let hint = child.size_hint(ctx, child_constraints);
            ctx.pop_path();
            // For Column, width is cross-axis - respect the constraint
            w = w.max(hint.x).min(constraints.max_width);
            h += hint.y;
            if i < self.children.len() - 1 {
                h += spacing;
            }
        }

        // Ensure we respect minimum width constraint
        w = w.max(constraints.min_width);

        // Apply constraints
        let result = Vec2::new(w, h);
        constraints.constrain(result)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut handled = false;
        for (i, child) in self.children.iter_mut().enumerate() {
            ctx.push_path(i);
            if child.handle_event(ctx, event) {
                handled = true;
                match event {
                    Event::Key(..) | Event::Char(_) => {
                        ctx.pop_path();
                        break;
                    }
                    _ => {}
                }
            }
            ctx.pop_path();
        }
        handled
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        for (i, child) in self.children.iter_mut().enumerate() {
            ctx.push_path(i);
            child.draw_overlay(ctx);
            ctx.pop_path();
        }
    }
}

pub struct Row {
    pub children: Vec<FlexWidget>,
    pub rect: Rect,
    pub align_items: Align,
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Row {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            rect: Rect::default(),
            align_items: Align::Start,
        }
    }

    pub fn push(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(FlexWidget::new(widget));
        self
    }

    pub fn push_flex(mut self, widget: impl Widget + 'static, flex: FlexConfig) -> Self {
        self.children.push(FlexWidget::new(widget).with_flex(flex));
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }
}

impl Widget for Row {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Note: For viewport culling in a ScrollView, you can check
        // if each child is visible before drawing. This is left to the user to
        // implement when needed.
        for (i, child) in self.children.iter_mut().enumerate() {
            ctx.push_path(i);
            child.draw(ctx);
            ctx.pop_path();
        }
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
        let spacing = ctx.theme.metrics.spacing;

        // Calculate total flex grow and shrink factors
        let total_flex_grow: f32 = self.children.iter().map(|c| c.flex.flex_grow).sum();
        let total_flex_shrink: f32 = self.children.iter().map(|c| c.flex.flex_shrink).sum();

        // First pass: get preferred sizes (natural size, but with proper constraints)
        let mut child_hints = Vec::new();
        let mut total_min_width = 0.0; // Minimum space needed (preferred for fixed, preferred for flex-grow as min)
        let total_spacing = spacing * (self.children.len().saturating_sub(1)) as f32;
        let height = available_space.size.y;

        for child in &self.children {
            // Pass constraints that include:
            // - Cross-axis (height) constraint for stretch alignment
            // - Flex min/max constraints
            let child_constraints = SizeConstraints::new(
                child.flex.min_width,
                child.flex.max_width,
                match self.align_items {
                    Align::Stretch => height.max(child.flex.min_height),
                    _ => child.flex.min_height,
                },
                match self.align_items {
                    Align::Stretch => height.min(child.flex.max_height),
                    _ => child.flex.max_height,
                },
            );
            let hint = child.size_hint(ctx, child_constraints);
            child_hints.push(hint);
            // All widgets need at least their preferred size (or min_width if larger)
            // This is the "flex-basis" - the minimum they need before flex distribution
            total_min_width += hint.x.max(child.flex.min_width);
        }

        let available_width = available_space.size.x - total_spacing;
        // Excess space is what's left after all widgets get their minimum (preferred) size
        // This excess will be distributed to flex-grow widgets proportionally
        let excess_width = available_width - total_min_width;

        // Second pass: distribute space using flex factors
        let mut x_cursor = available_space.pos.x;
        let y_start = available_space.pos.y;

        for (i, (child, hint)) in self.children.iter_mut().zip(child_hints.iter()).enumerate() {
            let mut final_width;

            if child.flex.flex_grow > 0.0 {
                // Flex-grow widget: gets proportional share of excess space
                // Minimum size is the preferred size (flex-basis) or min_width, whichever is larger
                let min_size = hint.x.max(child.flex.min_width);

                if excess_width > 0.0 && total_flex_grow > 0.0 {
                    // Distribute excess space proportionally
                    let grow_factor = child.flex.flex_grow / total_flex_grow;
                    // Each flex-grow widget gets: min_size + (excess * their_proportion)
                    final_width = min_size + (excess_width * grow_factor);
                } else {
                    // No excess or no flex-grow widgets, use minimum size
                    final_width = min_size;
                }
            } else {
                // Fixed widget: use preferred size, but may shrink if needed
                final_width = hint.x;

                if excess_width < 0.0 && total_flex_shrink > 0.0 && child.flex.flex_shrink > 0.0 {
                    // Shrink proportionally if there's a deficit
                    let shrink_factor = child.flex.flex_shrink / total_flex_shrink;
                    final_width += excess_width * shrink_factor;
                }
            }

            // Apply min/max constraints
            final_width = final_width
                .max(child.flex.min_width)
                .min(child.flex.max_width);

            // Calculate cross-axis alignment
            let child_height = match self.align_items {
                Align::Stretch => height, // Fill cross-axis
                _ => hint.y.min(height),  // Use preferred height, but don't exceed available
            };

            let child_y = match self.align_items {
                Align::Start => y_start,
                Align::Center => y_start + (height - child_height) / 2.0,
                Align::End => y_start + height - child_height,
                Align::Stretch => y_start,
            };

            let child_rect = Rect::new(x_cursor, child_y, final_width, child_height);
            ctx.push_path(i);
            child.layout(ctx, child_rect);
            ctx.pop_path();
            x_cursor += final_width + spacing;
        }
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let mut w = 0.0f32;
        let mut h = 0.0f32;
        let spacing = ctx.theme.metrics.spacing;

        for (i, child) in self.children.iter().enumerate() {
            // Pass constraints to children, ensuring cross-axis (height) constraint is respected
            // This is critical when Row is inside a Column with Align::Stretch
            let child_constraints = SizeConstraints::new(
                child.flex.min_width,
                child.flex.max_width,
                constraints.min_height.max(child.flex.min_height),
                constraints.max_height.min(child.flex.max_height),
            );
            ctx.push_path(i);
            let hint = child.size_hint(ctx, child_constraints);
            ctx.pop_path();
            // For Row, height is cross-axis - respect the constraint
            h = h.max(hint.y).min(constraints.max_height);
            w += hint.x;
            if i < self.children.len() - 1 {
                w += spacing;
            }
        }

        // Ensure we respect minimum height constraint
        h = h.max(constraints.min_height);

        // Apply constraints
        let result = Vec2::new(w, h);
        constraints.constrain(result)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut handled = false;
        for (i, child) in self.children.iter_mut().enumerate() {
            ctx.push_path(i);
            if child.handle_event(ctx, event) {
                handled = true;
                match event {
                    Event::Key(..) | Event::Char(_) => {
                        ctx.pop_path();
                        break;
                    }
                    _ => {}
                }
            }
            ctx.pop_path();
        }
        handled
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        for (i, child) in self.children.iter_mut().enumerate() {
            ctx.push_path(i);
            child.draw_overlay(ctx);
            ctx.pop_path();
        }
    }
}

type TabsCallback = Box<dyn FnMut(usize, &str)>;

struct TabDefinition {
    label: String,
    widget: Box<dyn Widget>,
}

impl TabDefinition {
    fn new(label: impl Into<String>, widget: impl Widget + 'static) -> Self {
        Self {
            label: label.into(),
            widget: Box::new(widget),
        }
    }
}

pub struct Tabs {
    tabs: Vec<TabDefinition>,
    pub rect: Rect,
    pub content_rect: Rect,
    tab_rects: Vec<Rect>,
    initial_selected: Option<usize>,
    on_change: Option<TabsCallback>,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            rect: Rect::default(),
            content_rect: Rect::default(),
            tab_rects: Vec::new(),
            initial_selected: None,
            on_change: None,
        }
    }

    pub fn push_tab(mut self, label: impl Into<String>, widget: impl Widget + 'static) -> Self {
        self.tabs.push(TabDefinition::new(label, widget));
        self
    }

    pub fn select(mut self, index: usize) -> Self {
        self.initial_selected = Some(index);
        self
    }

    pub fn on_change<F: FnMut(usize, &str) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    fn tab_bar_height(&self, ctx: &WidgetContext) -> f32 {
        ctx.theme.font.size_body + ctx.theme.metrics.padding * 2.0 + ctx.theme.metrics.border_width
    }

    fn rebuild_tab_rects(&mut self, ctx: &mut WidgetContext) {
        self.tab_rects.clear();

        if self.tabs.is_empty() {
            return;
        }

        let tab_height = self.tab_bar_height(ctx);
        let spacing = ctx.theme.metrics.spacing * 0.5;
        let mut cursor_x = self.rect.pos.x;
        let max_x = self.rect.pos.x + self.rect.size.x;

        for tab in &self.tabs {
            let text_size = ctx
                .painter
                .get_text_size(&tab.label, ctx.theme.font.size_body);
            let mut width = (text_size.x + ctx.theme.metrics.padding * 2.5).max(72.0);
            if max_x.is_finite() {
                let remaining = max_x - cursor_x;
                if remaining <= 0.0 {
                    width = 0.0;
                } else if remaining < width {
                    width = remaining;
                }
            }

            self.tab_rects
                .push(Rect::new(cursor_x, self.rect.pos.y, width, tab_height));
            cursor_x += width + spacing;
        }

        while self.tab_rects.len() < self.tabs.len() {
            self.tab_rects.push(Rect::default());
        }
    }

    fn ensure_tab_rects(&mut self, ctx: &mut WidgetContext) {
        if self.tab_rects.len() != self.tabs.len() {
            self.rebuild_tab_rects(ctx);
        }
    }

    fn hit_test_tabs(&self, pos: Vec2) -> Option<usize> {
        self.tab_rects
            .iter()
            .enumerate()
            .find(|(_, rect)| rect.contains(pos))
            .map(|(i, _)| i)
    }

    fn sync_state(&self, state: &mut TabsState) -> (Option<usize>, bool) {
        if self.tabs.is_empty() {
            let had_state = state.selected_index.is_some() || state.hovered_index.is_some();
            state.selected_index = None;
            state.hovered_index = None;
            return (None, had_state);
        }

        let mut dirty = false;
        if let Some(hover) = state.hovered_index
            && hover >= self.tabs.len()
        {
            state.hovered_index = None;
            dirty = true;
        }

        let default_index = self
            .initial_selected
            .unwrap_or(0)
            .min(self.tabs.len().saturating_sub(1));
        let current = state
            .selected_index
            .unwrap_or(default_index)
            .min(self.tabs.len() - 1);
        if state.selected_index != Some(current) {
            state.selected_index = Some(current);
            dirty = true;
        }

        (Some(current), dirty)
    }
}

impl Widget for Tabs {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        let mut state: TabsState = ctx.get_state();
        let (selected_index, state_dirty) = self.sync_state(&mut state);
        if selected_index.is_none() {
            if state_dirty {
                ctx.set_state(state);
            }
            return;
        }
        let selected_index = selected_index.unwrap();
        let hovered_index = state.hovered_index;

        self.ensure_tab_rects(ctx);

        let tab_height = self.tab_bar_height(ctx);
        let bar_rect = Rect::new(
            self.rect.pos.x,
            self.rect.pos.y,
            self.rect.size.x,
            tab_height,
        );
        ctx.painter
            .draw_rounded_rect(bar_rect, ctx.theme.colors.surface, ctx.theme.metrics.radius);

        for (i, rect) in self.tab_rects.iter().enumerate() {
            if i >= self.tabs.len() {
                break;
            }
            if rect.size.x <= 0.0 {
                continue;
            }

            let is_selected = i == selected_index;
            let is_hovered = hovered_index == Some(i);

            let bg_color = if is_selected {
                ctx.theme.colors.widget_bg
            } else {
                ctx.theme.colors.surface
            };

            ctx.painter
                .draw_rounded_rect(*rect, bg_color, ctx.theme.metrics.radius * 0.5);

            if is_hovered && !is_selected {
                ctx.painter.draw_rounded_rect(
                    *rect,
                    ctx.theme.colors.hover_overlay,
                    ctx.theme.metrics.radius * 0.5,
                );
            }

            let text_size = ctx
                .painter
                .get_text_size(&self.tabs[i].label, ctx.theme.font.size_body);
            let text_pos = Vec2::new(
                rect.pos.x + (rect.size.x - text_size.x) / 2.0,
                rect.pos.y + (rect.size.y - text_size.y) / 2.0,
            );
            let text_color = if is_selected {
                ctx.theme.colors.text
            } else {
                ctx.theme.colors.text_dim
            };
            ctx.painter.draw_text(
                &self.tabs[i].label,
                text_pos,
                text_color,
                ctx.theme.font.size_body,
                Wrap::None,
            );

            if is_selected {
                let indicator_height = 3.0;
                let indicator_rect = Rect::new(
                    rect.pos.x + 6.0,
                    rect.pos.y + rect.size.y - indicator_height,
                    (rect.size.x - 12.0).max(0.0),
                    indicator_height,
                );
                ctx.painter
                    .draw_rect(indicator_rect, ctx.theme.colors.primary);
            }
        }

        if self.content_rect.size.x > 0.0 && self.content_rect.size.y > 0.0 {
            ctx.painter.draw_rounded_rect(
                self.content_rect,
                ctx.theme.colors.surface,
                ctx.theme.metrics.radius,
            );
        }

        ctx.push_path(selected_index);
        self.tabs[selected_index].widget.draw(ctx);
        ctx.pop_path();

        if state_dirty {
            ctx.set_state(state);
        }
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;

        let mut state: TabsState = ctx.get_state();
        let (selected_index, state_dirty) = self.sync_state(&mut state);

        self.rebuild_tab_rects(ctx);

        let tab_height = self.tab_bar_height(ctx);
        let spacing = ctx.theme.metrics.spacing;
        let content_top = available_space.pos.y + tab_height + spacing;
        let content_height =
            (available_space.size.y - (content_top - available_space.pos.y)).max(0.0);
        self.content_rect = Rect::new(
            available_space.pos.x,
            content_top,
            available_space.size.x,
            content_height,
        );

        if let Some(index) = selected_index {
            ctx.push_path(index);
            self.tabs[index].widget.layout(ctx, self.content_rect);
            ctx.pop_path();
        }

        if state_dirty {
            ctx.set_state(state);
        }
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        if self.tabs.is_empty() {
            let size = Vec2::new(0.0, self.tab_bar_height(ctx));
            return constraints.constrain(size);
        }

        let tab_height = self.tab_bar_height(ctx);
        let available_height = (constraints.max_height - tab_height).max(0.0);
        let child_constraints = SizeConstraints::new(
            constraints.min_width,
            constraints.max_width,
            constraints.min_height.max(0.0),
            available_height,
        );

        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for (i, tab) in self.tabs.iter().enumerate() {
            ctx.push_path(i);
            let hint = tab.widget.size_hint(ctx, child_constraints);
            ctx.pop_path();
            width = width.max(hint.x);
            height = height.max(hint.y);
        }

        let total_height = height + tab_height + ctx.theme.metrics.spacing;
        constraints.constrain(Vec2::new(width, total_height))
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut state: TabsState = ctx.get_state();
        let (selected_index_opt, mut state_dirty) = self.sync_state(&mut state);

        let mut consumed = false;

        match event {
            Event::Move(pos) => {
                let hovered = self.hit_test_tabs(*pos);
                if hovered != state.hovered_index {
                    state.hovered_index = hovered;
                    state_dirty = true;
                }
                consumed = hovered.is_some();
            }
            Event::Press(pos) => {
                if let Some(index) = self.hit_test_tabs(*pos) {
                    state.hovered_index = Some(index);
                    state_dirty = true;
                    consumed = true;
                }
            }
            Event::Release(pos) => {
                if let Some(index) = self.hit_test_tabs(*pos) {
                    let previous = state.selected_index;
                    state.selected_index = Some(index);
                    state.hovered_index = Some(index);
                    state_dirty = true;
                    consumed = true;

                    if previous != Some(index)
                        && let Some(callback) = &mut self.on_change
                    {
                        (callback)(index, &self.tabs[index].label);
                    }
                }
            }
            _ => {}
        }

        let selected_index = selected_index_opt.and_then(|idx| {
            if idx < self.tabs.len() {
                Some(idx)
            } else {
                None
            }
        });

        if state_dirty {
            ctx.set_state(state);
        }

        if consumed {
            return true;
        }

        if let Some(index) = selected_index {
            ctx.push_path(index);
            let handled = self.tabs[index].widget.handle_event(ctx, event);
            ctx.pop_path();
            handled
        } else {
            false
        }
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        let mut state: TabsState = ctx.get_state();
        let (selected_index, state_dirty) = self.sync_state(&mut state);

        if state_dirty {
            ctx.set_state(state);
        }

        if let Some(idx) = selected_index {
            ctx.push_path(idx);
            self.tabs[idx].widget.draw_overlay(ctx);
            ctx.pop_path();
        }
    }
}

pub struct Panel {
    pub child: Box<dyn Widget>,
    pub rect: Rect,
}

impl Panel {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            rect: Rect::default(),
        }
    }
}

impl Widget for Panel {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Draw panel background
        let radius = ctx.theme.metrics.radius;
        ctx.painter
            .draw_rounded_rect(self.rect, ctx.theme.colors.surface, radius);
        ctx.push_path(0);
        self.child.draw(ctx);
        ctx.pop_path();
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
        // Add padding
        let padding = ctx.theme.metrics.padding;
        let child_rect = Rect::new(
            available_space.pos.x + padding,
            available_space.pos.y + padding,
            available_space.size.x - padding * 2.0,
            available_space.size.y - padding * 2.0,
        );
        ctx.push_path(0);
        self.child.layout(ctx, child_rect);
        ctx.pop_path();
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        let padding = ctx.theme.metrics.padding;
        // Account for padding in constraints
        let child_constraints = SizeConstraints::new(
            (constraints.min_width - padding * 2.0).max(0.0),
            constraints.max_width - padding * 2.0,
            (constraints.min_height - padding * 2.0).max(0.0),
            constraints.max_height - padding * 2.0,
        );
        ctx.push_path(0);
        let child_hint = self.child.size_hint(ctx, child_constraints);
        ctx.pop_path();
        let size = Vec2::new(child_hint.x + padding * 2.0, child_hint.y + padding * 2.0);
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        ctx.push_path(0);
        let result = self.child.handle_event(ctx, event);
        ctx.pop_path();
        result
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        ctx.push_path(0);
        self.child.draw_overlay(ctx);
        ctx.pop_path();
    }
}

pub struct Overlay {
    pub content: Box<dyn Widget>,
    pub overlay: Box<dyn Widget>,
}

impl Overlay {
    pub fn new(content: impl Widget + 'static, overlay: impl Widget + 'static) -> Self {
        Self {
            content: Box::new(content),
            overlay: Box::new(overlay),
        }
    }
}

impl Widget for Overlay {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.push_path(0);
        self.content.draw(ctx);
        ctx.pop_path();

        ctx.push_path(1);
        self.overlay.draw(ctx);
        ctx.pop_path();
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        ctx.push_path(0);
        self.content.layout(ctx, available_space);
        ctx.pop_path();

        ctx.push_path(1);
        self.overlay.layout(ctx, available_space);
        ctx.pop_path();
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        ctx.push_path(0);
        let hint = self.content.size_hint(ctx, constraints);
        ctx.pop_path();
        hint
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        ctx.push_path(1);
        let overlay_handled = self.overlay.handle_event(ctx, event);
        ctx.pop_path();
        if overlay_handled {
            return true;
        }

        ctx.push_path(0);
        let handled = self.content.handle_event(ctx, event);
        ctx.pop_path();
        handled
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        ctx.push_path(0);
        self.content.draw_overlay(ctx);
        ctx.pop_path();

        ctx.push_path(1);
        self.overlay.draw_overlay(ctx);
        ctx.pop_path();
    }
}

pub struct ScrollView {
    pub child: Box<dyn Widget>,
    pub rect: Rect,
    pub scroll_offset: f32,
    pub content_height: f32,
    pub is_dragging_scrollbar: bool,
}

impl ScrollView {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            rect: Rect::default(),
            scroll_offset: 0.0,
            content_height: 0.0,
            is_dragging_scrollbar: false,
        }
    }

    /// Check if a widget rect is visible in the viewport
    ///
    /// This can be used by child widgets to skip drawing when outside the visible area.
    /// Returns true if the widget overlaps with the scrollview's visible region.
    ///
    /// # Example
    /// ```ignore
    /// if scroll_view.is_visible(&widget.rect) {
    ///     widget.draw(ctx);
    /// }
    /// ```
    pub fn is_visible(&self, widget_rect: &Rect) -> bool {
        let view_top = self.rect.pos.y;
        let view_bottom = view_top + self.rect.size.y;
        let widget_bottom = widget_rect.pos.y + widget_rect.size.y;

        // Widget is visible if it overlaps with the viewport
        widget_rect.pos.y < view_bottom && widget_bottom > view_top
    }

    /// Get the visible viewport rect for this ScrollView
    pub fn viewport_rect(&self) -> Rect {
        self.rect
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> f32 {
        self.scroll_offset
    }
}

impl Widget for ScrollView {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Load state
        let state: ScrollViewState = ctx.get_state();

        // Sync to widget fields
        self.scroll_offset = state.offset;
        self.is_dragging_scrollbar = state.dragging;

        // Clip content
        let previous_scissor = ctx.painter.set_scissor(Some(self.rect));

        ctx.push_path(0);
        self.child.draw(ctx);
        ctx.pop_path();

        ctx.painter.set_scissor(previous_scissor);

        // Draw scrollbar if needed
        if self.content_height > self.rect.size.y {
            let scrollbar_width = ctx.theme.metrics.scrollbar_width;
            let view_h = self.rect.size.y;
            let ratio = view_h / self.content_height;
            let thumb_h = (view_h * ratio).max(20.0);
            let max_scroll = (self.content_height - view_h).max(0.0);
            let track_ratio = if max_scroll > 0.0 {
                self.scroll_offset / max_scroll
            } else {
                0.0
            };
            let thumb_y = self.rect.pos.y + track_ratio * (view_h - thumb_h);

            let thumb_rect = Rect::new(
                self.rect.pos.x + self.rect.size.x - scrollbar_width,
                thumb_y,
                scrollbar_width,
                thumb_h,
            );

            ctx.painter.draw_rect(thumb_rect, ctx.theme.colors.border);
        }
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        // Load state
        let mut state: ScrollViewState = ctx.get_state();

        // Sync from state
        self.scroll_offset = state.offset;

        self.rect = available_space;
        let child_constraints = SizeConstraints::new(
            0.0,
            available_space.size.x - ctx.theme.metrics.scrollbar_width,
            0.0,
            f32::INFINITY,
        );
        ctx.push_path(0);
        let content_hint = self.child.size_hint(ctx, child_constraints);
        ctx.pop_path();
        self.content_height = content_hint.y;

        // Clamp scroll and update state
        let max_scroll = (self.content_height - self.rect.size.y).max(0.0);
        let new_offset = self.scroll_offset.clamp(0.0, max_scroll);
        state.offset = new_offset;
        self.scroll_offset = new_offset;

        // Save state
        ctx.set_state(state);

        // Offset child
        let child_rect = Rect::new(
            available_space.pos.x,
            available_space.pos.y - self.scroll_offset,
            available_space.size.x - ctx.theme.metrics.scrollbar_width, // Reserve space for scrollbar
            self.content_height,
        );

        ctx.push_path(0);
        self.child.layout(ctx, child_rect);
        ctx.pop_path();
    }

    fn size_hint(&self, _ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        // ScrollView takes up available space, but we return a minimum
        let size = Vec2::new(100.0, 100.0);
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        // Load state at start
        let mut state: ScrollViewState = ctx.get_state();

        self.scroll_offset = state.offset;
        self.is_dragging_scrollbar = state.dragging;

        let result = match event {
            Event::Scroll(delta) => {
                // Try passing to child first (e.g. for open dropdowns)
                ctx.push_path(0);
                let child_handled = self.child.handle_event(ctx, event);
                ctx.pop_path();

                if child_handled {
                    return true;
                }

                // Only handle scroll if content_height is valid (layout has been called)
                // If content_height is 0, layout hasn't run yet, so consume event but don't process
                if self.content_height == 0.0 || self.rect.size.y == 0.0 {
                    // Layout hasn't been called yet or rect is invalid, consume event to prevent propagation
                    true
                } else if self.content_height > self.rect.size.y {
                    // Content is scrollable, handle the scroll
                    let scroll_speed = 20.0;
                    let new = state.offset - delta * scroll_speed;
                    let max_scroll = (self.content_height - self.rect.size.y).max(0.0);
                    let new_offset = new.clamp(0.0, max_scroll);
                    state.offset = new_offset;
                    self.scroll_offset = new_offset;
                    true
                } else {
                    // Content fits, consume the event (don't pass to child to avoid potential issues)
                    true
                }
            }
            Event::Press(pos) => {
                // Only handle scrollbar press if content_height is valid
                if self.content_height > self.rect.size.y
                    && self.content_height > 0.0
                    && self.rect.size.y > 0.0
                {
                    let scrollbar_width = 10.0; // Should match theme
                    let scrollbar_x = self.rect.pos.x + self.rect.size.x - scrollbar_width;

                    if pos.x >= scrollbar_x
                        && pos.x <= scrollbar_x + scrollbar_width
                        && pos.y >= self.rect.pos.y
                        && pos.y <= self.rect.pos.y + self.rect.size.y
                    {
                        // Clicked on scrollbar
                        state.dragging = true;
                        self.is_dragging_scrollbar = true;

                        // Calculate thumb position and check if clicked on it
                        let view_h = self.rect.size.y;
                        let ratio = view_h / self.content_height;
                        let thumb_h = (view_h * ratio).max(20.0);
                        let max_scroll = (self.content_height - view_h).max(0.0);
                        let track_ratio = if max_scroll > 0.0 {
                            self.scroll_offset / max_scroll
                        } else {
                            0.0
                        };
                        let thumb_y = self.rect.pos.y + track_ratio * (view_h - thumb_h);

                        // If clicked in track but not on thumb, jump to that position
                        if pos.y < thumb_y || pos.y > thumb_y + thumb_h {
                            let click_ratio = (pos.y - self.rect.pos.y) / view_h;
                            let new = click_ratio * max_scroll;
                            let new_offset = new.clamp(0.0, max_scroll);
                            state.offset = new_offset;
                            self.scroll_offset = new_offset;
                        }

                        state.offset = self.scroll_offset;
                        state.dragging = self.is_dragging_scrollbar;
                        ctx.set_state(state);
                        return true;
                    }
                }
                ctx.push_path(0);
                let result = self.child.handle_event(ctx, event);
                ctx.pop_path();
                result
            }
            Event::Move(pos) => {
                if state.dragging {
                    // Only handle dragging if content_height is valid
                    if self.content_height == 0.0 || self.rect.size.y == 0.0 {
                        return false;
                    }
                    let view_h = self.rect.size.y;
                    let ratio = view_h / self.content_height;
                    let thumb_h = (view_h * ratio).max(20.0);
                    let max_scroll = (self.content_height - view_h).max(0.0);

                    let mouse_ratio =
                        (pos.y - self.rect.pos.y - thumb_h / 2.0) / (view_h - thumb_h);
                    let new = mouse_ratio * max_scroll;
                    let new_offset = new.clamp(0.0, max_scroll);
                    state.offset = new_offset;
                    self.scroll_offset = new_offset;
                    true
                } else {
                    ctx.push_path(0);
                    let result = self.child.handle_event(ctx, event);
                    ctx.pop_path();
                    result
                }
            }
            Event::Release(_) => {
                if state.dragging {
                    state.dragging = false;
                    self.is_dragging_scrollbar = false;
                    true
                } else {
                    ctx.push_path(0);
                    let result = self.child.handle_event(ctx, event);
                    ctx.pop_path();
                    result
                }
            }
            _ => {
                ctx.push_path(0);
                let result = self.child.handle_event(ctx, event);
                ctx.pop_path();
                result
            }
        };

        // Sync widget fields to state before saving
        state.offset = self.scroll_offset;
        state.dragging = self.is_dragging_scrollbar;

        // Save state at end
        ctx.set_state(state);
        result
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        // Draw overlay for child (dropdown menus, etc.) without scissor clipping
        // Overlays should render on top, so we clear any scissor from ScrollView
        let previous_scissor = ctx.painter.set_scissor(None);
        ctx.push_path(0);
        self.child.draw_overlay(ctx);
        ctx.pop_path();
        ctx.painter.set_scissor(previous_scissor);
    }
}

#[derive(Debug, Clone, Default)]
pub struct TooltipState {
    pub hovered: bool,
    pub hover_frames: u32, // Frame counter for hover delay
    pub mouse_pos: Vec2,
}

pub struct Tooltip {
    pub child: Box<dyn Widget>,
    pub tooltip_text: String,
    pub rect: Rect,
    pub hover_delay_frames: u32, // Number of frames to wait before showing tooltip
}

impl Tooltip {
    pub fn new(child: impl Widget + 'static, tooltip_text: impl Into<String>) -> Self {
        Self {
            child: Box::new(child),
            tooltip_text: tooltip_text.into(),
            rect: Rect::default(),
            hover_delay_frames: 30, // Show after ~0.5 seconds at 60 FPS
        }
    }

    pub fn with_delay(mut self, frames: u32) -> Self {
        self.hover_delay_frames = frames;
        self
    }
}

impl Widget for Tooltip {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        ctx.push_path(0);
        self.child.draw(ctx);
        ctx.pop_path();
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        self.rect = available_space;
        ctx.push_path(0);
        self.child.layout(ctx, available_space);
        ctx.pop_path();
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        ctx.push_path(0);
        let hint = self.child.size_hint(ctx, constraints);
        ctx.pop_path();
        hint
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut state: TooltipState = ctx.get_state();
        let mut needs_update = false;

        if let Event::Move(pos) = event {
            let was_hovered = state.hovered;
            state.hovered = self.rect.contains(*pos);
            state.mouse_pos = *pos;

            if state.hovered {
                state.hover_frames += 1;
            } else {
                state.hover_frames = 0;
            }

            needs_update = was_hovered != state.hovered || state.hover_frames > 0;
        }

        if needs_update {
            ctx.set_state(state);
        }

        ctx.push_path(0);
        let handled = self.child.handle_event(ctx, event);
        ctx.pop_path();
        handled
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        // Draw child overlay first
        ctx.push_path(0);
        self.child.draw_overlay(ctx);
        ctx.pop_path();

        // Draw tooltip if hovered long enough
        let state: TooltipState = ctx.get_state();
        if state.hovered && state.hover_frames >= self.hover_delay_frames {
            // Clear any scissor so tooltip can appear outside clipped areas
            let previous_scissor = ctx.painter.set_scissor(None);

            let padding = ctx.theme.metrics.padding;
            let radius = ctx.theme.metrics.radius;
            let border_width = ctx.theme.metrics.border_width;

            // Calculate tooltip size
            let text_size = ctx
                .painter
                .get_text_size(&self.tooltip_text, ctx.theme.font.size_small);
            let tooltip_width = text_size.x + padding * 2.0;
            let tooltip_height = text_size.y + padding * 2.0;
            let tooltip_size = Vec2::new(tooltip_width, tooltip_height);

            // Position tooltip near mouse cursor (offset slightly to avoid covering cursor)
            let offset = Vec2::new(12.0, 20.0);
            let mut tooltip_pos = state.mouse_pos + offset;

            // Keep tooltip on screen (basic clamping)
            // In a real implementation, you might want more sophisticated positioning
            let screen_width = 1920.0; // Approximate - could be passed from context
            let screen_height = 1080.0;

            if tooltip_pos.x + tooltip_width > screen_width {
                tooltip_pos.x = state.mouse_pos.x - tooltip_width - offset.x;
            }
            if tooltip_pos.y + tooltip_height > screen_height {
                tooltip_pos.y = state.mouse_pos.y - tooltip_height - offset.y;
            }

            let tooltip_rect =
                Rect::new(tooltip_pos.x, tooltip_pos.y, tooltip_size.x, tooltip_size.y);

            // Draw tooltip border (outer rounded rect)
            ctx.painter
                .draw_rounded_rect(tooltip_rect, ctx.theme.colors.border, radius);

            // Draw tooltip background (inner rounded rect)
            let inset = border_width;
            let inner_rect = Rect::new(
                tooltip_rect.pos.x + inset,
                tooltip_rect.pos.y + inset,
                tooltip_rect.size.x - inset * 2.0,
                tooltip_rect.size.y - inset * 2.0,
            );
            ctx.painter.draw_rounded_rect(
                inner_rect,
                ctx.theme.colors.surface,
                (radius - inset).max(0.0),
            );

            // Draw tooltip text
            let text_pos = Vec2::new(tooltip_rect.pos.x + padding, tooltip_rect.pos.y + padding);
            ctx.painter.draw_text(
                &self.tooltip_text,
                text_pos,
                ctx.theme.colors.text,
                ctx.theme.font.size_small,
                glyphon::Wrap::Word,
            );

            // Restore previous scissor
            ctx.painter.set_scissor(previous_scissor);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModalState {
    pub open: bool,
    pub screen_size: Vec2, // Track screen size for backdrop
}

pub struct Modal {
    pub child: Box<dyn Widget>,
    pub rect: Rect,
    pub modal_rect: Rect, // The actual modal dialog rect
    pub close_on_backdrop_click: bool,
    pub on_close: Option<Box<dyn FnMut()>>,
    pub open_state: Option<Arc<Mutex<bool>>>, // External state control
}

impl Modal {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            rect: Rect::default(),
            modal_rect: Rect::default(),
            close_on_backdrop_click: true,
            on_close: None,
            open_state: None,
        }
    }

    pub fn close_on_backdrop_click(mut self, close: bool) -> Self {
        self.close_on_backdrop_click = close;
        self
    }

    pub fn on_close<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    pub fn bind_open_state(mut self, state: Arc<Mutex<bool>>) -> Self {
        self.open_state = Some(state);
        self
    }
}

impl Widget for Modal {
    fn draw(&mut self, ctx: &mut WidgetContext) {
        // Sync external state if bound
        if let Some(ref open_state) = self.open_state
            && let Ok(open) = open_state.lock()
        {
            let mut state: ModalState = ctx.get_state();
            if state.open != *open {
                state.open = *open;
                ctx.set_state(state);
            }
        }
        // Modal content is drawn in draw_overlay, nothing to draw here
    }

    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect) {
        // Store the available space (screen size) for backdrop rendering
        let mut state: ModalState = ctx.get_state();
        state.screen_size = available_space.size;
        ctx.set_state(state);

        self.rect = available_space;

        // Calculate modal size and position (centered)
        let padding = ctx.theme.metrics.padding * 2.0;
        let max_width = (available_space.size.x * 0.8).min(600.0);
        let max_height = (available_space.size.y * 0.8).min(500.0);

        let child_constraints = SizeConstraints::new(
            0.0,
            max_width - padding * 2.0,
            0.0,
            max_height - padding * 2.0,
        );

        ctx.push_path(0);
        let child_hint = self.child.size_hint(ctx, child_constraints);
        ctx.pop_path();
        let modal_width = (child_hint.x + padding * 2.0).min(max_width);
        let modal_height = (child_hint.y + padding * 2.0).min(max_height);

        // Center the modal
        let modal_x = available_space.pos.x + (available_space.size.x - modal_width) / 2.0;
        let modal_y = available_space.pos.y + (available_space.size.y - modal_height) / 2.0;

        self.modal_rect = Rect::new(modal_x, modal_y, modal_width, modal_height);

        // Layout child with padding
        let child_rect = Rect::new(
            self.modal_rect.pos.x + padding,
            self.modal_rect.pos.y + padding,
            self.modal_rect.size.x - padding * 2.0,
            self.modal_rect.size.y - padding * 2.0,
        );

        ctx.push_path(0);
        self.child.layout(ctx, child_rect);
        ctx.pop_path();
    }

    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2 {
        // Modal doesn't take up space when closed, but we need to calculate child size
        let state: ModalState = ctx.get_state();
        if !state.open {
            return Vec2::ZERO;
        }

        let padding = ctx.theme.metrics.padding * 2.0;
        let child_constraints = SizeConstraints::new(
            0.0,
            constraints.max_width - padding * 2.0,
            0.0,
            constraints.max_height - padding * 2.0,
        );

        ctx.push_path(0);
        let child_hint = self.child.size_hint(ctx, child_constraints);
        ctx.pop_path();

        let size = Vec2::new(child_hint.x + padding * 2.0, child_hint.y + padding * 2.0);
        constraints.constrain(size)
    }

    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool {
        let mut state: ModalState = ctx.get_state();

        // Sync external state if bound
        if let Some(ref open_state) = self.open_state
            && let Ok(open) = open_state.lock()
        {
            state.open = *open;
        }

        if !state.open {
            return false;
        }

        let result = match event {
            Event::Key(key, _) => {
                use winit::keyboard::NamedKey;
                // Close on Escape key
                if matches!(key, winit::keyboard::Key::Named(NamedKey::Escape)) {
                    state.open = false;
                    if let Some(ref open_state) = self.open_state
                        && let Ok(mut open) = open_state.lock()
                    {
                        *open = false;
                    }
                    if let Some(callback) = &mut self.on_close {
                        callback();
                    }
                    ctx.set_state(state);
                    return true;
                }
                false
            }
            Event::Press(pos) => {
                // Check if click is outside modal (on backdrop)
                if self.close_on_backdrop_click && !self.modal_rect.contains(*pos) {
                    state.open = false;
                    if let Some(ref open_state) = self.open_state
                        && let Ok(mut open) = open_state.lock()
                    {
                        *open = false;
                    }
                    if let Some(callback) = &mut self.on_close {
                        callback();
                    }
                    ctx.set_state(state);
                    return true;
                }

                // Let child handle the event if it's inside the modal
                if self.modal_rect.contains(*pos) {
                    ctx.push_path(0);
                    let handled = self.child.handle_event(ctx, event);
                    ctx.pop_path();
                    return handled;
                }

                false
            }
            _ => {
                // For other events, only handle if they're within the modal
                if self.modal_rect.contains(match event {
                    Event::Move(pos) | Event::Release(pos) => *pos,
                    _ => return false,
                }) {
                    ctx.push_path(0);
                    let handled = self.child.handle_event(ctx, event);
                    ctx.pop_path();
                    handled
                } else {
                    // Consume events outside modal to prevent interaction with background
                    matches!(event, Event::Move(_) | Event::Release(_))
                }
            }
        };

        ctx.set_state(state);
        result
    }

    fn draw_overlay(&mut self, ctx: &mut WidgetContext) {
        let mut state: ModalState = ctx.get_state();

        // Sync external state if bound
        if let Some(ref open_state) = self.open_state
            && let Ok(open) = open_state.lock()
        {
            state.open = *open;
        }

        if !state.open {
            ctx.set_state(state);
            return;
        }

        // Clear any scissor so modal can render on top of everything
        let previous_scissor = ctx.painter.set_scissor(None);

        // Draw backdrop (semi-transparent overlay)
        let backdrop_color = [0.0, 0.0, 0.0, 0.5]; // Black with 50% opacity
        let backdrop_rect = Rect::new(0.0, 0.0, state.screen_size.x, state.screen_size.y);
        ctx.painter.draw_rect(backdrop_rect, backdrop_color);

        // Draw modal dialog
        let radius = ctx.theme.metrics.radius;
        let border_width = ctx.theme.metrics.border_width;

        // Draw border (outer rounded rect)
        ctx.painter
            .draw_rounded_rect(self.modal_rect, ctx.theme.colors.border, radius);

        // Draw background (inner rounded rect)
        let inset = border_width;
        let inner_rect = Rect::new(
            self.modal_rect.pos.x + inset,
            self.modal_rect.pos.y + inset,
            self.modal_rect.size.x - inset * 2.0,
            self.modal_rect.size.y - inset * 2.0,
        );
        ctx.painter.draw_rounded_rect(
            inner_rect,
            ctx.theme.colors.surface,
            (radius - inset).max(0.0),
        );

        // Draw child content
        ctx.push_path(0);
        self.child.draw(ctx);
        ctx.pop_path();

        // Draw child overlay (for nested modals, dropdowns, etc.)
        ctx.push_path(0);
        self.child.draw_overlay(ctx);
        ctx.pop_path();

        // Restore previous scissor
        ctx.painter.set_scissor(previous_scissor);

        ctx.set_state(state);
    }
}

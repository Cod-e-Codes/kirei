use crate::gui::theme::Theme;
use glam::Vec2;
use glyphon::Wrap;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;

/// Rendering pass for controlling text z-order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawPass {
    Normal,
    Overlay,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(w, h),
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.pos.x
            && point.x <= self.pos.x + self.size.x
            && point.y >= self.pos.y
            && point.y <= self.pos.y + self.size.y
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Move(Vec2),
    Press(Vec2),
    Release(Vec2),
    /// Secondary (usually right) mouse button press used for context menus
    ContextClick(Vec2),
    Char(char),
    Key(winit::keyboard::Key, Modifiers), // Key + modifiers
    Scroll(f32),                          // Vertical scroll delta
    Ime(ImeEvent),                        // IME composition events
}

/// IME (Input Method Editor) events for non-ASCII input support
#[derive(Debug, Clone)]
pub enum ImeEvent {
    /// IME composition started
    Start,
    /// IME composition text update (pre-edit string)
    Update(String),
    /// IME composition completed (commit the composed text)
    Commit(String),
    /// IME composition cancelled
    Cancel,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

pub trait Painter {
    fn draw_rect(&mut self, rect: Rect, color: [f32; 4]);
    fn draw_rounded_rect(&mut self, rect: Rect, color: [f32; 4], radius: f32);
    fn draw_image(&mut self, rect: Rect, texture_id: usize);
    /// Draw text clipped by the active scissor (`set_scissor`), but shaped using `layout_size`
    /// (passed to cosmic-text `Buffer::set_size`). Width controls wrapping; height bounds vertical
    /// shaping for word-wrapped blocks. Always pass the widget content size from layout, not the
    /// viewport. Use [`text_layout_height_unbounded`] when measuring or drawing blocks whose height
    /// is driven only by content (intrinsic sizing).
    fn draw_text(
        &mut self,
        text: &str,
        pos: Vec2,
        color: [f32; 4],
        font_size: f32,
        wrap: Wrap,
        layout_size: Vec2,
    );
    fn get_text_size(&mut self, text: &str, font_size: f32) -> Vec2;
    /// Height of `text` when wrapped to `max_width`, using the same shaping as word-wrapped `draw_text`.
    fn get_wrapped_text_size(&mut self, text: &str, font_size: f32, max_width: f32) -> Vec2;
    fn set_scissor(&mut self, rect: Option<Rect>) -> Option<Rect>;
    fn set_draw_pass(&mut self, pass: DrawPass);
}

/// Use as `layout_size.y` when vertical extent must follow content (no fixed box height), for example
/// intrinsic height measurement in [`Painter::get_wrapped_text_size`].
#[inline]
pub fn text_layout_height_unbounded() -> f32 {
    f32::INFINITY
}

/// Unique identifier for a widget, used for state persistence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    /// Generate a widget ID from a path in the widget tree
    /// The path is a sequence of indices representing the position in the tree
    pub fn from_path(path: &[usize]) -> Self {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        for &idx in path {
            // Hash the usize index
            let bytes = idx.to_ne_bytes();
            for byte in bytes {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x100000001b3); // FNV prime
            }
        }
        WidgetId(hash)
    }

    /// Generate a widget ID from a string key (for manual assignment)
    pub fn from_key(key: &str) -> Self {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        for byte in key.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        }
        WidgetId(hash)
    }
}

/// Manages keyboard focus state and tab navigation
#[derive(Debug, Default)]
pub struct FocusManager {
    /// Currently focused widget ID (if any)
    current_focus: Option<WidgetId>,
    /// Ordered list of focusable widget IDs (built during layout)
    focus_chain: Vec<WidgetId>,
    /// Set of registered IDs for fast lookup (prevents duplicates)
    registered_ids: std::collections::HashSet<WidgetId>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the currently focused widget ID
    pub fn get_focused(&self) -> Option<WidgetId> {
        self.current_focus
    }

    /// Set focus to a specific widget
    pub fn set_focus(&mut self, id: Option<WidgetId>) {
        self.current_focus = id;
    }

    /// Check if a widget has focus
    pub fn has_focus(&self, id: WidgetId) -> bool {
        self.current_focus == Some(id)
    }

    /// Clear the focus chain (called at start of layout)
    pub fn clear_chain(&mut self) {
        self.focus_chain.clear();
        self.registered_ids.clear();
    }

    /// Register a focusable widget (called during layout)
    pub fn register_focusable(&mut self, id: WidgetId) {
        // Only add if not already in the chain (prevents duplicates on every frame)
        if self.registered_ids.insert(id) {
            self.focus_chain.push(id);
        }
    }

    /// Clear the focus chain (call before layout pass)
    pub fn clear_focus_chain(&mut self) {
        self.focus_chain.clear();
        self.registered_ids.clear();
    }

    /// Move focus to next widget in chain (Tab)
    pub fn focus_next(&mut self) -> Option<WidgetId> {
        if self.focus_chain.is_empty() {
            return None;
        }

        let next_index = if let Some(current) = self.current_focus {
            // Find current in chain and move to next
            self.focus_chain
                .iter()
                .position(|&id| id == current)
                .map(|i| (i + 1) % self.focus_chain.len())
                .unwrap_or(0)
        } else {
            // No current focus, start at beginning
            0
        };

        let next_id = self.focus_chain[next_index];
        self.current_focus = Some(next_id);
        Some(next_id)
    }

    /// Move focus to previous widget in chain (Shift+Tab)
    pub fn focus_prev(&mut self) -> Option<WidgetId> {
        if self.focus_chain.is_empty() {
            return None;
        }

        let prev_index = if let Some(current) = self.current_focus {
            // Find current in chain and move to previous
            self.focus_chain
                .iter()
                .position(|&id| id == current)
                .map(|i| {
                    if i == 0 {
                        self.focus_chain.len() - 1
                    } else {
                        i - 1
                    }
                })
                .unwrap_or(self.focus_chain.len() - 1)
        } else {
            // No current focus, start at end
            self.focus_chain.len() - 1
        };

        let prev_id = self.focus_chain[prev_index];
        self.current_focus = Some(prev_id);
        Some(prev_id)
    }
}

/// Storage for widget state, keyed by WidgetId
/// This allows state to persist across widget tree rebuilds
/// Uses type-erased storage (Box<dyn Any>) to allow each widget to have its own state type
#[derive(Debug, Default)]
pub struct WidgetStateStorage {
    states: HashMap<WidgetId, Box<dyn Any>>,
}

impl WidgetStateStorage {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Get typed state for a widget, creating default if it doesn't exist
    pub fn get_or_default<T: Default + Clone + 'static>(&mut self, id: WidgetId) -> T {
        self.states
            .entry(id)
            .or_insert_with(|| Box::new(T::default()))
            .downcast_ref::<T>()
            .cloned()
            .expect("Widget state type mismatch")
    }

    /// Set typed state for a widget
    pub fn set<T: 'static>(&mut self, id: WidgetId, state: T) {
        self.states.insert(id, Box::new(state));
    }

    /// Get immutable reference to typed state (returns None if not exists)
    pub fn get<T: 'static>(&self, id: WidgetId) -> Option<&T> {
        self.states.get(&id)?.downcast_ref::<T>()
    }

    /// Get mutable reference to typed state (returns None if not exists)
    pub fn get_mut<T: 'static>(&mut self, id: WidgetId) -> Option<&mut T> {
        self.states.get_mut(&id)?.downcast_mut::<T>()
    }

    /// Remove state for a widget
    pub fn remove(&mut self, id: WidgetId) {
        self.states.remove(&id);
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.states.clear();
    }
}

pub struct WidgetContext<'a> {
    pub painter: &'a mut dyn Painter,
    pub theme: &'a Theme,
    pub state_storage: &'a mut WidgetStateStorage,
    pub current_path: Vec<usize>, // Path in the widget tree for ID generation
    explicit_id_stack: Vec<Option<WidgetId>>, // Stack of explicit IDs for widgets with .with_id()
    pub focus_manager: &'a mut FocusManager, // Focus state and tab navigation
    pub request_redraw_fn: &'a dyn Fn(),
}

impl<'a> WidgetContext<'a> {
    pub fn new(
        painter: &'a mut dyn Painter,
        theme: &'a Theme,
        state_storage: &'a mut WidgetStateStorage,
        focus_manager: &'a mut FocusManager,
        request_redraw_fn: &'a dyn Fn(),
    ) -> Self {
        Self {
            painter,
            theme,
            state_storage,
            current_path: Vec::new(),
            explicit_id_stack: Vec::new(),
            focus_manager,
            request_redraw_fn,
        }
    }

    pub fn request_redraw(&self) {
        (self.request_redraw_fn)();
    }

    /// Get the current widget ID based on the current path or explicit ID
    /// Explicit IDs (set via .with_id()) take precedence over path-based IDs
    pub fn current_id(&self) -> WidgetId {
        if let Some(Some(id)) = self.explicit_id_stack.last() {
            return *id;
        }
        WidgetId::from_path(&self.current_path)
    }

    /// Push an index to the current path (for child widgets)
    pub fn push_path(&mut self, index: usize) {
        self.current_path.push(index);
    }

    /// Pop from the current path (after processing a child)
    pub fn pop_path(&mut self) {
        self.current_path.pop();
    }

    /// Push an explicit ID to the stack (for widgets with .with_id())
    /// Pass None to push a placeholder (widget has no explicit ID)
    pub fn push_explicit_id(&mut self, key: Option<&str>) {
        let id = key.map(WidgetId::from_key);
        self.explicit_id_stack.push(id);
    }

    /// Pop from the explicit ID stack (after processing a widget)
    pub fn pop_explicit_id(&mut self) {
        self.explicit_id_stack.pop();
    }

    /// Get or create typed state for the current widget
    pub fn get_state<T: Default + Clone + 'static>(&mut self) -> T {
        self.state_storage.get_or_default(self.current_id())
    }

    /// Set typed state for the current widget
    pub fn set_state<T: 'static>(&mut self, state: T) {
        self.state_storage.set(self.current_id(), state);
    }

    /// Get state by explicit ID (for reading other widgets' state)
    pub fn get_state_by_id<T: Default + Clone + 'static>(&mut self, id: WidgetId) -> T {
        self.state_storage.get_or_default(id)
    }

    /// Set state by explicit ID (for updating other widgets' state)
    pub fn set_state_by_id<T: 'static>(&mut self, id: WidgetId, state: T) {
        self.state_storage.set(id, state);
    }
}

pub trait Widget {
    fn draw(&mut self, ctx: &mut WidgetContext);
    fn layout(&mut self, ctx: &mut WidgetContext, available_space: Rect);
    /// Calculate the ideal size for this widget given constraints.
    /// The returned size should respect the constraints (min/max bounds).
    fn size_hint(&self, ctx: &mut WidgetContext, constraints: SizeConstraints) -> Vec2;
    fn handle_event(&mut self, ctx: &mut WidgetContext, event: &Event) -> bool;

    /// Draw overlay elements (e.g., dropdown menus, tooltips) that should appear on top of all widgets.
    /// This is called after all widgets have been drawn in the normal draw() phase.
    fn draw_overlay(&mut self, _ctx: &mut WidgetContext) {
        // Default implementation does nothing - only widgets with overlays need to implement this
    }

    /// Returns true if this widget can receive keyboard focus (for Tab navigation)
    fn is_focusable(&self) -> bool {
        false // Default: not focusable
    }
}

/// Animated value for smooth transitions (e.g., hover effects, scrolling)
#[derive(Debug, Clone, Copy)]
pub struct AnimatedValue {
    current: f32,
    target: f32,
    speed: f32, // units per second
}

impl AnimatedValue {
    /// Create a new animated value starting at the given value
    pub fn new(initial: f32, speed: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            speed,
        }
    }

    /// Set the target value to animate towards
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Get the current value
    pub fn get(&self) -> f32 {
        self.current
    }

    /// Set the current value instantly (no animation)
    pub fn set_instant(&mut self, value: f32) {
        self.current = value;
        self.target = value;
    }

    /// Check if the value is still animating
    pub fn is_animating(&self) -> bool {
        (self.current - self.target).abs() >= 0.01
    }

    /// Update the animated value, returns true if still animating
    pub fn update(&mut self, dt: f32) -> bool {
        if (self.current - self.target).abs() < 0.01 {
            self.current = self.target;
            return false;
        }

        let diff = self.target - self.current;
        let delta = diff.signum() * (self.speed * dt).min(diff.abs());
        self.current += delta;
        true
    }

    /// Update with spring-like smooth interpolation (exponential decay)
    pub fn update_smooth(&mut self, dt: f32, smoothness: f32) -> bool {
        if (self.current - self.target).abs() < 0.01 {
            self.current = self.target;
            return false;
        }

        // Exponential decay for smooth animation
        let t = 1.0 - (-smoothness * dt).exp();
        self.current += (self.target - self.current) * t;
        true
    }
}

impl Default for AnimatedValue {
    fn default() -> Self {
        Self::new(0.0, 100.0)
    }
}

/// Size constraints for widgets, similar to Flutter's BoxConstraints
/// Used to communicate available space and limits during layout
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SizeConstraints {
    /// Minimum width (0.0 means no minimum)
    pub min_width: f32,
    /// Maximum width (f32::INFINITY means no maximum)
    pub max_width: f32,
    /// Minimum height (0.0 means no minimum)
    pub min_height: f32,
    /// Maximum height (f32::INFINITY means no maximum)
    pub max_height: f32,
}

impl SizeConstraints {
    /// Create unbounded constraints (no min/max limits)
    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Create constraints with a fixed size
    pub fn tight(size: Vec2) -> Self {
        Self {
            min_width: size.x,
            max_width: size.x,
            min_height: size.y,
            max_height: size.y,
        }
    }

    /// Create constraints with min/max bounds
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    /// Constrain a size to fit within these constraints
    pub fn constrain(&self, size: Vec2) -> Vec2 {
        Vec2::new(
            size.x.max(self.min_width).min(self.max_width),
            size.y.max(self.min_height).min(self.max_height),
        )
    }

    /// Get the maximum size allowed by these constraints
    pub fn max_size(&self) -> Vec2 {
        Vec2::new(self.max_width, self.max_height)
    }

    /// Get the minimum size required by these constraints
    pub fn min_size(&self) -> Vec2 {
        Vec2::new(self.min_width, self.min_height)
    }

    /// Check if constraints are tight (min == max)
    pub fn is_tight(&self) -> bool {
        (self.min_width - self.max_width).abs() < f32::EPSILON
            && (self.min_height - self.max_height).abs() < f32::EPSILON
    }
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self::unbounded()
    }
}

# kirei

A GUI library for Rust built on wgpu and winit.

## Overview

kirei provides a widget-based GUI system with GPU-accelerated rendering. It includes a layout system, event handling, and a set of built-in widgets.

## Dependencies

Version requirements live in `Cargo.toml` (semver-compatible ranges, not pinned patch versions). The resolved dependency graph is in `Cargo.lock` after you run `cargo update`.

Inspect what gets built:

```bash
cargo tree -p kirei -e normal
```

Refresh lockfile to the newest releases allowed by those ranges:

```bash
cargo update
```

Bump the version fields in `Cargo.toml` to newer compatible releases (requires [cargo-edit](https://github.com/killercup/cargo-edit)):

```bash
cargo upgrade
```

## Widgets

Available widgets:

- Label
- Button
- Checkbox
- RadioButtons
- Slider
- ProgressBar
- TextInput
- TextArea (supports `with_id` for stable `WidgetStateStorage` keys, optional `font_size`)
- Dropdown
- ContextMenu (global or widget-scoped right-click menus)
- Tabs (tabbed interface with multiple panels)
- ImageWidget
- Column (vertical layout)
- Row (horizontal layout)
- Panel (container)
- Modal (centered dialog / popup overlay widget)
- ScrollView (scrollable container with viewport culling)

### Text layout vs clipping

`Painter::draw_text` takes an explicit `layout_size` for cosmic-text shaping (wrap width and vertical extent). That must be the widget content size from layout, not the scroll viewport. The active scissor only clips pixels. For intrinsic height only (for example `get_wrapped_text_size`), use `text_layout_height_unbounded` as the `y` component of `layout_size`. Built-in widgets already pass the correct sizes.
- Tooltip (hover tooltip wrapper for any widget)

## Usage

See the examples directory for complete usage examples:

- `demo.rs` - Basic widget demonstration
- `demo_fps.rs` - A demo with animations (includes real-time FPS counter)
- `demo_column.rs` - Column layout examples
- `demo_row.rs` - Row layout examples
- `demo_complex.rs` - Complex layout examples

### Running Examples

Run an example (recommended with `--release` for better performance):

```bash
cargo run --example demo --release
```

### Building

Build the library in release mode:

```bash
cargo build --release
```

Build all examples in release mode:

```bash
cargo build --examples --release
```

### Development

Format code and run the linter:

```bash
cargo fmt
cargo clippy --all-targets
```

## Architecture

The library consists of:

- `gui/core` - Widget trait, event system, layout system, state management, focus management
- `gui/renderer` - GPU rendering backend with two-pass rendering for proper overlay z-ordering
- `gui/theme` - Theming system
- `gui/widgets` - Widget implementations

Widgets implement the `Widget` trait and participate in layout and event handling. The renderer batches draw calls and manages GPU resources.

### Two-Pass Rendering

The renderer uses a two-pass system to ensure proper z-ordering:
- **Normal Pass**: Main UI shapes and text are rendered first
- **Overlay Pass**: Modal backdrops and overlay text are rendered last

This ensures modal dialogs and context menus appear correctly on top of the main UI.

**Known Issue**: Text rendering through glyphon may batch all text together regardless of pass, causing some overlay text to render incorrectly or bleed through modal backgrounds in certain scenarios.

### Performance

The library is designed for real-time interactive applications. The `demo_fps.rs` example includes a real-time FPS counter widget drawn using the normal widget system. Typical framerates on modern hardware are 120-140 FPS, demonstrating efficient rendering even with complex widget hierarchies, text rendering, and layout calculations.

### State Management

Widgets use a centralized state management system:
- Each widget has a unique `WidgetId` for state persistence
- **Explicit IDs**: Use `.with_id("unique_key")` to assign stable IDs that survive UI refactoring
- **Path-based IDs**: Automatic IDs based on widget tree position (fragile, use explicit IDs instead)
- Widget state (hover, press, values, etc.) is stored in `WidgetStateStorage`
- Widgets are stateless - all state lives in the storage, enabling persistence and easier state management
- Type-safe state storage with per-widget state types

### Focus Management

The library includes keyboard focus management for accessible navigation:
- **Tab Navigation**: Press Tab to cycle forward through focusable widgets, Shift+Tab to cycle backward
- **Focus Chain**: Automatically built during layout from widgets that implement `is_focusable()`
- **Visual Indicators**: Focused widgets display focus rings for keyboard navigation clarity
- **Focusable Widgets**: Currently includes `TextInput` and `TextArea`
- **Focus Manager**: Centralized `FocusManager` tracks focus state and handles keyboard navigation

**Known Issue**: Tab navigation requires mouse movement or rapid key presses to trigger - single Tab key presses may not immediately change focus without additional input events.

#### Widget ID Management

For widgets that need persistent state across UI changes, use explicit IDs:

```rust
// Good: Explicit ID survives UI refactoring
Slider::new(0.5, 0.0, 1.0).with_id("volume_slider")
Button::new("Click Me").with_id("submit_button")

// Access state elsewhere
let slider_id = WidgetId::from_key("volume_slider");
let state: SliderState = widget_state.get_or_default(slider_id);
```

Without explicit IDs, widget state is tied to tree position and breaks when the UI structure changes. Currently supported on: `Label`, `Button`, `Slider`, `TextInput`.

### Layout System

The layout system supports:
- Flexbox-style growing and shrinking via `FlexConfig`
- Minimum and maximum size constraints
- Alignment options: `Start`, `Center`, `End`, `Stretch`
- Dynamic sizing based on constraints through `size_hint()`

### Text Input

TextInput widget includes:
- IME (Input Method Editor) support for non-ASCII input
- Clipboard operations (copy, cut, paste) with error handling
- Character position caching for efficient rendering
- Text selection and cursor navigation

## License

This project is licensed under the MIT License.
use kirei::gui::core::{DrawPass, FocusManager, Rect, Widget, WidgetContext, WidgetStateStorage};
use kirei::gui::renderer::{RenderContext, Renderer};
use kirei::gui::theme::Theme;
use kirei::gui::widgets::{
    Align, Button, Checkbox, Column, ContextMenu, ContextMenuItem, Dropdown, ImageWidget, Label,
    Modal, Overlay, Panel, ProgressBar, RadioButtons, Row, ScrollView, Slider, Tabs, TextArea,
    TextInput, Tooltip,
};
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

// Input State
struct InputState {
    mouse_pos: glam::Vec2,
    mouse_pressed: bool,
    modifiers: winit::keyboard::ModifiersState,
}

impl InputState {
    fn default() -> Self {
        Self {
            mouse_pos: glam::Vec2::ZERO,
            mouse_pressed: false,
            modifiers: winit::keyboard::ModifiersState::default(),
        }
    }
}

// GUI Event Wrapper
pub struct GuiEvent {
    pub winit_event: Option<WindowEvent>, // For raw events if needed
    pub event: kirei::gui::core::Event,
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    renderer: Renderer,
    root_widget: Box<dyn Widget>,
    input_state: InputState,
    theme: Theme,
    is_dark_theme: Arc<Mutex<bool>>,
    window: Arc<Window>,
    widget_state: WidgetStateStorage,
    focus_manager: FocusManager,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Use Arc<Window> to create a 'static surface
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut renderer = Renderer::new(&device, &queue, config.format, size.width, size.height);

        // Load example texture (handle Result from improved error handling)
        let texture_id = renderer
            .load_texture(&device, &queue, "sphere.png")
            .expect("Failed to load sphere.png texture");

        // Theme toggle state
        let is_dark_theme = Arc::new(Mutex::new(true));
        let is_dark_theme_clone = is_dark_theme.clone();

        // Password visibility toggle
        let show_password = Arc::new(Mutex::new(false));
        let show_password_for_input = show_password.clone();
        let show_password_for_checkbox = show_password.clone();

        // Modal open state
        let modal_open = Arc::new(Mutex::new(false));
        let modal_open_for_button = modal_open.clone();
        let modal_open_for_modal = modal_open.clone();

        // Create UI
        let login_panel = Panel::new(
            Column::new()
                .push(Label::new("Account Login").with_color([0.2, 0.6, 1.0, 1.0]))
                .push(
                    TextInput::new("Email address")
                        .max_chars(64)
                        .sanitize_with(|text| text.trim().to_lowercase())
                        .validate(|text| {
                            let trimmed = text.trim();
                            if trimmed.is_empty() {
                                Some("Email is required".into())
                            } else if !trimmed.contains('@') || !trimmed.contains('.') {
                                Some("Enter a valid email address".into())
                            } else {
                                None
                            }
                        })
                        .on_change(|email| {
                            println!("Email captured: {}", email);
                        }),
                )
                .push(
                    TextInput::password("Password")
                        .bind_password_visibility(show_password_for_input)
                        .max_chars(32)
                        .sanitize_with(|text| text.trim().to_string())
                        .validate(|text| {
                            if text.chars().count() < 8 {
                                Some("Password must be at least 8 characters".into())
                            } else {
                                None
                            }
                        })
                        .on_change(|_| {
                            println!("Password updated");
                        }),
                )
                .push(
                    Row::new().push(Checkbox::new("Show password text", false).on_change(
                        move |checked| {
                            if let Ok(mut visible) = show_password_for_checkbox.lock() {
                                *visible = checked;
                            }
                        },
                    )),
                )
                .push(Label::new("Tell us about your project:"))
                .push(
                    TextArea::new("Goals, constraints, timeline...")
                        .with_min_lines(3)
                        .auto_grow(true)
                        .resizable(true)
                        .allow_tab(true)
                        .max_chars(240)
                        .sanitize_with(|text| text.replace('\r', ""))
                        .validate(|text| {
                            let trimmed = text.trim();
                            if trimmed.is_empty() {
                                Some("A short project description is required".into())
                            } else if trimmed.chars().count() < 10 {
                                Some("Please provide at least 10 characters".into())
                            } else {
                                None
                            }
                        })
                        .on_change(|text| {
                            println!("Project notes updated ({} chars)", text.len());
                        }),
                )
                .push(Button::new("Log In").on_click(|| {
                    println!("Attempting login...");
                })),
        );

        let tabs_demo = Tabs::new()
            .push_tab(
                "Overview",
                Column::new()
                    .push(Label::new("Live metrics"))
                    .push(ProgressBar::new(0.82))
                    .push(Label::new("Throughput steady and within limits")),
            )
            .push_tab(
                "Settings",
                Column::new()
                    .push(Label::new("Notification threshold"))
                    .push(Slider::new(0.35, 0.0, 1.0).on_change(|level| {
                        println!("Alert level set to {:.2}", level);
                    }))
                    .push(Checkbox::new("Enable alerts", true).on_change(|checked| {
                        println!("Alerts {}", if checked { "enabled" } else { "disabled" });
                    })),
            )
            .push_tab(
                "Activity",
                Column::new()
                    .push(Label::new("Recent events"))
                    .push(Label::new("09:45    Provisioned staging cluster"))
                    .push(Label::new("10:05    Kicked off integration suite"))
                    .push(Button::new("View full log").on_click(|| {
                        println!("Activity log requested");
                    })),
            )
            .on_change(|index, label| {
                println!("Switched to tab {} ({})", index, label);
            });

        let content = Column::new()
            .push(Label::new("UI Toolkit Demo").with_color([0.2, 0.6, 1.0, 1.0]))
            .push(
                Row::new()
                    .push(Label::new("Theme:"))
                    .push(Button::new("Switch Theme").on_click(move || {
                        let mut is_dark = is_dark_theme_clone.lock().unwrap();
                        *is_dark = !*is_dark;
                        println!(
                            "Theme switched to {}",
                            if *is_dark { "dark" } else { "light" }
                        );
                    })),
            )
            .push(login_panel)
            .push(Panel::new(
                Column::new()
                    .push(Label::new("Controls Panel"))
                    .push(Checkbox::new("Check me!", false).on_change(|checked| {
                        println!("Checkbox changed: {}", checked);
                    }))
                    .push(
                        Tooltip::new(
                            Slider::new(0.5, 0.0, 1.0).on_change(|val| {
                                println!("Slider value: {:.2}", val);
                            }),
                            "Drag to adjust the value from 0.0 to 1.0",
                        )
                        .with_delay(20),
                    )
                    .push(
                        Tooltip::new(
                            ProgressBar::new(0.7),
                            "This progress bar shows 70% completion",
                        )
                        .with_delay(20),
                    )
                    .push(Label::new("Context Menu:"))
                    .push(
                        ContextMenu::new(Panel::new(
                            Column::new()
                                .push(Label::new("Scene Actions").with_color([0.2, 0.6, 1.0, 1.0]))
                                .push(Label::new(
                                    "Right-click anywhere in the window to open this menu",
                                ))
                                .align_items(Align::Start),
                        ))
                        .global(true)
                        .with_items(vec![
                            ContextMenuItem::new("Refresh Data")
                                .shortcut("Ctrl+R")
                                .on_select(|| {
                                    println!("Context menu: refresh data");
                                }),
                            ContextMenuItem::new("Duplicate Panel")
                                .shortcut("Ctrl+D")
                                .on_select(|| {
                                    println!("Context menu: duplicate panel");
                                }),
                            ContextMenuItem::new("Delete Panel")
                                .destructive(true)
                                .on_select(|| {
                                    println!("Context menu: delete panel");
                                }),
                        ]),
                    )
                    .push(Label::new("Radio Buttons:"))
                    .push(
                        RadioButtons::new(vec![
                            "Alpha".to_string(),
                            "Beta".to_string(),
                            "Gamma".to_string(),
                        ])
                        .select(1)
                        .on_change(|index, label| {
                            println!("Radio selection {}: {}", index, label);
                        }),
                    )
                    .push(Label::new("Dropdown:"))
                    .push(
                        Dropdown::new(vec![
                            "Option 1".to_string(),
                            "Option 2".to_string(),
                            "Option 3".to_string(),
                            "Option 4".to_string(),
                            "Option 5".to_string(),
                            "Option 6".to_string(),
                            "Option 7".to_string(),
                            "Option 8".to_string(),
                            "Option 9".to_string(),
                            "Option 10".to_string(),
                        ])
                        .with_placeholder("Select an option...")
                        .on_change(|index, text| {
                            println!("Dropdown selected: {} ({})", index, text);
                        }),
                    ),
            ))
            .push(Label::new("Image:"))
            .push(ImageWidget::new(texture_id).with_size(glam::Vec2::new(338.0, 100.0)))
            .push(Label::new("Tabs:"))
            .push(Panel::new(tabs_demo))
            .push(
                Tooltip::new(
                    Button::new("Click Me!").on_click(|| {
                        println!("Button clicked!");
                    }),
                    "This button triggers an action when clicked",
                )
                .with_delay(20),
            )
            .push(Label::new("Modal Dialog:"))
            .push(Button::new("Open Modal").on_click(move || {
                if let Ok(mut open) = modal_open_for_button.lock() {
                    *open = true;
                    println!("Opening modal dialog");
                }
            }))
            .align_items(Align::Start);

        // Create modal content
        let modal_open_for_close = modal_open_for_modal.clone();
        let modal_open_for_cancel = modal_open_for_modal.clone();
        let modal_open_for_on_close = modal_open_for_modal.clone();
        let modal_content = Panel::new(
            Column::new()
                .push(Label::new("Modal Dialog").with_color([0.2, 0.6, 1.0, 1.0]))
                .push(Label::new("This is a modal dialog example."))
                .push(Label::new("You can close it by:"))
                .push(Label::new("• Pressing the Escape key"))
                .push(Label::new("• Clicking outside the dialog"))
                .push(Label::new("• Clicking the Close button"))
                .push(
                    Row::new()
                        .push(Button::new("Close").on_click(move || {
                            if let Ok(mut open) = modal_open_for_close.lock() {
                                *open = false;
                                println!("Closing modal dialog");
                            }
                        }))
                        .push(Button::new("Cancel").on_click(move || {
                            if let Ok(mut open) = modal_open_for_cancel.lock() {
                                *open = false;
                                println!("Modal cancelled");
                            }
                        })),
                )
                .align_items(Align::Start),
        );

        // Create modal widget
        let modal = Modal::new(modal_content)
            .bind_open_state(modal_open_for_modal.clone())
            .close_on_backdrop_click(true)
            .on_close(move || {
                if let Ok(mut open) = modal_open_for_on_close.lock() {
                    *open = false;
                }
            });

        // Base UI wrapped in overlay so modal can render across the entire viewport
        let base_ui = Panel::new(ScrollView::new(content));
        let root_widget = Overlay::new(base_ui, modal);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            renderer,
            root_widget: Box::new(root_widget),
            input_state: InputState::default(),
            theme: Theme::dark(),
            is_dark_theme,
            window,
            widget_state: WidgetStateStorage::new(),
            focus_manager: FocusManager::new(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.renderer
                .resize(new_size.width, new_size.height, &self.queue);
            self.renderer.prepare();

            // Layout and draw
            let rect = Rect::new(0.0, 0.0, new_size.width as f32, new_size.height as f32);
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            // Clear focus chain before layout to rebuild it
            ctx.focus_manager.clear_focus_chain();
            self.root_widget.layout(&mut ctx, rect);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        use kirei::gui::core::Event as GuiEvent;

        let gui_event = match event {
            WindowEvent::CursorMoved { position, .. } => {
                let pos = glam::Vec2::new(position.x as f32, position.y as f32);
                self.input_state.mouse_pos = pos;
                Some(GuiEvent::Move(pos))
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => match button {
                MouseButton::Left => {
                    self.input_state.mouse_pressed = true;
                    Some(GuiEvent::Press(self.input_state.mouse_pos))
                }
                MouseButton::Right => Some(GuiEvent::ContextClick(self.input_state.mouse_pos)),
                _ => None,
            },
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.input_state.mouse_pressed = false;
                Some(GuiEvent::Release(self.input_state.mouse_pos))
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            } => None,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: winit::keyboard::Key::Character(c),
                        ..
                    },
                ..
            } => {
                // Don't send Char events if Ctrl or Alt is pressed (let Key handler deal with it)
                if self.input_state.modifiers.control_key() || self.input_state.modifiers.alt_key()
                {
                    let modifiers = kirei::gui::core::Modifiers {
                        ctrl: self.input_state.modifiers.control_key(),
                        shift: self.input_state.modifiers.shift_key(),
                        alt: self.input_state.modifiers.alt_key(),
                    };
                    Some(GuiEvent::Key(
                        winit::keyboard::Key::Character(c.clone()),
                        modifiers,
                    ))
                } else {
                    // Only first char for simplicity
                    c.chars().next().map(GuiEvent::Char)
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        ..
                    },
                ..
            } => {
                let modifiers = kirei::gui::core::Modifiers {
                    ctrl: self.input_state.modifiers.control_key(),
                    shift: self.input_state.modifiers.shift_key(),
                    alt: self.input_state.modifiers.alt_key(),
                };
                Some(GuiEvent::Key(logical_key.clone(), modifiers))
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.input_state.modifiers = new_modifiers.state();
                None
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => *y * 3.0,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
                };
                Some(GuiEvent::Scroll(scroll_delta))
            }
            _ => None,
        };

        if let Some(e) = gui_event {
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            self.root_widget.handle_event(&mut ctx, &e);
            // Always request redraw after handling events (hover, clicks, scroll, etc.)
            self.window.request_redraw();
            true
        } else {
            false
        }
    }

    fn update(&mut self) {
        // Check if theme should be switched
        let is_dark = *self.is_dark_theme.lock().unwrap();
        let current_is_dark =
            matches!(self.theme, Theme { colors: ref c, .. } if c.background[0] < 0.15);

        if is_dark != current_is_dark {
            self.theme = if is_dark {
                Theme::dark()
            } else {
                Theme::light()
            };
            // Request redraw when theme changes
            self.window.request_redraw();
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Clear screen
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.theme.colors.background[0] as f64,
                            g: self.theme.colors.background[1] as f64,
                            b: self.theme.colors.background[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // Prepare GUI (clear batches/text from previous frame)
        self.renderer.prepare();

        // Layout widgets first
        {
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            // Clear focus chain before layout to rebuild it
            ctx.focus_manager.clear_focus_chain();
            let rect = Rect::new(0.0, 0.0, self.size.width as f32, self.size.height as f32);
            self.root_widget.layout(&mut ctx, rect);
        }

        // Draw normal widgets (accumulate batches/text)
        {
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            // Set draw pass to Normal so text goes to normal_text_areas
            ctx.painter.set_draw_pass(DrawPass::Normal);
            self.root_widget.draw(&mut ctx);
        }

        // Get counts before drawing overlays to split rendering
        let (normal_batch_count, normal_text_count) = self.renderer.get_content_counts();

        // Draw overlays (accumulate more batches/text on top)
        // Layout was already done, so widget rects should be correct
        // Reset scissor state before drawing overlays so they aren't clipped
        {
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
            // Clear any scissor that might have been set by normal widget drawing
            ctx.painter.set_scissor(None);
            // Set draw pass to Overlay so text goes to overlay_text_areas
            ctx.painter.set_draw_pass(DrawPass::Overlay);
            self.root_widget.draw_overlay(&mut ctx);
        }

        // Render with split: normal widgets first, then overlays on top
        let mut render_ctx = RenderContext {
            device: &self.device,
            queue: &self.queue,
            view: &view,
            encoder: &mut encoder,
            width: self.size.width,
            height: self.size.height,
        };
        self.renderer.render_split(
            &mut render_ctx,
            Some((normal_batch_count, normal_text_count)),
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes =
                Window::default_attributes().with_title("RS Graphics 2 - Modern UI Toolkit");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let state = pollster::block_on(State::new(window));
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        if !state.input(&event) {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                _ => {}
            }
        } else {
            // If input handled by GUI, don't call update() to avoid duplicate request_redraw()
            // The input() method already calls request_redraw() if needed by event handling.
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

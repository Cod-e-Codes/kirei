use kirei::gui::core::{
    AnimatedValue, Event, FocusManager, Modifiers, Rect, Widget, WidgetContext, WidgetId,
    WidgetStateStorage,
};
use kirei::gui::renderer::Renderer;
use kirei::gui::theme::Theme;
use kirei::gui::widgets::{
    Button, Checkbox, Column, Label, LabelState, Panel, ProgressBar, ScrollView, Slider,
    SliderState, TextInput,
};
use std::sync::Arc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct InputState {
    mouse_pos: glam::Vec2,
    mouse_pressed: bool,
    modifiers: winit::keyboard::ModifiersState,
    ime_composing: bool, // Track if IME composition is active
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_pos: glam::Vec2::ZERO,
            mouse_pressed: false,
            modifiers: winit::keyboard::ModifiersState::default(),
            ime_composing: false,
        }
    }
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
    window: Arc<Window>,
    widget_state: WidgetStateStorage,
    focus_manager: FocusManager,

    // Demo-specific: Animation tracking
    last_frame_time: Instant,
    progress_animation: AnimatedValue,
    frame_count: u64,
    last_fps_update: Instant,
    fps: f32,

    // Widget IDs for dynamic updates
    fps_label_id: WidgetId,
    progress_bar_id: WidgetId,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

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

        let renderer = Renderer::new(&device, &queue, config.format, size.width, size.height);
        let theme = Theme::dark();
        let now = Instant::now();

        let progress_animation = AnimatedValue::new(0.0, 0.5);

        // Generate widget IDs based on paths in the tree
        // FPS label path: ScrollView[0] -> Column[16] (17th child, index 16)
        // Progress bar path: ScrollView[0] -> Column[3] -> Panel[0] -> Column[1]
        let fps_label_id = WidgetId::from_path(&[0, 16]);
        let progress_bar_id = WidgetId::from_path(&[0, 3, 0, 1]);

        let mut state = Self {
            surface,
            device,
            queue,
            config,
            size,
            renderer,
            root_widget: Box::new(Column::new()), // Placeholder, will be rebuilt
            input_state: InputState::default(),
            theme,
            window,
            widget_state: WidgetStateStorage::new(),
            focus_manager: FocusManager::new(),
            last_frame_time: now,
            progress_animation,
            frame_count: 0,
            last_fps_update: now,
            fps: 0.0,
            fps_label_id,
            progress_bar_id,
        };

        // Build initial UI
        state.rebuild_ui();
        state
    }

    fn rebuild_ui(&mut self) {
        // Build UI once - state will be updated via WidgetState
        let progress_bar = ProgressBar::new(0.0);
        let text_input = TextInput::new("Type here");

        // Create scrollable content with enough items to demonstrate scrolling
        let scrollable_content = Column::new()
            .push(Panel::new(
                Column::new()
                    .push(
                        Label::new("🚀 GUI System Improvements Demo")
                            .with_color([0.2, 0.6, 1.0, 1.0]),
                    )
                    .push(Label::new("Hover and interact to see smooth animations!"))
                    .push(Label::new(
                        "Scroll down to see more content and test viewport culling!",
                    )),
            ))
            .push(Panel::new(Column::new().push(
                Label::new("📊 Features:").with_color([0.2, 0.8, 0.2, 1.0]),
            )))
            .push({
                let button = Button::new("Hover Me - Smooth Animations!");

                let slider = Slider::new(0.5, 0.0, 1.0);

                let checkbox = Checkbox::new("AnimatedValue: FPS-Independent", false);

                Panel::new(
                    Column::new()
                        .push(
                            Label::new("🎨 Interactive Elements:").with_color([0.8, 0.6, 0.2, 1.0]),
                        )
                        .push(button)
                        .push(slider)
                        .push(checkbox)
                        .push(text_input),
                )
            })
            .push(Panel::new(
                Column::new()
                    .push(
                        Label::new("📈 Animated Progress (using AnimatedValue):")
                            .with_color([1.0, 0.2, 0.6, 1.0]),
                    )
                    .push(progress_bar),
            ))
            .push(Panel::new(
                Column::new()
                    .push(Label::new("📜 ScrollView Demo:").with_color([0.6, 0.2, 1.0, 1.0]))
                    .push(Label::new("This content is inside a ScrollView!"))
                    .push(Label::new(
                        "Use mouse wheel or drag the scrollbar to scroll.",
                    ))
                    .push(Label::new("The ScrollView has an is_visible() API.")),
            ))
            .push(Panel::new(Label::new(
                "Item 1: Scroll to see more items below",
            )))
            .push(Panel::new(Label::new(
                "Item 2: Each panel is a separate widget",
            )))
            .push(Panel::new(Label::new(
                "Item 3: ScrollView provides smooth scrolling",
            )))
            .push(Panel::new(Label::new(
                "Item 4: The scrollbar appears when needed",
            )))
            .push(Panel::new(Label::new(
                "Item 5: You can drag it to jump to positions",
            )))
            .push(Panel::new(Label::new(
                "Item 6: Or use mouse wheel for smooth scrolling",
            )))
            .push(Panel::new(Label::new(
                "Item 7: Keep scrolling to see more!",
            )))
            .push(Panel::new(Label::new(
                "Item 8: ScrollView has viewport culling API",
            )))
            .push(Panel::new(Label::new(
                "Item 9: Using the is_visible() method",
            )))
            .push(Panel::new(Label::new(
                "Item 10: Try scrolling up and down!",
            )))
            .push(Label::new("FPS: Calculating...")); // Will be updated via state

        // Create ScrollView with content
        let scroll_view = ScrollView::new(scrollable_content);

        // Update root_widget
        self.root_widget = Box::new(scroll_view);
    }

    fn update(&mut self) {
        // Calculate delta time
        let now = Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Animate progress bar (loops 0 -> 1)
        self.progress_animation.update(dt);
        if self.progress_animation.get() >= 1.0 {
            self.progress_animation.set_instant(0.0);
            self.progress_animation.set_target(1.0);
        } else {
            self.progress_animation.set_target(1.0);
        }

        // Update progress bar via state (reuse SliderState)
        let mut progress_state: SliderState =
            self.widget_state.get_or_default(self.progress_bar_id);
        progress_state.value = self.progress_animation.get();
        self.widget_state.set(self.progress_bar_id, progress_state);

        // Update FPS counter
        self.frame_count += 1;
        let fps_elapsed = (now - self.last_fps_update).as_secs_f32();
        if fps_elapsed >= 0.5 {
            self.fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.last_fps_update = now;

            // Update FPS label via state
            let mut label_state: LabelState = self.widget_state.get_or_default(self.fps_label_id);
            label_state.text = Some(format!(
                "⚡ FPS: {:.1} | Showing smooth AnimatedValue in action!",
                self.fps
            ));
            self.widget_state.set(self.fps_label_id, label_state);
        }

        // Request continuous redraw for animations
        self.window.request_redraw();
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.renderer
                .resize(new_size.width, new_size.height, &self.queue);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.prepare();

        let request_redraw = || self.window.request_redraw();
        let mut ctx = WidgetContext::new(
            &mut self.renderer,
            &self.theme,
            &mut self.widget_state,
            &mut self.focus_manager,
            &request_redraw,
        );
        let screen_rect = Rect::new(
            10.0,
            10.0,
            self.size.width as f32 - 20.0,
            self.size.height as f32 - 20.0,
        );

        self.root_widget.layout(&mut ctx, screen_rect);
        self.root_widget.draw(&mut ctx);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.renderer.render(
            &self.device,
            &self.queue,
            &view,
            &mut encoder,
            self.size.width,
            self.size.height,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("GUI Improvements Demo - Hover & Interact!")
                .with_inner_size(winit::dpi::LogicalSize::new(900, 800));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.state = Some(pollster::block_on(State::new(window)));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
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
            WindowEvent::CursorMoved { position, .. } => {
                state.input_state.mouse_pos = glam::Vec2::new(position.x as f32, position.y as f32);
                let request_redraw = || state.window.request_redraw();
                let mut ctx = WidgetContext::new(
                    &mut state.renderer,
                    &state.theme,
                    &mut state.widget_state,
                    &mut state.focus_manager,
                    &request_redraw,
                );
                state
                    .root_widget
                    .handle_event(&mut ctx, &Event::Move(state.input_state.mouse_pos));
                state.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: element_state,
                button,
                ..
            } => {
                if button == MouseButton::Left {
                    match element_state {
                        ElementState::Pressed => {
                            state.input_state.mouse_pressed = true;
                            let request_redraw = || state.window.request_redraw();
                            let mut ctx = WidgetContext::new(
                                &mut state.renderer,
                                &state.theme,
                                &mut state.widget_state,
                                &mut state.focus_manager,
                                &request_redraw,
                            );
                            state
                                .root_widget
                                .handle_event(&mut ctx, &Event::Press(state.input_state.mouse_pos));
                        }
                        ElementState::Released => {
                            state.input_state.mouse_pressed = false;
                            let request_redraw = || state.window.request_redraw();
                            let mut ctx = WidgetContext::new(
                                &mut state.renderer,
                                &state.theme,
                                &mut state.widget_state,
                                &mut state.focus_manager,
                                &request_redraw,
                            );
                            state.root_widget.handle_event(
                                &mut ctx,
                                &Event::Release(state.input_state.mouse_pos),
                            );
                        }
                    }
                    state.window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 20.0,
                };
                let request_redraw = || state.window.request_redraw();
                let mut ctx = WidgetContext::new(
                    &mut state.renderer,
                    &state.theme,
                    &mut state.widget_state,
                    &mut state.focus_manager,
                    &request_redraw,
                );
                state
                    .root_widget
                    .handle_event(&mut ctx, &Event::Scroll(scroll_delta));
                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: element_state,
                        text,
                        ..
                    },
                ..
            } => {
                if element_state == ElementState::Pressed {
                    let request_redraw = || state.window.request_redraw();
                    let mut ctx = WidgetContext::new(
                        &mut state.renderer,
                        &state.theme,
                        &mut state.widget_state,
                        &mut state.focus_manager,
                        &request_redraw,
                    );
                    // Handle character input from text field
                    if let Some(text_str) = text {
                        for c in text_str.chars() {
                            if !c.is_control() || c == ' ' {
                                state.root_widget.handle_event(&mut ctx, &Event::Char(c));
                            }
                        }
                    }

                    // Handle key events (arrows, backspace, etc.)
                    let modifiers = Modifiers {
                        ctrl: state.input_state.modifiers.control_key(),
                        shift: state.input_state.modifiers.shift_key(),
                        alt: state.input_state.modifiers.alt_key(),
                    };
                    state
                        .root_widget
                        .handle_event(&mut ctx, &Event::Key(logical_key.clone(), modifiers));
                    state.window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                state.input_state.modifiers = new_modifiers.state();
            }
            WindowEvent::Ime(ime) => {
                use kirei::gui::core::{Event as GuiEvent, ImeEvent};
                let request_redraw = || state.window.request_redraw();
                let mut ctx = WidgetContext::new(
                    &mut state.renderer,
                    &state.theme,
                    &mut state.widget_state,
                    &mut state.focus_manager,
                    &request_redraw,
                );
                match ime {
                    winit::event::Ime::Enabled => {
                        // IME enabled - ready for composition
                    }
                    winit::event::Ime::Disabled => {
                        // IME disabled - cancel any active composition
                        if state.input_state.ime_composing {
                            state
                                .root_widget
                                .handle_event(&mut ctx, &GuiEvent::Ime(ImeEvent::Cancel));
                            state.input_state.ime_composing = false;
                            state.window.request_redraw();
                        }
                    }
                    winit::event::Ime::Commit(text) => {
                        // Commit the composition
                        if state.input_state.ime_composing {
                            state
                                .root_widget
                                .handle_event(&mut ctx, &GuiEvent::Ime(ImeEvent::Commit(text)));
                            state.input_state.ime_composing = false;
                            state.window.request_redraw();
                        }
                    }
                    winit::event::Ime::Preedit(text, _cursor_offset) => {
                        // Preedit text update
                        if !state.input_state.ime_composing {
                            // First preedit - start composition
                            state
                                .root_widget
                                .handle_event(&mut ctx, &GuiEvent::Ime(ImeEvent::Start));
                            state.input_state.ime_composing = true;
                        }
                        // Update composition text
                        state
                            .root_widget
                            .handle_event(&mut ctx, &GuiEvent::Ime(ImeEvent::Update(text)));
                        state.window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    println!("🚀 GUI Improvements Visual Demo");
    println!("================================\n");
    println!("This demo showcases various GUI features:");
    println!("  • Smooth animations using AnimatedValue");
    println!("  • Real-time FPS counter\n");
    println!("Try:");
    println!("  - Hover over buttons to see smooth transitions");
    println!("  - Type in the text input");
    println!("  - Watch the animated progress bar loop");
    println!("  - Move the slider smoothly");
    println!("  - Check the FPS counter\n");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}

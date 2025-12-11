use kirei::gui::core::{FocusManager, Rect, Widget, WidgetContext, WidgetId, WidgetStateStorage};
use kirei::gui::renderer::Renderer;
use kirei::gui::theme::Theme;
use kirei::gui::widgets::{
    Align, Button, Column, Label, LabelState, Panel, ProgressBar, Row, ScrollView, Slider,
    SliderState,
};
use std::sync::Arc;
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
}

impl InputState {
    fn default() -> Self {
        Self {
            mouse_pos: glam::Vec2::ZERO,
            mouse_pressed: false,
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
    // Widget ID for dynamic label
    slider_value_label_id: WidgetId,
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

        // Use explicit widget ID instead of brittle path-based ID
        let slider_value_label_id = WidgetId::from_key("slider_value_label");

        // Create complex nested layouts
        let content = Column::new()
            .push(Label::new("Complex Layout Demos").with_color([0.2, 0.6, 1.0, 1.0]))
            .push(Label::new("Combining different layout patterns"))
            .push(Label::new(""))
            // Dashboard-style layout
            .push(Label::new("Dashboard Layout:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Panel::new(
                        Column::new()
                            .push(Label::new("Panel 1").with_color([1.0, 0.8, 0.2, 1.0]))
                            .push(Button::new("Action 1"))
                            .push(Button::new("Action 2"))
                            .align_items(Align::Start),
                    ))
                    .push(Panel::new(
                        Column::new()
                            .push(Label::new("Panel 2").with_color([0.2, 1.0, 0.8, 1.0]))
                            .push(ProgressBar::new(0.6))
                            .push(Label::new("Progress: 60%"))
                            .align_items(Align::Start),
                    ))
                    .push(Panel::new(
                        Column::new()
                            .push(Label::new("Panel 3").with_color([1.0, 0.5, 0.8, 1.0]))
                            .push(Slider::new(0.3, 0.0, 1.0).with_id("volume_slider"))
                            .push(Label::new("Value: 30%").with_id("slider_value_label"))
                            .align_items(Align::Start),
                    ))
                    .align_items(Align::Start),
            ))
            .push(Label::new(""))
            // Form-style layout with labels and controls
            .push(Label::new("Form Layout:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(Label::new("Volume:"))
                            .push(Slider::new(0.8, 0.0, 1.0))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Label::new("Brightness:"))
                            .push(Slider::new(0.5, 0.0, 1.0))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(Label::new("Quality:"))
                            .push(ProgressBar::new(0.9))
                            .align_items(Align::Center),
                    )
                    .align_items(Align::Start),
            ))
            .push(Label::new(""))
            // Toolbar layout
            .push(Label::new("Toolbar Layout:").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(Button::new("File"))
                    .push(Button::new("Edit"))
                    .push(Button::new("View"))
                    .push(Button::new("Help"))
                    .align_items(Align::Center),
            ))
            .push(Label::new(""))
            // Card layout with centered content
            .push(Label::new("Card Layout (Centered):").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Column::new()
                    .push(Label::new("Welcome!").with_color([0.2, 0.8, 1.0, 1.0]))
                    .push(Label::new("This is a centered card"))
                    .push(
                        Row::new()
                            .push(Button::new("Cancel"))
                            .push(Button::new("Accept"))
                            .align_items(Align::Center),
                    )
                    .align_items(Align::Center),
            ))
            .push(Label::new(""))
            // Split layout with different alignments
            .push(Label::new("Split Layout (Start vs End):").with_color([0.8, 0.8, 0.8, 1.0]))
            .push(Panel::new(
                Row::new()
                    .push(
                        Column::new()
                            .push(Label::new("Left Side"))
                            .push(Button::new("Left 1"))
                            .push(Button::new("Left 2"))
                            .align_items(Align::Start),
                    )
                    .push(
                        Column::new()
                            .push(Label::new("Right Side"))
                            .push(Button::new("Right 1"))
                            .push(Button::new("Right 2"))
                            .align_items(Align::End),
                    )
                    .align_items(Align::Start),
            ))
            .align_items(Align::Start);

        let root_widget = Panel::new(ScrollView::new(content));

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
            window,
            widget_state: WidgetStateStorage::new(),
            focus_manager: FocusManager::new(),
            slider_value_label_id,
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

            let rect = Rect::new(0.0, 0.0, new_size.width as f32, new_size.height as f32);
            let request_redraw = || self.window.request_redraw();
            let mut ctx = WidgetContext::new(
                &mut self.renderer,
                &self.theme,
                &mut self.widget_state,
                &mut self.focus_manager,
                &request_redraw,
            );
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
                button: MouseButton::Left,
                ..
            } => {
                self.input_state.mouse_pressed = true;
                Some(GuiEvent::Press(self.input_state.mouse_pos))
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.input_state.mouse_pressed = false;
                Some(GuiEvent::Release(self.input_state.mouse_pos))
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

        self.renderer.prepare();

        // Read slider value from state using explicit ID
        let slider_id = WidgetId::from_key("volume_slider");
        let slider_state: SliderState = self.widget_state.get_or_default(slider_id);
        let slider_value = slider_state.value;

        // Update label text via state
        let mut label_state: LabelState =
            self.widget_state.get_or_default(self.slider_value_label_id);
        label_state.text = Some(format!("Value: {}%", (slider_value * 100.0) as u32));
        self.widget_state
            .set(self.slider_value_label_id, label_state);

        // Layout and draw
        let rect = Rect::new(0.0, 0.0, self.size.width as f32, self.size.height as f32);
        let request_redraw = || self.window.request_redraw();
        let mut ctx = WidgetContext::new(
            &mut self.renderer,
            &self.theme,
            &mut self.widget_state,
            &mut self.focus_manager,
            &request_redraw,
        );
        self.root_widget.layout(&mut ctx, rect);
        self.root_widget.draw(&mut ctx);

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

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes().with_title("Complex Layout Demos");
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
                WindowEvent::RedrawRequested => match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprintln!("{:?}", e),
                },
                _ => {}
            }
        } else {
            // If input handled by GUI, don't call update() to avoid duplicate request_redraw()
            // The input() method already calls request_redraw() if needed
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
